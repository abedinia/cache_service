mod cache;
mod handlers;
mod routes;

use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use log::error;
use once_cell::sync::Lazy;
use prometheus::Registry;
use std::sync::Arc;
use std::time::Duration;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

static INIT: Lazy<()> = Lazy::new(|| {
    dotenv().ok();
    env_logger::init();
});

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Lazy::force(&INIT);

    let cache = cache::initialize_cache().await;
    let cache_clone = Arc::clone(&cache);

    tokio::spawn(async move {
        loop {
            cache_clone.invalidate_expired(Duration::from_secs(1)).await;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    let registry = Registry::new();
    handlers::cache_handlers::init_metrics(&registry);

    let api_doc = routes::ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&cache)))
            .app_data(web::Data::new(registry.clone()))
            .configure(routes::init)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", api_doc.clone()),
            )
    })
    .workers(num_cpus::get() * 2)
    .bind("0.0.0.0:8080")?
    .run()
    .await
    .map_err(|e| {
        error!("Failed to start server: {}", e);
        e
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache;
    use actix_web::test;
    use prometheus::Registry;
    use std::sync::Arc;

    #[actix_rt::test]
    async fn test_index_ok() {
        Lazy::force(&INIT);

        let cache = cache::initialize_cache().await;
        let registry = Registry::new();

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::clone(&cache)))
                .app_data(web::Data::new(registry.clone()))
                .configure(routes::init),
        )
        .await;

        let req = test::TestRequest::with_uri("/").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_swagger_ui() {
        Lazy::force(&INIT);

        let cache = cache::initialize_cache().await;
        let registry = Registry::new();
        let api_doc = routes::ApiDoc::openapi();

        let mut app = test::init_service(
            App::new()
                .app_data(web::Data::new(Arc::clone(&cache)))
                .app_data(web::Data::new(registry.clone()))
                .configure(routes::init)
                .service(
                    SwaggerUi::new("/swagger-ui/{_:.*}")
                        .url("/api-doc/openapi.json", api_doc.clone()),
                ),
        )
        .await;

        let req = test::TestRequest::with_uri("/swagger-ui/index.html").to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
