use log::{info, warn};
use rev_buf_reader::RevBufReader;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::BufRead;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Socket = String;
pub type UserID = String;

pub struct SharedClientCache {
    clients: HashMap<Socket, UserID>,
}

impl SharedClientCache {
    pub fn new_cache() -> Arc<Mutex<SharedClientCache>> {
        Arc::new(Mutex::new(SharedClientCache {
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
    pub fn empty_chat_history() -> Arc<Mutex<ChatHistory>> {
        Arc::new(Mutex::new(ChatHistory {
            history: VecDeque::new(),
        }))
    }

    pub fn from_local_log_file(file: &str) -> Arc<Mutex<ChatHistory>> {
        let file = File::open(file);
        if file.is_err() {
            warn!("Could not open log file.",);
            return Self::empty_chat_history();
        }
        let file = file.unwrap();
        let buf = RevBufReader::new(file);

        let lines: Vec<String> = buf
            .lines()
            .take(TO_DRAIN)
            .map(|l| l.unwrap_or("".to_owned()))
            .filter(|l| l.contains("INFO: Message: ["))
            .collect();

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

        Arc::new(Mutex::new(ChatHistory {
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
            .lock()
            .await
            .history
            .clone();

        assert_eq!(chat.len(), 9);

        for msg in chat {
            assert_eq!(msg, "[user]: example test message\r\n");
        }
    }
}
