// use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Passing {
    pub passing_number: String,
    pub transponder: String,
    pub date: String, // ISO 8601 format
    pub time: String,
    pub event_id: String,
    pub hits: String,
    pub max_rssi: String,
    pub internal_data: String, // hex
    pub is_active: String, // 1/0
    pub channel: String,
    pub loop_id: String,
    pub loop_id_wakeup: String,
    pub battery: String,
    pub temperature: String,
    pub internal_active_data: String, // hex
    pub box_temp: String,
    pub box_reader_id: String,
}

pub struct Decoder {
    ip: IpAddr,
    port: u16,
}

impl Decoder {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    pub async fn run(&self, tx: broadcast::Sender<Passing>) {
        loop {
            println!("Connecting to decoder at {}:{}", self.ip, self.port);
            match TcpStream::connect((self.ip, self.port)).await {
                Ok(socket) => {
                    println!("Connected to decoder");
                    if let Err(e) = self.handle_connection(socket, &tx).await {
                        eprintln!("Connection error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect: {}", e);
                }
            }
            // Wait before reconnecting
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn handle_connection(
        &self,
        socket: TcpStream,
        tx: &broadcast::Sender<Passing>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut framed = Framed::new(socket, LinesCodec::new());

        // Initialize protocol
        framed.send("SETPROTOCOL;2.0").await?;
        framed.send("SETPUSHPASSINGS;1;1").await?;

        // Ping interval
        let mut ping_interval = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                line = framed.next() => {
                    match line {
                        Some(Ok(msg)) => {
                            self.process_message(&msg, tx);
                        }
                        Some(Err(e)) => return Err(Box::new(e)),
                        None => return Err("Connection closed".into()),
                    }
                }
                _ = ping_interval.tick() => {
                    framed.send("PING").await?;
                }
            }
        }
    }

    fn process_message(&self, msg: &str, tx: &broadcast::Sender<Passing>) {
        // println!("Received: {}", msg);
        let parts: Vec<&str> = msg.split(';').collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "#P" => {
                // Format: #P;PassingNo;Transponder;Date;Time;EventID;Hits;MaxRSSI;InternalData;IsActive;Channel;LoopID;LoopIDWakeup;Battery;Temperature;InternalActiveData;BoxTemp;BoxReaderID
                // Note: Fields might be empty or missing depending on device.
                // We'll try to get as many as possible, defaulting to empty string.
                
                let get_part = |idx: usize| -> String {
                    parts.get(idx).unwrap_or(&"").to_string()
                };

                if parts.len() >= 5 {
                    let passing_number = get_part(1);
                    let transponder = get_part(2);
                    let date_str = get_part(3);
                    let time_str = get_part(4);
                    
                    // Combine date and time for ISO string
                    let iso_date = format!("{}T{}", date_str, time_str.replace(':', "."));

                    let passing = Passing {
                        passing_number,
                        transponder,
                        date: iso_date,
                        time: time_str,
                        event_id: get_part(5),
                        hits: get_part(6),
                        max_rssi: get_part(7),
                        internal_data: get_part(8),
                        is_active: get_part(9),
                        channel: get_part(10),
                        loop_id: get_part(11),
                        loop_id_wakeup: get_part(12),
                        battery: get_part(13),
                        temperature: get_part(14),
                        internal_active_data: get_part(15),
                        box_temp: get_part(16),
                        box_reader_id: get_part(17),
                    };
                    
                    println!("Passing: {:?}", passing);
                    let _ = tx.send(passing);
                }
            }
            "PING" => {
                // Ignore
            }
            _ => {
                // Ignore other messages
            }
        }
    }
}
