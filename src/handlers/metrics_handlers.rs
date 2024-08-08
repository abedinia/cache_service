use actix_web::{web, HttpResponse, Responder};
use prometheus::{Encoder, TextEncoder};

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn metrics(registry: web::Data<prometheus::Registry>) -> impl Responder {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    if encoder.encode(&metric_families, &mut buffer).is_err() {
        log::error!("Failed to encode metrics");
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer)
}
