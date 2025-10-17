use backend::create_app;

#[tokio::main]
async fn main() {
    let app = create_app().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();

    println!("Backend server running on http://localhost:8888");
    println!("OpenAPI JSON available at http://localhost:8888/api/openapi.json");
    println!("Swagger UI available at http://localhost:8888/api/docs");
    axum::serve(listener, app).await.unwrap();
}
