use axum::{routing::get, Router};
use std::env;

// #[tokio::main] is a macro that sets up the asynchronous runtime for us.
// It wraps our main function so we can use `async` and `await`.
#[tokio::main]
async fn main() {
    // 1. Build our application router with a single route.
    // When someone visits the root URL ("/"), it returns a simple text string.
    let app = Router::new()
        .route("/", get(|| async { "Hello, World! RustyTasks is live!" }));

    // 2. Cloud platforms like Render assign a specific PORT dynamically using an environment variable.
    // Here we try to read "PORT". If it doesn't exist (like on your local machine), we default to "3000".
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    
    // 3. We bind to "0.0.0.0" instead of "127.0.0.1" (localhost) so external traffic can reach the app in the cloud.
    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", addr);

    // 4. Create a TCP listener on the specified address.
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    // 5. Start serving the app!
    axum::serve(listener, app).await.unwrap();
}