//! Built-in M2 authoring vocabulary and its lossless embeddings.
use super::properties::Target as PropertySelector;
use crate::model::profiles::Unavailable;
use crate::model::profiles::UniversalProperty;
use crate::model::profiles::{m2, Extends, NoControls, Profile, Universal};
use crate::model::Link;
use crate::model::PropertyIdentityKey;
use crate::model::{
    Basis, Bounds, ClipControls, DrawControls, DrawLooks, Error, FeatureKind, IdentityBinding,
    LengthUnit, M2Geometry, MaterialSurface, MeshControls, Model, ModelMetadata, NodeControls,
    PropertyPayload, System, TextureControls, UniversalAppearance, UniversalBinding,
};
use crate::model::{FeatureSet, FeatureView, FeatureVocabulary};
/// Vocabulary that maps mechanically to M2 logical features.
#[derive(Clone, Debug, Default)]
pub struct M2;
impl Profile for M2 {
    type Property = PropertySelector;
    fn default_bounds() -> Option<Bounds> {
        Some(Default::default())
    }
    type Appearance<'a> = DrawLooks<'a>;
    type Binding<'a> = IdentityBinding;
    type Geometry<'a> = M2Geometry<'a>;
    type Draw = Option<m2::ShadowControls>;
    type ModelMetadata<'a> = m2::ModelMetadata;
    type System<'a> = m2::System<'a>;
    type Surface<'a> = m2::Material<'a>;
    type Feature<'a> = m2::ActorKind<'a>;
    type Node<'a> = Option<m2::BoneControls>;
    type Texture<'a> = m2::TextureControls<'a>;
    type Mesh<'a> = Option<m2::GeometryControls>;
    type Instance<'a> = NoControls;
    type Clip<'a> = m2::SequenceControls;
    type Controller<'a> = NoControls;
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        super::texture::resources(model)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        if model.bounds().is_none() {
            return Err(Error::new(
                "model.bounds",
                "this profile requires an authored bounding volume",
            ));
        }

        if model.basis() != Basis::default() || model.units() != LengthUnit::Unspecified {
            return Err(Error::new(
                "m2.coordinates",
                "M2 has fixed authored axes and no physical-unit declaration",
            ));
        }
        if model.clips().any(|(_, clip)| !clip.is_looping()) {
            return Err(Error::new(
                "m2.clips.looping",
                "nonlooping playback has no established native M2 mapping",
            ));
        }

        let declared = |id| model.node(id).is_some_and(|node| node.controls.is_some());
        if model.controllers().len() != 0 {
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
            .any(|(_, skin)| skin.joints.iter().any(|id| !declared(*id)))
        {
            return Err(Error::new(
                "m2.skin.joints",
                "joint palette must reference declared bones",
            ));
        }
        if model.meshes().any(|(_, mesh)| {
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
    pub(crate) fn profile_supports_property(&self, key: &PropertyIdentityKey) -> bool {
        super::properties::Target::from_identity(key).is_some_and(|key| key.supports_feature(self))
    }

    pub(crate) fn profile_payload_kind(&self) -> FeatureKind {
        FeatureKind::Other
    }
}
impl Extends<M2> for Universal {
    fn property(value: <M2 as Profile>::Property) -> Self::Property {
        UniversalProperty::M2(value)
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
        Some(ModelMetadata::M2(value))
    }
    fn system<'a>(value: <M2 as Profile>::System<'a>) -> Self::System<'a> {
        System::M2(value)
    }
    fn surface<'a>(value: <M2 as Profile>::Surface<'a>) -> Self::Surface<'a> {
        MaterialSurface::ModelShader(Box::new(value))
    }
    fn feature<'a>(value: <M2 as Profile>::Feature<'a>) -> Self::Feature<'a> {
        value.into()
    }
    fn node<'a>(value: <M2 as Profile>::Node<'a>) -> Self::Node<'a> {
        value.map(NodeControls::M2)
    }
    fn texture<'a>(value: <M2 as Profile>::Texture<'a>) -> Self::Texture<'a> {
        Some(TextureControls::M2(value))
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
        None
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
    type Extent = Unavailable;
    type ParticleCopy = Unavailable;
    type Projector = Unavailable;
    type Force = Unavailable;
    type Warp = Unavailable;
    type HitTest = Unavailable;
    type AttachmentVolume = Unavailable;
    type Billboard = Unavailable;
    type Turret = Unavailable;
    type IkJoint = Unavailable;
    type IkTwoJoint = Unavailable;
    type IkCcd = Unavailable;
    type OneBoneSolver = Unavailable;
    type ShadowBox = Unavailable;
    type ViewVolume = Unavailable;
    type AtlasParticles = Unavailable;
    type PointLight = Unavailable;
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link)) {
        value.visit_profile_links(visitor);
    }
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a> {
        value.profile_feature()
    }
    fn supports_property(value: &FeatureSet<'a, Self>, key: &PropertyIdentityKey) -> bool {
        value.profile_supports_property(key)
    }
    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind {
        value.profile_payload_kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Controller, Node, Skin};
    use m2::{BoneControls, ModelMetadata};

    #[test]
    fn native_joint_palettes_require_declared_bone_nodes() {
        let mut invalid = Model::<M2>::builder("palette");
        let helper = invalid.add_node(Node::named("helper")).unwrap();
        invalid.add_skin(Skin::identity(vec![helper])).unwrap();
        assert!(invalid.finish().is_err());
        let mut valid = Model::<M2>::builder("palette");
        let bone = valid
            .add_node(Node::named("bone").with_controls(BoneControls::default()))
            .unwrap();
        valid.add_skin(Skin::identity(vec![bone])).unwrap();
        valid.finish().unwrap();
    }

    #[test]
    fn key_bone_categories_and_layers_are_checked_at_publication() {
        let mut invalid = Model::<M2>::builder("key bone");
        let helper = invalid.add_node(Node::named("helper")).unwrap();
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
