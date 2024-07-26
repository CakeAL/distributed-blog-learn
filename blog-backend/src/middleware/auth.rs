use std::sync::Arc;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts, Extension,
};
use blog_auth::Claims;

use crate::{handler::cookie, model::AppState};

pub struct Auth(Claims);

#[async_trait]
impl<S> FromRequestParts<S> for Auth
where
    S: Send + Sync,
{
    type Rejection = String;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(state) = Extension::<Arc<AppState>>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.to_string())?;
        let claims = match cookie::get(&parts.headers, "token") {
            Some(token) => state
                .jwt
                .verify_and_get(&token)
                .map_err(|err| err.to_string())?,
            None => return Err("请登录".to_string()),
        };
        Ok(Self(claims))
    }
}