use std::sync::Arc;

use axum::{routing::get, Extension, Router};
use blog_proto::{
    category_service_client::CategoryServiceClient, topic_service_client::TopicServiceClient,
};
use tera::Tera;
use tokio::net::TcpListener;

mod handler;
mod model;

#[tokio::main]
async fn main() {
    let addr = "[::1]:19529";

    let cate = CategoryServiceClient::connect("http://[::1]:19527")
        .await
        .unwrap();

    let topic = TopicServiceClient::connect("http://[::1]:19528")
        .await
        .unwrap();

    let tera = Tera::new("blog-frontend/templates/*.html").unwrap();

    let app = Router::new()
        .route("/", get(handler::index))
        .route("/detail/:id", get(handler::detail))
        .layer(Extension(Arc::new(model::AppState::new(cate, topic, tera))));

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
