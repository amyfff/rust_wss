use crate::{
    auth::auth_middleware,
    handlers::{auth_handler, book_handler, email_handler, ws_handler},
    AppState,
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new().nest("/api/v1", api_routes(app_state))
}

fn api_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .merge(create_auth_routes(app_state.clone()))
        .merge(create_book_routes(app_state.clone()))
        .merge(create_email_routes(app_state.clone()))
        .merge(create_ws_route(app_state))
}

fn create_auth_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/signup", post(auth_handler::signup))
        .route("/signin", post(auth_handler::signin))
        .with_state(app_state)
}

fn create_book_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/books", get(book_handler::get_all_books).post(book_handler::create_book))
        .route(
            "/books/:id",
            get(book_handler::get_book_by_id)
                .put(book_handler::update_book)
                .delete(book_handler::delete_book),
        )
        .route_layer(middleware::from_fn(auth_middleware))
        .with_state(app_state)
}

fn create_email_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/emails", get(email_handler::get_all_emails).post(email_handler::create_email))
        .route(
            "/emails/:id",
            get(email_handler::get_email_by_id)
                .put(email_handler::update_email)
                .delete(email_handler::delete_email),
        )
        // This line adds the authentication requirement
        .route_layer(middleware::from_fn(auth_middleware))
        .with_state(app_state)
}

fn create_ws_route(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler::websocket_handler))
        .route_layer(middleware::from_fn(auth_middleware))
        .with_state(app_state)
}