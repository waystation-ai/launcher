use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

pub fn ensure_config_file(config_path: &PathBuf) -> Result<(), String> {
    if !config_path.exists() {
        let initial_config = json!({
            "mcpServers": {}
        });

        let config_str = serde_json::to_string_pretty(&initial_config)
            .map_err(|e| format!("Failed to create initial config: {}", e))?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        fs::write(config_path, config_str)
            .map_err(|e| format!("Failed to write initial config file: {}", e))?;
    }
    Ok(())
}

pub fn ensure_mcp_servers(config_json: &mut Value) -> Result<(), String> {
    if !config_json.is_object() {
        *config_json = json!({
            "mcpServers": {}
        });
    } else if !config_json
        .get("mcpServers")
        .map_or(false, |v| v.is_object())
    {
        config_json["mcpServers"] = json!({});
    }
    Ok(())
}
