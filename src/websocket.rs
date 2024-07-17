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
        let mut unhealthy_indexes = Vec::<usize>::new();

        for (index, websocket) in self.websockets.iter_mut().enumerate() {
            if let Err(e) = websocket.send(Message::Text(message.to_string())) {
                unhealthy_indexes.push(index);
                eprintln!("Error sending message: {}", e);
            }
        }

        for index in unhealthy_indexes.iter().rev() {
            println!(
                "Removing websocket at index {} due to it being unhealthy",
                index
            );

            // Remove all websockets that failed to be written to
            self.websockets.remove(*index);
        }

        Ok(())
    }
}
