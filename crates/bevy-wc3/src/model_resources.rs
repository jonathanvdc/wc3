//! Model resources referenced by attachments and Classic particle emitters.
use bevy::prelude::*;
use wc3::model::{Model, V1800};

use crate::asset::Wc3ModelAsset;

/// Resolved child models in source record order. Empty or unresolved paths keep
/// their slots. PREM image resources (`EmitterUsesTga`) are not model resources.
#[derive(Component, Clone, Default)]
pub struct Wc3ModelResources {
    attachments: Vec<Option<Handle<Wc3ModelAsset>>>,
    particles: Vec<Option<Handle<Wc3ModelAsset>>>,
}

impl Wc3ModelResources {
    /// Resolve literal paths using a consumer's asset source. The callback owns
    /// path normalization and any MDL-to-MDX fallback policy.
    pub fn resolve(
        model: &Model<V1800>,
        mut resolve: impl FnMut(&str) -> Option<Handle<Wc3ModelAsset>>,
    ) -> Self {
        let attachments = model
            .attachments()
            .iter()
            .map(|attachment| {
                let path = attachment.path.text();
                if path.is_empty() {
                    None
                } else {
                    resolve(&path)
                }
            })
            .collect();
        let particles = model
            .particle_emitters()
            .iter()
            .map(|emitter| {
                let path = emitter.path.text();
                if emitter.flags().emitter_uses_mdl() && !path.is_empty() {
                    resolve(&path)
                } else {
                    None
                }
            })
            .collect();
        Self {
            attachments,
            particles,
        }
    }

    /// Attachment record index, not attachment ID or node object ID.
    pub fn attachment(&self, index: usize) -> Option<Handle<Wc3ModelAsset>> {
        self.attachments.get(index).cloned().flatten()
    }

    /// Classic PREM emitter record index.
    pub fn particle(&self, index: usize) -> Option<Handle<Wc3ModelAsset>> {
        self.particles.get(index).cloned().flatten()
    }

    pub(crate) fn from_handles(
        attachments: Vec<Option<Handle<Wc3ModelAsset>>>,
        particles: Vec<Option<Handle<Wc3ModelAsset>>>,
    ) -> Self {
        Self {
            attachments,
            particles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Wc3LayerMaterial;
    use crate::model::Wc3Model;
    use crate::spawn::{prepare_model_with_resources, spawn_prepared_model};
    use bevy::ecs::world::CommandQueue;
    use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
    use wc3::model::mdl::Read as _;

    #[test]
    fn resolver_preserves_record_slots_and_skips_empty_and_image_paths() {
        let model = Model::<V1800>::decode_mdl(
            r#"
            Version { FormatVersion 1800, }
            Model "Resources" {}
            Attachment "Empty" { ObjectId 0, AttachmentID 20, }
            Attachment "Weapon" { ObjectId 1, AttachmentID 7, Path "Weapon.mdl", }
            Attachment "Missing" { ObjectId 2, Path "Missing.mdx", }
            ParticleEmitter "Empty" { ObjectId 3, EmitterUsesMdl, }
            ParticleEmitter "Model" { ObjectId 4, EmitterUsesMdl, Path "Spark.mdl", }
            ParticleEmitter "Image" { ObjectId 5, EmitterUsesTga, Path "Image.tga", }
        "#,
        )
        .unwrap();
        let mut assets = Assets::<Wc3ModelAsset>::default();
        let handle = assets.add(Wc3ModelAsset {
            source: Wc3Model::decode_mdl("Version { FormatVersion 800, } Model \"Child\" {}")
                .unwrap(),
            textures: default(),
            models: default(),
        });
        let mut requested = Vec::new();
        let resources = Wc3ModelResources::resolve(&model, |path| {
            requested.push(path.to_owned());
            (path != "Missing.mdx").then(|| handle.clone())
        });
        assert_eq!(requested, ["Weapon.mdl", "Missing.mdx", "Spark.mdl"]);
        assert!(resources.attachment(0).is_none());
        assert_eq!(resources.attachment(1), Some(handle.clone()));
        assert!(resources.attachment(2).is_none());
        assert!(resources.attachment(7).is_none());
        assert!(resources.particle(0).is_none());
        assert_eq!(resources.particle(1), Some(handle.clone()));
        assert!(resources.particle(2).is_none());

        let source = Wc3Model {
            source_version: 1800,
            model,
        };
        let mut meshes = Assets::<Mesh>::default();
        let mut materials = Assets::<Wc3LayerMaterial>::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let prepared = prepare_model_with_resources(
            &mut meshes,
            &mut materials,
            &mut bindposes,
            &source,
            |_| None,
            |path| (path != "Missing.mdx").then(|| handle.clone()),
        )
        .unwrap();
        assert_eq!(
            prepared.model_resources().attachment(1),
            Some(handle.clone())
        );
        let mut world = World::new();
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let root = spawn_prepared_model(&mut commands, &mut meshes, &mut materials, &prepared);
        queue.apply(&mut world);
        let instance_resources = world.get::<Wc3ModelResources>(root).unwrap();
        assert_eq!(instance_resources.attachment(1), Some(handle.clone()));
        assert_eq!(instance_resources.particle(1), Some(handle));
    }
}
