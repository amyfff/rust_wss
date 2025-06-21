use crate::{auth::Claims, error::AppError, models::book::{Book, CreateBook, UpdateBook}, ws, AppState};
use axum::{extract::{Path, State}, Extension, Json};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[axum::debug_handler]
pub async fn get_all_books(State(state): State<Arc<AppState>>, _claims: Extension<Claims>) -> Result<Json<Vec<Book>>, AppError> {
    let books = sqlx::query_as!(Book, "SELECT * FROM books ORDER BY created_at DESC")
        .fetch_all(&state.db_pool)
        .await?;
    Ok(Json(books))
}

#[axum::debug_handler]
pub async fn create_book(State(state): State<Arc<AppState>>, _claims: Extension<Claims>, Json(payload): Json<CreateBook>) -> Result<Json<Book>, AppError> {
    payload.validate()?;
    let book = sqlx::query_as!(
        Book,
        "INSERT INTO books (title, author, publication_year) VALUES ($1, $2, $3) RETURNING *",
        payload.title,
        payload.author,
        payload.publication_year
    )
    .fetch_one(&state.db_pool)
    .await?;
    ws::broadcast_event(ws::WsEvent::BookCreated(book.clone()));
    Ok(Json(book))
}

#[axum::debug_handler]
pub async fn get_book_by_id(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, _claims: Extension<Claims>) -> Result<Json<Book>, AppError> {
    let book = sqlx::query_as!(Book, "SELECT * FROM books WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Buku".to_string()))?;
    Ok(Json(book))
}

#[axum::debug_handler]
pub async fn update_book(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, _claims: Extension<Claims>, Json(payload): Json<UpdateBook>) -> Result<Json<Book>, AppError> {
    payload.validate()?;
    let book = sqlx::query_as!(Book, "SELECT * FROM books WHERE id = $1", id)
        .fetch_optional(&state.db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Buku".to_string()))?;

    let title = payload.title.unwrap_or(book.title);
    let author = payload.author.unwrap_or(book.author);
    let pub_year = payload.publication_year.or(book.publication_year);

    let updated_book = sqlx::query_as!(
        Book,
        "UPDATE books SET title = $1, author = $2, publication_year = $3, updated_at = NOW() WHERE id = $4 RETURNING *",
        title, author, pub_year, id
    )
    .fetch_one(&state.db_pool)
    .await?;
    ws::broadcast_event(ws::WsEvent::BookUpdated(updated_book.clone()));
    Ok(Json(updated_book))
}

#[axum::debug_handler]
pub async fn delete_book(State(state): State<Arc<AppState>>, Path(id): Path<Uuid>, _claims: Extension<Claims>) -> Result<(), AppError> {
    let result = sqlx::query!("DELETE FROM books WHERE id = $1", id)
        .execute(&state.db_pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Buku".to_string()));
    }
    ws::broadcast_event(ws::WsEvent::BookDeleted(id));
    Ok(())
}