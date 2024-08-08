use crate::handlers::{cache_handlers, metrics_handlers};
use actix_web::{web, HttpResponse};
use utoipa::OpenApi;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/")
            .route(web::get().to(|| async { HttpResponse::Ok().body("..: Cache Service") })),
    )
    .service(web::resource("/cache").route(web::post().to(cache_handlers::create_item)))
    .service(
        web::resource("/cache/{key}")
            .route(web::get().to(cache_handlers::retrieve_item))
            .route(web::delete().to(cache_handlers::remove_item)),
    )
    .service(web::resource("/metrics").route(web::get().to(metrics_handlers::metrics)));
}

#[derive(OpenApi)]
#[openapi(
    paths(
        cache_handlers::create_item,
        cache_handlers::retrieve_item,
        cache_handlers::remove_item,
        metrics_handlers::metrics
    ),
    components(schemas(cache_handlers::CacheItem))
)]
pub struct ApiDoc;
