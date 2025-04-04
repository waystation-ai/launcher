// Modified from original Apache 2.0 licensed code: Removed unused commands and adjusted for WayStation MCP

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

    #[cfg(target_os = "macos")]
    {
        // Use macOS-specific path
        let default_path = dirs::home_dir()
            .ok_or("Could not find home directory".to_string())?
            .join("Library/Application Support/Claude/claude_desktop_config.json");
        debug!("Using default config path: {}", default_path.display());
        return Ok(default_path);
    }

    #[cfg(target_os = "windows")]
    {
        // Use Windows-specific path
        let default_path = dirs::config_dir()
            .ok_or("Could not find config directory".to_string())?
            .join("Claude/claude_desktop_config.json");
        debug!("Using default config path: {}", default_path.display());
        return Ok(default_path);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // Fallback for other platforms
        let default_path = dirs::config_dir()
            .ok_or("Could not find config directory".to_string())?
            .join("claude/claude_desktop_config.json");
        debug!("Using default config path: {}", default_path.display());
        return Ok(default_path);
    }
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

fn get_claude_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let claude_app_path = PathBuf::from("/Applications/Claude.app");
        debug!("Checking for Claude.app at: {}", claude_app_path.display());
        if claude_app_path.exists() {
            return Some(claude_app_path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let program_files = std::env::var("ProgramFiles").unwrap_or_default();
        let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_default();
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

        let possible_paths = [
            format!("{}\\AnthropicClaude\\Claude.exe", program_files),
            format!("{}\\AnthropicClaude\\Claude.exe", program_files_x86),
            format!("{}\\AnthropicClaude\\claude.exe", local_app_data),
        ];

        for path in possible_paths.iter() {
            debug!("Checking for Claude.exe at: {}", path);
            let path_buf = PathBuf::from(path);
            if path_buf.exists() {
                return Some(path_buf);
            }
        }
    }

    None
}

#[tauri::command]
pub fn restart_claude_app() -> Result<String, String> {
    info!("Restarting Claude app...");

    #[cfg(target_os = "macos")]
    {
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
    }

    #[cfg(target_os = "windows")]
    {
        // Kill the Claude app using taskkill
        Command::new("taskkill")
            .args(&["/F", "/IM", "Claude.exe"])
            .output()
            .map_err(|e| format!("Failed to kill Claude app: {}", e))?;

        // Wait a moment to ensure it's fully closed
        sleep(Duration::from_millis(500));

        if let Some(path) = get_claude_path() {
            debug!("Claude installation found at: {}", path.display());

            // Relaunch the app
            Command::new(path.to_str().unwrap())
            .spawn()
            .map_err(|e| format!("Failed to relaunch Claude app: {}", e))?;
        } else {
            debug!("Claude installation not found");
            return Err("Claude installation not found".to_string());
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        return Err("Restarting Claude app is not supported on this platform".to_string());
    }

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

pub fn get_app_directory() -> Result<std::path::PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, use AppData/Roaming directory
        let app_data_dir = dirs::data_dir()
            .ok_or("Could not determine AppData directory")?
            .join("WayStation");
        return Ok(app_data_dir);
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On macOS/Linux, use ~/.waystation
        let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
        let waystation_dir = home_dir.join(".waystation");
        return Ok(waystation_dir);
    }
}


#[tauri::command]
pub fn check_onboarding_completed() -> Result<bool, String> {
    let app_directory = get_app_directory()?;
    let onboarding_file = app_directory.join("onboarding_completed");

    debug!("Checking onboarding file at: {}", onboarding_file.display());
    Ok(onboarding_file.exists())
}

#[tauri::command]
pub fn check_claude_installed() -> Result<bool, String> {
    if let Some(path) = get_claude_path() {
        debug!("Claude installation found at: {}", path.display());
        Ok(true)
    } else {
        debug!("Claude installation not found");
        Ok(false)
    }
}
