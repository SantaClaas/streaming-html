use std::convert::Infallible;

use tokio::sync::mpsc::Sender;

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn streamer(sender: Sender<Result<String, Infallible>>) {
    let mut counter = 0;
    loop {
        counter += 1;
        sender
            .send(Ok(format!("Hello, World! {}", counter)))
            .await
            .unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        if counter == 10 {
            break;
        }
    }
}

async fn handler() -> impl IntoResponse {
    let (sender, receiver) = tokio::sync::mpsc::channel(8);
    tokio::spawn(streamer(sender));

    // let stream = ReceiverStream::from(receiver);
    let stream = ReceiverStream::new(receiver);
    let body = axum::body::Body::from_stream(stream);
    Html(body)
}
