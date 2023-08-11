use axum::{
    extract::TypedHeader,
    headers::authorization::{Authorization, Basic},
    http::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Extension, Router,
};

use dotenvy::dotenv;
mod controllers;
mod models;
use controllers::link::{create_link, redirect, update_link};
use controllers::user::{create_user, delete_user, get_users};

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap_or_else(|_| panic!("Failed to create Postgres connection pool! URL: {}", url));

    sqlx::migrate!("./migrations").run(&pool).await?;

    let addr: std::net::SocketAddr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    println!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app().layer(Extension(pool)).into_make_service())
        .await
        .unwrap();

    Ok(())
}

/// Having a function that produces our app makes it easy to call it from tests
/// without having to create an HTTP server.
#[allow(dead_code)]
fn app() -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/user", post(create_user))
        .route("/users", get(get_users))
        .route("/user/:id", delete(delete_user))
        .route_layer(middleware::from_fn(auth))
        .route("/link", post(create_link).put(update_link))
        .route("/:id", get(redirect))
}

async fn auth<B>(
    // run the `TypedHeader` extractor
    TypedHeader(auth): TypedHeader<Authorization<Basic>>,
    // you can also add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if token_is_valid(auth.username(), auth.password()) {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn token_is_valid(username: &str, password: &str) -> bool {
    username == "admin" && password == "password"
}

async fn handler() -> &'static str {
    "Let's Get Rusty!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"Let's Get Rusty!");
    }
}
