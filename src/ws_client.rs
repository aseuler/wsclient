use super::Handler;
// use futures_util::FutureExt;
use futures_util::{future, pin_mut, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

#[derive(Clone, Default)]
struct ChatHandler {}
impl Handler for ChatHandler {
    fn process(&self, data: String) {
        println!("{:?}", data);
    }
}

#[derive(Clone, Debug)]
pub struct WsClient {
    pub tx: Arc<Mutex<Option<futures_channel::mpsc::UnboundedSender<Message>>>>,
    pub runtime: Arc<Mutex<tokio::runtime::Runtime>>,
}

impl WsClient {
    pub fn new() -> Self {
        Self {
            tx: Arc::new(Mutex::new(None)),
            runtime: Arc::new(Mutex::new(
                tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(4)
                    .thread_name("wsclient")
                    .enable_all()
                    .build()
                    .unwrap(),
            )),
        }
    }

    pub fn send(&self, data: Vec<u8>) {
        if let Some(tx) = self.tx.lock().unwrap().clone() {
            // tx.unbounded_send(Message::Text(data)).unwrap();
            let _ = tx.unbounded_send(Message::Binary(data));
        } else {
            warn!("tx is none, not valid");
        }
    }

    pub fn send_ignore_error(&self, data: String) {
        if let Some(tx) = self.tx.lock().unwrap().clone() {
            let _ = tx.unbounded_send(Message::Text(data));
        } else {
            warn!("tx is none, not valid");
        }
    }

    pub async fn start(&mut self, url: String, handler: Box<dyn Handler>) -> &mut Self {
        let url = url::Url::parse(&url).unwrap();

        // info!("start: {}", url);
        let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
        self.tx = Arc::new(Mutex::new(Some(stdin_tx.clone())));

        let func = |url: url::Url,
                    stdin_rx: futures_channel::mpsc::UnboundedReceiver<Message>,
                    handler: Box<dyn Handler>| async move {
            // let (ws_stream, _resp) = connect_async(url).await.expect("Failed to connect");
            let tmp_conn = connect_async(url).await;
            if tmp_conn.is_err() {
                println!("connect with error");
                for (key, value) in std::env::vars() {
                    info!("{key}: {value}");
                }
                error!("error: {}", tmp_conn.as_ref().err().unwrap());
                handler.process("connect_error".to_string());
            }
            let (ws_stream, _resp) = tmp_conn.unwrap();

            info!("websocket handshake has been successfully completed");
            let (write, read) = ws_stream.split();
            let stdin_to_ws = stdin_rx.map(Ok).forward(write);
            let ws_to_stdout = {
                read.for_each(|message| async {
                    match message {
                        Ok(message) => {
                            let data = message.into_data();
                            let data_string = String::from_utf8_lossy(&data).to_string();
                            if !data_string.eq("ping") {
                                handler.process(data_string);
                            }
                        }
                        Err(e) => {
                            println!("ws read error: {}", e);
                            handler.process("connect_error".to_string());
                            // std::process::exit(1);
                        }
                    }
                })
            };
            pin_mut!(stdin_to_ws, ws_to_stdout);
            future::select(stdin_to_ws, ws_to_stdout).await;
        };

        // info!("tokio::spawn: {}", url);
        // tokio::spawn(func(url, stdin_rx, handler));
        self.runtime
            .lock()
            .unwrap()
            .spawn(func(url, stdin_rx, handler));
        // info!("tokio::spawn finish");
        self
    }

    pub async fn chat(&mut self, url: String) {
        self.start(url, Box::new(ChatHandler::default())).await;

        let mut stdin = tokio::io::stdin();
        loop {
            let mut buf = vec![0; 65536];
            let n = match stdin.read(&mut buf).await {
                Err(_) | Ok(0) => break,
                Ok(n) => n,
            };
            buf.truncate(n);

            if let Some(tx) = self.tx.lock().unwrap().clone() {
                tx.unbounded_send(Message::binary(buf)).unwrap();
            }
        }
    }
}
