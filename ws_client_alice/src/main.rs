use serde_json::json;
use futures::{SinkExt, StreamExt}; // Import both traits
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use serde::Deserialize;
use std::io::{self, Write}; // Import `Write` for flushing stdout

#[derive(Debug, Deserialize)]
struct MessagePayload {
    message_id: String,
    sender: String,
    recipient: String,
    content: String,
    timestamp: Option<String>, // Handle the timestamp field from the server
}

#[tokio::main]
async fn main() {
    let server_url = "ws://127.0.0.1:3000/ws/alice";

    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(server_url).await.expect("Failed to connect");
    println!("Connected to the server at {}", server_url);

    let (mut write, mut read) = ws_stream.split();

    // Spawn a task to read messages from the server
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Attempt to deserialize the message payload
                    match serde_json::from_str::<MessagePayload>(&text) {
                        Ok(payload) => {
                            println!(
                                "Received message from {}: {} [timestamp: {}]",
                                payload.sender,
                                payload.content,
                                payload.timestamp.unwrap_or_else(|| "N/A".to_string())
                            );
                        }
                        Err(_) => println!("Received: {}", text),
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("Server closed the connection");
                    break;
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Read input from the terminal and send messages
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        print!("Enter message (or type 'exit' to disconnect): ");
        io::stdout().flush().unwrap(); // Flush the prompt to stdout

        input.clear();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            println!("Disconnecting...");
            let disconnect_payload = json!({
                "action_type": "disconnect",
                "user": "alice",
            });
            write
                .send(Message::Text(disconnect_payload.to_string().into()))
                .await
                .expect("Failed to send disconnect message");
            break;
        }

        // Construct a message payload
        let message_payload = json!({
            "action_type": "send_message",
            "message_id": uuid::Uuid::new_v4().to_string(),
            "sender": "alice",
            "recipient": "bob",
            "content": input,
        });

        write
            .send(Message::Text(message_payload.to_string().into()))
            .await
            .expect("Failed to send message");
    }

    println!("Disconnected from the server.");
}
