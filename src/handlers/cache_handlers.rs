use crate::cache::Cache;
use actix_web::{web, HttpResponse, Responder};
use prometheus::{Counter, Opts};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CacheItem {
    key: String,
    data: String,
    ttl: u64,
}

lazy_static::lazy_static! {
    static ref REQUEST_COUNTER: Counter = Counter::with_opts(Opts::new("requests", "Number of requests")).unwrap();
    static ref WRITE_COUNTER: Counter = Counter::with_opts(Opts::new("writes", "Number of write requests")).unwrap();
    static ref READ_COUNTER: Counter = Counter::with_opts(Opts::new("reads", "Number of read requests")).unwrap();
}

pub fn init_metrics(registry: &prometheus::Registry) {
    registry
        .register(Box::new(REQUEST_COUNTER.clone()))
        .unwrap();
    registry.register(Box::new(WRITE_COUNTER.clone())).unwrap();
    registry.register(Box::new(READ_COUNTER.clone())).unwrap();
}

#[utoipa::path(
    post,
    path = "/cache",
    request_body = CacheItem,
    responses(
    (status = 200, description = "Cache item created"),
    (status = 500, description = "Internal server error")
    )
)]
pub async fn create_item(
    cache: web::Data<Arc<dyn Cache<String>>>,
    item: web::Json<CacheItem>,
) -> impl Responder {
    REQUEST_COUNTER.inc();
    WRITE_COUNTER.inc();
    match cache
        .insert_item(item.key.clone(), item.data.clone(), item.ttl)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Failed to insert item: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[utoipa::path(
    get,
    path = "/cache/{key}",
    responses(
        (status = 200, description = "Cache item retrieved"),
        (status = 404, description = "Cache item not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn retrieve_item(
    cache: web::Data<Arc<dyn Cache<String>>>,
    key: web::Path<String>,
) -> impl Responder {
    REQUEST_COUNTER.inc();
    READ_COUNTER.inc();
    match cache.retrieve_item(&key.into_inner()).await {
        Some(data) => HttpResponse::Ok().body(data),
        None => HttpResponse::NotFound().finish(),
    }
}

#[utoipa::path(
    delete,
    path = "/cache/{key}",
    responses(
        (status = 200, description = "Cache item deleted"),
        (status = 404, description = "Cache item not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn remove_item(
    cache: web::Data<Arc<dyn Cache<String>>>,
    key: web::Path<String>,
) -> impl Responder {
    REQUEST_COUNTER.inc();
    WRITE_COUNTER.inc();
    match cache.remove_item(&key.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            log::error!("Failed to remove item: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
