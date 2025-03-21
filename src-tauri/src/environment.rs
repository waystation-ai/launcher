use log::{debug, error, info};
use once_cell::sync::Lazy;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

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

    Ok((node_path, npx_path))
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

    // Check NVM-installed node first
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

    // If not found in NVM, check system node
    let version_command = Command::new("node")
        .arg("--version")
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

    // Double-check node version to avoid race conditions
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

    // Verify nvm is properly installed before using it
    if !check_nvm_installed() {
        return Err("nvm is required to install Node.js".to_string());
    }

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

    // First check if .nvm directory exists
    let nvm_dir = dirs::home_dir()
        .map(|path| path.join(".nvm"))
        .filter(|path| path.exists());

    if nvm_dir.is_none() {
        info!("NVM directory not found");
        return false;
    }

    // Then check if we can run nvm to confirm it's properly installed
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

fn install_nvm() -> Result<(), String> {
    if is_test_mode() {
        return Ok(());
    }

    // Double-check nvm installation to avoid race conditions
    if check_nvm_installed() {
        info!("nvm is already installed, skipping installation");
        return Ok(());
    }

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

// New synchronous environment setup function for config.rs to use
pub fn ensure_environment_sync() -> Result<String, String> {
    if is_test_mode() {
        return Ok("Environment setup completed".to_string());
    }

    // If environment setup is already completed, return early
    if ENVIRONMENT_SETUP_COMPLETED.load(Ordering::SeqCst) {
        debug!("Environment setup already completed");
        return Ok("Environment setup already completed".to_string());
    }

    info!("Starting synchronous environment setup");

    // Use a mutex to prevent concurrent setup operations
    let _lock = match ENVIRONMENT_SETUP_LOCK.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            info!("Another environment setup is already in progress, waiting...");
            // Block until lock is available for synchronous operation
            ENVIRONMENT_SETUP_LOCK.lock().unwrap()
        }
    };

    // Check again if setup was completed while waiting
    if ENVIRONMENT_SETUP_COMPLETED.load(Ordering::SeqCst) {
        return Ok("Environment setup completed while waiting".to_string());
    }

    // Ensure node environment is ready
    ensure_node_environment()?;

    info!("Synchronous environment setup completed");
    Ok("Environment setup completed".to_string())
}

#[tauri::command]
pub fn ensure_environment() -> Result<String, String> {
    if is_test_mode() {
        return Ok("Environment setup started".to_string());
    }

    // Use a more reliable way to check if we're already setting up the environment
    if ENVIRONMENT_SETUP_STARTED.swap(true, Ordering::SeqCst) {
        info!("Environment setup already in progress, skipping");
        return Ok("Environment setup already in progress".to_string());
    }

    // Use a thread-safe approach for environment setup
    std::thread::spawn(|| {
        // Use a mutex to prevent concurrent setup operations
        let _lock = match ENVIRONMENT_SETUP_LOCK.try_lock() {
            Ok(guard) => guard,
            Err(_) => {
                info!("Another environment setup is already in progress");
                ENVIRONMENT_SETUP_STARTED.store(false, Ordering::SeqCst);
                return;
            }
        };

        info!("Starting environment setup");

        // Ensure node environment is ready
        if let Err(e) = ensure_node_environment() {
            error!("Failed to ensure node environment: {}", e);
        }

        info!("Environment setup completed");
        ENVIRONMENT_SETUP_STARTED.store(false, Ordering::SeqCst);
    });

    Ok("Environment setup started".to_string())
}
