use axum::routing::get;
use axum::Router;
use env_logger::Env;
use log::info;
use malloc_test::api::routes::dump_heap_stats;
use malloc_test::rocksdb;
use std::net::ToSocketAddrs;
use tikv_jemallocator as jemallocator;
use tokio::runtime::Builder;
use tokio::{signal, spawn};

#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() -> std::io::Result<()> {
    let runtime = Builder::new_multi_thread()
        .thread_name("malloc_test")
        .enable_all()
        .build()?;

    runtime.block_on(async {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

        let _kv = match rocksdb::DB::init("/tmp/kv_store") {
            Ok(kv) => kv,
            Err(err) => panic!("failed to initialize KV store: {:#}", err),
        };

        let metrics_server = spawn(async move {
            let metrics_app = Router::new().route("/pprof/heap_stats", get(dump_heap_stats));

            let metric_addr = "127.0.0.1:8085"
                .to_socket_addrs()
                .unwrap_or_else(|err| panic!("failed to resolve metrics address: {}", err))
                .next()
                .expect("failed to get metrics socket address");

            info!("Starting metrics server at http://127.0.0.1:8085");

            axum::Server::bind(&metric_addr)
                .serve(metrics_app.into_make_service())
                .with_graceful_shutdown(shutdown_signal())
                .await
                .unwrap_or_else(|err| panic!("failed to start metrics server: {}", err));

            info!("metrics server is shutdown");
        });

        let (_metrics_server,) = tokio::join!(metrics_server,);

        info!("service is shutdown");

        Ok(())
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    let quit = async {
        signal::unix::signal(signal::unix::SignalKind::quit())
            .expect("failed to install SIGQUIT signal handler")
            .recv()
            .await;
    };

    if cfg!(unix) {
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
            _ = quit => {},
        }
    } else {
        ctrl_c.await;
    }

    info!("signal received, starting graceful shutdown");
}
