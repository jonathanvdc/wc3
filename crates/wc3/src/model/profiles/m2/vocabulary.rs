//! Built-in M2 authoring vocabulary and its lossless embeddings.
use crate::model::error::{Location, ProfileError, ProfileKind, ProfileViolation};
use crate::model::profiles::Unavailable;
use crate::model::profiles::{m2, Extends, NoProperties, Profile, Universal};
use crate::model::resources::{M2ImageResource, M2ModelResource};
use crate::model::M2ResourceLocator;
use crate::model::PropertyValues;
use crate::model::{
    ClipProperties, DrawLooks, DrawProperties, Error, FeatureKind, IdentityBinding, M2Geometry,
    MaterialSurface, MeshProperties, Model, ModelMetadata, NodeProperties, System,
    UniversalAppearance, UniversalBinding,
};
use crate::model::{EventSource, TimeSource};
use crate::model::{FeatureSet, FeatureView, FeatureVocabulary};
use crate::model::{LightShadows, Link};
/// Vocabulary that maps mechanically to M2 logical features.
#[derive(Clone, Debug, Default)]
pub struct M2;
impl Profile for M2 {
    type ImageResource<'a> = M2ImageResource<'a>;
    type ModelResource<'a> = M2ModelResource<'a>;
    type EffectResource<'a> = Unavailable;
    type PhysicsResource<'a> = M2ResourceLocator<'a>;
    type BoneOverrideResource<'a> = M2ResourceLocator<'a>;

    type Appearance<'a> = DrawLooks<'a>;
    type Binding<'a> = IdentityBinding;
    type Geometry<'a> = M2Geometry<'a>;
    type DrawProperties = Option<m2::DrawProperties>;
    type ModelMetadata<'a> = m2::ModelMetadata<'a>;
    type EventSource<'a> = m2::EventSource;
    type System<'a> = m2::System<'a>;
    type Surface<'a> = m2::Material<'a>;
    type Feature<'a> = m2::ActorKind<'a>;
    type NodeProperties<'a> = Option<m2::BoneProperties>;
    type MeshProperties<'a> = Option<m2::MeshProperties>;
    type InstanceProperties<'a> = NoProperties;
    type ClipProperties<'a> = m2::SequenceProperties;

    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        super::texture::resources(model)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        model.validate_animation_consumers()?;
        for (id, feature) in model.features() {
            match &feature.payload {
                m2::ActorKind::Camera(camera) if !camera.is_m2() => {
                    return Err(Error::at(
                        Location::for_owner(id).field("projection"),
                        ProfileError::new(ProfileKind::M2, ProfileViolation::CameraProjection),
                    ));
                }
                m2::ActorKind::Light(light)
                    if light.shadows != LightShadows::default() || light.falloff.is_some() =>
                {
                    return Err(Error::at(
                        Location::for_owner(id),
                        ProfileError::new(ProfileKind::M2, ProfileViolation::LightParameters),
                    ));
                }
                _ => {}
            }
        }
        for event in model.global_events() {
            if model
                .event_source(event.source)
                .expect("validated source")
                .clock
                == TimeSource::Animation
            {
                return Err(Error::at(
                    Location::for_owner(event.source).field("clock"),
                    ProfileError::new(ProfileKind::M2, ProfileViolation::EventSelection),
                ));
            }
        }
        for (id, _) in model.clips() {
            for event in model.clip_events(id) {
                if matches!(
                    model
                        .event_source(event.source)
                        .expect("validated source")
                        .clock,
                    TimeSource::Shared(_)
                ) {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .field("animations")
                            .with_owner(event.source),
                        ProfileError::new(ProfileKind::M2, ProfileViolation::EventSelection),
                    ));
                }
            }
        }

        for (id, clip) in model.clips() {
            if let Some(alias) = clip.properties.alias {
                let target = model.clip(alias).expect("validated alias reference");
                if clip.animations != target.animations {
                    return Err(Error::at(
                        Location::for_owner(id).field("properties").field("alias"),
                        ProfileError::new(
                            ProfileKind::M2,
                            ProfileViolation::AliasAnimationSelection {
                                expected: target.animations.clone(),
                                actual: clip.animations.clone(),
                            },
                        ),
                    ));
                }
            }
        }
        if let Some((id, _)) = model.clips().iter().find(|(_, clip)| !clip.is_looping()) {
            return Err(Error::at(
                Location::for_owner(id),
                ProfileError::new(ProfileKind::M2, ProfileViolation::NonloopingClip),
            ));
        }

        let declared = |id| model.node(id).is_some_and(|node| node.properties.is_some());
        for (index, node) in model.metadata().key_bone_lookup.iter().enumerate() {
            if let Some(node) = node.filter(|node| !declared(*node)) {
                return Err(Error::at(
                    Location::new()
                        .field("metadata")
                        .field("key_bone_lookup")
                        .index(index)
                        .with_owner(node),
                    ProfileError::new(ProfileKind::M2, ProfileViolation::UndeclaredBone),
                ));
            }
        }
        for (id, skin) in model.skins() {
            if let Some((index, node)) = skin
                .joints
                .iter()
                .enumerate()
                .find(|(_, node)| !declared(**node))
            {
                return Err(Error::at(
                    Location::for_owner(id)
                        .field("joints")
                        .index(index)
                        .with_owner(*node),
                    ProfileError::new(ProfileKind::M2, ProfileViolation::UndeclaredBone),
                ));
            }
        }
        for (id, mesh) in model.meshes() {
            if let Some(node) = mesh
                .properties
                .as_ref()
                .and_then(|controls| controls.center_bone)
                .filter(|node| !declared(*node))
            {
                return Err(Error::at(
                    Location::for_owner(id)
                        .field("properties")
                        .field("center_bone")
                        .with_owner(node),
                    ProfileError::new(ProfileKind::M2, ProfileViolation::UndeclaredBone),
                ));
            }
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
    fn draw(value: <M2 as Profile>::DrawProperties) -> Self::DrawProperties {
        value.map(DrawProperties::M2)
    }
    fn model_metadata<'a>(value: <M2 as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        ModelMetadata::M2(value)
    }
    fn event_source<'a>(value: <M2 as Profile>::EventSource<'a>) -> Self::EventSource<'a> {
        EventSource::M2(value)
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
    fn node<'a>(value: <M2 as Profile>::NodeProperties<'a>) -> Self::NodeProperties<'a> {
        value.map(NodeProperties::M2)
    }
    fn mesh<'a>(value: <M2 as Profile>::MeshProperties<'a>) -> Self::MeshProperties<'a> {
        value.map(MeshProperties::M2)
    }
    fn instance<'a>(
        value: <M2 as Profile>::InstanceProperties<'a>,
    ) -> Self::InstanceProperties<'a> {
        let _ = value;
        None
    }
    fn clip<'a>(value: <M2 as Profile>::ClipProperties<'a>) -> Self::ClipProperties<'a> {
        Some(ClipProperties::M2(value))
    }
}

impl<'a> FeatureVocabulary<'a> for M2 {
    type Light = m2::Light;
    type Camera = m2::Camera;
    type Attachment = m2::Attachment;
    type Particle = Box<m2::ParticleEmitter<'a>>;
    type Ribbon = Box<m2::RibbonEmitter>;
    type GeometryAnimation = Unavailable;
    type CollisionShape = Unavailable;
    type ParticleProgram = Unavailable;
    type FaceActor = Unavailable;
    type LightProgram = Unavailable;

    type Particle2 = Unavailable;
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
    use crate::model::Skin;
    use m2::{BoneProperties, ModelMetadata};

    #[test]
    fn native_joint_palettes_require_declared_bone_nodes() {
        let mut invalid = Model::<M2>::builder("palette");
        let helper = invalid.add_named_node("helper").unwrap();
        invalid.add_skin(Skin::identity(vec![helper])).unwrap();
        assert!(invalid.finish().is_err());
        let mut valid = Model::<M2>::builder("palette");
        let definition = valid
            .new_node("bone")
            .with_properties(BoneProperties::default());
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
    }
}
