#[derive(PartialEq)]
pub enum Profile {
    Dev,
    Release,
}

pub fn get_active_profile() -> Profile {
    if cfg!(debug_assertions) {
        Profile::Dev
    } else {
        Profile::Release
    }
}

pub fn get_log_file() -> &'static str {
    "./log/chat-server.log"
}
