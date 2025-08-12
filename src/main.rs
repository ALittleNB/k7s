use std::net::SocketAddr;

use tracing::info;

mod routes;
mod settings;
mod schema;
mod utils;

use routes::create_router;
use settings::SETTINGS;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .event_format(utils::formatter::JsonFormatter)
        .init();

    let app = create_router();

    let addr: SocketAddr = format!("{}:{}", SETTINGS.server.host, SETTINGS.server.port)
        .parse()
        .expect("invalid server host/port");
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
