use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_schedule::{every, Job};

pub fn clean(to_clean: Arc<Mutex<VecDeque<String>>>, interval_minutes: u32) {
    tokio::task::spawn(async move {
        let to_drain = 50;

        every(interval_minutes)
            .second()
            .perform(|| async {
                println!("task run");
                let mut q = to_clean.lock().await;
                if q.len() > to_drain {
                    q.drain(..to_drain);
                }
            })
            .await;
    });
}
