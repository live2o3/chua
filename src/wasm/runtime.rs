use futures::task::{Context, Poll};
use futures_channel::oneshot;
use futures_channel::oneshot::{Canceled, Receiver};
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen::{throw_str, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;

pub fn spawn<T>(task: T) -> JoinHandle<T::Output>
where
    T: Future + 'static,
    T::Output: 'static,
{
    let (sender, receiver) = oneshot::channel();

    spawn_local(async move {
        let value = task.await;
        sender.send(value).unwrap_or_default();
    });

    JoinHandle { receiver }
}

pub struct JoinHandle<T> {
    receiver: Receiver<T>,
}

impl<T: 'static> Future for JoinHandle<T> {
    type Output = Result<T, Canceled>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx)
    }
}

pub async fn promise<T>(promise: js_sys::Promise) -> Result<T, JsValue>
where
    T: JsCast,
{
    use wasm_bindgen_futures::JsFuture;

    let js_val = JsFuture::from(promise).await?;

    js_val.dyn_into::<T>()
}

// 来自 gloo-file
fn safe_u64_to_f64(number: u64) -> f64 {
    // Max integer stably representable by f64
    // todo use js_sys::Number::MAX_SAFE_INTEGER once stable
    const MAX_SAFE_INTEGER: u64 = 9007199254740991; // (2^53 - 1)
    if number > MAX_SAFE_INTEGER {
        throw_str("a rust number was too large and could not be represented in JavaScript");
    }
    number as f64
}

pub(crate) fn get_slice(blob: &web_sys::Blob, start: u64, end: u64) -> web_sys::Blob {
    use wasm_bindgen::UnwrapThrowExt;

    let start = safe_u64_to_f64(start);
    let end = safe_u64_to_f64(end);

    blob.slice_with_f64_and_f64(start, end).unwrap_throw()
}
