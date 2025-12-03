//! Kāraṇa OS - Web Simulator Server
//!
//! A lightweight server that serves the beautiful web-based AR glasses simulator.
//! Run with: cargo run
//!
//! Then open http://localhost:3000 in your browser

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main() {
    println!();
    println!("\x1b[36m\x1b[1m🕶️  Kāraṇa OS - Smart Glasses Web Simulator\x1b[0m");
    println!("\x1b[90m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!();
    
    let listener = match TcpListener::bind("0.0.0.0:3000") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("❌ Failed to bind to port 3000: {}", e);
            eprintln!("   Try: sudo lsof -i :3000 to find what's using the port");
            return;
        }
    };
    
    println!("✨ \x1b[32mServer started successfully!\x1b[0m");
    println!();
    println!("📡 Open your browser and go to:");
    println!();
    println!("   \x1b[36m\x1b[1m➜  http://localhost:3000\x1b[0m");
    println!();
    println!("\x1b[90m━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\x1b[0m");
    println!("\x1b[90mPress Ctrl+C to stop the server\x1b[0m");
    println!();
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 4096];
    
    if stream.read(&mut buffer).is_err() {
        return;
    }
    
    let request = String::from_utf8_lossy(&buffer);
    
    // Parse the request path
    let path = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or("/");
    
    let (status, content_type, body) = match path {
        "/" | "/index.html" => {
            // Try to load the HTML file
            let html_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("static/index.html");
            
            match fs::read_to_string(&html_path) {
                Ok(contents) => {
                    println!("\x1b[32m✓\x1b[0m Serving simulator to client");
                    ("200 OK", "text/html", contents)
                }
                Err(_) => {
                    println!("\x1b[33m⚠\x1b[0m HTML file not found, using embedded version");
                    ("200 OK", "text/html", include_str!("../static/index.html").to_string())
                }
            }
        }
        "/favicon.ico" => {
            ("204 No Content", "image/x-icon", String::new())
        }
        _ => {
            println!("\x1b[31m✗\x1b[0m 404: {}", path);
            ("404 NOT FOUND", "text/plain", "404 Not Found".to_string())
        }
    };
    
    let response = format!(
        "HTTP/1.1 {}\r\n\
         Content-Type: {}; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Cache-Control: no-cache\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        status,
        content_type,
        body.len(),
        body
    );
    
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}
