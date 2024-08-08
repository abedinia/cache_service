use actix_web::{web, App, HttpServer};
use cache_service::{cache, handlers, routes};
use dotenv::dotenv;
use log::info;
use once_cell::sync::Lazy;
use prometheus::Registry;
use reqwest;
use std::sync::Arc;
use std::time::Duration;
use tokio::spawn;
use utoipa::OpenApi;

static INIT: Lazy<()> = Lazy::new(|| {
    dotenv().ok();
    env_logger::init();
});

async fn start_test_server() -> std::io::Result<()> {
    Lazy::force(&INIT);

    let cache = cache::initialize_cache().await;
    let cache_clone = Arc::clone(&cache);

    spawn(async move {
        cache_clone.invalidate_expired(Duration::from_secs(1)).await;
    });

    let registry = Registry::new();
    handlers::cache_handlers::init_metrics(&registry);

    info!("Starting HTTP server");

    let api_doc = routes::ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::clone(&cache)))
            .app_data(web::Data::new(registry.clone()))
            .configure(routes::init)
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", api_doc.clone()),
            )
    })
    .workers(num_cpus::get() * 2)
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde_json::json;

    #[actix_rt::test]
    async fn test_index_ok() {
        let server = actix_rt::spawn(start_test_server());

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        let client = reqwest::Client::new();
        let res = client.get("http://127.0.0.1:8080/").send().await.unwrap();
        assert!(res.status().is_success());

        server.abort();
    }

    #[actix_rt::test]
    async fn test_swagger_ui() {
        let server = actix_rt::spawn(start_test_server());

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        let client = reqwest::Client::new();
        let res = client
            .get("http://127.0.0.1:8080/swagger-ui/index.html")
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        server.abort();
    }

    #[actix_rt::test]
    async fn test_set_cache() {
        let server = actix_rt::spawn(start_test_server());

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        let client = reqwest::Client::new();
        let res = client
            .post("http://127.0.0.1:8080/cache")
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "key": "aydin",
                    "data": "exampleData",
                    "ttl": 1
                })
                .to_string(),
            )
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        server.abort();
    }

    #[actix_rt::test]
    async fn test_get_cache() {
        let server = actix_rt::spawn(start_test_server());

        actix_rt::time::sleep(Duration::from_secs(2)).await;

        let client = reqwest::Client::new();

        let response = client
            .post("http://127.0.0.1:8080/cache")
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "key": "aydin",
                    "data": "exampleData",
                    "ttl": 10
                })
                .to_string(),
            )
            .send()
            .await
            .expect("Failed to set cache");

        assert!(
            response.status().is_success(),
            "Failed to set cache: {:?}",
            response
        );

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        let res = client
            .get("http://127.0.0.1:8080/cache/aydin")
            .send()
            .await
            .expect("Failed to get cache");

        assert!(
            res.status().is_success(),
            "Cache retrieval was not successful: {}",
            res.status()
        );

        let body = res.text().await.expect("Failed to read response body");
        assert_eq!(body, "exampleData", "Unexpected cache value: {}", body);

        server.abort();
    }

    #[actix_rt::test]
    async fn test_delete_cache() {
        let server = actix_rt::spawn(start_test_server());

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        let client = reqwest::Client::new();

        client
            .post("http://127.0.0.1:8080/cache")
            .header("Content-Type", "application/json")
            .body(
                json!({
                    "key": "aydin",
                    "data": "exampleData",
                    "ttl": 1
                })
                .to_string(),
            )
            .send()
            .await
            .unwrap();

        let res = client
            .delete("http://127.0.0.1:8080/cache/aydin")
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());

        let get_res = client
            .get("http://127.0.0.1:8080/cache/aydin")
            .send()
            .await
            .unwrap();
        assert!(get_res.status().is_client_error());

        server.abort();
    }

    #[actix_rt::test]
    async fn test_metrics() {
        let server = actix_rt::spawn(start_test_server());

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        let client = reqwest::Client::new();
        let res = client
            .get("http://127.0.0.1:8080/metrics")
            .send()
            .await
            .unwrap();
        assert!(res.status().is_success());
        let body = res.text().await.unwrap();

        println!("Metrics response body: {}", body);

        assert!(body.contains("reads"));
        assert!(body.contains("requests"));
        assert!(body.contains("writes"));

        server.abort();
    }
}
