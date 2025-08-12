use std::env;
use std::process::Command;

use axum::{http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use serde::Serialize;
use tracing::error;
use crate::settings::SETTINGS;
use crate::schema::response::ApiResponse;

#[derive(Serialize)]
pub struct ApplyResult {
    message: String,
    stdout: String,
    stderr: String,
}

pub async fn apply(headers: HeaderMap) -> impl IntoResponse {
    // Simple password auth with Authorization header. Expect either:
    // - Authorization: Bearer <password>
    // - Authorization: <password>
    let is_authorized = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|raw| {
            if let Some(rest) = raw.strip_prefix("Bearer ") {
                rest == SETTINGS.auth.password
            } else {
                raw == SETTINGS.auth.password
            }
        })
        .unwrap_or(false);

    if !is_authorized {
        return (StatusCode::UNAUTHORIZED, "unauthorized").into_response();
    }

    let (ok, stdout, stderr) = run_compose_up().await;

    if ok {
        let body = ApiResponse::success(serde_json::json!({
            "status": "success",
            "response": stdout,
        }));
        (StatusCode::OK, Json(body)).into_response()
    } else {
        error!("docker compose failed: {}", stderr);
        let body = ApiResponse::error(
            "apply_failed",
            serde_json::json!({
                "status": "failed",
                "response": stderr,
            }),
        );
        (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
    }
}

async fn run_compose_up() -> (bool, String, String) {
    // Prefer `docker compose` (v2). If that fails to spawn, try `docker-compose`.
    match run_command(["docker", "compose", "up", "-d"]).await {
        Ok((ok, out, err)) if ok => (true, out, err),
        Ok((ok, out, err)) => {
            if let Ok((ok2, out2, err2)) = run_command(["docker-compose", "up", "-d"]).await {
                return (ok2, out2, format!("{}\n(prev err) {}", err2, err));
            }
            (ok, out, err)
        }
        Err(e) => {
            // Could not spawn `docker compose`. Try legacy.
            if let Ok((ok, out, err)) = run_command(["docker-compose", "up", "-d"]).await {
                return (ok, out, err);
            }
            (false, String::new(), format!("spawn error: {}", e))
        }
    }
}

async fn run_command<const N: usize>(args: [&str; N]) -> Result<(bool, String, String), String> {
    let mut cmd = Command::new(args[0]);
    for a in &args[1..] {
        cmd.arg(a);
    }
    cmd.envs(env::vars());

    match cmd.output() {
        Ok(output) => Ok((
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )),
        Err(e) => Err(e.to_string()),
    }
}


