use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use heisenberg::SpaExt;
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
    let count = todos.len();
    println!("GET /api/todos 200 - {} items", count);
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
    println!("POST /api/todos 200 - created #{}", id);
    Ok(Json(todo))
}

async fn toggle_todo(
    Path(id): Path<u32>,
    axum::extract::State(store): axum::extract::State<TodoStore>,
) -> Result<Json<Todo>, StatusCode> {
    let mut todos = store.lock().await;
    if let Some(todo) = todos.get_mut(&id) {
        todo.completed = !todo.completed;
        println!("POST /api/todos/{}/toggle 200", id);
        Ok(Json(todo.clone()))
    } else {
        println!("POST /api/todos/{}/toggle 404", id);
        Err(StatusCode::NOT_FOUND)
    }
}

#[tokio::main]
async fn main() {
    let store: TodoStore = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/api/todos", get(get_todos).post(create_todo))
        .route("/api/todos/:id/toggle", post(toggle_todo))
        .with_state(store)
        .spa_at_from("/*", "./web"); // One line! References the app, not the output

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to 127.0.0.1:3001"));

    println!("🚀 Server running on http://127.0.0.1:3001\n");

    axum::serve(listener, app)
        .with_graceful_shutdown(heisenberg::shutdown_signal())
        .await
        .unwrap();
}
