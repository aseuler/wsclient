use std::env;
use wsclient;

#[tokio::main]
pub async fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));
    wsclient::WsClient::new(url).chat().await;
}
