use crate::file_utils::{ensure_config_file, ensure_mcp_servers};
use dirs;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    static ref CONFIG_CACHE: Mutex<Option<Value>> = Mutex::new(None);
    static ref ENV_SETUP_COMPLETE: Mutex<bool> = Mutex::new(false);
}

// Function to get the config path - uses test path if set
fn get_config_path() -> Result<PathBuf, String> {
    debug!(
        "Getting config path, test_mode: {}",
        crate::environment::is_test_mode()
    );

    // Otherwise use the default path
    let default_path = dirs::home_dir()
        .ok_or("Could not find home directory".to_string())?
        .join("Library/Application Support/Claude/claude_desktop_config.json");

    debug!("Using default config path: {}", default_path.display());
    Ok(default_path)
}

pub fn get_config() -> Result<Value, String> {
    debug!(
        "Getting config, test_mode: {}",
        crate::environment::is_test_mode()
    );

    let mut cache = CONFIG_CACHE.lock().unwrap();
    if let Some(ref config) = *cache {
        debug!("Using cached config");
        return Ok(config.clone());
    }

    let config_path = get_config_path()?;
    debug!("Using config path: {}", config_path.display());

    if !config_path.exists() {
        info!("Config file does not exist, creating it");
        ensure_config_file(&config_path)?;
    }

    let config_str = fs::read_to_string(&config_path).map_err(|e| {
        error!("Failed to read config file: {}", e);
        format!("Failed to read config file: {}", e)
    })?;

    let mut config_json: Value = serde_json::from_str(&config_str).map_err(|e| {
        error!("Failed to parse config JSON: {}", e);
        format!("Failed to parse config JSON: {}", e)
    })?;

    ensure_mcp_servers(&mut config_json)?;

    *cache = Some(config_json.clone());
    debug!("Config loaded and cached successfully");
    Ok(config_json)
}

pub fn save_config(config: &Value) -> Result<(), String> {
    let config_path = get_config_path()?;
    debug!("Saving config to {}", config_path.display());

    let updated_config = serde_json::to_string_pretty(config).map_err(|e| {
        error!("Failed to serialize config: {}", e);
        format!("Failed to serialize config: {}", e)
    })?;

    fs::write(&config_path, updated_config).map_err(|e| {
        error!("Failed to write config file: {}", e);
        format!("Failed to write config file: {}", e)
    })?;

    // Update cache
    let mut cache = CONFIG_CACHE.lock().unwrap();
    *cache = Some(config.clone());
    info!("Config saved successfully");

    Ok(())
}

#[tauri::command]
pub fn restart_claude_app() -> Result<String, String> {
    info!("Restarting Claude app...");

    // Kill the Claude app
    Command::new("pkill")
        .arg("-x")
        .arg("Claude")
        .output()
        .map_err(|e| format!("Failed to kill Claude app: {}", e))?;

    // Wait a moment to ensure it's fully closed
    sleep(Duration::from_millis(500));

    // Relaunch the app
    Command::new("open")
        .arg("-a")
        .arg("Claude")
        .output()
        .map_err(|e| format!("Failed to relaunch Claude app: {}", e))?;

    Ok("Claude app restarted successfully".to_string())
}

#[tauri::command]
pub fn install_waystation_mcp() -> Result<String, String> {
    info!("Installing waystation-mcp...");

    let mut config_json = get_config()?;

    if let Some(mcp_servers) = config_json
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
    {
        let app_config = json!({
            "command": "npx",
            "args": ["-y", "@waystation/mcp"]
        });

        debug!("Adding config for waystation: {:?}", app_config);
        mcp_servers.insert("WayStation".to_string(), app_config);
        save_config(&config_json)?;

        info!("Successfully installed waystation-mcp");
        Ok("Added waystation-mcp configuration".to_string())
    } else {
        let err = "Failed to find mcpServers in config".to_string();
        error!("{}", err);
        Err(err)
    }
}

#[tauri::command]
pub fn uninstall_waystation_mcp() -> Result<String, String> {
    info!("Uninstalling waystation-mcp...");

    let mut config_json = get_config()?;

    if let Some(mcp_servers) = config_json
        .get_mut("mcpServers")
        .and_then(|v| v.as_object_mut())
    {
        if mcp_servers.remove("WayStation").is_some() {
            save_config(&config_json)?;
            info!("Successfully uninstalled waystation-mcp");
            Ok("Removed waystation-mcp configuration".to_string())
        } else {
            warn!("waystation-mcp configuration was not found");
            Ok("waystation-mcp configuration was not found".to_string())
        }
    } else {
        let err = "Failed to find mcpServers in config".to_string();
        error!("{}", err);
        Err(err)
    }
}

#[tauri::command]
pub fn check_onboarding_completed() -> Result<bool, String> {
    let home = match dirs::home_dir() {
        Some(path) => path,
        None => return Err("Could not determine home directory".to_string()),
    };
    let onboarding_file = home.join(".waystation/onboarding_completed");

    debug!("Checking onboarding file at: {}", onboarding_file.display());
    Ok(onboarding_file.exists())
}

#[tauri::command]
pub fn check_claude_installed() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        let claude_app_path = std::path::PathBuf::from("/Applications/Claude.app");
        debug!("Checking for Claude.app at: {}", claude_app_path.display());
        return Ok(claude_app_path.exists());
    }

    #[cfg(not(target_os = "macos"))]
    {
        return Ok(false);
    }
}
