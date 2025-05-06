use std::{error::Error, net::SocketAddr, path::PathBuf};

use axum::{extract::{ws::{Message, Utf8Bytes}, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use futures::{SinkExt, StreamExt};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new().route("/ping", get(ping_handler));
    let port = std::env::args()
        .find(|arg| arg.starts_with("-port="))
        .and_then(|arg| arg.trim_start_matches("-port=").parse().ok())
        .unwrap_or(7777);
    let cert = std::env::args()
        .find(|arg| arg.starts_with("-cert="))
        .map(|arg| arg.trim_start_matches("-cert=").to_string())
        .unwrap_or_else(|| "cert.pem".to_string());
    let key = std::env::args()
        .find(|arg| arg.starts_with("-key="))
        .map(|arg| arg.trim_start_matches("-key=").to_string())
        .unwrap_or_else(|| "key.pem".to_string());
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let service = axum_server::bind(address).serve(app.clone().into_make_service());

    match RustlsConfig::from_pem_file(PathBuf::from(&cert), PathBuf::from(&key)).await {
        Ok(config) => {
            tracing::info!("started secured server on {}", address);
            let (main, ws) = futures::join!(
                axum_server::bind_rustls(address, config).serve(app.into_make_service()),
                service
            );
            main?;
            ws?;
        }
        Err(e) => {
            tracing::warn!("ssl config not created {}", e);
            tracing::info!("started non-secured server on {}", address);
            let (main, ws) = futures::join!(
                axum_server::bind(address).serve(app.into_make_service()),
                service
            );
            main?;
            ws?;
        }
    }

    Ok(())
}

async fn ping_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        let (mut sender, mut receiver) = socket.split();

        while let Some(message) = receiver.next().await {
            if let Ok(_) = message {
                let _ = sender
                    .send(Message::Text(Utf8Bytes::from("Ok")))
                    .await;
            }
        }
    })
}
