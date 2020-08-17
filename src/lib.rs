pub(crate) mod common;

macro_rules! if_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

macro_rules! if_tokio {
    ($($item:item)*) => {$(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*}
}

if_tokio! {
    mod internal;
    pub use internal::upload;
    pub use internal::Exception;
}

if_wasm! {
    mod wasm;
    pub use wasm::*;
}
