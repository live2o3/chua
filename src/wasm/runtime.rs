use futures::task::{Context, Poll};
use futures_channel::oneshot;
use futures_channel::oneshot::{Canceled, Receiver};
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;

pub fn spawn<T>(task: T) -> JoinHandle<T::Output>
where
    T: Future + 'static,
    T::Output: 'static,
{
    let (sender, receiver) = oneshot::channel();

    spawn_local(async move {
        let value = task.await;
        sender
            .send(value)
            .map_err(|_| format!("join error"))
            .unwrap_throw();
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
