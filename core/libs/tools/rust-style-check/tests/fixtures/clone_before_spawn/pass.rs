use std::sync::Arc;

pub async fn example(session: Arc<String>) {
    tokio::spawn({
        let session = Arc::clone(&session);
        async move {
            let _owned = session;
        }
    });
}
