use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use log::{error, info, warn};
use reqwest::header;
use sqlx::PgPool;

use crate::{
    api::ApiError,
    entities::{Permissions, User},
};

const BASIC_PREFIX: &str = "Basic ";

fn verify_password(password: &str, encoded_hash: &str) -> Result<(), String> {
    let hash = PasswordHash::new(encoded_hash).map_err(|error| error.to_string())?;

    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .map_err(|error| error.to_string())
}

async fn find_user(pool: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        "SELECT username, password_hash, permissions
         FROM users
         WHERE username = $1",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

fn decode_basic_credentials(request: &Request) -> Result<(String, String), ApiError> {
    let header = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(ApiError::Unauthorized)?
        .to_str()
        .map_err(|_| ApiError::bad_request("invalid authorization header"))?;

    let encoded = header
        .strip_prefix(BASIC_PREFIX)
        .ok_or(ApiError::bad_request(
            "authorization must use Basic authentication",
        ))?;
    let decoded = BASE64.decode(encoded).map_err(|error| {
        error!("Failed to decode authorization header: {error}");
        ApiError::bad_request("invalid base64 credentials")
    })?;
    let credentials = String::from_utf8(decoded).map_err(|error| {
        error!("Authorization header is not UTF-8: {error}");
        ApiError::bad_request("credentials are not valid UTF-8")
    })?;
    let (username, password) = credentials.split_once(':').ok_or_else(|| {
        error!("Authorization header does not contain a username/password separator");
        ApiError::bad_request("credentials must contain a username and password")
    })?;

    Ok((username.to_owned(), password.to_owned()))
}

pub struct RequirePermissions<const REQUIRED_BITS: u32>;

impl<S, const REQUIRED_BITS: u32> FromRequestParts<S> for RequirePermissions<REQUIRED_BITS>
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let user = parts
            .extensions
            .get::<User>()
            .ok_or(ApiError::Unauthorized)?;
        let required = Permissions::from_bits_truncate(REQUIRED_BITS);

        if user.permissions.contains(required) {
            Ok(Self)
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

pub async fn require_auth(
    State(state): State<crate::api::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let (username, password) = decode_basic_credentials(&request)?;
    let user = find_user(&state.database, &username)
        .await
        .map_err(|error| {
            error!("Query for {username} failed: {error}");
            ApiError::Internal
        })?
        .ok_or_else(|| {
            warn!("Authentication failed for unknown user {username}");
            ApiError::Unauthorized
        })?;

    if verify_password(&password, &user.password_hash).is_err() {
        warn!("Authentication failed for user {username}");
        return Err(ApiError::Unauthorized);
    }

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}

pub async fn request_logger(request: Request, next: Next) -> Response {
    info!("Got a request for {}", request.uri());
    next.run(request).await
}

#[cfg(test)]
mod tests {
    mod decode_basic_credentials {
        use axum::{
            body::Body,
            extract::Request,
            http::{HeaderValue, header::AUTHORIZATION},
        };
        use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

        use crate::api::{ApiError, auth::decode_basic_credentials};

        fn request_with_authorization(value: &str) -> Request {
            Request::builder()
                .header(AUTHORIZATION, value)
                .body(Body::empty())
                .expect("request should be valid")
        }

        #[test]
        fn path_ok_basic_credentials() {
            // Given
            let encoded = BASE64.encode("alice:correct horse battery staple");
            let request = request_with_authorization(&format!("Basic {encoded}"));

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_ok());
            assert_eq!(
                (
                    "alice".to_owned(),
                    "correct horse battery staple".to_owned()
                ),
                result.expect("credentials should decode")
            );
        }

        #[test]
        fn anxiety_password_contains_colon() {
            // Given
            let encoded = BASE64.encode("alice:part:part");
            let request = request_with_authorization(&format!("Basic {encoded}"));

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_ok());
            assert_eq!(
                ("alice".to_owned(), "part:part".to_owned()),
                result.expect("credentials should split at the first colon")
            );
        }

        #[test]
        fn path_err_authorization_header_missing() {
            // Given
            let request = Request::new(Body::empty());

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::Unauthorized)));
        }

        #[test]
        fn path_err_authorization_header_is_not_text() {
            // Given
            let request = Request::builder()
                .header(
                    AUTHORIZATION,
                    HeaderValue::from_bytes(&[0x80]).expect("header should accept an opaque value"),
                )
                .body(Body::empty())
                .expect("request should be valid");

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_err());
            match result.expect_err("non-text header should fail") {
                ApiError::BadRequest { message } => {
                    assert_eq!("invalid authorization header", message);
                }
                error => panic!("expected BadRequest, got {error:?}"),
            }
        }

        #[test]
        fn path_err_authentication_scheme_is_not_basic() {
            // Given
            let request = request_with_authorization("Bearer token");

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_err());
            match result.expect_err("non-Basic scheme should fail") {
                ApiError::BadRequest { message } => {
                    assert_eq!("authorization must use Basic authentication", message);
                }
                error => panic!("expected BadRequest, got {error:?}"),
            }
        }

        #[test]
        fn path_err_credentials_are_not_base64() {
            // Given
            let request = request_with_authorization("Basic !!!");

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_err());
            match result.expect_err("invalid base64 should fail") {
                ApiError::BadRequest { message } => {
                    assert_eq!("invalid base64 credentials", message);
                }
                error => panic!("expected BadRequest, got {error:?}"),
            }
        }

        #[test]
        fn path_err_credentials_are_not_utf8() {
            // Given
            let encoded = BASE64.encode([0xff]);
            let request = request_with_authorization(&format!("Basic {encoded}"));

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_err());
            match result.expect_err("non-UTF-8 credentials should fail") {
                ApiError::BadRequest { message } => {
                    assert_eq!("credentials are not valid UTF-8", message);
                }
                error => panic!("expected BadRequest, got {error:?}"),
            }
        }

        #[test]
        fn path_err_credentials_have_no_separator() {
            // Given
            let encoded = BASE64.encode("alice");
            let request = request_with_authorization(&format!("Basic {encoded}"));

            // When
            let result = decode_basic_credentials(&request);

            // Then
            assert!(result.is_err());
            match result.expect_err("credentials without a colon should fail") {
                ApiError::BadRequest { message } => {
                    assert_eq!("credentials must contain a username and password", message);
                }
                error => panic!("expected BadRequest, got {error:?}"),
            }
        }
    }

    mod require_permissions {
        use axum::{
            body::Body,
            extract::FromRequestParts,
            http::{Request, request::Parts},
        };

        use crate::{
            api::{ApiError, auth::RequirePermissions},
            entities::{Permissions, User},
        };

        fn request_parts(user: Option<User>) -> Parts {
            let mut request = Request::new(Body::empty());
            if let Some(user) = user {
                request.extensions_mut().insert(user);
            }
            request.into_parts().0
        }

        fn user_with_permissions(permissions: Permissions) -> User {
            User {
                username: "test".into(),
                password_hash: "hash".into(),
                permissions,
            }
        }

        #[tokio::test]
        async fn path_ok_user_has_required_permission() {
            // Given
            let user = user_with_permissions(Permissions::STOCK_READ | Permissions::SALES_READ);
            let mut parts = request_parts(Some(user));

            // When
            let result =
                RequirePermissions::<{ Permissions::STOCK_READ.bits() }>::from_request_parts(
                    &mut parts,
                    &(),
                )
                .await;

            // Then
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn path_ok_no_permissions_are_required() {
            // Given
            let user = user_with_permissions(Permissions::empty());
            let mut parts = request_parts(Some(user));

            // When
            let result = RequirePermissions::<0>::from_request_parts(&mut parts, &()).await;

            // Then
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn path_err_user_is_missing() {
            // Given
            let mut parts = request_parts(None);

            // When
            let result =
                RequirePermissions::<{ Permissions::STOCK_READ.bits() }>::from_request_parts(
                    &mut parts,
                    &(),
                )
                .await;

            // Then
            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::Unauthorized)));
        }

        #[tokio::test]
        async fn path_err_user_lacks_required_permission() {
            // Given
            let user = user_with_permissions(Permissions::SALES_READ);
            let mut parts = request_parts(Some(user));

            // When
            let result =
                RequirePermissions::<{ Permissions::STOCK_READ.bits() }>::from_request_parts(
                    &mut parts,
                    &(),
                )
                .await;

            // Then
            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::Forbidden)));
        }

        #[tokio::test]
        async fn path_err_user_has_only_some_required_permissions() {
            // Given
            let user = user_with_permissions(Permissions::STOCK_READ);
            let mut parts = request_parts(Some(user));
            const REQUIRED: u32 = Permissions::STOCK_READ.bits() | Permissions::STOCK_WRITE.bits();

            // When
            let result = RequirePermissions::<REQUIRED>::from_request_parts(&mut parts, &()).await;

            // Then
            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::Forbidden)));
        }
    }

    mod verify_password {
        use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

        use crate::api::auth::verify_password;

        fn hash_password(password: &str) -> String {
            let salt = SaltString::from_b64("dGVzdHNhbHQ").expect("test salt should be valid");
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .expect("password should hash")
                .to_string()
        }

        #[test]
        fn path_ok_password_matches_hash() {
            // Given
            let password = "test";
            let encoded_hash = hash_password(password);

            // When
            let result = verify_password(password, &encoded_hash);

            // Then
            assert!(result.is_ok());
            assert_eq!((), result.expect("matching password should verify"));
        }

        #[test]
        fn path_err_password_does_not_match_hash() {
            // Given
            let encoded_hash = hash_password("test");

            // When
            let result = verify_password("wrong", &encoded_hash);

            // Then
            assert!(result.is_err());
            assert_eq!(
                "invalid password",
                result.expect_err("incorrect password should fail")
            );
        }

        #[test]
        fn path_err_hash_is_malformed() {
            // Given
            let encoded_hash = "not-a-password-hash";

            // When
            let result = verify_password("test", encoded_hash);

            // Then
            assert!(result.is_err());
            assert_eq!(
                "password hash string missing field",
                result.expect_err("malformed hash should fail")
            );
        }
    }
}
