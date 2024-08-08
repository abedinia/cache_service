pub mod in_memory_cache;
pub mod redis_cache;
pub mod schema;

pub use in_memory_cache::InMemoryCache;
pub use redis_cache::RedisCache;
pub use schema::Cache;

use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::env;
use std::sync::Arc;

pub async fn initialize_cache() -> Arc<dyn Cache<String>> {
    let cache_backend = env::var("CACHE_BACKEND").unwrap_or_else(|_| "in_memory".to_string());
    match cache_backend.as_str() {
        "redis" => {
            let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
            let manager = RedisConnectionManager::new(redis_url).expect("Invalid Redis URL");
            let pool = Pool::builder()
                .build(manager)
                .await
                .expect("Failed to create Redis pool");
            Arc::new(RedisCache::new(pool))
        }
        _ => Arc::new(InMemoryCache::new()),
    }
}
