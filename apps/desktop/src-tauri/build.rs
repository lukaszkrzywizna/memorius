fn main() {
    // The tray icon is embedded with `include_image!`, which cargo does not
    // track on its own.
    println!("cargo:rerun-if-changed=icons/tray.png");
    tauri_build::build();
}
