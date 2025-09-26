use axum::{Json, response::IntoResponse};
use axum::{Router, http::header, routing::get};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_exporter_prometheus::PrometheusHandle;
use serde::Serialize;

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

pub fn build_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("metrics recorder")
}

/// 返回一个 Router，暴露 `/` 路径给 Prometheus 抓取
pub fn routes<S>(metrics: PrometheusHandle) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/healthz", get(|| async { Json(Health { status: "ok" }) }))
        .route(
            "/metrics",
            get({
                let h = metrics.clone();
                move || {
                    let h = h.clone();
                    async move {
                        // Prometheus 抓取需要 text/plain; version=0.0.4
                        let body = h.render(); // String: 实现了 IntoResponse
                        (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            body,
                        )
                            .into_response()
                        // 若不需要显式 header，也可： body.into_response()
                    }
                }
            }),
        )
}
