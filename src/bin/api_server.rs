// Cofre Vault — HTTP API Server
// Provides REST endpoints for the semantic chat feature

use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use uuid::Uuid;
use cofre_vault::{
    models::ChatRequest,
    services::{EmbeddingService, RagChatService},
};

#[derive(Clone)]
struct AppState {
    rag_chat: Arc<RagChatService>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Build sqlx PgPool from DATABASE_URL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");
    let pool = Arc::new(pool);

    // Build services
    let embedding_service = Arc::new(
        EmbeddingService::from_env().expect("Failed to initialize EmbeddingService")
    );
    let rag_chat = Arc::new(
        RagChatService::from_env(embedding_service, pool)
            .expect("Failed to initialize RagChatService")
    );

    let state = AppState { rag_chat };

    let app = Router::new()
        .route("/api/vaults/:vault_id/chat", post(chat_handler))
        .with_state(state);

    let addr = std::env::var("API_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    tracing::info!("API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn chat_handler(
    State(state): State<AppState>,
    Path(vault_id): Path<Uuid>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    // user_id placeholder — in production this would come from auth middleware
    let user_id = Uuid::nil();

    match state.rag_chat.process_message(vault_id, user_id, &payload.message).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(cofre_vault::Error::RateLimitExceeded) => {
            (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded — Gemini quota hit, try again shortly").into_response()
        }
        Err(cofre_vault::Error::ChatGenerationFailed(msg)) => {
            (StatusCode::BAD_GATEWAY, msg).into_response()
        }
        Err(e) => {
            tracing::error!("Chat handler error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        }
    }
}
