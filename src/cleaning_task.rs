use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_schedule::{every, Job};

pub fn clean(to_clean: Arc<Mutex<Vec<String>>>, interval: u32) {
    tokio::task::spawn(async move {
        every(interval)
            .second()
            .perform(|| async {
                println!("task run");
                to_clean.lock().await.clear();
            }).await;
    });
}
