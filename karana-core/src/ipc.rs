use std::sync::mpsc::Sender;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn start_ipc_server(port: u16, tx: Sender<String>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    log::info!("IPC Server listening on 0.0.0.0:{}", port);

    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut socket, addr)) => {
                    log::info!("IPC: New connection from {}", addr);
                    let tx_clone = tx.clone();
                    
                    tokio::spawn(async move {
                        let (reader, _) = socket.split();
                        let mut buf_reader = BufReader::new(reader);
                        let mut line = String::new();

                        loop {
                            line.clear();
                            match buf_reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF
                                Ok(_) => {
                                    let intent = line.trim().to_string();
                                    if !intent.is_empty() {
                                        log::info!("IPC: Received intent: '{}'", intent);
                                        if let Err(e) = tx_clone.send(intent) {
                                            log::error!("IPC: Failed to forward intent: {}", e);
                                            break;
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("IPC: Read error: {}", e);
                                    break;
                                }
                            }
                        }
                        log::info!("IPC: Connection closed for {}", addr);
                    });
                }
                Err(e) => {
                    log::error!("IPC: Accept error: {}", e);
                }
            }
        }
    });

    Ok(())
}
