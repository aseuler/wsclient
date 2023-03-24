use futures::executor::block_on;
use std::env;
use wsclient;

#[tokio::main]
pub async fn run(url: String) {
    wsclient::WsClient::new(url).chat().await;
}

pub fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));
    block_on(async { run(url) });
}
