use chua::{CompleteResult, InitializeResult, UploadChunkResult};
use warp::http::header::CONTENT_TYPE;
use warp::http::HeaderValue;
use warp::http::StatusCode;
use warp::hyper::Body;
use warp::reply::Response;
use warp::Reply;

macro_rules! impl_reply_for_result {
    ($result:ident, $reply:ident) => {
        pub struct $reply($result);

        impl From<$result> for $reply {
            fn from(r: $result) -> Self {
                Self(r)
            }
        }

        impl Reply for $reply {
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
    };
}

impl_reply_for_result!(InitializeResult, InitializeReply);
impl_reply_for_result!(UploadChunkResult, UploadChunkReply);
impl_reply_for_result!(CompleteResult, CompleteReply);
