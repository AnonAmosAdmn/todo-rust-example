use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post, put, delete},
    Json, Router,
};

use hyper::Server;

use serde::{Deserialize, Serialize};

use sqlx::SqlitePool;

use std::net::SocketAddr;

use uuid::Uuid;

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

type Db = SqlitePool;

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    title: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    fs::create_dir_all("data")?;

    let path = PathBuf::from("data/todos.db");

    if !path.exists() {
        fs::File::create(&path)?;
    }

    let db_url = format!("sqlite://{}", path.display());

    let db = SqlitePool::connect(&db_url).await?;
    println!(" ");
    println!("🟢 Connected to SQLite DB at {}", db_url);

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL
        )
        "#,
    )
    .execute(&db)
    .await?;

    let app = Router::new()
        .route("/", get(root))
        .route("/todos", get(list_todos))
        .route("/todos", post(create_todo))
        .route("/todos/:id", get(get_todo))
        .route("/todos/:id", put(update_todo))
        .route("/todos/:id", delete(delete_todo))
        .with_state(db.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!(" ");
    println!("✅ Running Todo API on http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn list_todos(State(db): State<Db>) -> Result<Json<Vec<Todo>>, StatusCode> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos")
        .fetch_all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(todos))
}

async fn create_todo(State(db): State<Db>, Json(payload): Json<CreateTodo>) -> Result<Json<Todo>, StatusCode> {
    let id = Uuid::new_v4().to_string();
    let todo = Todo {
        id: id.clone(),
        title: payload.title,
        completed: false,
    };

    sqlx::query("INSERT INTO todos (id, title, completed) VALUES (?, ?, ?)")
        .bind(&todo.id)
        .bind(&todo.title)
        .bind(todo.completed)
        .execute(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(todo))
}

async fn get_todo(Path(id): Path<String>, State(db): State<Db>) -> Result<Json<Todo>, StatusCode> {
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos WHERE id = ?")
        .bind(&id)
        .fetch_optional(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(todo) = todo {
        Ok(Json(todo))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn update_todo(
    Path(id): Path<String>,
    State(db): State<Db>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
     let existing = sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos WHERE id = ?")
        .bind(&id)
        .fetch_optional(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(mut todo) = existing {
        if let Some(title) = payload.title {
            todo.title = title;
        }

        sqlx::query("UPDATE todos SET title = ?, completed = ? WHERE id = ?")
            .bind(&todo.title)
            .bind(todo.completed)
            .bind(&todo.id)
            .execute(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(todo))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn delete_todo(
    Path(id): Path<String>,
    State(db): State<Db>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(&id)
        .execute(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 1 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn root() -> impl IntoResponse {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>Todo API Frontend</title>
<style>
  body { font-family: Arial, sans-serif; margin: 2rem; }
  button { margin: 0.5rem; padding: 0.5rem 1rem; }
  input { padding: 0.3rem; margin-left: 0.5rem; }
  pre { background: #eee; padding: 1rem; }
</style>
</head>
<body>
  <h1>Todo API Frontend</h1>

  <button onclick="listTodos()">List All Todos</button>
  <br />

  <label>New Todo Title:
    <input type="text" id="newTodoTitle" placeholder="New title" />
  </label>
  <button onclick="createTodo()">Create Todo</button>
  <br />

  <label>Get Todo by ID:
    <input type="text" id="todoId" placeholder="ID" />
  </label>
  <button onclick="getTodo()">Get Todo</button>
  <br />

  <label>Update Todo by ID:
    <input type="text" id="updateTodoId" placeholder="ID" />
    <input type="text" id="updateTitle" placeholder="New title" />
  </label>
  <button onclick="updateTodo()">Update Todo</button>
  <br />

  <label>Delete Todo by ID:
    <input type="text" id="deleteTodoId" placeholder="ID" />
  </label>
  <button onclick="deleteTodo()">Delete Todo</button>

  <h2>Result:</h2>
  <pre id="result">No results yet</pre>

  <script>
    async function listTodos() {
      const res = await fetch('/todos');
      const data = await res.json();
      document.getElementById('result').textContent = JSON.stringify(data, null, 2);
    }

    async function createTodo() {
      const title = document.getElementById('newTodoTitle').value.trim();
      if (!title) {
        alert('Please enter a title');
        return;
      }

      const res = await fetch('/todos', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ title })
      });

      const data = await res.json();
      document.getElementById('result').textContent = JSON.stringify(data, null, 2);
    }

    async function getTodo() {
      const id = document.getElementById('todoId').value.trim();
      if (!id) {
        alert('Please enter an ID');
        return;
      }

      const res = await fetch('/todos/' + id);

      if (res.status === 404) {
        document.getElementById('result').textContent = 'Todo not found!';
        return;
      }

      const data = await res.json();
      document.getElementById('result').textContent = JSON.stringify(data, null, 2);
    }

    async function updateTodo() {
      const id = document.getElementById('updateTodoId').value.trim();
      if (!id) {
        alert('Please enter an ID to update');
        return;
      }

      const title = document.getElementById('updateTitle').value.trim();

      // Build payload only with fields that are provided
      const payload = {};
      if (title) payload.title = title;

      const res = await fetch('/todos/' + id, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (res.status === 404) {
        document.getElementById('result').textContent = 'Todo not found!';
        return;
      }

      const data = await res.json();
      document.getElementById('result').textContent = JSON.stringify(data, null, 2);
    }

    async function deleteTodo() {
      const id = document.getElementById('deleteTodoId').value.trim();
      if (!id) {
        alert('Please enter an ID to delete');
        return;
      }

      const res = await fetch('/todos/' + id, {
        method: 'DELETE',
      });

      if (res.status === 204) {
        document.getElementById('result').textContent = `Todo with ID ${id} deleted successfully`;
      } else if (res.status === 404) {
        document.getElementById('result').textContent = 'Todo not found!';
      } else {
        document.getElementById('result').textContent = 'Error deleting todo';
      }
    }
  </script>
</body>
</html>
    "#;

    Html(html)
}
