use crate::models::{book::Book, email::Email};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{stream::StreamExt, SinkExt};
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum WsEvent {
    BookCreated(Book), BookUpdated(Book), BookDeleted(Uuid),
    EmailCreated(Email), EmailUpdated(Email), EmailDeleted(Uuid),
}

static CHANNEL: once_cell::sync::Lazy<broadcast::Sender<String>> =
    once_cell::sync::Lazy::new(|| { let (tx, _rx) = broadcast::channel(100); tx });

pub fn broadcast_event(event: WsEvent) {
    match serde_json::to_string(&event) {
        Ok(json_message) => {
            if let Err(e) = CHANNEL.send(json_message) {
                 tracing::warn!("Gagal menyiarkan pesan WebSocket: {}", e);
            }
        }
        Err(e) => tracing::error!("Gagal serialisasi event WebSocket ke JSON: {}", e),
    }
}

pub async fn handle_socket(mut socket: WebSocket, user_id: String) {
    let user_id_clone = user_id.clone();
    tracing::info!("WebSocket client terhubung: {}", user_id);
    let mut rx = CHANNEL.subscribe();

    let welcome_msg = serde_json::json!({
        "event": "CONNECTED",
        "data": format!("Welcome, user {}! You are now listening for updates.", user_id)
    }).to_string();

    if socket.send(Message::Text(welcome_msg)).await.is_err() {
        tracing::warn!("Gagal mengirim pesan selamat datang ke {}", user_id);
        return;
    }

    loop {
        tokio::select! {
            // Menerima pesan baru dari channel broadcast dan mengirimkannya ke client
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg)).await.is_err() {
                    // Client terputus
                    break;
                }
            }
            // Menerima pesan dari client (misalnya, ping atau close)
            Some(Ok(msg)) = socket.next() => {
                if let Message::Close(_) = msg {
                    // Client meminta untuk menutup koneksi
                    break;
                }
            }
            // Kedua channel ditutup, keluar dari loop
            else => {
                break;
            }
        }
    }

    tracing::info!("Koneksi WebSocket untuk {} telah ditutup.", user_id_clone);
}