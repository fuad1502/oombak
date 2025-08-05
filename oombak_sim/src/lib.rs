pub mod request;
pub mod response;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

pub use request::Request;
pub use response::Response;

pub use oombak_rs::probe::{InstanceNode, Probe, Signal, SignalType};
pub use request::ProbePointsModification;
pub use response::{CompactWaveValue, LoadedDut, SimulationResult, Wave};

#[async_trait]
pub trait Simulator: Send + Sync {
    async fn serve(&self, request: &Request);
    async fn set_channel(&self, channel: Sender<Message>);
}

pub enum Message {
    Request(request::Request),
    Response(response::Response),
}

impl Message {
    pub fn request(id: usize, payload: request::Payload) -> Self {
        Self::Request(request::Request { id, payload })
    }

    pub fn response(id: usize, payload: response::Payload) -> Self {
        Self::Response(response::Response { id, payload })
    }
}
