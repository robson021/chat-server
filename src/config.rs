use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;

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
            Profile::Dev => {
                let stderr = ConsoleAppender::builder()
                    .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
                    .target(Target::Stderr)
                    .build();

                let error_trace_level =
                    Root::builder().appender("stderr").build(LevelFilter::Trace);

                let console_cfg = log4rs::Config::builder()
                    .appender(Appender::builder().build("stderr", Box::new(stderr)))
                    .build(error_trace_level)
                    .unwrap();

                log4rs::init_config(console_cfg).unwrap();
                log::set_max_level(LevelFilter::Debug);
            }
        };
    }
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
        if args.len() < 2 {
            panic!("No password specified");
        }
        let password = args[1].trim().to_owned();
        if password.len() < 5 {
            panic!("Password must be at least 5 characters");
        }
        Config {
            profile: Profile::Release,
            password: Some(password),
            host: "0.0.0.0:8080".to_owned(),
            log_file: Some("./log/chat-server.log".to_owned()),
        }
    }
}
