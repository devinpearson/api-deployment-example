use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};

use crate::models::user::User;
//use serde_json::{json, Value};
use sqlx::{Pool, Postgres};

pub async fn get_users(state: Extension<Pool<Postgres>>) -> Json<Vec<User>> {
    let Extension(pool) = state;

    let records = sqlx::query!("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .expect("failed to fetch users");

    let records = records
        .iter()
        .map(|r| User {
            id: Some(r.id),
            name: r.name.to_string(),
            email: r.email.clone(),
        })
        .collect();

    Json(records)
}

pub async fn create_user(
    state: Extension<Pool<Postgres>>,
    extract::Json(user): extract::Json<User>,
) -> Json<User> {
    let Extension(pool) = state;

    let row = sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email",
        user.name,
        user.email
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create user");

    Json(User {
        id: Some(row.id),
        name: row.name,
        email: row.email,
    })
}

pub async fn delete_user(state: Extension<Pool<Postgres>>, Path(user_id): Path<i32>) -> StatusCode {
    let Extension(pool) = state;

    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&pool)
        .await
        .expect("Failed to delete user");

    StatusCode::NO_CONTENT
}
