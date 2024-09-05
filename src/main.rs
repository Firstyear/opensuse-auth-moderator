use anyhow::{Context, Result};
use axum::extract::DefaultBodyLimit;
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tracing::*;

mod configuration;

async fn status_view() -> Html<&'static str> {
    Html(r#"Ok"#)
}

async fn index_view() -> Html<&'static str> {
    Html(r#"Ok"#)
}

struct AppState {
    oidc: (),
}

#[derive(Debug, clap::Parser)]
#[clap(about = "OpenSUSE Auth Moderator")]
struct Opt {
    config: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);

    // let console_layer = ConsoleLayer::builder().with_default_env().spawn();

    // Parse CLI
    let opt = Opt::parse();

    let config = configuration::parse(&opt.config)?;

    Registry::default()
        // .with(console_layer)
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_chain_file(
        &config.tls_pem_chain,
        &config.tls_pem_key,
    )
    .await
    .context("Unable to setup rustls certificate and/or key")?;

    let app_state = Arc::new(AppState { oidc: () });

    let tls_addr = config.tls_bind_addr.clone();

    let app = Router::new()
        .route("/", get(index_view))
        .route("/_status", get(status_view))
        // TODO Double check this.
        .layer(DefaultBodyLimit::disable())
        // .layer(session_layer)
        .layer(CompressionLayer::new());

    info!("listening on {}", tls_addr);

    axum_server::bind_rustls(tls_addr, tls_config)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
