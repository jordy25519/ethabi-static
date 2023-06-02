#![cfg_attr(feature = "bump", feature(allocator_api))]
pub use ethabi_static_derive::*;

#[cfg(feature = "bump")]
pub use bumpalo::Bump;
mod types;
pub use types::*;
