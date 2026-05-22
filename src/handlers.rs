//! Axum handlers for helloworld。
//!
//! 共用 `AppCtx`：DB connection + 延迟绑定的 BusClient（用于 cross-app 调用）。

use std::sync::{Arc, OnceLock};

use axum::{
    Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokimo_bus_auth::TokimoUser;
use tokimo_bus_client::BusClient;
use tokimo_bus_protocol::CallerCtx;
use tracing::{info, warn};
use ts_rs::TS;
use uuid::Uuid;

use crate::db::{entities::items, repos::items_repo::ItemsRepo};

pub struct AppCtx {
    pub db: DatabaseConnection,
    pub client: Arc<OnceLock<Arc<BusClient>>>,
}

/// 统一错误响应。
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: msg.into(),
        }
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: msg.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({ "error": self.message });
        (self.status, Json(body)).into_response()
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(e: sea_orm::DbErr) -> Self {
        Self::internal(format!("db: {e}"))
    }
}

// ─── greet / echo ────────────────────────────────────────────────────────

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct GreetReq {
    name: String,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GreetResp {
    message: String,
}

pub async fn greet(Json(req): Json<GreetReq>) -> Result<Json<GreetResp>, AppError> {
    Ok(Json(GreetResp {
        message: format!("Hello, {}!", req.name),
    }))
}

pub async fn echo(body: Bytes) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/octet-stream")],
        body,
    )
        .into_response()
}

// ─── items CRUD ──────────────────────────────────────────────────────────

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ItemDto {
    #[ts(type = "string")]
    pub id: Uuid,
    pub content: String,
    #[ts(type = "string")]
    pub created_at: DateTime<Utc>,
}

impl From<items::Model> for ItemDto {
    fn from(model: items::Model) -> Self {
        Self {
            id: model.id,
            content: model.content,
            created_at: model.created_at.with_timezone(&Utc),
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ItemsListResp {
    items: Vec<ItemDto>,
}

pub async fn items_list(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
) -> Result<Json<ItemsListResp>, AppError> {
    let user_id = parse_user_id(&user_id)?;
    let items = ItemsRepo::list_by_user(&ctx.db, user_id)
        .await?
        .into_iter()
        .map(ItemDto::from)
        .collect();
    Ok(Json(ItemsListResp { items }))
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct AddReq {
    content: String,
}

pub async fn items_add(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<AddReq>,
) -> Result<Json<ItemDto>, AppError> {
    if req.content.trim().is_empty() {
        return Err(AppError::bad_request("content is empty"));
    }
    let user_id = parse_user_id(&user_id)?;
    let item = ItemsRepo::create(&ctx.db, user_id, req.content).await?;
    Ok(Json(ItemDto::from(item)))
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct DeleteResp {
    #[ts(type = "number")]
    deleted: u64,
}

pub async fn items_delete(
    State(ctx): State<Arc<AppCtx>>,
    Path(id): Path<Uuid>,
    TokimoUser { user_id }: TokimoUser,
) -> Result<Json<DeleteResp>, AppError> {
    let user_id = parse_user_id(&user_id)?;
    let deleted = ItemsRepo::delete(&ctx.db, id, user_id).await?;
    Ok(Json(DeleteResp { deleted }))
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct UpdateReq {
    content: String,
}

pub async fn items_update(
    State(ctx): State<Arc<AppCtx>>,
    Path(id): Path<Uuid>,
    TokimoUser { user_id }: TokimoUser,
    Json(req): Json<UpdateReq>,
) -> Result<Json<ItemDto>, AppError> {
    if req.content.trim().is_empty() {
        return Err(AppError::bad_request("content is empty"));
    }

    let user_id = parse_user_id(&user_id)?;
    let item = ItemsRepo::update(&ctx.db, id, user_id, req.content)
        .await?
        .ok_or_else(|| AppError::not_found("item not found"))?;

    Ok(Json(ItemDto::from(item)))
}

pub async fn items_add_with_notify(
    State(ctx): State<Arc<AppCtx>>,
    TokimoUser { user_id }: TokimoUser,
    headers: HeaderMap,
    Json(req): Json<AddReq>,
) -> Result<Json<ItemDto>, AppError> {
    if req.content.trim().is_empty() {
        return Err(AppError::bad_request("content is empty"));
    }

    let parsed_user_id = parse_user_id(&user_id)?;
    let item = ItemsRepo::create(&ctx.db, parsed_user_id, req.content).await?;
    let dto = ItemDto::from(item);

    let caller_user_id = Some(user_id);

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let client = ctx
        .client
        .get()
        .ok_or_else(|| AppError::internal("BusClient not yet bound"))?;

    let notify_payload = serde_json::json!({
        "user_id": caller_user_id,
        "app_id": "helloworld",
        "category_id": "item_added",
        "category_label": "helloworld.notifications.itemAdded",
        "title": "Helloworld",
        "body": format!("New item added: {}", dto.content),
        "level": "info",
    });
    let bytes =
        serde_json::to_vec(&notify_payload).map_err(|e| AppError::internal(format!("serialize notify: {e}")))?;

    info!(item_id = %dto.id, "helloworld: dispatching notification_center.notify");
    if let Err(e) = client
        .invoke(
            "notification_center",
            "notify",
            bytes,
            CallerCtx {
                user_id: caller_user_id,
                caller_app_id: None,
                request_id,
                workspace: None,
            },
        )
        .await
    {
        warn!(error = %e, "notification dispatch failed (item still saved)");
    }

    Ok(Json(dto))
}

// ─── data plane 示例 ─────────────────────────────────────────────────────

pub async fn data_hello() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        "hello from helloworld data-plane\n",
    )
        .into_response()
}

fn parse_user_id(user_id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(user_id).map_err(|_| AppError::bad_request("invalid user id"))
}
