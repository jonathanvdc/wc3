#[path = "blp/container.rs"]
mod container;

#[cfg(feature = "blp-decode")]
#[path = "blp/decode.rs"]
mod decode;

#[cfg(all(feature = "blp-decode", feature = "blp-encode"))]
#[path = "blp/encode.rs"]
mod encode;

#[cfg(feature = "blp-encode")]
#[path = "blp/encode_only.rs"]
mod encode_only;
