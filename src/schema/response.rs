use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub msg: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            msg: "ok".to_string(),
            data,
        }
    }

    pub fn error(msg: impl Into<String>, data: T) -> Self {
        Self {
            code: -1,
            msg: msg.into(),
            data,
        }
    }
}


