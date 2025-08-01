fn main() {
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/icedm.ico")
        .compile()
        .unwrap();
}
