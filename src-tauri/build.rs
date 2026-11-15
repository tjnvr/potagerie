#[path = "src/command_names.rs"]
mod command_names;

fn main() {
    // App commands must be allowed by a capability, like plugin commands (S05 §3).
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(command_names::COMMANDS)),
    )
    .expect("failed to run tauri-build");
}
