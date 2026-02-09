#[derive(Clone)]
pub struct Config {
    pub index_url: String,
    pub zip_bases: Vec<String>,
}

#[derive(Clone)]
pub struct DownloadOptions {
    pub concurrency: usize,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self { concurrency: 4 }
    }
}

#[derive(Clone, Debug)]
pub struct ResourceItem {
    pub dest: String,
    pub md5: Option<String>,
}
