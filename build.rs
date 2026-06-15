fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("FileDescription", "Oxide Shell");
        res.set("ProductName", "Oxide Shell");
        res.compile().expect("Failed to compile Windows resources");
    }
}
