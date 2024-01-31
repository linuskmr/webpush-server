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