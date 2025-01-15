use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::console::Target;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

const LOG_PATTERN: &str = "{d} |{l}|: {m}{n}";

pub fn setup_logger() {
    debug_config();
    // if cfg!(debug_assertions) {
    //     debug_config();
    // } else {
    //     prod_config();
    // }
}

fn prod_config() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::default()))
        .build("chat-server.log")
        .unwrap();

    let file_cfg = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(file_cfg).unwrap();
    log::set_max_level(LevelFilter::Info);
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
