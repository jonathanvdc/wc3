//! Attachment form selected by the programmed-scene profile.
use crate::model::SocketDefinition;
use std::borrow::Cow;
/// Named attachment carried by its owning node feature.
pub type Attachment<'a> = SocketDefinition<Cow<'a, str>, (), u32>;
