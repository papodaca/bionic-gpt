mod index;
mod new;
use axum::{
    routing::{get, post},
    Router,
};
use ui_components::routes::datasets::{INDEX, NEW};

pub fn routes() -> Router {
    Router::new()
        .route(INDEX, get(index::index))
        .route(NEW, post(new::new))
}
