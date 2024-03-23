use std::fs::File;
use std::io::Read;
use std::path::Path;
use once_cell::sync::Lazy;
use async_lock::RwLock;
use serde::Deserialize;


pub static CONF: Lazy<RwLock<Conf>> = Lazy::new(|| {
    RwLock::new(Conf::new())
});


#[derive(Debug, Deserialize)]
pub struct ServerConf {
    pub name: String,
    pub hostnames: Vec<String>,
    pub socket_path: String,
    pub static_dir: String,
    pub dev_static_dir: String,
    pub index_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub ip: String,
    pub https_port: u16,
    pub http_port: u16,
    pub acme_challenge_dir: String,
    pub acme_challenge_url: String,
    pub index_url: String,
    pub tmp_dir: String,
    pub servers: Vec<ServerConf>,
	pub max_request_size_mb: usize,
}

impl Conf {
    pub fn new() -> Self {
        let path = Path::new("miarh.toml");
        let mut file = File::open(&path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let conf: Conf = toml::from_str(&contents).unwrap();
        return conf;
    }
}
