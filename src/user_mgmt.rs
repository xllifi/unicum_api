use std::process::exit;

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use axum::{
    extract::{FromRequestParts, Request},
    http::request::Parts,
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::Engine;
use inquire::{Confirm, MultiSelect, Text};
use log::{error, info, warn};
use reqwest::{StatusCode, header};
use sqlx::PgPool;

use crate::{
    ENV_SALT_B64,
    entities::{Permissions, User},
};

fn hash_password(password: &str) -> Result<String, String> {
    let argon2 = Argon2::default();
    let salt = SaltString::from_b64(&ENV_SALT_B64.value).map_err(|e| e.to_string())?;

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|x| x.to_string())
        .map_err(|e| e.to_string())
}

pub async fn useradd(username: Option<String>, pool: PgPool) {
    let username = if username.is_some() {
        username.unwrap()
    } else {
        Text::new("Username").prompt().unwrap()
    };
    let existing: Option<User> = sqlx::query_as(
        "
            SELECT username, password_hash, permissions
            FROM users
            WHERE username = $1
            ",
    )
    .bind(&username)
    .fetch_optional(&pool)
    .await
    .unwrap();
    if existing.is_some() {
        warn!("User exists, updating existing. Press ESC to skip prompts and leave unchanged.")
    }

    let password = Text::new("Password").prompt_skippable().unwrap();
    let password_hash = if let Some(password) = password {
        hash_password(&password).unwrap()
    } else if let Some(password_hash) = existing.clone().map(|x| x.password_hash) {
        password_hash.clone()
    } else {
        error!("Please provide a password.");
        exit(2)
    };

    let permissions = MultiSelect::new("Permissions", Permissions::all().iter().collect())
        .prompt_skippable()
        .unwrap();
    let permissions = if let Some(permissions) = permissions {
        permissions
            .into_iter()
            .fold(Permissions::empty(), |acc, flag| acc | flag)
    } else if let Some(permissions) = existing.map(|x| x.permissions) {
        permissions
    } else {
        error!("Please select some permissions.");
        exit(2)
    };

    let confirmation = Confirm::new("Confirm user creation with the above settings")
        .prompt()
        .unwrap();
    if !confirmation {
        info!("Cancelled.");
        exit(0);
    } else {
        let result = sqlx::query(
            "INSERT INTO users (username, password_hash, permissions) VALUES ($1, $2, $3)",
        )
        .bind(username)
        .bind(password_hash)
        .bind(i32::from(permissions))
        .fetch_all(&pool)
        .await;
        match result {
            Ok(_) => info!("User added succesfully!"),
            Err(e) => error!("Failed to add user: {e}"),
        }
    }
}

pub async fn userdel(username: Option<String>, pool: PgPool) {
    let username = if username.is_some() {
        username.unwrap()
    } else {
        Text::new("Username").prompt().unwrap()
    };

    let confirmation = Confirm::new("Are you sure you want to delete the user?")
        .prompt()
        .unwrap();
    if !confirmation {
        info!("Cancelled.");
        exit(0);
    } else {
        let result = sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .fetch_all(&pool)
            .await;
        match result {
            Ok(_) => info!("User deleted succesfully!"),
            Err(e) => error!("Failed to delete user: {e}"),
        }
    }
}

pub struct RequirePermissions<const REQ_BITS: u32>;

impl<S, const REQ_BITS: u32> FromRequestParts<S> for RequirePermissions<REQ_BITS>
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Extract the user context (typically placed there by your Auth Middleware)
        let user = parts.extensions.get::<User>().ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "Missing authentication context").into_response()
        })?;

        // 2. Decode the required bits representation
        let required = Permissions::from_bits_truncate(REQ_BITS);

        // 3. Perform the bitwise validation
        if user.permissions.contains(required) {
            Ok(RequirePermissions)
        } else {
            Err((
                StatusCode::FORBIDDEN,
                "Access denied: Insufficient permissions",
            )
                .into_response())
        }
    }
}

pub async fn require_auth(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|x| x.to_str().ok());
    let auth_header: &str = match auth_header {
        Some(str) if auth_header.is_some_and(|s| s.starts_with("Basic ")) => &str[6..str.len()],
        Some(_) => return Err(StatusCode::BAD_REQUEST),
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    macro_rules! bad_request {
        ($($arg:tt)+) => {{
            // Replace `log::error` with `tracing::error` or your preferred logger
            log::error!($($arg)+);
            StatusCode::BAD_REQUEST
        }};
    }

    let unbase64 = base64::prelude::BASE64_STANDARD
        .decode(auth_header)
        .map(String::from_utf8)
        .map_err(|e| bad_request!("Failed to decode base64 auth header {auth_header}: {e}"))?
        .map_err(|e| bad_request!("Failed to decode auth header bytes to string: {e}"))?;
    let split = unbase64.split(':').collect::<Vec<_>>();
    let username = split.first();
    let username = match username {
        Some(v) => v,
        None => return Err(bad_request!("Couldn't find username in {split:?}")),
    };
    let password = split.get(1);
    let password = match password {
        Some(v) => v,
        None => return Err(bad_request!("Couldn't find password in {split:?}")),
    };
    let password_hash =
        hash_password(password).map_err(|e| bad_request!("Failed to hash password: {e}"))?;

    let pool = req.extensions().get::<PgPool>().ok_or_else(|| {
        error!("PgPool is missing from request extensions!");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let db_pool: &PgPool = pool;

    let user: User = sqlx::query_as(
        "
        SELECT *
        FROM users
        WHERE username = $1 AND password_hash = $2
        ",
    )
    .bind(username)
    .bind(password_hash)
    .fetch_one(db_pool)
    .await
    .map_err(|e| {
        error!("Query for {username} failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
