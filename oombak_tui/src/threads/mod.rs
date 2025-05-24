mod error;
mod event;
mod render;
pub mod simulator_request_dispatcher;
mod thread_group;
mod util;

pub use error::ThreadError;
pub use error::ThreadResult;
pub use event::EventThread;
pub use render::Message as RendererMessage;
pub use render::RendererThread;
pub use simulator_request_dispatcher::SimulatorRequestDispatcher;
pub use thread_group::ThreadGroup;
pub use util::setup_terminate_group_panic_hook;

pub trait Thread {
    fn terminate(&mut self) -> ThreadResult;
}
