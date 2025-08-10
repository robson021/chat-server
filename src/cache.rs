use log::{info, warn};
use rev_buf_reader::RevBufReader;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufRead;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type Socket = String;
pub type UserID = String;

pub struct ClientCache {
    clients: HashMap<Socket, UserID>,
}

pub type SharedClientCache = Arc<RwLock<ClientCache>>;

impl ClientCache {
    pub fn new_cache() -> SharedClientCache {
        Arc::new(RwLock::new(ClientCache {
            clients: HashMap::new(),
        }))
    }
    pub fn insert(&mut self, socket: Socket, user_id: UserID) {
        self.clients.insert(socket, user_id);
    }

    pub fn get(&self, socket: &Socket) -> Option<UserID> {
        self.clients.get(socket).cloned()
    }

    pub fn remove(&mut self, socket: &Socket) -> Option<UserID> {
        self.clients.remove(socket)
    }
}

pub struct ChatHistory {
    pub history: VecDeque<String>,
}

pub type SharedChatHistory = Arc<RwLock<ChatHistory>>;

const TO_DRAIN: usize = 999;

impl ChatHistory {
    pub fn insert(&mut self, msg: String) {
        self.history.push_back(msg);
    }
    pub fn drain(&mut self) {
        if TO_DRAIN < self.history.len() {
            self.history.drain(..TO_DRAIN);
            info!(
                "Message queue drained. Current length: {}",
                self.history.len()
            );
        }
    }
    pub fn empty_chat_history() -> SharedChatHistory {
        Arc::new(RwLock::new(ChatHistory {
            history: VecDeque::new(),
        }))
    }

    pub fn from_local_log_file(file: &str) -> SharedChatHistory {
        let file = File::open(file);
        if file.is_err() {
            warn!("Could not open log file.",);
            return Self::empty_chat_history();
        }
        let file = file.unwrap();
        let buf = RevBufReader::new(file);

        let lines: Vec<String> = buf
            .lines()
            .map(|l| l.unwrap_or("".to_owned()))
            .filter(|l| l.contains("INFO: Message: |"))
            .take(TO_DRAIN)
            .collect();

        dbg!("Loaded chat lines (reversed): {:?}", &lines);

        let lines: Vec<String> = lines
            .iter()
            .map(|line| {
                let mut s = String::from(&line[37..]); // 37 is log prefix todo: should be calculated
                s.push_str("\r\n");
                s
            })
            .rev()
            .collect();

        info!("Loaded {} lines from the old log file.", lines.len());
        // dbg!("Lines: {:?}", &lines);

        Arc::new(RwLock::new(ChatHistory {
            history: VecDeque::from(lines),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_chat_from_log() {
        let chat = ChatHistory::from_local_log_file("./resources/test/chat-server.log")
            .read()
            .await
            .history
            .clone();

        assert_eq!(chat.len(), 11);

        for msg in chat {
            assert!(msg.ends_with("| [user]: Test message.\r\n"));
        }
    }
}
