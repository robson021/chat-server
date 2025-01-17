#[derive(PartialEq)]
pub enum Profile {
    Dev,
    Release,
}

pub struct Config {
    pub profile: Profile,
    pub password: Option<String>,
    pub log_file: Option<String>,
    pub host: String,
}

pub fn get_config() -> Config {
    if cfg!(debug_assertions) {
        Config {
            profile: Profile::Dev,
            password: None,
            log_file: None,
            host: "localhost:8080".to_owned(),
        }
    } else {
        let args: Vec<String> = std::env::args().collect();
        let password = args[1].trim().to_owned();
        Config {
            profile: Profile::Release,
            password: Some(password),
            host: "0.0.0.0:8080".to_owned(),
            log_file: Some("./log/chat-server.log".to_owned()),
        }
    }
}
