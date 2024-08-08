use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CacheItem {
    pub key: String,
    pub data: String,
    pub ttl: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_cache_item_serialization() {
        let cache_item = CacheItem {
            key: "test_key".to_string(),
            data: "test_data".to_string(),
            ttl: 3600,
        };

        let serialized = serde_json::to_string(&cache_item).expect("Failed to serialize CacheItem");
        let deserialized: CacheItem = serde_json::from_str(&serialized).expect("Failed to deserialize CacheItem");

        assert_eq!(cache_item.key, deserialized.key);
        assert_eq!(cache_item.data, deserialized.data);
        assert_eq!(cache_item.ttl, deserialized.ttl);
    }

    #[test]
    fn test_cache_item_schema() {

        let schema = CacheItem::schema();


        let schema_json = serde_json::to_string(&schema).expect("Failed to serialize schema");
        println!("CacheItem Schema: {}", schema_json);


        assert!(schema_json.contains("\"key\""));
        assert!(schema_json.contains("\"data\""));
        assert!(schema_json.contains("\"ttl\""));
    }
}
