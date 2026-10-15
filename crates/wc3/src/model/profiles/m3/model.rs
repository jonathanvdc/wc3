//! Attachment form selected by the programmed-scene profile.
use crate::model::AttachmentPointDefinition;
use std::borrow::Cow;
/// Named attachment carried by its owning node feature.
pub type Attachment<'a> = AttachmentPointDefinition<Cow<'a, str>, (), u32>;
