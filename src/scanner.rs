use local_ip_address::local_ip;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

pub async fn scan_for_decoder(port: u16, subnet_prefix: Option<String>) -> Option<IpAddr> {
    let subnet_prefix = if let Some(prefix) = subnet_prefix {
        prefix
    } else {
        let my_local_ip = match local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                eprintln!("Failed to get local IP: {}", e);
                return None;
            }
        };

        let IpAddr::V4(ipv4) = my_local_ip else {
            eprintln!("IPv6 not supported for scanning yet");
            return None;
        };

        let octets = ipv4.octets();
        format!("{}.{}.{}.", octets[0], octets[1], octets[2])
    };

    println!("Scanning subnet {}0/24 for port {}", subnet_prefix, port);

    let mut tasks = vec![];

    for i in 1..255 {
        let ip_str = format!("{}{}", subnet_prefix, i);
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            // if ip == my_local_ip {
            //     continue;
            // }
            tasks.push(check_ip(ip, port));
        }
    }

    // Run all checks concurrently
    let results = futures::future::join_all(tasks).await;

    for res in results {
        if let Some(ip) = res {
            println!("Found decoder at {}", ip);
            return Some(ip);
        }
    }

    println!("Decoder not found");
    None
}

async fn check_ip(ip: IpAddr, port: u16) -> Option<IpAddr> {
    let addr = SocketAddr::new(ip, port);
    let connect_future = TcpStream::connect(addr);
    
    // Short timeout for scanning
    match timeout(Duration::from_millis(200), connect_future).await {
        Ok(Ok(_)) => Some(ip),
        _ => None,
    }
}
