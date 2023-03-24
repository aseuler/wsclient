use futures_util::{future, pin_mut, StreamExt};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, warn};

#[derive(Clone, Debug)]
pub struct WsClient {
    pub url: url::Url,
    pub tx: Option<futures_channel::mpsc::UnboundedSender<Message>>,
}

impl WsClient {
    pub fn new(url: String) -> Self {
        let url = url::Url::parse(&url).unwrap();
        Self { url: url, tx: None }
    }

    pub fn send(&self, data: String) {
        if let Some(tx) = self.tx.clone() {
            tx.unbounded_send(Message::Text(data)).unwrap();
        } else {
            warn!("tx is none, not valid");
        }
    }

    pub fn send_ignore_error(&self, data: String) {
        if let Some(tx) = self.tx.clone() {
            let _ = tx.unbounded_send(Message::Text(data));
        } else {
            warn!("tx is none, not valid");
        }
    }

    pub async fn start(&mut self, callback: fn(String)) -> &mut Self {
        let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
        self.tx = Some(stdin_tx);

        let func = |url: url::Url,
                    stdin_rx: futures_channel::mpsc::UnboundedReceiver<Message>,
                    callback: fn(String)| async move {
            let (ws_stream, _resp) = connect_async(url).await.expect("Failed to connect");
            info!("websocket handshake has been successfully completed");

            let (write, read) = ws_stream.split();
            let stdin_to_ws = stdin_rx.map(Ok).forward(write);

            let ws_to_stdout = {
                read.for_each(|message| async move {
                    match message {
                        Ok(message) => {
                            let data = message.into_data();
                            let data_string = String::from_utf8_lossy(&data).to_string();
                            if !data_string.eq("ping") {
                                // println!("{:?}", data_string);
                                callback(data_string);
                            }
                        }
                        Err(e) => {
                            println!("ws read error: {}", e);
                            std::process::exit(1);
                        }
                    }
                })
            };
            pin_mut!(stdin_to_ws, ws_to_stdout);
            future::select(stdin_to_ws, ws_to_stdout).await;
        };

        tokio::spawn(func(self.url.clone(), stdin_rx, callback));
        self
    }

    pub async fn chat(&mut self) {
        let callback = |data: String| {
            println!("{:?}", data);
        };
        self.start(callback).await.send("init".to_string());

        let mut stdin = tokio::io::stdin();
        loop {
            let mut buf = vec![0; 65536];
            let n = match stdin.read(&mut buf).await {
                Err(_) | Ok(0) => break,
                Ok(n) => n,
            };
            buf.truncate(n);

            if let Some(tx) = self.tx.clone() {
                tx.unbounded_send(Message::binary(buf)).unwrap();
            }
        }
    }
}
