use axum::http::StatusCode;
use axum::response::IntoResponse;
use hyper::header;
use tikv_jemalloc_ctl as jemalloc_ctl;

pub async fn dump_heap_stats() -> impl IntoResponse {
    let mut buf = Vec::new();
    jemalloc_ctl::stats_print::stats_print(&mut buf, jemalloc_ctl::stats_print::Options::default())
        .unwrap();
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        buf,
    )
}
