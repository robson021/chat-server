pub fn get_host() -> &'static str {
    if cfg!(debug_assertions) {
        "localhost:8080"
    } else {
        "0.0.0.0:8080"
    }
}
