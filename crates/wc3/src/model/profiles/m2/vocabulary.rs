//! Built-in M2 authoring vocabulary and its lossless embeddings.
use crate::model::profiles::Unavailable;
use crate::model::profiles::{m2, Extends, NoControls, Profile, Universal};
use crate::model::resources::{M2ImageResource, PathResource};
use crate::model::Link;
use crate::model::M2ResourceLocator;
use crate::model::PropertyValues;
use crate::model::{
    ClipControls, DrawControls, DrawLooks, Error, FeatureKind, IdentityBinding, M2Geometry,
    MaterialSurface, MeshControls, Model, ModelMetadata, NodeControls, System, UniversalAppearance,
    UniversalBinding,
};
use crate::model::{FeatureSet, FeatureView, FeatureVocabulary};
/// Vocabulary that maps mechanically to M2 logical features.
#[derive(Clone, Debug, Default)]
pub struct M2;
impl Profile for M2 {
    type ImageResource<'a> = M2ImageResource<'a>;
    type ModelResource<'a> = PathResource<'a>;
    type EffectResource<'a> = Unavailable;
    type PhysicsResource<'a> = M2ResourceLocator<'a>;
    type BoneOverrideResource<'a> = M2ResourceLocator<'a>;

    type Appearance<'a> = DrawLooks<'a>;
    type Binding<'a> = IdentityBinding;
    type Geometry<'a> = M2Geometry<'a>;
    type Draw = Option<m2::DrawControls>;
    type ModelMetadata<'a> = m2::ModelMetadata<'a>;
    type System<'a> = m2::System<'a>;
    type Surface<'a> = m2::Material<'a>;
    type Feature<'a> = m2::ActorKind<'a>;
    type Node<'a> = Option<m2::BoneControls>;
    type Mesh<'a> = Option<m2::GeometryControls>;
    type Instance<'a> = NoControls;
    type Clip<'a> = m2::SequenceControls;
    type Controller<'a> = NoControls;
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        super::texture::resources(model)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        model.validate_animation_consumers()?;
        for (_, clip) in model.clips() {
            if let Some(alias) = clip.controls.alias {
                let target = model.clip(alias).expect("validated alias reference");
                if clip.animations != target.animations {
                    return Err(Error::new(
                        "m2.clip.alias",
                        "native aliases must select the same animation identities",
                    ));
                }
            }
        }
        if model.clips().iter().any(|(_, clip)| !clip.is_looping()) {
            return Err(Error::new(
                "m2.clips.looping",
                "nonlooping playback has no established native M2 mapping",
            ));
        }

        let declared = |id| model.node(id).is_some_and(|node| node.controls.is_some());
        if !model.controllers().is_empty() {
            return Err(Error::new(
                "m2.controllers",
                "M2 animation has no layer-controller objects",
            ));
        }
        if model
            .metadata()
            .key_bone_lookup
            .iter()
            .flatten()
            .any(|id| !declared(*id))
        {
            return Err(Error::new(
                "m2.key_bone_lookup",
                "key-bone category must reference a declared bone",
            ));
        }
        if model
            .skins()
            .iter()
            .any(|(_, skin)| skin.joints.iter().any(|id| !declared(*id)))
        {
            return Err(Error::new(
                "m2.skin.joints",
                "joint palette must reference declared bones",
            ));
        }
        if model.meshes().iter().any(|(_, mesh)| {
            mesh.controls
                .as_ref()
                .and_then(|controls| controls.center_bone)
                .is_some_and(|id| !declared(id))
        }) {
            return Err(Error::new(
                "m2.geometry.center_bone",
                "sorting frame must reference a declared bone",
            ));
        }
        Ok(())
    }
}
impl m2::ActorKind<'_> {
    pub(crate) fn profile_payload_kind(&self) -> FeatureKind {
        FeatureKind::Other
    }
}
impl Extends<M2> for Universal {
    fn image_resource<'a>(value: <M2 as Profile>::ImageResource<'a>) -> Self::ImageResource<'a> {
        value.into()
    }
    fn model_resource<'a>(value: <M2 as Profile>::ModelResource<'a>) -> Self::ModelResource<'a> {
        value.into()
    }
    fn effect_resource<'a>(value: <M2 as Profile>::EffectResource<'a>) -> Self::EffectResource<'a> {
        match value {}
    }
    fn physics_resource<'a>(
        value: <M2 as Profile>::PhysicsResource<'a>,
    ) -> Self::PhysicsResource<'a> {
        value.into()
    }
    fn bone_override_resource<'a>(
        value: <M2 as Profile>::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        value.into()
    }

    fn appearance<'a>(value: <M2 as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Draw(value)
    }
    fn binding<'a>(value: <M2 as Profile>::Binding<'a>) -> Self::Binding<'a> {
        UniversalBinding::Identity(value)
    }
    fn geometry<'a>(value: <M2 as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn draw(value: <M2 as Profile>::Draw) -> Self::Draw {
        value.map(DrawControls::M2)
    }
    fn model_metadata<'a>(value: <M2 as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        ModelMetadata::M2(value)
    }
    fn system<'a>(value: <M2 as Profile>::System<'a>) -> Self::System<'a> {
        System::M2(value)
    }
    fn surface<'a>(value: <M2 as Profile>::Surface<'a>) -> Self::Surface<'a> {
        MaterialSurface::from(value)
    }
    fn feature<'a>(
        value: <M2 as Profile>::Feature<'a>,
        values: &mut PropertyValues,
    ) -> Self::Feature<'a> {
        crate::model::features::FromFeature::from_feature(value, values)
    }
    fn node<'a>(value: <M2 as Profile>::Node<'a>) -> Self::Node<'a> {
        value.map(NodeControls::M2)
    }
    fn mesh<'a>(value: <M2 as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        value.map(MeshControls::M2)
    }
    fn instance<'a>(value: <M2 as Profile>::Instance<'a>) -> Self::Instance<'a> {
        let _ = value;
        None
    }
    fn clip<'a>(value: <M2 as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Some(ClipControls::M2(value))
    }
    fn controller<'a>(value: <M2 as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        NoControls
    }
}

impl<'a> FeatureVocabulary<'a> for M2 {
    type Light = m2::Light;
    type Camera = m2::Camera;
    type Attachment = m2::Attachment;
    type Particle = Box<m2::Particle<'a>>;
    type Ribbon = Box<m2::Ribbon>;
    type Event = m2::Event;
    type GeometryAnimation = Unavailable;
    type CollisionShape = Unavailable;
    type ParticleProgram = Unavailable;
    type FaceActor = Unavailable;
    type LightProgram = Unavailable;

    type AtlasParticles = Unavailable;
    type PointLight = Unavailable;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        value.visit_profile_links(visitor);
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        value.profile_feature()
    }

    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind {
        value.profile_payload_kind()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::model::{Controller, Skin};
    use m2::{BoneControls, ModelMetadata};

    #[test]
    fn native_joint_palettes_require_declared_bone_nodes() {
        let mut invalid = Model::<M2>::builder("palette");
        let helper = invalid.add_named_node("helper").unwrap();
        invalid.add_skin(Skin::identity(vec![helper])).unwrap();
        assert!(invalid.finish().is_err());
        let mut valid = Model::<M2>::builder("palette");
        let definition = valid
            .new_node("bone")
            .with_controls(BoneControls::default());
        let bone = valid.add_node(definition).unwrap();
        valid.add_skin(Skin::identity(vec![bone])).unwrap();
        valid.finish().unwrap();
    }

    #[test]
    fn key_bone_categories_and_layers_are_checked_at_publication() {
        let mut invalid = Model::<M2>::builder("key bone");
        let helper = invalid.add_named_node("helper").unwrap();
        invalid.set_metadata(ModelMetadata {
            key_bone_lookup: vec![Some(helper)],
            ..ModelMetadata::default()
        });
        assert!(invalid.finish().is_err());
        let mut invalid = Model::<M2>::builder("layer");
        invalid.add_controller(Controller::default()).unwrap();
        assert!(invalid.finish().is_err());
    }
}
