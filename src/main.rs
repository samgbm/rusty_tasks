use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put}, // Note: We only import get and put here, post is used as a chained method below.
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

// --- 1. Data Models ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
}

// Structs to define what incoming JSON data should look like
#[derive(Deserialize)]
pub struct CreateTaskPayload {
    pub title: String,
}

#[derive(Deserialize)]
pub struct UpdateTaskPayload {
    pub status: TaskStatus,
}

// --- 2. Shared State ---
// We use Arc (Atomic Reference Counted) and Mutex to share data across threads safely.
type AppState = Arc<Mutex<HashMap<String, Task>>>;

// --- 3. Main Function ---
#[tokio::main]
async fn main() {
    // Initialize our in-memory database (an empty HashMap)
    let shared_state: AppState = Arc::new(Mutex::new(HashMap::new()));

    // Build the router and attach our handlers and state
    let app = Router::new()
        .route("/", get(|| async { "RustyTasks API is live!" }))
        .route("/tasks", get(get_tasks).post(create_task))
        // In Axum 0.7, path variables use {id} syntax instead of :id
        .route("/tasks/{id}", put(update_task))
        .with_state(shared_state); // Inject the state into our routes

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- 4. Handlers ---

// GET /tasks: Returns a list of all tasks
async fn get_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let db = state.lock().unwrap();
    let tasks: Vec<Task> = db.values().cloned().collect();
    Json(tasks) // Automatically serializes the Vec<Task> to JSON
}

// POST /tasks: Creates a new task
async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskPayload>,
) -> (StatusCode, Json<Task>) {
    let new_task = Task {
        id: Uuid::new_v4().to_string(), // Generate a random UUID
        title: payload.title,
        status: TaskStatus::ToDo,       // All new tasks start as ToDo
    };

    // Lock the database to safely write to it
    let mut db = state.lock().unwrap();
    db.insert(new_task.id.clone(), new_task.clone());

    (StatusCode::CREATED, Json(new_task))
}

// PUT /tasks/{id}: Updates a task's status
async fn update_task(
    Path(id): Path<String>, // Extracts the {id} from the URL
    State(state): State<AppState>,
    Json(payload): Json<UpdateTaskPayload>,
) -> Result<Json<Task>, StatusCode> {
    let mut db = state.lock().unwrap();

    // Check if the task exists in our HashMap
    if let Some(task) = db.get_mut(&id) {
        task.status = payload.status; // Update the status
        Ok(Json(task.clone()))
    } else {
        Err(StatusCode::NOT_FOUND) // Return 404 if the ID doesn't exist
    }
}