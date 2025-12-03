//! Kāraṇa OS Smart Glasses - Web Simulator Server
//!
//! Serves the beautiful web-based AR glasses simulator.
//! Run with: cargo run --example web_simulator
//!
//! Then open http://localhost:3000 in your browser

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

fn main() {
    println!("\n🕶️  Kāraṇa OS - Smart Glasses Web Simulator");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    let listener = TcpListener::bind("127.0.0.1:3000").expect("Failed to bind to port 3000");
    
    println!("✨ Server started!");
    println!("📡 Open your browser and go to:");
    println!();
    println!("   \x1b[36m\x1b[1mhttp://localhost:3000\x1b[0m");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Press Ctrl+C to stop the server");
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
    let mut buffer = [0; 1024];
    
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
            // Serve the main HTML file
            let html_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("examples/web_simulator/index.html");
            
            match fs::read_to_string(&html_path) {
                Ok(contents) => ("200 OK", "text/html", contents),
                Err(_) => {
                    // If file not found, serve embedded HTML
                    ("200 OK", "text/html", get_embedded_html())
                }
            }
        }
        _ => {
            ("404 NOT FOUND", "text/plain", "404 Not Found".to_string())
        }
    };
    
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
        status,
        content_type,
        body.len(),
        body
    );
    
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
    
    // Log request
    if path == "/" || path == "/index.html" {
        println!("📱 Client connected - serving simulator");
    }
}

fn get_embedded_html() -> String {
    // Fallback embedded HTML if file not found
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Kāraṇa OS Simulator</title>
    <style>
        body { 
            font-family: system-ui; 
            background: #0a0a1a; 
            color: #fff; 
            display: flex; 
            justify-content: center; 
            align-items: center; 
            height: 100vh; 
            margin: 0;
        }
        .container { text-align: center; }
        h1 { font-size: 3em; margin-bottom: 20px; }
        p { color: #888; }
        .reload { 
            margin-top: 20px;
            padding: 15px 30px;
            background: linear-gradient(90deg, #00d4ff, #7b68ee);
            border: none;
            border-radius: 30px;
            color: white;
            font-size: 16px;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🕶️ Kāraṇa OS</h1>
        <p>Smart Glasses Simulator</p>
        <p>Please rebuild the example to load the full simulator.</p>
        <button class="reload" onclick="location.reload()">Reload</button>
    </div>
</body>
</html>"#.to_string()
}
