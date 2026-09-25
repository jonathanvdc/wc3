//! Semantic checks for all currently known MDX chunk layouts.

use crate::{Error, Model, ModelInfo};

impl Model {
    /// Validates known chunks and animation records without changing their bytes.
    /// Unknown top-level chunks remain valid and round-trip unchanged.
    pub fn validate(&self) -> Result<(), Error> {
        if self
            .chunks()
            .iter()
            .any(|chunk| chunk.tag == *b"VERS" && chunk.data.len() < 4)
        {
            return Err(Error::InvalidVersionChunk);
        }
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == *b"MODL") {
            ModelInfo::parse(&chunk.data)?;
        }
        self.sequences()?;
        self.global_sequences()?;
        self.textures()?;
        self.pivot_points()?;
        self.bind_poses()?;
        self.face_fx()?;
        self.materials()?;
        self.geosets()?;
        self.geoset_animations()?;
        for bone in self.bones()? {
            bone.node().tracks();
        }
        for helper in self.helpers()? {
            helper.tracks();
        }
        self.attachments()?;
        for event in self.event_objects()? {
            event.node().tracks();
        }
        for shape in self.collision_shapes()? {
            shape.node().tracks();
        }
        self.particle_emitters()?;
        for emitter in self.particle_emitters2()? {
            emitter.node().tracks();
            emitter.tracks()?;
        }
        self.ribbon_emitters()?;
        for emitter in self.popcorn_emitters()? {
            emitter.node().tracks();
            emitter.tracks()?;
        }
        self.cameras()?;
        self.lights()?;
        self.texture_animations()?;
        Ok(())
    }
}
