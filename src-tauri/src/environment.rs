// Modified from original Apache 2.0 licensed code: Removed UVX support and adjusted for WayStation MCP

use log::{debug, error, info};
use once_cell::sync::Lazy;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Windows-specific constants
#[cfg(target_os = "windows")]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

static NVM_INSTALLED: AtomicBool = AtomicBool::new(false);
static NODE_INSTALLED: AtomicBool = AtomicBool::new(false);
static ENVIRONMENT_SETUP_STARTED: AtomicBool = AtomicBool::new(false);
static ENVIRONMENT_SETUP_COMPLETED: AtomicBool = AtomicBool::new(false);
static NODE_VERSION: &str = "v20.9.0";
static IS_TEST_MODE: AtomicBool = AtomicBool::new(false);

// Lock to prevent concurrent environment setup operations
static ENVIRONMENT_SETUP_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub fn is_test_mode() -> bool {
    IS_TEST_MODE.load(Ordering::SeqCst)
}

pub fn get_nvm_node_paths() -> Result<(String, String), String> {
    debug!("get_nvm_node_paths called, test_mode: {}", is_test_mode());

    if is_test_mode() {
        debug!("Using test mode paths for nvm/node");
        return Ok((
            "/test/.nvm/versions/node/v20.9.0/bin/node".to_string(),
            "/test/.nvm/versions/node/v20.9.0/bin/npx".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        let shell_command = format!(
            r#"
          export NVM_DIR="$HOME/.nvm"
          [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
          nvm use {} > /dev/null 2>&1
          which node
          which npx
      "#,
            NODE_VERSION
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(shell_command)
            .output()
            .map_err(|e| format!("Failed to get node paths: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get node and npx paths".to_string());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut lines = output_str.lines();

        let node_path = lines
            .next()
            .ok_or("Failed to get node path")?
            .trim()
            .to_string();

        let npx_path = lines
            .next()
            .ok_or("Failed to get npx path")?
            .trim()
            .to_string();

        // Only validate paths in non-test mode
        if !is_test_mode() && !node_path.contains(".nvm/versions/node") {
            debug!("Node path validation failed: {}", node_path);
            return Err("Node path is not from nvm installation".to_string());
        }

        return Ok((node_path, npx_path));
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we'll check for Node.js in common installation locations
        // and use the system-installed Node.js rather than NVM
        
        // First try to find node in PATH
        let node_output = Command::new("where")
            .arg("node.exe")
            .output()
            .map_err(|e| format!("Failed to locate node.exe: {}", e))?;
            
        let npx_output = Command::new("where")
            .arg("npx.cmd")
            .output()
            .map_err(|e| format!("Failed to locate npx.cmd: {}", e))?;
            
        if node_output.status.success() && npx_output.status.success() {
            let node_path = String::from_utf8_lossy(&node_output.stdout)
                .lines()
                .next()
                .ok_or("Failed to get node path")?
                .trim()
                .to_string();
                
            let npx_path = String::from_utf8_lossy(&npx_output.stdout)
                .lines()
                .next()
                .ok_or("Failed to get npx path")?
                .trim()
                .to_string();
                
            return Ok((node_path, npx_path));
        }
        
        // If not found in PATH, check common installation locations
        let program_files = std::env::var("ProgramFiles").unwrap_or_default();
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        
        let possible_node_paths = [
            format!("{}\\nodejs\\node.exe", program_files),
            format!("{}\\nodejs\\node.exe", appdata),
        ];
        
        let possible_npx_paths = [
            format!("{}\\nodejs\\npx.cmd", program_files),
            format!("{}\\nodejs\\npx.cmd", appdata),
        ];
        
        for (node_path, npx_path) in possible_node_paths.iter().zip(possible_npx_paths.iter()) {
            if std::path::Path::new(node_path).exists() && std::path::Path::new(npx_path).exists() {
                return Ok((node_path.clone(), npx_path.clone()));
            }
        }
        
        return Err("Node.js installation not found".to_string());
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        // For other Unix-like systems
        let shell_command = format!(
            r#"
          export NVM_DIR="$HOME/.nvm"
          [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
          nvm use {} > /dev/null 2>&1
          which node
          which npx
      "#,
            NODE_VERSION
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(shell_command)
            .output()
            .map_err(|e| format!("Failed to get node paths: {}", e))?;

        if !output.status.success() {
            return Err("Failed to get node and npx paths".to_string());
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut lines = output_str.lines();

        let node_path = lines
            .next()
            .ok_or("Failed to get node path")?
            .trim()
            .to_string();

        let npx_path = lines
            .next()
            .ok_or("Failed to get npx path")?
            .trim()
            .to_string();

        return Ok((node_path, npx_path));
    }
}

fn check_node_version() -> Result<String, String> {
    if is_test_mode() {
        return Ok(NODE_VERSION.to_string());
    }

    // If we already confirmed node is installed with correct version, return early
    if NODE_INSTALLED.load(Ordering::SeqCst) {
        debug!("Node.js already confirmed as installed");
        return Ok(NODE_VERSION.to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let shell_command = format!(
            r#"
          export NVM_DIR="$HOME/.nvm"
          [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
          nvm list | grep -w "{}" || true
      "#,
            NODE_VERSION
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(shell_command)
            .output()
            .map_err(|e| format!("Failed to check nvm node version: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains(NODE_VERSION) {
            info!("Node.js {} is already installed via nvm", NODE_VERSION);
            NODE_INSTALLED.store(true, Ordering::SeqCst);
            return Ok(NODE_VERSION.to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        if check_nvm_installed() {
            let nvm_cmd = Command::new("nvm")
                .arg("list")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Failed to check nvm node version: {}", e))?;

            let output_str = String::from_utf8_lossy(&nvm_cmd.stdout);
            let version_no_v = NODE_VERSION.trim_start_matches('v');

            if output_str.contains(NODE_VERSION) || output_str.contains(version_no_v) {
                info!("Node.js {} is already installed via nvm", NODE_VERSION);
                NODE_INSTALLED.store(true, Ordering::SeqCst);
                return Ok(NODE_VERSION.to_string());
            }

            let nvm_root = std::env::var("NVM_HOME")
                .ok()
                .map(std::path::PathBuf::from)
                .or_else(|| dirs::home_dir().map(|p| p.join("AppData").join("Roaming").join("nvm")))
                .ok_or("Could not determine NVM_HOME")?;

            let node_exists = nvm_root.join(version_no_v).join("node.exe").exists()
                || nvm_root
                    .join(format!("v{}", version_no_v))
                    .join("node.exe")
                    .exists();

            if node_exists {
                info!("Node.js {} binary found via nvm", NODE_VERSION);
                NODE_INSTALLED.store(true, Ordering::SeqCst);
                return Ok(NODE_VERSION.to_string());
            }
        }
    }

    #[cfg(target_os = "macos")]
    let version_command = Command::new("node")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to check node version: {}", e))?;

    #[cfg(target_os = "windows")]
    let version_command = Command::new("node")
        .arg("--version")
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("Failed to check node version: {}", e))?;

    if version_command.status.success() {
        let version = String::from_utf8_lossy(&version_command.stdout)
            .trim()
            .to_string();

        if version == NODE_VERSION {
            info!("Node.js {} is already installed system-wide", NODE_VERSION);
            NODE_INSTALLED.store(true, Ordering::SeqCst);
            return Ok(version);
        }

        info!("Found Node.js {} but {} is required", version, NODE_VERSION);
        return Ok(version);
    }

    Err("Node.js not found".to_string())
}

fn check_nvm_version() -> Result<String, String> {
    if is_test_mode() {
        return Ok("0.40.1".to_string());
    }

    let shell_command = r#"
        export NVM_DIR="$HOME/.nvm"
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
        nvm --version
    "#;

    let output = Command::new("bash")
        .arg("-c")
        .arg(shell_command)
        .output()
        .map_err(|e| format!("Failed to check nvm version: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get nvm version".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn install_node() -> Result<(), String> {
    if is_test_mode() {
        return Ok(());
    }

    match check_node_version() {
        Ok(version) if version == NODE_VERSION => {
            info!(
                "Node.js {} is already installed, skipping installation",
                NODE_VERSION
            );
            NODE_INSTALLED.store(true, Ordering::Relaxed);
            return Ok(());
        }
        _ => {}
    }

    info!("Installing Node.js {}", NODE_VERSION);

    if !check_nvm_installed() {
        return Err("nvm is required to install Node.js".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let shell_command = format!(
            r#"
          export NVM_DIR="$HOME/.nvm"
          [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
          nvm install {} --no-progress
      "#,
            NODE_VERSION
        );

        let output = Command::new("bash")
            .arg("-c")
            .arg(shell_command)
            .output()
            .map_err(|e| format!("Failed to run node installation: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Node installation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let version_without_v = NODE_VERSION.trim_start_matches('v');

        let output = Command::new("nvm")
            .arg("install")
            .arg(version_without_v)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run node installation: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "Node installation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let use_output = Command::new("nvm")
            .arg("use")
            .arg(version_without_v)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to set node version: {}", e))?;

        if !use_output.status.success() {
            return Err(format!(
                "Failed to set node version: {}",
                String::from_utf8_lossy(&use_output.stderr)
            ));
        }

        let nvm_root = std::env::var("NVM_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| dirs::home_dir().map(|p| p.join("AppData").join("Roaming").join("nvm")))
            .unwrap_or_default();

        let node_exists = nvm_root.join(version_without_v).join("node.exe").exists()
            || nvm_root
                .join(format!("v{}", version_without_v))
                .join("node.exe")
                .exists();

        if !node_exists {
            return Err(format!(
                "Node.js {} installation verification failed. Binary not found at expected locations.",
                NODE_VERSION
            ));
        }
    }

    NODE_INSTALLED.store(true, Ordering::Relaxed);
    info!("Node.js {} installed successfully", NODE_VERSION);
    Ok(())
}

fn check_nvm_installed() -> bool {
    if is_test_mode() {
        return true;
    }

    // If we've already confirmed nvm is installed, return early
    if NVM_INSTALLED.load(Ordering::Relaxed) {
        debug!("NVM already confirmed as installed");
        return true;
    }

    #[cfg(target_os = "macos")]
    {
        let nvm_dir = dirs::home_dir()
            .map(|path| path.join(".nvm"))
            .filter(|path| path.exists());

        if nvm_dir.is_none() {
            info!("NVM directory not found");
            return false;
        }

        match check_nvm_version() {
            Ok(version) => {
                info!("NVM version {} is installed", version);
                NVM_INSTALLED.store(true, Ordering::Relaxed);
                true
            }
            Err(_) => {
                info!("NVM directory exists but nvm command failed");
                false
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let nvm_home = std::env::var("NVM_HOME")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| dirs::home_dir().map(|p| p.join("AppData").join("Roaming").join("nvm")));

        if let Some(nvm_path) = nvm_home {
            if nvm_path.exists() {
                let nvm_exe = nvm_path.join("nvm.exe");
                if nvm_exe.exists() {
                    info!("NVM for Windows found at {}", nvm_path.display());
                    NVM_INSTALLED.store(true, Ordering::Relaxed);
                    return true;
                }
            }
        }

        info!("NVM for Windows not found");
        false
    }
}

fn install_nvm() -> Result<(), String> {
    if is_test_mode() {
        return Ok(());
    }

    if check_nvm_installed() {
        info!("nvm is already installed, skipping installation");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        info!("Installing nvm...");

        let shell_command = r#"
          curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
      "#;

        let output = Command::new("bash")
            .arg("-c")
            .arg(shell_command)
            .output()
            .map_err(|e| format!("Failed to install nvm: {}", e))?;

        if !output.status.success() {
            return Err(format!(
                "nvm installation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        NVM_INSTALLED.store(true, Ordering::Relaxed);
        info!("nvm installed successfully");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        info!("Installing nvm for Windows...");

        let temp_dir = std::env::temp_dir().join("waystation_nvm_install");
        let _ = std::fs::create_dir_all(&temp_dir);
        let installer_path = temp_dir.join("nvm-setup.exe");

        let nvm_installer_url =
            "https://github.com/coreybutler/nvm-windows/releases/download/1.1.11/nvm-setup.exe";

        let download_cmd = format!(
            "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
            nvm_installer_url,
            installer_path.to_string_lossy()
        );

        let dl_output = Command::new("powershell")
            .arg("-Command")
            .arg(&download_cmd)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to download nvm installer: {}", e))?;

        if !dl_output.status.success() {
            return Err(format!(
                "Failed to download nvm installer: {}",
                String::from_utf8_lossy(&dl_output.stderr)
            ));
        }

        info!("Starting NVM for Windows installer. Please follow the on-screen instructions.");
        let installer_output = Command::new(&installer_path)
            .arg("/SILENT")
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to run nvm installer: {}", e))?;

        if !installer_output.status.success() {
            return Err(format!(
                "NVM installation failed: {}",
                String::from_utf8_lossy(&installer_output.stderr)
            ));
        }

        let _ = std::fs::remove_file(&installer_path);
        let _ = std::fs::remove_dir(&temp_dir);

        if check_nvm_installed() {
            NVM_INSTALLED.store(true, Ordering::Relaxed);
            info!("nvm for Windows installed successfully");
            Ok(())
        } else {
            Err("nvm for Windows installation completed but verification failed".to_string())
        }
    }
}

fn ensure_node_environment() -> Result<String, String> {
    if is_test_mode() {
        return Ok("Node environment is ready".to_string());
    }

    // First check if we have nvm installed, install if needed
    if !check_nvm_installed() {
        install_nvm()?;
    }

    // Check if we have the correct node version, install if needed
    match check_node_version() {
        Ok(version) => {
            if version != NODE_VERSION {
                info!(
                    "Node.js {} found, but {} required. Installing...",
                    version, NODE_VERSION
                );
                install_node()?;
            } else {
                debug!("Node.js {} is already installed", NODE_VERSION);
                NODE_INSTALLED.store(true, Ordering::Relaxed);
            }
        }
        Err(_) => {
            info!("Node.js not found. Installing...");
            install_node()?;
        }
    }

    // Mark environment setup as completed
    ENVIRONMENT_SETUP_COMPLETED.store(true, Ordering::SeqCst);

    Ok("Node environment is ready".to_string())
}

#[tauri::command]
pub async fn ensure_environment() -> Result<String, String> {
    if is_test_mode() {
        return Ok("Environment setup started".to_string());
    }

    if ENVIRONMENT_SETUP_STARTED.swap(true, Ordering::SeqCst) {
        info!("Environment setup already in progress, skipping");
        return Ok("Environment setup already in progress".to_string());
    }

    match tauri::async_runtime::spawn_blocking(|| {
        let _lock = match ENVIRONMENT_SETUP_LOCK.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                info!("Another environment setup is already in progress");
                ENVIRONMENT_SETUP_STARTED.store(false, Ordering::SeqCst);
                return Err("Another environment setup is already in progress".to_string());
            }
        };

        info!("Starting environment setup");
        let mut setup_failed = false;

        if let Err(e) = ensure_node_environment() {
            error!("Failed to ensure node environment: {}", e);
            setup_failed = true;
        }

        ENVIRONMENT_SETUP_STARTED.store(false, Ordering::SeqCst);

        if setup_failed {
            ENVIRONMENT_SETUP_COMPLETED.store(false, Ordering::SeqCst);
            Err("Environment setup failed. Please check the logs for details.".to_string())
        } else {
            ENVIRONMENT_SETUP_COMPLETED.store(true, Ordering::SeqCst);
            info!("Environment setup completed successfully");
            Ok("Environment setup completed".to_string())
        }
    })
    .await
    {
        Ok(result) => result,
        Err(e) => {
            error!("Environment setup task panicked: {}", e);
            ENVIRONMENT_SETUP_STARTED.store(false, Ordering::SeqCst);
            ENVIRONMENT_SETUP_COMPLETED.store(false, Ordering::SeqCst);
            Err("Environment setup failed unexpectedly".to_string())
        }
    }
}

#[cfg(target_os = "windows")]
pub fn create_windowless_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn create_windowless_command(program: &str) -> Command {
    Command::new(program)
}