mod common;

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

pub use common::json::*;
pub use common::Exception;

if_tokio! {
    mod internal;
    pub use internal::upload;
}

if_wasm! {
    mod wasm;
    pub use wasm::*;
}
