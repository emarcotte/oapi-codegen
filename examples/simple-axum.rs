use axum::{response::Html, routing::get, Router};

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = std::net::SocketAddr::from(([127,0,0,1], 3000));

    let axum_server = axum::Server::bind(&listener)
            .serve(app.into_make_service());

    let duration = std::time::Duration::from_secs(2);
    let timeout_result = tokio::time::timeout(duration, axum_server).await;

    match timeout_result {
        Err(_) => {
            println!("example shutdown via timeout");
        },
        // Do nothing on natrual shut down.
        Ok(r) => {
            println!("Shutdown via something else! {:?}", r);
        }
    }
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
