use std::sync::Arc;

use sqlx::Row;
use crate::{AppState, PushSubscription};

pub async fn add_push_subscription(
    axum::extract::State(app_state): axum::extract::State<Arc<AppState>>,
    axum::extract::Json(push_subscription): axum::extract::Json<PushSubscription>,
) -> impl axum::response::IntoResponse {

    sqlx::query("INSERT INTO push_subscriptions (endpoint, auth, p256dh) VALUES (?, ?, ?)")
        .bind(push_subscription.endpoint)
        .bind(push_subscription.auth)
        .bind(push_subscription.p256dh)
        .execute(&app_state.db_pool).await.unwrap();
    "OK"
}

pub async fn show_subscriptions(
    axum::extract::State(app_state): axum::extract::State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {

    let push_subscriptions = sqlx::query("SELECT * FROM push_subscriptions")
        .map(|row: sqlx::sqlite::SqliteRow| {
            PushSubscription {
                endpoint: row.get("endpoint"),
                auth: row.get("auth"),
                p256dh: row.get("p256dh"),
            }
        })
        .fetch_all(&app_state.db_pool).await.unwrap();
    axum::Json(push_subscriptions)
}

pub async fn send_pushes(
    axum::extract::State(app_state): axum::extract::State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    use web_push::WebPushClient;

    // Load all push subscriptions from the DB
    let push_subscriptions = sqlx::query("SELECT * FROM push_subscriptions")
        .map(|row: sqlx::sqlite::SqliteRow| {
            PushSubscription {
                endpoint: row.get("endpoint"),
                auth: row.get("auth"),
                p256dh: row.get("p256dh"),
            }
        })
        .fetch_all(&app_state.db_pool).await.unwrap();
    tracing::debug!(amount=push_subscriptions.len(), "Sending pushes");

    // Build push payloads
    let private_key = tokio::fs::read("private.pem").await.unwrap();
    let push_payloads = push_subscriptions.into_iter().map(|push_subscription| {
        let subscription_info = web_push::SubscriptionInfo::new(
            push_subscription.endpoint,
            push_subscription.p256dh,
            push_subscription.auth,
        );

        let sig_builder = web_push::VapidSignatureBuilder::from_pem(private_key.as_slice(), &subscription_info).unwrap().build().unwrap();
        let mut builder = web_push::WebPushMessageBuilder::new(&subscription_info);
        let content = "Encrypted payload to be sent in the notification".as_bytes();
        builder.set_payload(web_push::ContentEncoding::Aes128Gcm, content);
        builder.set_vapid_signature(sig_builder);
        builder.build().unwrap()
    });

    // Send all pushes concurrently
    let http_client = Arc::new(web_push::IsahcWebPushClient::new().unwrap());
    futures::future::join_all(push_payloads.map(|push_payload| async {
        tracing::trace!(?push_payload, "Sending push");
        http_client.send(push_payload).await.unwrap();
    })).await;
}