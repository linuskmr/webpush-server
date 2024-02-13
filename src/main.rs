mod routes;
mod cli;

use std::sync::Arc;
use clap::Parser;
use anyhow::Context;
use axum::handler::Handler;
use tracing_subscriber::util::SubscriberInitExt;


const ENV_AUTH_BEARER_TOKEN: &str = "WEBPUSH_AUTH_BEARER_TOKEN";


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger().await;

    let cli_args = cli::Args::parse();

    let db_pool = sqlx::SqlitePool::connect(&cli_args.db).await.context(format!("Connecting to DB at '{}'", cli_args.db))?;
    setup_db(&db_pool).await.context("Setup DB")?;
    tracing::debug!("Connected to DB");
    let app_state = Arc::new(AppState { db_pool });

    // The authorization created by this function is used to protect routes that shouldn't be public like sending a push.
    // Therefore, clients need to provide the token specified in an environment variable in the Authorization header to get access.
    let authorization_layer = {
        let authorization_bearer_token = std::env::var(ENV_AUTH_BEARER_TOKEN).context(format!("Environment variable {} not set", ENV_AUTH_BEARER_TOKEN))?;
        tower_http::validate_request::ValidateRequestHeaderLayer::bearer(&authorization_bearer_token)
    };

    let app = axum::Router::new()
        .route(
            "/subscriptions",
            axum::routing::MethodRouter::new()
                .get(routes::get_subscriptions.layer(authorization_layer.clone()))
                .post(routes::add_subscription)
        )
        .route(
            "/push",
            axum::routing::post(routes::send_push.layer(authorization_layer.clone()))
        )
        // Set CORS header to allow the JS to subscribe to the push service by `fetch()`ing the subscribe route
        .layer(tower_http::cors::CorsLayer::new()
            // allow `GET` and `POST` when accessing the resource
            .allow_methods([http::Method::GET, http::Method::POST])
            // allow header `Content-Type: application/json`
            .allow_headers([http::header::CONTENT_TYPE])
            // allow requests from any origin
            .allow_origin(tower_http::cors::Any)
        )
        .with_state(app_state);

    let listen_addr: std::net::SocketAddr = (std::net::Ipv6Addr::UNSPECIFIED, cli_args.port).into();
    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
    tracing::info!(?listen_addr, "HTTP server listening");
    axum::serve(listener, app).await?;
    Ok(())
}


struct AppState {
    db_pool: sqlx::SqlitePool,
}

/// The PushSubscription interface of the Push API provides a subscription's URL endpoint and credentials.
/// 
/// See <https://developer.mozilla.org/en-US/docs/Web/API/PushSubscription>
#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct PushSubscription {
    endpoint: String,
    /// Web Push authentication secret
    auth: String,
    /// A [jwt_simple::algorithms::ES256KeyPair]
    p256dh: String,
}


/// Creates necessary tables if they don't exist.
async fn setup_db(db: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let query = "CREATE TABLE IF NOT EXISTS push_subscriptions (
        id INTEGER PRIMARY KEY,
        endpoint TEXT NOT NULL,
        auth TEXT NOT NULL,
        p256dh TEXT NOT NULL
    )";
    sqlx::query(query).execute(db).await.context("Error creating table 'push_subscriptions' if not exists")?;
    Ok(())
}

/// Configures the logger to use the default environment variable (`RUST_LOG`),
/// or the info level for webpush_server if it's not present.
async fn init_logger() {
    use tracing_subscriber::layer::SubscriberExt;

    tracing_subscriber::Registry::default()
		.with(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or(tracing_subscriber::EnvFilter::new("webpush_server=info")),
		)
        .with(tracing_subscriber::fmt::layer())
        .init();
}