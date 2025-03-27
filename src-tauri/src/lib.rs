pub mod app;
pub mod environment;
pub mod file_utils;

use base64::{engine::general_purpose, Engine as _};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use url::Url;

use tauri_plugin_opener::OpenerExt;

// Constants for OAuth configuration
const AUTH_URL: &str = "https://clerk.waystation.ai/oauth/authorize";
const TOKEN_URL: &str = "https://clerk.waystation.ai/oauth/token";
const USER_INFO_URL: &str = "https://clerk.waystation.ai/oauth/userinfo";
const CLIENT_ID: &str = "5xEs1bi3TY8JNVHx";
const REDIRECT_URI: &str = "waystation://oauth/callback";
const STORE_PATH: &str = ".auth.dat";

// Structs for OAuth data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthState {
    pub code_verifier: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_at: Option<u64>,
    pub user_info: Option<UserInfo>,
}

// State management
struct AuthStateManager(Mutex<Option<AuthState>>);

// Helper functions for PKCE
fn generate_code_verifier() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    general_purpose::URL_SAFE_NO_PAD.encode(hash)
}

fn generate_state() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

// Tauri commands
#[tauri::command]
async fn login(
    state: State<'_, AuthStateManager>,
    app_handle: AppHandle,
) -> Result<String, String> {
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state_value = generate_state();

    // Store PKCE and state values
    let auth_state = AuthState {
        code_verifier,
        state: state_value.clone(),
    };
    *state.0.lock().unwrap() = Some(auth_state);

    // Build authorization URL
    let mut url = Url::parse(AUTH_URL).map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair("client_id", CLIENT_ID)
        .append_pair("redirect_uri", REDIRECT_URI)
        .append_pair("response_type", "code")
        .append_pair("scope", "profile email")
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", &state_value);

    // Open the URL in the default browser
    let _ = app_handle.opener().open_path(url.as_str(), None::<&str>);

    Ok("Authorization URL opened in browser".to_string())
}

#[tauri::command]
async fn handle_redirect_uri(
    url: String,
    state: State<'_, AuthStateManager>,
    app_handle: AppHandle,
) -> Result<AuthData, String> {
    // Parse the URL
    let url = Url::parse(&url).map_err(|e| e.to_string())?;

    // Extract query parameters
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();

    // Get the authorization code and state
    let code = query_params
        .get("code")
        .ok_or("No authorization code found in redirect URI")?;
    let received_state = query_params
        .get("state")
        .ok_or("No state found in redirect URI")?;

    // Verify state
    let auth_state = state
        .0
        .lock()
        .unwrap()
        .clone()
        .ok_or("No auth state found")?;
    if received_state != &auth_state.state {
        return Err("State mismatch, possible CSRF attack".to_string());
    }

    // Exchange code for tokens
    let client = Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("code", code),
        ("code_verifier", &auth_state.code_verifier),
        ("grant_type", "authorization_code"),
        ("redirect_uri", REDIRECT_URI),
    ];

    let token_response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.map_err(|e| e.to_string())?;
        return Err(format!("Token request failed: {}", error_text));
    }

    let tokens: TokenResponse = token_response.json().await.map_err(|e| e.to_string())?;

    // Get user info
    let user_info_response = client
        .get(USER_INFO_URL)
        .bearer_auth(&tokens.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let user_info: Option<UserInfo> = if user_info_response.status().is_success() {
        Some(user_info_response.json().await.map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Calculate token expiration
    let expires_at = tokens.expires_in.map(|expires_in| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + expires_in
    });

    // Store tokens
    let auth_data = AuthData {
        access_token: tokens.access_token.clone(),
        refresh_token: tokens.refresh_token,
        id_token: tokens.id_token,
        expires_at,
        user_info,
    };

    // Save to persistent store
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let store_path = app_data_dir.join(STORE_PATH);

    // Create a simple JSON file to store the auth data
    let json_data = serde_json::to_string_pretty(&auth_data).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(app_data_dir).map_err(|e| e.to_string())?;
    std::fs::write(&store_path, json_data).map_err(|e| e.to_string())?;

    // Fetch and save MCP token
    save_way_key(tokens.access_token).await.ok(); // Ignore errors

    Ok(auth_data)
}

#[tauri::command]
async fn get_auth_data(app_handle: AppHandle) -> Result<Option<AuthData>, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let store_path = app_data_dir.join(STORE_PATH);

    // Read the auth data from the file
    if !store_path.exists() {
        return Ok(None);
    }

    let json_data = std::fs::read_to_string(&store_path).map_err(|e| e.to_string())?;
    let auth_data: AuthData = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;
    Ok(Some(auth_data))
}

#[tauri::command]
async fn logout(app_handle: AppHandle) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let store_path = app_data_dir.join(STORE_PATH);

    // Delete the auth data file
    if store_path.exists() {
        std::fs::remove_file(&store_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn save_way_key(access_token: String) -> Result<(), String> {
    let app_dir = app::get_app_directory()?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;

    // Write token to file
    let token_path = app_dir.join("token");
    std::fs::write(&token_path, format!("Bearer {}", &access_token))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn refresh_token(app_handle: AppHandle) -> Result<AuthData, String> {
    // Get current auth data
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let store_path = app_data_dir.join(STORE_PATH);

    if !store_path.exists() {
        return Err("No auth data found".to_string());
    }

    let json_data = std::fs::read_to_string(&store_path).map_err(|e| e.to_string())?;
    let auth_data: AuthData = serde_json::from_str(&json_data).map_err(|e| e.to_string())?;

    let refresh_token = auth_data
        .refresh_token
        .clone()
        .ok_or("No refresh token available")?;

    // Exchange refresh token for new tokens
    let client = Client::new();
    let params = [
        ("client_id", CLIENT_ID),
        ("refresh_token", &refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let token_response = client
        .post(TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !token_response.status().is_success() {
        let error_text = token_response.text().await.map_err(|e| e.to_string())?;
        return Err(format!("Token refresh failed: {}", error_text));
    }

    let tokens: TokenResponse = token_response.json().await.map_err(|e| e.to_string())?;

    // Calculate token expiration
    let expires_at = tokens.expires_in.map(|expires_in| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + expires_in
    });

    // Update auth data
    let new_auth_data = AuthData {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token.or(auth_data.refresh_token),
        id_token: tokens.id_token.or(auth_data.id_token),
        expires_at,
        user_info: auth_data.user_info,
    };

    // Save to persistent store
    let json_data = serde_json::to_string_pretty(&new_auth_data).map_err(|e| e.to_string())?;
    std::fs::write(&store_path, json_data).map_err(|e| e.to_string())?;

    save_way_key(new_auth_data.access_token.clone()).await.ok();

    Ok(new_auth_data)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();

            println!("a new app instance was opened with {_args:?} and the deep link event was already triggered");

        })); 
    }

    // Create the Tauri application builder
    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_deep_link::init())        
        .plugin(tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Warn)
            .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview, ))
            .target(tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: Some("logs".to_string()), }, ))
            .build())
        .setup(|app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AuthStateManager(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            login,
            handle_redirect_uri,
            get_auth_data,
            logout,
            refresh_token,
            app::install_waystation_mcp,
            app::check_claude_installed,
            app::restart_claude_app,
            app::check_onboarding_completed
        ])
        .setup(|_app| {
            // No custom setup needed, deep link handling is done in the frontend
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
