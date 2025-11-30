mod decoder;
mod scanner;
mod server;
mod json_server;

use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use tokio::sync::broadcast;
use crate::decoder::WsMessage;

use clap::Parser;
use std::net::IpAddr;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Optional IP range to scan (e.g. "192.168.1.")
    #[arg(long)]
    ip_range: Option<String>,

    /// Specific decoder IP to connect to (skips scanning)
    #[arg(long)]
    decoder_ip: Option<IpAddr>,

    /// Decoder port
    #[arg(long, default_value_t = 3601)]
    decoder_port: u16,

    /// Enable JSON Server Mode (listens for JSON connections instead of connecting to decoder)
    #[arg(long, default_value_t = false)]
    listen_mode: bool,

    /// Port to listen on in JSON Server Mode
    #[arg(long, default_value_t = 3602)]
    listen_port: u16,

    /// Enable debug logging
    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("Starting rrconverter...");

    let (tx, _rx) = broadcast::channel::<WsMessage>(100);
    let is_connected = Arc::new(AtomicBool::new(false));

    // Start WebSocket server in the background
    let tx_clone = tx.clone();
    let is_connected_clone = is_connected.clone();
    tokio::spawn(async move {
        server::start_server(tx_clone, 8080, is_connected_clone).await;
    });

    if args.listen_mode {
        println!("Starting in JSON Server Mode...");
        json_server::run_server(tx, args.listen_port, is_connected, args.debug).await;
    } else {
        loop {
            let ip = if let Some(ip) = args.decoder_ip {
                println!("Using specified decoder IP: {}", ip);
                Some(ip)
            } else {
                println!("Scanning for decoder...");
                scanner::scan_for_decoder(args.decoder_port, args.ip_range.clone()).await
            };

            if let Some(ip) = ip {
                println!("Found decoder at {}", ip);
                let decoder = decoder::Decoder::new(ip, args.decoder_port);
                
                // Run decoder. If it returns, it means it disconnected.
                decoder.run(tx.clone(), is_connected.clone()).await;
                
                // Disconnected
                println!("Decoder disconnected, retrying...");
                // Status updates are now handled inside decoder.run

            } else {
                println!("Decoder not found, retrying in 5 seconds...");
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
