use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use crate::decoder::{WsMessage, Passing};

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct JsonPassingInner {
    Transponder: String,
    Hits: Option<i64>,
    RSSI: Option<i64>,
    Battery: Option<f64>,
    Temperature: Option<f64>,
    LoopID: Option<i64>,
    Channel: Option<i64>,
    InternalData: Option<String>,
    PassingNo: Option<i64>,
    UTCTime: String, // "2024-01-12T09:06:35.944Z"
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct JsonPassingWrapper {
    Passing: JsonPassingInner,
}

pub async fn run_server(tx: broadcast::Sender<WsMessage>, port: u16, is_connected: Arc<AtomicBool>) {
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind JSON server to {}: {}", addr, e);
            return;
        }
    };

    println!("JSON Server listening on {}", addr);

    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
                continue;
            }
        };

        println!("New JSON client connection from {}", addr);
        
        // Mark as connected when a client connects
        is_connected.store(true, Ordering::SeqCst);
        let _ = tx.send(WsMessage::Status { event: "connected".to_string() });

        let tx = tx.clone();
        let is_connected = is_connected.clone();

        tokio::spawn(async move {
            let reader = BufReader::new(socket);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }

                match serde_json::from_str::<JsonPassingWrapper>(&line) {
                    Ok(wrapper) => {
                        let inner = wrapper.Passing;
                        
                        // Parse UTCTime to date and time
                        // Format: "2024-01-12T09:06:35.944Z"
                        let (date_str, time_str) = if let Some((d, t)) = inner.UTCTime.split_once('T') {
                            (d.to_string(), t.trim_end_matches('Z').to_string())
                        } else {
                            ("".to_string(), "".to_string())
                        };

                        let passing = Passing {
                            passing_number: inner.PassingNo.map(|v| v.to_string()).unwrap_or_default(),
                            transponder: inner.Transponder,
                            date: inner.UTCTime.clone(), // Keep full ISO string for date field as per previous logic? 
                                                         // Wait, decoder.rs combines them: let iso_date = format!("{}T{}", date_str, time_str);
                                                         // But here we already have ISO.
                                                         // Let's check decoder.rs Passing struct usage.
                                                         // In decoder.rs: pub date: String, // ISO 8601 format
                                                         // So we should put the full ISO string in `date`.
                            time: time_str,
                            event_id: "".to_string(),
                            hits: inner.Hits.map(|v| v.to_string()).unwrap_or_default(),
                            max_rssi: inner.RSSI.map(|v| v.to_string()).unwrap_or_default(),
                            internal_data: inner.InternalData.unwrap_or_default(),
                            is_active: "1".to_string(), // Assume active
                            channel: inner.Channel.map(|v| v.to_string()).unwrap_or_default(),
                            loop_id: inner.LoopID.map(|v| v.to_string()).unwrap_or_default(),
                            loop_id_wakeup: "".to_string(),
                            battery: inner.Battery.map(|v| v.to_string()).unwrap_or_default(),
                            temperature: inner.Temperature.map(|v| v.to_string()).unwrap_or_default(),
                            internal_active_data: "".to_string(),
                            box_temp: "".to_string(),
                            box_reader_id: "".to_string(),
                        };

                        println!("JSON Passing: {:?}", passing);
                        if let Err(e) = tx.send(WsMessage::Passing(passing)) {
                            eprintln!("Error broadcasting passing: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing JSON: {}. Data: {}", e, line);
                    }
                }
            }

            println!("JSON client {} disconnected", addr);
            // Only mark disconnected if this was the active connection? 
            // For simplicity, we'll mark disconnected. 
            // If multiple clients are supported, this might be flaky, but usually only one source.
            is_connected.store(false, Ordering::SeqCst);
            let _ = tx.send(WsMessage::Status { event: "disconnected".to_string() });
        });
    }
}
