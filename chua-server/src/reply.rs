use chua::{CompleteResult, InitializeResult, UploadChunkResult};
use warp::reply::Response;
use warp::Reply;

pub struct InitializeReply(InitializeResult);

impl From<InitializeResult> for InitializeReply {
    fn from(r: InitializeResult) -> Self {
        Self(r)
    }
}

impl Reply for InitializeReply {
    fn into_response(self) -> Response {
        unimplemented!()
    }
}

pub struct UploadChunkReply(UploadChunkResult);

impl From<UploadChunkResult> for UploadChunkReply {
    fn from(r: UploadChunkResult) -> Self {
        Self(r)
    }
}

impl Reply for UploadChunkReply {
    fn into_response(self) -> Response {
        unimplemented!()
    }
}

pub struct CompleteReply(CompleteResult);

impl From<CompleteResult> for CompleteReply {
    fn from(r: CompleteResult) -> Self {
        Self(r)
    }
}

impl Reply for CompleteReply {
    fn into_response(self) -> Response {
        unimplemented!()
    }
}
