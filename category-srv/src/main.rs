use std::env;

use blog_proto::category_service_server::CategoryServiceServer;

mod server;

#[tokio::main]
async fn main() {
    let addr = "[::1]:19527";
    println!("category-srv run at: {}", addr);

    let dsn =
        env::var("PG_DSN").unwrap_or("postgres://cakeal:20030214@localhost:5432/distributed-blog".to_string());
    let pool = sqlx::postgres::PgPool::connect(&dsn).await.unwrap();
    let category_srv = server::Category::new(pool);
    tonic::transport::Server::builder()
        .add_service(CategoryServiceServer::new(category_srv))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
