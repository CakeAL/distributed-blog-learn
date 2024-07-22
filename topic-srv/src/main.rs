use std::env;

use blog_proto::topic_service_server::TopicServiceServer;

mod server;

#[tokio::main]
async fn main() {
    let addr = "[::1]:19528";
    println!("topic-srv run at: {}", addr);

    let dsn = env::var("PG_DSN")
        .unwrap_or("postgres://cakeal:20030214@localhost:5432/distributed-blog".to_string());
    let pool = sqlx::postgres::PgPool::connect(&dsn).await.unwrap();
    let topic_srv = server::Topic::new(pool);
    tonic::transport::Server::builder()
        .add_service(TopicServiceServer::new(topic_srv))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
