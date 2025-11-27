mod decoder;
mod scanner;
mod server;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use tokio::sync::broadcast;
use crate::decoder::WsMessage;

#[tokio::main]
async fn main() {
    println!("Starting rrconverter...");

    let (tx, _rx) = broadcast::channel::<WsMessage>(100);
    let is_connected = Arc::new(AtomicBool::new(false));

    // Start WebSocket server in the background
    let tx_clone = tx.clone();
    let is_connected_clone = is_connected.clone();
    tokio::spawn(async move {
        server::start_server(tx_clone, 8080, is_connected_clone).await;
    });

    loop {
        println!("Scanning for decoder...");
        if let Some(ip) = scanner::scan_for_decoder(3601).await {
            println!("Found decoder at {}", ip);
            let decoder = decoder::Decoder::new(ip, 3601);
            
            // Run decoder. If it returns, it means it disconnected.
            decoder.run(tx.clone(), is_connected.clone()).await;
            
            // Disconnected
            println!("Decoder disconnected, rescanning...");
            // Status updates are now handled inside decoder.run

        } else {
            println!("Decoder not found, retrying in 5 seconds...");
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}
