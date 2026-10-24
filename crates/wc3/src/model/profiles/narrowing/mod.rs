//! Exact checked projections of complete owner payloads.
use super::Profile;
use crate::model::error::{
    FeatureFamily, PayloadForm, PayloadKind, ProfileError, ProfileKind, ProfileViolation,
};
use crate::model::PropertyValues;
use crate::model::{
    ClipProperties, DrawProperties, Error, EventSource, InstanceProperties, MeshProperties, Model,
    ModelMetadata, NodeFeature, NodeProperties, System, UniversalAppearance, UniversalBinding,
};
mod m2;

mod mdlx;
mod wmo;
/// A target vocabulary that can check and consume payloads from another profile.
/// Implementations must preserve every accepted authored value and relationship.
/// Returning an error is required when an exact projection is unavailable.
pub trait Narrows<Source: Profile>: Profile {
    /// Checks exactness constraints requiring the complete source graph.
    fn validate_projection(source: &Model<'_, Source>) -> Result<(), Error> {
        for (id, feature) in source.features() {
            Self::validate_feature(source, &feature.payload)
                .map_err(|error| error.with_owner(id))?;
        }
        Ok(())
    }

    /// Checks a feature's target constraints that depend on its owning graph.
    ///
    /// This is also used when policies inspect whether a selected feature can
    /// be retained. Payload-only projection remains the responsibility of `feature`.
    fn validate_feature<'a>(
        source: &Model<'a, Source>,
        feature: &Source::Feature<'a>,
    ) -> Result<(), Error> {
        let _ = (source, feature);
        Ok(())
    }

    /// Projects image dependency declarations exactly.
    fn image_resource<'a>(
        value: Source::ImageResource<'a>,
    ) -> Result<Self::ImageResource<'a>, Error>;
    /// Projects model dependency declarations exactly.
    fn model_resource<'a>(
        value: Source::ModelResource<'a>,
    ) -> Result<Self::ModelResource<'a>, Error>;
    /// Projects effect dependency declarations exactly.
    fn effect_resource<'a>(
        value: Source::EffectResource<'a>,
    ) -> Result<Self::EffectResource<'a>, Error>;
    /// Projects physics dependency declarations exactly.
    fn physics_resource<'a>(
        value: Source::PhysicsResource<'a>,
    ) -> Result<Self::PhysicsResource<'a>, Error>;
    /// Projects boneoverride dependency declarations exactly.
    fn bone_override_resource<'a>(
        value: Source::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error>;

    /// Projects the complete appearance vocabulary while preserving identities.
    fn appearance<'a>(value: Source::Appearance<'a>) -> Result<Self::Appearance<'a>, Error>;
    /// Projects the authored joint binding without changing its interpretation.
    fn binding<'a>(value: Source::Binding<'a>) -> Result<Self::Binding<'a>, Error>;
    /// Projects complete geometry without changing its logical values.
    fn geometry<'a>(value: Source::Geometry<'a>) -> Result<Self::Geometry<'a>, Error>;
    /// Projects model-wide metadata without changing their meaning.
    fn model_metadata<'a>(
        value: Source::ModelMetadata<'a>,
    ) -> Result<Self::ModelMetadata<'a>, Error>;
    /// Projects a complete event source without changing its meaning.
    fn event_source<'a>(value: Source::EventSource<'a>) -> Result<Self::EventSource<'a>, Error>;
    /// Projects an independent scene system without changing its meaning.
    fn system<'a>(value: Source::System<'a>) -> Result<Self::System<'a>, Error>;
    /// Projects a surface without changing its meaning.
    fn surface<'a>(value: Source::Surface<'a>) -> Result<Self::Surface<'a>, Error>;
    /// Projects an attached feature without changing its meaning.
    fn feature<'a>(
        value: Source::Feature<'a>,
        values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error>;
    /// Projects node properties without changing meaning.
    fn node<'a>(value: Source::NodeProperties<'a>) -> Result<Self::NodeProperties<'a>, Error>;
    /// Projects mesh properties without changing meaning.
    fn mesh<'a>(value: Source::MeshProperties<'a>) -> Result<Self::MeshProperties<'a>, Error>;
    /// Projects instance properties without changing meaning.
    fn instance<'a>(
        value: Source::InstanceProperties<'a>,
    ) -> Result<Self::InstanceProperties<'a>, Error>;
    /// Projects draw-pass behavior without changing meaning.
    fn draw(value: Source::DrawProperties) -> Result<Self::DrawProperties, Error>;
    /// Projects clip properties without changing meaning.
    fn clip<'a>(value: Source::ClipProperties<'a>) -> Result<Self::ClipProperties<'a>, Error>;
}
impl<P: Profile> Narrows<P> for P {
    fn image_resource<'a>(value: P::ImageResource<'a>) -> Result<Self::ImageResource<'a>, Error> {
        Ok(value)
    }
    fn model_resource<'a>(value: P::ModelResource<'a>) -> Result<Self::ModelResource<'a>, Error> {
        Ok(value)
    }
    fn effect_resource<'a>(
        value: P::EffectResource<'a>,
    ) -> Result<Self::EffectResource<'a>, Error> {
        Ok(value)
    }
    fn physics_resource<'a>(
        value: P::PhysicsResource<'a>,
    ) -> Result<Self::PhysicsResource<'a>, Error> {
        Ok(value)
    }
    fn bone_override_resource<'a>(
        value: P::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error> {
        Ok(value)
    }

    fn appearance<'a>(value: P::Appearance<'a>) -> Result<Self::Appearance<'a>, Error> {
        Ok(value)
    }
    fn binding<'a>(value: P::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        Ok(value)
    }
    fn geometry<'a>(value: P::Geometry<'a>) -> Result<Self::Geometry<'a>, Error> {
        Ok(value)
    }
    fn model_metadata<'a>(value: P::ModelMetadata<'a>) -> Result<Self::ModelMetadata<'a>, Error> {
        Ok(value)
    }
    fn event_source<'a>(value: P::EventSource<'a>) -> Result<Self::EventSource<'a>, Error> {
        Ok(value)
    }
    fn system<'a>(value: P::System<'a>) -> Result<Self::System<'a>, Error> {
        Ok(value)
    }
    fn surface<'a>(value: P::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        Ok(value)
    }
    fn feature<'a>(
        value: P::Feature<'a>,
        _values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        Ok(value)
    }
    fn node<'a>(value: P::NodeProperties<'a>) -> Result<Self::NodeProperties<'a>, Error> {
        Ok(value)
    }
    fn mesh<'a>(value: P::MeshProperties<'a>) -> Result<Self::MeshProperties<'a>, Error> {
        Ok(value)
    }
    fn instance<'a>(
        value: P::InstanceProperties<'a>,
    ) -> Result<Self::InstanceProperties<'a>, Error> {
        Ok(value)
    }
    fn draw(value: P::DrawProperties) -> Result<Self::DrawProperties, Error> {
        Ok(value)
    }
    fn clip<'a>(value: P::ClipProperties<'a>) -> Result<Self::ClipProperties<'a>, Error> {
        Ok(value)
    }
}
fn incompatible(target: ProfileKind, owner: PayloadKind, actual: PayloadForm) -> Error {
    let violation = if actual == PayloadForm::None {
        ProfileViolation::RequiredPayload { owner }
    } else {
        ProfileViolation::UnsupportedPayload { owner, actual }
    };
    ProfileError::new(target, violation).into()
}
fn appearance(value: &UniversalAppearance<'_>) -> PayloadForm {
    match value {
        UniversalAppearance::Single(_) => PayloadForm::SingleAppearance,
        UniversalAppearance::Draw(_) => PayloadForm::DrawAppearance,
    }
}
fn binding(value: &UniversalBinding<'_>) -> PayloadForm {
    match value {
        UniversalBinding::Identity(_) => PayloadForm::IdentityBinding,
        UniversalBinding::Explicit(_) => PayloadForm::ExplicitBinding,
    }
}
fn metadata(value: &ModelMetadata<'_>) -> PayloadForm {
    match value {
        ModelMetadata::Mdlx(_) => PayloadForm::Mdlx,
        ModelMetadata::M2(_) => PayloadForm::M2,
        ModelMetadata::Wmo(_) => PayloadForm::Wmo,
    }
}
fn event(value: &EventSource) -> PayloadForm {
    match value {
        EventSource::Mdlx(_) => PayloadForm::Mdlx,
        EventSource::M2(_) => PayloadForm::M2,
    }
}
fn system(value: &System<'_>) -> PayloadForm {
    match value {
        System::M2(_) => PayloadForm::M2,
        System::Wmo(_) => PayloadForm::Wmo,
        System::Physics(_) => PayloadForm::PortablePhysics,
    }
}
fn node(value: &Option<NodeProperties>) -> PayloadForm {
    match value {
        None => PayloadForm::None,
        Some(NodeProperties::Mdlx(_)) => PayloadForm::Mdlx,
        Some(NodeProperties::M2(_)) => PayloadForm::M2,
    }
}
fn mesh(value: &Option<MeshProperties>) -> PayloadForm {
    match value {
        None => PayloadForm::None,
        Some(MeshProperties::Mdlx(_)) => PayloadForm::Mdlx,
        Some(MeshProperties::M2(_)) => PayloadForm::M2,
    }
}
fn instance(value: &Option<InstanceProperties<'_>>) -> PayloadForm {
    match value {
        None => PayloadForm::None,
        Some(InstanceProperties::Wmo(_)) => PayloadForm::Wmo,
    }
}
fn draw(value: &Option<DrawProperties>) -> PayloadForm {
    match value {
        None => PayloadForm::None,
        Some(DrawProperties::M2(_)) => PayloadForm::M2,
        Some(DrawProperties::Wmo(_)) => PayloadForm::Wmo,
    }
}
fn clip(value: &Option<ClipProperties>) -> PayloadForm {
    match value {
        None => PayloadForm::None,
        Some(ClipProperties::Mdlx(_)) => PayloadForm::Mdlx,
        Some(ClipProperties::M2(_)) => PayloadForm::M2,
    }
}
fn feature_family(value: &NodeFeature<'_>) -> FeatureFamily {
    match value {
        NodeFeature::Light(_) => FeatureFamily::Light,
        NodeFeature::Camera(_) => FeatureFamily::Camera,
        NodeFeature::Attachment(_) => FeatureFamily::Attachment,
        NodeFeature::Particle(_) => FeatureFamily::Particle,
        NodeFeature::Ribbon(_) => FeatureFamily::Ribbon,
        NodeFeature::GeometryAnimation(_) => FeatureFamily::GeometryAnimation,
        NodeFeature::CollisionShape(_) => FeatureFamily::CollisionShape,
        NodeFeature::ParticleProgram(_) => FeatureFamily::ParticleProgram,
        NodeFeature::FaceActor(_) => FeatureFamily::FaceActor,
        NodeFeature::LightProgram(_) => FeatureFamily::LightProgram,
        NodeFeature::Particle2(_) => FeatureFamily::Particle2,
        NodeFeature::PointLight(_) => FeatureFamily::PointLight,
    }
}
fn incompatible_feature(target: ProfileKind, value: &NodeFeature<'_>) -> Error {
    ProfileError::new(
        target,
        ProfileViolation::UnsupportedFeature {
            family: feature_family(value),
        },
    )
    .into()
}
