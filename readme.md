## Usage
```rust
use std::env;
use ctrlc;
use wsclient;

fn handler(data: String) {
    println!("{:?}", data);
}

#[tokio::main]
pub async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));
    wsclient::WsClient::new(url).start(handler).await;

    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    rx.recv().expect("Could not receive from channel.");
    println!("Ctrl-C and exiting...");
    std::process::exit(0);
}
```