use std::net::TcpStream;
use tungstenite::{Error, Message, WebSocket};

pub struct WebSocketHandler {
    websockets: Vec<WebSocket<TcpStream>>,
}

impl WebSocketHandler {
    pub fn new() -> Self {
        Self {
            websockets: Vec::new(),
        }
    }

    pub fn add_websocket(&mut self, websocket: WebSocket<TcpStream>) {
        self.websockets.push(websocket);
    }

    pub fn broadcast(&mut self, message: &str) -> Result<(), Error> {
        for websocket in self.websockets.iter_mut() {
            // if let Err(e) = websocket.read() {
            //     eprintln!("Error reading from websocket: {}", e);
            // }

            if let Err(e) = websocket.send(Message::Text(message.to_string())) {
                eprintln!("Error sending message: {}", e);
            }
        }

        Ok(())
    }
}
