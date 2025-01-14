// use crate::cache::ChatHistory;
// use std::collections::VecDeque;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tokio_schedule::{every, Job};
//
// const TO_DRAIN: usize = 50;
//
// pub async fn clean(history: &mut ChatHistory, interval_minutes: u32) {
//     tokio::task::spawn(async move {
//         every(interval_minutes)
//             .second()
//             .perform(|| async {
//                 println!("Cleaning task running...");
//                 drain_old_messages(history.history.clone()).await;
//             })
//             .await;
//     });
// }
//
// async fn drain_old_messages(history: Arc<Mutex<VecDeque<String>>>) {
//     let mut guard = history.lock().await;
//     if guard.len() > TO_DRAIN {
//         guard.drain(..TO_DRAIN);
//     }
// }
