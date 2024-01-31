mod routes;

use std::sync::Arc;

use anyhow::Context;
use tracing_subscriber::util::SubscriberInitExt;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_url = "push_subscriptions.db";
    let listen_port: u16 = 3000;

    init_logger().await;
    let db_pool = sqlx::SqlitePool::connect(db_url).await.context(format!("Connecting to DB at '{}'", db_url))?;
    setup_db(&db_pool).await.context("Setup DB")?;
    tracing::debug!("Connected to DB");
    let app_state = Arc::new(AppState { db_pool });

    let app = axum::Router::new()
        .route("/add_push_subscription", axum::routing::post(routes::add_push_subscription))
        .route("/show_subscriptions", axum::routing::get(routes::show_subscriptions))
        .with_state(app_state);

    let listen_addr: std::net::SocketAddr = (std::net::Ipv6Addr::UNSPECIFIED, listen_port).into();
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