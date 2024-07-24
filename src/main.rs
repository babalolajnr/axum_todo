use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let v1_routes = Router::new()
        .route("/", get(todos))
        .route("/todos", post(create_todo))
        .route("/todos/:id", put(update_todo))
        .route("/todos/:id", delete(delete_todo));

    let app = Router::new().nest("/v1", v1_routes);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running on http://localhost:3000");
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
