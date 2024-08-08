use async_trait::async_trait;
use std::io;
use tokio::time::Duration;

#[async_trait]
pub trait Cache<T>: Send + Sync {
    async fn insert_item(&self, key: String, value: T, ttl: u64) -> io::Result<()>;
    async fn retrieve_item(&self, key: &str) -> Option<T>
    where
        T: Clone;
    async fn remove_item(&self, key: &str) -> io::Result<()>;
    async fn invalidate_expired(&self, interval: Duration);
}
