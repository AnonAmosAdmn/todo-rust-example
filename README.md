# Rust Todo API

A lightweight and fully functional Todo API built with Rust, Axum, SQLx, and SQLite.


# This project features:

RESTful endpoints to create, read, update, and delete todos.

SQLite-backed storage with auto-initialization.

Simple HTML+JS frontend for testing the API interactively.

Axum web framework for routing and async handlers.

SQLite for easy file-based persistence.

UUIDs for unique Todo IDs.

Auto-created database and table in a local data/ directory.

Minimal frontend served from the root (/) route.


# Installation Requirements

Rust & Cargo

sqlite3 (optional, for inspecting DB manually)


# Clone the repo

git clone https://github.com/AnonAmosAdmn/todo-rust-example

cd todo-rust-example


# Run the server

cargo run

By default, the server runs at http://127.0.0.1:3000 and creates data/todos.db.


# API Endpoints

Method	Endpoint	Description

GET	/todos	     List all todos

GET	/todos/:id	     Get a specific todo

POST	/todos       	Create a new todo

PUT	/todos/:id	      Update a todo (title)

DELETE	/todos/:id	       Delete a todo by ID

GET	/	      Basic HTML frontend


# Example POST /todos body:
{
  "title": "new todo"
}


# Example PUT /todos/:id body:
{
  "title": "Updated title"
}


# Project Structure

├── src/

│   └── main.rs         # Main server and route logic

├── data/               # SQLite DB auto-generated here

├── Cargo.toml          # Dependencies and project info

└── README.md


# Dependencies

axum,
hyper,
serde,
sqlx,
uuid,
tokio


# License

MIT License


# Todo

 Add completed status toggle,
 
 Add filtering (e.g. /todos?completed=true),
 
 Add tests,
 
 Dockerize
