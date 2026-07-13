use std::process::exit;

use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use inquire::{Confirm, MultiSelect, Text};
use log::{error, info, warn};
use sqlx::PgPool;

use crate::entities::Permissions;

fn hash_password(password: &str, salt: &str) -> Result<String, String> {
    let salt = SaltString::from_b64(salt).map_err(|error| error.to_string())?;

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| error.to_string())
}

async fn find_user(
    pool: &PgPool,
    username: &str,
) -> Result<Option<crate::entities::User>, sqlx::Error> {
    sqlx::query_as::<_, crate::entities::User>(
        "SELECT username, password_hash, permissions
         FROM users
         WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

fn prompt_username(username: Option<String>) -> String {
    username.unwrap_or_else(|| {
        Text::new("Username").prompt().unwrap_or_else(|error| {
            error!("Failed to read username: {error}");
            exit(2);
        })
    })
}

fn prompt_permissions(existing: Option<Permissions>) -> Permissions {
    let selected = MultiSelect::new("Permissions", Permissions::all().iter().collect())
        .prompt_skippable()
        .unwrap_or_else(|error| {
            error!("Failed to read permissions: {error}");
            exit(2);
        });

    match selected {
        Some(values) => values
            .into_iter()
            .fold(Permissions::empty(), |permissions, value| {
                permissions | value
            }),
        None => existing.unwrap_or_else(|| {
            error!("Please select some permissions.");
            exit(2);
        }),
    }
}

pub async fn useradd(username: Option<String>, pool: PgPool, salt: &str) {
    let username = prompt_username(username);
    let existing = match find_user(&pool, &username).await {
        Ok(user) => user,
        Err(error) => {
            error!("Failed to look up user {username}: {error}");
            exit(2);
        }
    };

    if existing.is_some() {
        warn!("User exists; updating it. Press ESC to keep each existing value.");
    }

    let password = Text::new("Password")
        .prompt_skippable()
        .unwrap_or_else(|error| {
            error!("Failed to read password: {error}");
            exit(2);
        });
    let password_hash = match password {
        Some(password) => hash_password(&password, salt).unwrap_or_else(|error| {
            error!("Failed to hash password: {error}");
            exit(2);
        }),
        None => existing
            .as_ref()
            .map(|user| user.password_hash.clone())
            .unwrap_or_else(|| {
                error!("Please provide a password.");
                exit(2);
            }),
    };

    let permissions = prompt_permissions(existing.as_ref().map(|user| user.permissions));
    let confirmed = Confirm::new("Confirm user creation with the above settings")
        .prompt()
        .unwrap_or_else(|error| {
            error!("Failed to read confirmation: {error}");
            exit(2);
        });

    if !confirmed {
        info!("Cancelled.");
        return;
    }

    match sqlx::query(
        "INSERT INTO users (username, password_hash, permissions)
         VALUES ($1, $2, $3)
         ON CONFLICT (username) DO UPDATE
         SET password_hash = EXCLUDED.password_hash,
             permissions = EXCLUDED.permissions",
    )
    .bind(&username)
    .bind(password_hash)
    .bind(i32::from(permissions))
    .execute(&pool)
    .await
    {
        Ok(_) => info!("User added successfully!"),
        Err(error) => error!("Failed to add user: {error}"),
    }
}

pub async fn userdel(username: Option<String>, pool: PgPool) {
    let username = prompt_username(username);
    let confirmed = Confirm::new("Are you sure you want to delete the user?")
        .prompt()
        .unwrap_or_else(|error| {
            error!("Failed to read confirmation: {error}");
            exit(2);
        });

    if !confirmed {
        info!("Cancelled.");
        return;
    }

    match sqlx::query("DELETE FROM users WHERE username = $1")
        .bind(&username)
        .execute(&pool)
        .await
    {
        Ok(result) if result.rows_affected() == 0 => warn!("User {username} does not exist."),
        Ok(_) => info!("User deleted successfully!"),
        Err(error) => error!("Failed to delete user: {error}"),
    }
}
