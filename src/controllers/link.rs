use axum::{
    extract::{self, Path},
    http::StatusCode,
    response::Redirect,
    Extension, Json,
};

use crate::models::link::Link;
//use serde_json::{json, Value};
use sqlx::{Pool, Postgres};

pub async fn create_link(
    state: Extension<Pool<Postgres>>,
    extract::Json(link): extract::Json<Link>,
) -> Json<Link> {
    let Extension(pool) = state;

    let row = sqlx::query!(
        "INSERT INTO links (id, link) VALUES ($1, $2) RETURNING id, link",
        link.id,
        link.link
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create link");

    Json(Link {
        id: row.id,
        link: row.link,
    })
}

pub async fn redirect(
    state: Extension<Pool<Postgres>>,
    Path(short_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    let Extension(pool) = state;

    let row = sqlx::query!("SELECT link FROM links WHERE id = $1 LIMIT 1", short_id)
        .fetch_one(&pool)
        .await
        .expect("Failed to find short");
    let redirect = Redirect::to(&row.link);
    Ok(redirect)
}
