use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use crate::{Message, Request};

#[async_trait]
pub trait Simulator: Send + Sync {
    async fn serve(&self, request: Request);
    async fn set_channel(&self, channel: Sender<Message>);
}
