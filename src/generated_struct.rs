
use axum::{                                                                                                                                                                      
    extract::{self, Extension, Path},                                                                                                                                            
    routing::{get, post},                                                                                                                                                        
    Json, Router,                                                                                                                                                                
};                                                                                                                                                                               
use serde::Deserialize;                                                                                                                                                          
use serde_json::{json, Value};                                                                                                                                                   
use sqlx::PgPool;                                                                                                                                                                
use sqlx::{postgres::PgPoolOptions, prelude::FromRow};                                                                                                                           
use std::env;                                                                                                                                                                    
use std::net::SocketAddr;                                                                                                                                                        
use std::result::Result;                                                                                                                                                         
use std::sync::Arc;                                                                                                                                                              
use axum::http::StatusCode;                                                                                                                                                      
use sqlx::types::chrono::Utc; 


#[derive(Debug, Deserialize, FromRow)]
struct Users {
    id: uuid::Uuid,
    username: String,
    email: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}


async fn add_users(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<Users>,
) -> Json<Value> {
    let query = "INSERT INTO users (id, username, email, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)";
    sqlx::query(query)
    	.bind(payload.id)
	.bind(payload.username)
	.bind(payload.email)
	.bind(payload.created_at)
	.bind(payload.updated_at)
        .execute(&pool)
        .await;
        Json(json!({"res": "sucsess"}))
}


async fn get_users(
    extract::State(pool): extract::State<PgPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let query = "SELECT * FROM users";
    let q = sqlx::query_as::<_, Users>(query);

    let elemints: Vec<Users> = q.fetch_all(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let res_json: Vec<Value> = elemints.into_iter().map(|elemint| {
        json!({
    	"id": elemint.id, 
	"username": elemint.username, 
	"email": elemint.email, 
	"created_at": elemint.created_at, 
	"updated_at": elemint.updated_at, 

        })
    
    }).collect();

    Ok(Json(json!({ "payload": res_json })))
}
#[derive(Debug, Deserialize, FromRow)]
struct Runs {
    id: uuid::Uuid,
    user_id: uuid::Uuid,
    start_time: chrono::DateTime<Utc>,
    time_running: PgInterval,
    distance: f64,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}


async fn add_runs(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<Runs>,
) -> Json<Value> {
    let query = "INSERT INTO runs (id, user_id, start_time, time_running, distance, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)";
    sqlx::query(query)
    	.bind(payload.id)
	.bind(payload.user_id)
	.bind(payload.start_time)
	.bind(payload.time_running)
	.bind(payload.distance)
	.bind(payload.created_at)
	.bind(payload.updated_at)
        .execute(&pool)
        .await;
        Json(json!({"res": "sucsess"}))
}


async fn get_runs(
    extract::State(pool): extract::State<PgPool>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let query = "SELECT * FROM runs";
    let q = sqlx::query_as::<_, Runs>(query);

    let elemints: Vec<Runs> = q.fetch_all(&pool).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let res_json: Vec<Value> = elemints.into_iter().map(|elemint| {
        json!({
    	"id": elemint.id, 
	"user_id": elemint.user_id, 
	"start_time": elemint.start_time, 
	"time_running": elemint.time_running, 
	"distance": elemint.distance, 
	"created_at": elemint.created_at, 
	"updated_at": elemint.updated_at, 

        })
    
    }).collect();

    Ok(Json(json!({ "payload": res_json })))
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_url = "postgres://dbuser:p@localhost:1111/work";
    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(db_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    let app = Router::new()
    	.route("/add_users", post(add_users))
	.route("/get_users", get(get_users))
	.route("/add_runs", post(add_runs))
	.route("/get_runs", get(get_runs))

        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await.unwrap();

    axum::serve(listener, app).await.unwrap();
    Ok(())
}


