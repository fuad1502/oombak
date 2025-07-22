use std::sync::{Arc, RwLock};

use oombak_sim::Response;

pub type Listeners = Vec<Arc<RwLock<dyn Listener>>>;

pub trait Listener: Send + Sync {
    fn on_receive_reponse(&mut self, response: &Response);
}
