fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("jianxin.ico");
        res.compile().unwrap();
    }
}
