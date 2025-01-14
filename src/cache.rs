use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Socket = String;
pub type UserID = String;

pub struct SharedClientCache {
    pub clients: HashMap<Socket, UserID>,
}

impl SharedClientCache {
    pub fn new_cache() -> Arc<Mutex<SharedClientCache>> {
        Arc::new(Mutex::new(SharedClientCache {
            clients: HashMap::new(),
        }))
    }
}

pub struct ChatHistory {
    pub history: VecDeque<String>,
}

impl ChatHistory {
    pub fn insert(&mut self, msg: String) {
        self.history.push_back(msg);
    }
    pub fn new_chat_history() -> Arc<Mutex<ChatHistory>> {
        Arc::new(Mutex::new(ChatHistory {
            history: VecDeque::new(),
        }))
    }
}
