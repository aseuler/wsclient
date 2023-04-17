## Usage
```rust
use ctrlc;
use std::env;
use wsclient;

#[derive(Default)]
struct DataListener {}

impl wsclient::Handler for DataListener {
    fn process(&self, data: String) {
        println!("{:?}", data);
    }
}

#[tokio::main]
pub async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));
    let listener = DataListener::default();
    wsclient::WsClient::new()
        .start(url, Box::new(listener))
        .await;

    use std::sync::mpsc::channel;
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    rx.recv().expect("Could not receive from channel.");
    println!("Ctrl-C and exiting...");
    std::process::exit(0);
}

```