fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("zani.ico");
        res.compile().unwrap();
    }
}