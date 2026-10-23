//! Image publication independent of assembly preparation.
use bevy::prelude::{Assets, Handle, Image};

/// Receives fully interpreted assembly images and returns their retained handles.
///
/// The compiler has already selected the pixel data, color space and sampler.
/// Implementations may insert immediately or reserve handles and defer insertion.
/// Deferred handles must belong to the image store used by the eventual renderer.
/// Keep those handles strongly retained until publication. Compilation may emit
/// images before a later error; the sink owns publication and rollback of them.
pub trait ImageSink {
    /// Retain an image and return a handle that remains valid after compilation.
    fn add_image(&mut self, image: Image) -> Handle<Image>;
}

impl ImageSink for Assets<Image> {
    fn add_image(&mut self, image: Image) -> Handle<Image> {
        self.add(image)
    }
}

impl<F: FnMut(Image) -> Handle<Image>> ImageSink for F {
    fn add_image(&mut self, image: Image) -> Handle<Image> {
        self(image)
    }
}
