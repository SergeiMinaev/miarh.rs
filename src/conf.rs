use std::fs::File;
use std::io::Read;
use std::path::Path;
use lazy_static::lazy_static;
use async_lock::RwLock;
use serde::Deserialize;


lazy_static! {
    pub static ref CONF: RwLock<Conf> = RwLock::new(Conf::new());
}


#[derive(Debug, Deserialize)]
pub struct ServerConf {
    pub name: String,
    pub hostnames: Vec<String>,
    pub socket_path: String,
    pub static_dir: String,
    pub index_path: String,
}

#[derive(Debug, Deserialize)]
pub struct Conf {
    pub https_host_port: String,
    pub http_host_port: String,
    pub acme_challenge_dir: String,
    pub acme_challenge_url: String,
    pub servers: Vec<ServerConf>,
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
