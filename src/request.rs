use crate::conf::CONF;
use serde::Serialize;


#[derive(Serialize)]
pub struct Request {
    pub method: String,
    pub host: String,
    pub path: String,
}

impl Request {
    pub async fn app_socket_path(&self) -> Option<String> {
        let conf = CONF.read().await;
        for srv in &conf.servers {
            for hostname in &srv.hostnames {
                if hostname == &self.host {
                    return Some(srv.socket_path.to_string().clone());
                }
            }
        }
        return None;
    }
}
