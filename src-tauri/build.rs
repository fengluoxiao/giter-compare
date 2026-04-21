fn main() {
    // 检查目标平台，支持交叉编译
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    
    if target_os == "macos" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let objc_dir = format!("{}/src/swift/Sources", manifest_dir);
        
        cc::Build::new()
            .file(format!("{}/SwiftUIPanel.m", objc_dir))
            .file(format!("{}/NativeToolbar.m", objc_dir))
            .flag("-fobjc-arc")
            .flag("-framework")
            .flag("AppKit")
            .compile("native_ui");
    }
    
    tauri_build::build()
}
