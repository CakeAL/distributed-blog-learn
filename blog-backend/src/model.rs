use blog_auth::Jwt;
use blog_proto::{
    admin_service_client::AdminServiceClient, category_service_client::CategoryServiceClient, topic_service_client::TopicServiceClient
};
use tera::Tera;

pub struct AppState {
    pub cate: CategoryServiceClient<tonic::transport::Channel>,
    pub topic: TopicServiceClient<tonic::transport::Channel>,
    pub admin: AdminServiceClient<tonic::transport::Channel>,
    pub tera: Tera,
    pub jwt: Jwt,
}

impl AppState {
    pub fn new(
        cate: CategoryServiceClient<tonic::transport::Channel>,
        topic: TopicServiceClient<tonic::transport::Channel>,
        admin: AdminServiceClient<tonic::transport::Channel>,
        tera: Tera,
        jwt: Jwt,
    ) -> Self {
        Self { cate, topic, admin, tera, jwt }
    }
}