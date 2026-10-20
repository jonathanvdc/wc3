//! Typed immutable event payloads, checked route layouts and bounded ownership.
mod arena;
mod element;
mod frame;
mod layout;
pub(crate) use arena::{Arena, Handle};
pub(crate) use element::Element;
pub(crate) use frame::{select, snapshot};
pub(crate) use layout::{Field, Layout};

mod storage;
pub(crate) use storage::Storage;
