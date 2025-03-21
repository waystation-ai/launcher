use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Run the default Tauri build process
    tauri_build::build();

    // Copy Info.plist for macOS builds to register the custom URL scheme
    if cfg!(target_os = "macos") {
        println!("cargo:warning=Building for macOS, copying Info.plist...");

        // Get the output directory from Cargo
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);

        // Navigate up to find the target directory
        let target_dir = out_path
            .ancestors()
            .find(|p| p.file_name().map_or(false, |name| name == "target"))
            .unwrap_or_else(|| Path::new("target"));

        // Create the path where Info.plist should be copied
        let info_plist_dest = target_dir.join("Info.plist");

        // Copy the Info.plist file
        match fs::copy("Info.plist", &info_plist_dest) {
            Ok(_) => println!(
                "cargo:warning=Successfully copied Info.plist to {:?}",
                info_plist_dest
            ),
            Err(e) => println!("cargo:warning=Failed to copy Info.plist: {}", e),
        }
    }
}
