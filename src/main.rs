mod decoder;
mod scanner;
mod server;

use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    println!("Starting rrconverter...");

    let (tx, _rx) = broadcast::channel(100);

    // Start WebSocket server in the background
    // Note: In the plan I said "Spawn Decoder and Server" after scan.
    // But the server can probably run independently, or it should only run when we have a connection?
    // The request said "host a websocket server on which it can pass on the received passings".
    // It's probably fine to have the server always running, or running once we start.
    // Let's spawn it once globally.
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        server::start_server(tx_clone, 8080).await;
    });

    loop {
        println!("Scanning for decoder...");
        if let Some(ip) = scanner::scan_for_decoder(3601).await {
            println!("Found decoder at {}", ip);
            let decoder = decoder::Decoder::new(ip, 3601);
            
            // Run decoder. If it returns, it means it disconnected.
            decoder.run(tx.clone()).await;
            
            println!("Decoder disconnected, rescanning...");
        } else {
            println!("Decoder not found, retrying in 5 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}
