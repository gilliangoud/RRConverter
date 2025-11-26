use crate::decoder::Passing;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use warp::Filter;

pub async fn start_server(tx: broadcast::Sender<Passing>, port: u16) {
    let tx = warp::any().map(move || tx.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(tx)
        .map(|ws: warp::ws::Ws, tx| {
            ws.on_upgrade(move |socket| handle_connection(socket, tx))
        })
        .boxed();

    println!("WebSocket server listening on 0.0.0.0:{}", port);
    warp::serve(ws_route).run(([0, 0, 0, 0], port)).await;
}

async fn handle_connection(ws: warp::ws::WebSocket, tx: broadcast::Sender<Passing>) {
    let (mut ws_tx, _) = ws.split();
    let mut rx = tx.subscribe();

    println!("New WebSocket client connected");

    while let Ok(passing) = rx.recv().await {
        if let Ok(json) = serde_json::to_string(&passing) {
            if let Err(e) = ws_tx.send(warp::ws::Message::text(json)).await {
                eprintln!("WebSocket send error: {}", e);
                break;
            }
        }
    }
    println!("WebSocket client disconnected");
}
