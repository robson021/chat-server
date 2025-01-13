use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Socket = String;
pub type UserID = String;

pub struct Shared {
    pub clients: HashMap<Socket, UserID>,
}

pub fn new_cache() -> Arc<Mutex<Shared>> {
    Arc::new(Mutex::new(Shared {
        clients: HashMap::new(),
    }))
}
