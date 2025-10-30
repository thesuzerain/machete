// As described here: https://github.com/tokio-rs/axum/discussions/1423
// It's not actively recommended to have tests in the integration tests directory, but I personally prefer it for organization.

use axum::{body::Body, http::Request};
use machete::app;
use reqwest::StatusCode;
use sqlx::PgPool;
use tower::util::ServiceExt;
use http_body_util::BodyExt;

#[sqlx::test]
async fn basic_test(pool: PgPool) -> sqlx::Result<()>  {
    let app = app(pool);

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello, World!");
    Ok(())
}