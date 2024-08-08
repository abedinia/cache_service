use super::schema::Cache;
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use log::info;
use redis::AsyncCommands;
use std::io;
use tokio::time::Duration;

pub struct RedisCache {
    pool: Pool<RedisConnectionManager>,
}

impl RedisCache {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<T> Cache<T> for RedisCache
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    async fn insert_item(&self, key: String, value: T, ttl: u64) -> io::Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        let value = serde_json::to_string(&value)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        conn.set_ex(key, value, ttl as usize as u64)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }

    async fn retrieve_item(&self, key: &str) -> Option<T>
    where
        T: Clone,
    {
        let mut conn = self.pool.get().await.ok()?;
        let value: Option<String> = conn.get(key).await.ok()?;
        value.and_then(|v| serde_json::from_str(&v).ok())
    }

    async fn remove_item(&self, key: &str) -> io::Result<()> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        conn.del(key)
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }

    async fn invalidate_expired(&self, _interval: Duration) {
        info!("Redis handles expiration internally, no need to manually invalidate")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bb8::Pool;
    use bb8_redis::RedisConnectionManager;
    use dotenv::dotenv;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::io;
    use tokio;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    struct TestData {
        value: String,
    }

    async fn get_redis_pool() -> Pool<RedisConnectionManager> {
        dotenv().ok();
        let redis_url = env::var("TEST_REDIS_URL").expect("REDIS_URL must be set");
        let manager = RedisConnectionManager::new(redis_url).unwrap();
        Pool::builder().build(manager).await.unwrap()
    }

    #[tokio::test]
    async fn test_insert_item() -> io::Result<()> {
        let pool = get_redis_pool().await;
        let cache = RedisCache::new(pool);
        let key = "test_key".to_string();
        let value = TestData {
            value: "test_value".to_string(),
        };
        let ttl = 10;

        cache.insert_item(key.clone(), value.clone(), ttl).await?;

        let retrieved_value: TestData = cache.retrieve_item(&key).await.unwrap();

        assert_eq!(value, retrieved_value);
        Ok(())
    }

    #[tokio::test]
    async fn test_retrieve_item() {
        let pool = get_redis_pool().await;
        let cache = RedisCache::new(pool);
        let key = "test_key".to_string();
        let value = TestData {
            value: "test_value".to_string(),
        };

        let mut conn = cache.pool.get().await.unwrap();
        let _: () = conn
            .set(&key, serde_json::to_string(&value).unwrap())
            .await
            .unwrap();

        let retrieved_value: TestData = cache.retrieve_item(&key).await.unwrap();

        assert_eq!(value, retrieved_value);
    }
}
