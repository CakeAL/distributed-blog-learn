use axum::http::{HeaderMap, StatusCode};

pub use auth::*;
pub use cate::*;

mod auth;
mod cate;

pub async fn index() -> (StatusCode, HeaderMap) {
    redirect("/login")
}

pub fn redirect(url: &str) -> (StatusCode, HeaderMap) {
    redirect_with_cookie(url, None)
}

pub fn redirect_with_cookie(url: &str, cookie: Option<&str>) -> (StatusCode, HeaderMap) {
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::LOCATION, url.parse().unwrap());
    if let Some(cookie) = cookie {
        headers.insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    }
    (StatusCode::FOUND, headers)
}
