fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico")
            .set("InternalName", "FAST-MIC.EXE")
            .set_version_info(winres::VersionInfo::PRODUCTVERSION, 0x0001000000000000);
        if let Err(e) = res.compile() {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
