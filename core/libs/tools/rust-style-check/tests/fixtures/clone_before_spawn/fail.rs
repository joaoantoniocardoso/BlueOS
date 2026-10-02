use std::sync::Arc;

pub async fn example(session: Arc<String>) {
    let session = Arc::clone(&session);
    tokio::spawn(async move {
        let _owned = session;
    });
}
