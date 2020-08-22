mod common;

macro_rules! if_wasm {
    ($($item:item)*) => {$(
        #[cfg(target_arch = "wasm32")]
        $item
    )*}
}

macro_rules! if_native {
    ($($item:item)*) => {$(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*}
}

pub use common::json::*;
pub use common::Exception;
pub use common::{FILE_ROUTE, PART_NAME};

if_native! {
    mod native;
    pub use native::upload;
}

if_wasm! {
    mod wasm;
    pub use wasm::upload;
}
