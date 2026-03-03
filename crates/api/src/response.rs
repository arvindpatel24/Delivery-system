use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                data: Some(data),
                error: None,
            }),
        )
    }

    pub fn created(data: T) -> (StatusCode, Json<Self>) {
        (
            StatusCode::CREATED,
            Json(Self {
                success: true,
                data: Some(data),
                error: None,
            }),
        )
    }
}

impl ApiResponse<()> {
    pub fn error(status: StatusCode, message: String) -> (StatusCode, Json<Self>) {
        (
            status,
            Json(Self {
                success: false,
                data: None,
                error: Some(message),
            }),
        )
    }
}
