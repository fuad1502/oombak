pub mod request;
pub mod response;

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
