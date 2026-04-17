fn main() {
    #[cfg(target_os = "macos")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let objc_dir = format!("{}/src/swift/Sources", manifest_dir);
        
        cc::Build::new()
            .file(format!("{}/SwiftUIPanel.m", objc_dir))
            .flag("-fobjc-arc")
            .flag("-framework")
            .flag("AppKit")
            .compile("native_panel");
    }
    
    tauri_build::build()
}
