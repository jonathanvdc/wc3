//! Semantic checks for all currently known MDX chunk layouts.

use crate::{Error, Model};

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
        let version = match self.version() {
            Some(version) => version,
            None if self.chunk(*b"VERS").is_some() => return Err(Error::InvalidVersionChunk),
            None => 800,
        };
        self.model_info()?;
        self.sequences()?;
        self.global_sequences()?;
        self.textures()?;
        self.pivot_points()?;
        self.bind_poses()?;
        self.face_fx()?;
        for material in self.materials()? {
            for layer in material.layers(version)? {
                layer.texture_slots(version)?;
                layer.tracks(version)?;
            }
        }
        for geoset in self.geosets()? {
            geoset.vertices()?;
            geoset.normals()?;
            geoset.face_indices()?;
            geoset.vertex_groups(version)?;
            geoset.matrix_group_sizes(version)?;
            geoset.matrix_indices(version)?;
            geoset.material_id(version)?;
            geoset.selection_group(version)?;
            geoset.unselectable(version)?;
            geoset.level_of_detail(version)?;
            geoset.name(version)?;
            geoset.extent(version)?;
            geoset.sequence_extents(version)?;
            geoset.tangents(version)?;
            geoset.skin_weights(version)?;
            geoset.skin_bone_indices(version)?;
            geoset.uv_sets(version)?;
        }
        for animation in self.geoset_animations()? {
            animation.tracks()?;
        }
        for bone in self.bones()? {
            bone.node().tracks()?;
        }
        for helper in self.helpers()? {
            helper.tracks()?;
        }
        for attachment in self.attachments()? {
            attachment.node().tracks()?;
            attachment.visibility_track()?;
        }
        for event in self.event_objects()? {
            event.node().tracks()?;
        }
        for shape in self.collision_shapes()? {
            shape.node().tracks()?;
        }
        for emitter in self.particle_emitters()? {
            emitter.node().tracks()?;
            emitter.tracks()?;
        }
        for emitter in self.particle_emitters2()? {
            emitter.node().tracks()?;
            emitter.tracks()?;
        }
        for emitter in self.ribbon_emitters()? {
            emitter.node().tracks()?;
            emitter.tracks()?;
        }
        for emitter in self.popcorn_emitters()? {
            emitter.node().tracks()?;
            emitter.tracks()?;
        }
        for camera in self.cameras()? {
            camera.tracks()?;
        }
        for light in self.lights()? {
            light.node().tracks()?;
            light.tracks()?;
        }
        for animation in self.texture_animations()? {
            animation.tracks()?;
        }
        Ok(())
    }
}
