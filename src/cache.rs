use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Socket = String;
pub type UserID = String;

pub struct SharedClientCache {
    pub clients: HashMap<Socket, UserID>,
}

pub fn new_cache() -> Arc<Mutex<SharedClientCache>> {
    Arc::new(Mutex::new(SharedClientCache {
        clients: HashMap::new(),
    }))
}

pub struct ChatHistory {
    history: Arc<Mutex<VecDeque<String>>>,
}
impl ChatHistory {
    pub async fn insert(&mut self, msg: String) {
        self.history.lock().await.push_back(msg);
    }
    pub async fn drain_front(&mut self, to_drain: usize) {
        if self.history.lock().await.len() <= to_drain {
            return;
        }
        self.history.lock().await.drain(..to_drain);
    }
}
