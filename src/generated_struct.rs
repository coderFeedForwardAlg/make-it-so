
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
    user_uuid: uuid::Uuid,
    username: String,
    email: String,
    created_at: chrono::DateTime<Utc>,
}


async fn add_users(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<Users>,
) -> Json<Value> {
    let query = "INSERT INTO users (user_uuid, username, email, created_at) VALUES ($1, $2, $3, $4)";
    let result = sqlx::query(query)
    	.bind(payload.user_uuid)
	.bind(payload.username)
	.bind(payload.email)
	.bind(payload.created_at)
        .execute(&pool)
        .await;
    match result {
        Ok(value) => Json(json!({"res": "added"})),
        Err(e) => Json(json!({"res": format!("error: {}", e)}))

    }
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
    	"user_uuid": elemint.user_uuid, 
	"username": elemint.username, 
	"email": elemint.email, 
	"created_at": elemint.created_at, 

        })
    
    }).collect();

    Ok(Json(json!({ "payload": res_json })))
}
#[derive(Debug, Deserialize, FromRow)]
struct Runs {
    run_uuid: uuid::Uuid,
    user_uuid: uuid::Uuid,
    distance_km: f64,
    completion_time_seconds: i32,
    start_time: chrono::DateTime<Utc>,
}


async fn add_runs(
    extract::State(pool): extract::State<PgPool>,
    Json(payload): Json<Runs>,
) -> Json<Value> {
    let query = "INSERT INTO runs (run_uuid, user_uuid, distance_km, completion_time_seconds, start_time) VALUES ($1, $2, $3, $4, $5)";
    let result = sqlx::query(query)
    	.bind(payload.run_uuid)
	.bind(payload.user_uuid)
	.bind(payload.distance_km)
	.bind(payload.completion_time_seconds)
	.bind(payload.start_time)
        .execute(&pool)
        .await;
    match result {
        Ok(value) => Json(json!({"res": "added"})),
        Err(e) => Json(json!({"res": format!("error: {}", e)}))

    }
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
    	"run_uuid": elemint.run_uuid, 
	"user_uuid": elemint.user_uuid, 
	"distance_km": elemint.distance_km, 
	"completion_time_seconds": elemint.completion_time_seconds, 
	"start_time": elemint.start_time, 

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


