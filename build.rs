fn main() {
    #[cfg(windows)]
    winresource::WindowsResource::new()
        .set_icon("assets/meowsn.ico")
        .compile()
        .unwrap();
}
