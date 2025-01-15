use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::console::Target;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

const LOG_PATTERN: &str = "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}";

pub fn setup_logger() {
    if cfg!(debug_assertions) {
        debug_config();
    } else {
        log4rs::init_file("./resources/logger_cfg.yml", Default::default()).unwrap();
    }
}

fn debug_config() {
    let stderr = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .target(Target::Stderr)
        .build();

    let error_trace_level = Root::builder().appender("stderr").build(LevelFilter::Trace);

    let console_cfg = Config::builder()
        .appender(Appender::builder().build("stderr", Box::new(stderr)))
        .build(error_trace_level)
        .unwrap();

    log4rs::init_config(console_cfg).unwrap();
    log::set_max_level(LevelFilter::Debug);
}
