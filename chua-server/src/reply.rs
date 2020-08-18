use chua::{CompleteResult, InitializeResult, UploadChunkResult};
use warp::http::header::CONTENT_TYPE;
use warp::http::HeaderValue;
use warp::http::StatusCode;
use warp::hyper::Body;
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
        match serde_json::to_string(&self.0) {
            Ok(json) => {
                let mut res = Response::new(Body::from(json));
                res.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                res
            }
            Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
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
        match serde_json::to_string(&self.0) {
            Ok(json) => {
                let mut res = Response::new(Body::from(json));
                res.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                res
            }
            Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
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
        match serde_json::to_string(&self.0) {
            Ok(json) => {
                let mut res = Response::new(Body::from(json));
                res.headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                res
            }
            Err(_e) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
