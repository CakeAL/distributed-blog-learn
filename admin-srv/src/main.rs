use std::env;

use blog_proto::admin_service_server::AdminServiceServer;

mod server;
#[tokio::main]
async fn main() {
    let addr = "[::1]:19530";
    println!("admin-srv runs at: {}", addr);

    let dsn = env::var("PG_DSN")
        .unwrap_or("postgres://cakeal:20030214@localhost:5432/distributed-blog".to_string());
    let pool = sqlx::postgres::PgPool::connect(&dsn).await.unwrap();
    let srv = server::Admin::new(pool);
    tonic::transport::Server::builder()
        .add_service(AdminServiceServer::new(srv))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
