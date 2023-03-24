pub mod ws_client;
pub use ws_client::*;

pub trait Handler: Send + Sync {
    fn process(&self, data: String);
}
