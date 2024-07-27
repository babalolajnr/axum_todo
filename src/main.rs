use std::time::Duration;

use axum::{
    body::Bytes,
    extract::{MatchedPath, Path},
    http::{HeaderMap, Request},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{info, info_span, Span};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_span_events(FmtSpan::FULL),
        )
        .init();

    // Log a test message to verify logging is working
    tracing::info!("Logging initialized");

    let v1_routes = Router::new()
        .route("/", get(todos))
        .route("/todos", post(create_todo))
        .route("/todos/:id", put(update_todo))
        .route("/todos/:id", delete(delete_todo));

    let app = Router::new().nest("/v1", v1_routes).layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request<_>| {
                // Log the matched route's path (with placeholders not filled in).
                // Use request.uri() or OriginalUri if you want the real path.
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                info_span!(
                    "http_request",
                    method = ?request.method(),
                    uri = ?request.uri(),
                    matched_path,
                    user_agent = ?request.headers().get("user-agent").and_then(|v| v.to_str().ok()),
                )
            })
            .on_request(|request: &Request<_>, _span: &Span| {
                // You can use `_span.record("some_other_field", value)` in one of these
                // closures to attach a value to the initially empty field in the info_span
                // created above.
                info!("Started {} request to {}", request.method(), request.uri());

                // // Log headers
                // for (name, value) in request.headers() {
                //     info!("Header: {}: {:?}", name, value);
                // }
            })
            .on_response(|response: &Response, latency: Duration, _span: &Span| {
                info!(
                    "Finished request with status {} in {:?}",
                    response.status(),
                    latency
                );
            })
            .on_body_chunk(|chunk: &Bytes, _latency: Duration, _span: &Span| {
                info!("Sent {} bytes", chunk.len());
            })
            .on_eos(
                |trailers: Option<&HeaderMap>, stream_duration: Duration, _span: &Span| {
                    info!(
                        "Stream closed after {:?}, trailers: {:?}",
                        stream_duration, trailers
                    );
                },
            )
            .on_failure(
                |error: ServerErrorsFailureClass, latency: Duration, _span: &Span| {
                    info!("Request failed after {:?}: {:?}", latency, error);
                },
            ),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct UpdateTodo {
    title: Option<String>,
    body: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Todo {
    id: u64,
    title: String,
    body: String,
}

async fn create_todo(Json(todo): Json<CreateTodo>) -> Json<Todo> {
    Json(Todo {
        id: 42,
        title: todo.title,
        body: todo.body,
    })
}

async fn todos() -> Json<Vec<Todo>> {
    Json(vec![
        Todo {
            id: 1,
            title: "Do the dishes".to_string(),
            body: "Make sure to do the dishes before bed".to_string(),
        },
        Todo {
            id: 2,
            title: "Walk the dog".to_string(),
            body: "Take Fido for a walk around the block".to_string(),
        },
    ])
}

async fn update_todo(Path(id): Path<u64>, Json(todo): Json<UpdateTodo>) -> Json<Todo> {
    Json(Todo {
        id,
        title: todo.title.unwrap_or_else(|| "Do the dishes".to_string()),
        body: todo
            .body
            .unwrap_or_else(|| "Make sure to do the dishes before bed".to_string()),
    })
}

async fn delete_todo(Path(id): Path<u64>) -> Json<Todo> {
    Json(Todo {
        id,
        title: "Do the dishes".to_string(),
        body: "Make sure to do the dishes before bed".to_string(),
    })
}
