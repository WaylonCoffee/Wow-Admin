use crate::{error::AppError, state::SharedState};
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use domain::user::User;
use infra::user_repo;
use uuid::Uuid;

pub fn routes() -> Router<SharedState> {
    Router::new().route("/{id}", get(get_user))
}

async fn get_user(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, AppError> {
    let user = user_repo::find_user_by_id(&state.pool, id).await?;
    match user {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::BadRequest(format!("user {id} not found"))),
    }
}
