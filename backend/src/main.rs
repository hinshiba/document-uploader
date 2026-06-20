#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }));

    axum::serve(listener, app).await?;

    Ok(())
}
