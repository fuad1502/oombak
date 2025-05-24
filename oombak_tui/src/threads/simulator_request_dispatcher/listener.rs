use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use oombak_sim::Response;

pub type Listeners = Vec<Arc<RwLock<dyn Listener>>>;

#[async_trait]
pub trait Listener: Send + Sync {
    async fn on_receive_reponse(&mut self, response: &Response);
}
