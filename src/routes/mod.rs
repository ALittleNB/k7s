use axum::{routing::put, Router};

pub mod apply;

pub fn create_router() -> Router {
    Router::new().route("/apply", put(apply::apply))
}


