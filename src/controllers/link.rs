use axum::{
    extract::{self, Path},
    http::StatusCode,
    response::Redirect,
    response::{IntoResponse, Response, Result},
    Extension, Json,
};
use url::Url;

use crate::models::link::Link;
//use serde_json::{json, Value};
use sqlx::{Pool, Postgres};

pub async fn create_link(
    state: Extension<Pool<Postgres>>,
    extract::Json(link): extract::Json<Link>,
) -> Response {
    let Extension(pool) = state;
    let cleaned_link = Url::parse(&link.link);
    if let Err(res) = cleaned_link {
        return (StatusCode::BAD_REQUEST, Json(res).to_string()).into_response();
    }
    let cleaned_link = cleaned_link.unwrap();
    if (cleaned_link.scheme() != "http") && (cleaned_link.scheme() != "https") {
        return (StatusCode::BAD_REQUEST, "invalid url").into_response();
    }
    let row = sqlx::query!(
        "INSERT INTO links (id, link) VALUES ($1, $2) RETURNING id, link",
        link.id,
        cleaned_link.to_string()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create link");

    Json(Link {
        id: row.id,
        link: row.link,
    })
    .into_response()
}

pub async fn update_link(
    state: Extension<Pool<Postgres>>,
    extract::Json(link): extract::Json<Link>,
) -> Response {
    let Extension(pool) = state;
    let cleaned_link = Url::parse(&link.link);
    if let Err(res) = cleaned_link {
        return (StatusCode::BAD_REQUEST, Json(res).to_string()).into_response();
    }
    let cleaned_link = cleaned_link.unwrap();
    if (cleaned_link.scheme() != "http") && (cleaned_link.scheme() != "https") {
        return (StatusCode::BAD_REQUEST, "invalid url").into_response();
    }
    let row = sqlx::query!(
        "UPDATE links SET link=$2 WHERE id=$1 RETURNING id, link",
        link.id,
        cleaned_link.as_str()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to update link");

    Json(Link {
        id: row.id,
        link: row.link,
    })
    .into_response()
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
