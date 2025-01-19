const LOG_FILE_PATH: &str = "./resources/logger_cfg.yml";
const LOG_PATTERN: &str = "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}";

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

impl Config {
    pub fn setup_logger(&self) {
        match self.profile {
            Profile::Release => log4rs::init_file(LOG_FILE_PATH, Default::default()).unwrap(),
            Profile::Dev => log4rs::init_file(LOG_FILE_PATH, Default::default()).unwrap(),
        };
    }
}

pub fn get_config() -> Config {
    Config {
        profile: Profile::Dev,
        password: None,
        host: "localhost:8081".to_owned(),
        log_file: Some("./log/chat-server.log".to_owned()),
    }
}
