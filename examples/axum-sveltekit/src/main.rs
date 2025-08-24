use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

type TodoStore = Arc<Mutex<HashMap<u32, Todo>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Todo {
    id: u32,
    title: String,
    completed: bool,
}

#[derive(Deserialize)]
struct CreateTodo {
    title: String,
}

async fn get_todos(store: axum::extract::State<TodoStore>) -> Json<Vec<Todo>> {
    let todos = store.lock().await;
    Json(todos.values().cloned().collect())
}

async fn create_todo(
    axum::extract::State(store): axum::extract::State<TodoStore>,
    Json(payload): Json<CreateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = store.lock().await;
    let id = todos.len() as u32 + 1;
    let todo = Todo {
        id,
        title: payload.title,
        completed: false,
    };
    todos.insert(id, todo.clone());
    Ok(Json(todo))
}
async fn toggle_todo(
    Path(id): Path<u32>,
    axum::extract::State(store): axum::extract::State<TodoStore>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = store.lock().await;
    if let Some(todo) = todos.get_mut(&id) {
        todo.completed = !todo.completed;
        Ok(Json(todo.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {
    let store: TodoStore = Arc::new(Mutex::new(HashMap::new()));

    let api_routes = Router::new()
        .route("/todos", get(get_todos).post(create_todo))
        .route("/todos/:id/toggle", post(toggle_todo))
        .with_state(store);

    let app = Router::new()
        .nest("/api", api_routes)
        .layer(heisenberg::HeisenbergLayer::new(
            heisenberg::Heisenberg::new().spa("./web/build").build(),
        ));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!("🚀 Server running on http://127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}
