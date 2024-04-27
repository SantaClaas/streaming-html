use std::convert::Infallible;

use tokio::sync::mpsc::{Sender, UnboundedSender};

use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use tokio_stream::wrappers::{ReceiverStream, UnboundedReceiverStream};

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

async fn streamer(sender: UnboundedSender<Result<String, Infallible>>) {
    // Write initial template
    // Notes on not obvious things:
    // - The template element has to nested within another element to attach the shadow root to
    // - The elements to slot have to be siblings of the template element
    // - Sibling elements next to the template will not get rendered
    //   Meaning only elements to be slotted should be placed as siblings to the template
    const TEMPLATE: &str = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Document</title>
    </head>
    <body>
    <main>
        <template shadowrootmode="open">
            <ol>
                <li>
                    <slot name="slot-1">
                        <span>Loading slot 1</span>
                    </slot>
                </li>
                <li>
                    <slot name="slot-2">
                        <span>Loading slot 2</span>
                    </slot>
                </li>
                <li>
                    <slot name="slot-3">
                        <span>Loading slot 3</span>
                    </slot>
                </li>
                <li>
                    <slot name="slot-4">
                        <span>Loading slot 4</span>
                    </slot>
                </li>
            </ol>
            <form>
              <fieldset>
                <legend>Choose your favorite monster</legend>

                <input type="radio" id="kraken" name="monster" value="K" />
                <label for="kraken">Kraken</label><br />

                <input type="radio" id="sasquatch" name="monster" value="S" />
                <label for="sasquatch">Sasquatch</label><br />

                <input type="radio" id="mothman" name="monster" value="M" />
                <label for="mothman">Mothman</label>
              </fieldset>
            </form>
        </template>

    "#;

    sender.send(Ok(TEMPLATE.to_string())).unwrap();
    // Showcase out of order
    let slot = |slot: u8| format!("<span slot=\"slot-{}\">Slot {}</span>", slot, slot);

    let next = slot(3);
    sender.send(Ok(next)).unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let next = slot(1);
    sender.send(Ok(next)).unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let next = slot(4);
    sender.send(Ok(next)).unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let next = slot(2);
    sender.send(Ok(next)).unwrap();

    // Close the document
    sender
        .send(Ok("</main></body></html>".to_string()))
        .unwrap();
}

async fn handler() -> impl IntoResponse {
    // We know the response is max size will be the HTML document size
    // so we can use an unbounded channel
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(streamer(sender));

    let stream = UnboundedReceiverStream::new(receiver);
    let body = axum::body::Body::from_stream(stream);
    Html(body)
}
