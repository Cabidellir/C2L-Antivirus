fn main() {
    println!("cargo:rerun-if-changed=assets/c2l-shield.ico");
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/c2l-shield.ico")
        .compile()
        .expect("Windows icon resource");
}
