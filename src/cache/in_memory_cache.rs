use super::schema::Cache;
use async_trait::async_trait;
use std::collections::HashMap;
use std::io;
use tokio::sync::RwLock;
use tokio::time::{self, Duration, Instant};

pub struct InMemoryCache<T> {
    store: RwLock<HashMap<String, (T, Instant)>>,
}

impl<T> Default for InMemoryCache<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> InMemoryCache<T> {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Cache<T> for InMemoryCache<T> {
    async fn insert_item(&self, key: String, value: T, ttl: u64) -> io::Result<()> {
        let mut store = self.store.write().await;
        store.insert(key, (value, Instant::now() + Duration::from_secs(ttl)));
        Ok(())
    }

    async fn retrieve_item(&self, key: &str) -> Option<T> {
        let store = self.store.read().await;
        if let Some((value, expiry)) = store.get(key) {
            if Instant::now() < *expiry {
                return Some(value.clone());
            }
        }
        None
    }

    async fn remove_item(&self, key: &str) -> io::Result<()> {
        let mut store = self.store.write().await;
        store.remove(key);
        Ok(())
    }

    async fn invalidate_expired(&self, interval: Duration) {
        loop {
            time::sleep(interval).await;
            let mut store = self.store.write().await;
            let now = Instant::now();
            store.retain(|_, (_, expiry)| *expiry > now);
        }
    }
}
