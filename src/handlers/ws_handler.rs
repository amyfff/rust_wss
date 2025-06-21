use crate::auth::Claims;
use crate::ws;
use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    Extension,
};

#[axum::debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(claims): Extension<Claims>,
) -> Response {
    let user_id = claims.sub.clone();
    ws.on_upgrade(move |socket: WebSocket| ws::handle_socket(socket, user_id))
}