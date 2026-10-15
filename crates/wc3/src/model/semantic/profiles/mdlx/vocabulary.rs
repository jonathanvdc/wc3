//! Built-in MDLX authoring vocabulary and its lossless embeddings.
use super::properties::Target as PropertySelector;
use crate::model::semantic::profiles::UniversalProperty;
use crate::model::semantic::profiles::{
    mdlx, Extends, FeaturePayload, NoControls, Profile, SurfacePayload, Unavailable, Universal,
};
use crate::model::semantic::Model;
use crate::model::semantic::PropertyIdentityKey;
use crate::model::semantic::{
    ClipControls, Error, FeatureKind, IdentityBinding, MaterialSurface, MdlxGeometry, MeshControls,
    ModelMetadata, NodeControls, NodeFeature, SingleLook, TextureControls, UniversalAppearance,
    UniversalBinding,
};
/// Vocabulary that maps mechanically to Warcraft III logical features.
///
/// The graph cannot accept surfaces outside its vocabulary:
/// ```compile_fail
/// use wc3::model::semantic::{Material, MaterialSurface};
/// use wc3::model::semantic::profiles::Mdlx;
/// let material = Material::<Mdlx> {
///     name: "invalid surface family".into(),
///     surface: MaterialSurface::Generic(Box::default()),
/// };
/// ```
#[derive(Clone, Debug, Default)]
pub struct Mdlx;
impl Profile for Mdlx {
    type Property = PropertySelector;
    fn default_bounds() -> Option<crate::model::semantic::Bounds> {
        Some(Default::default())
    }
    type Appearance<'a> = SingleLook<'a>;
    type Binding<'a> = IdentityBinding;
    type Geometry<'a> = MdlxGeometry<'a>;
    type Draw = NoControls;
    type ModelMetadata<'a> = mdlx::ModelMetadata<'a>;
    type System<'a> = Unavailable;
    type Surface<'a> = mdlx::Material<'a>;
    type Feature<'a> = mdlx::Node<'a>;
    type Node<'a> = mdlx::NodeControls;
    type Texture<'a> = mdlx::TextureControls<'a>;
    type Mesh<'a> = mdlx::Geometry;
    type Instance<'a> = NoControls;
    type Clip<'a> = mdlx::Sequence;
    type Controller<'a> = NoControls;
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        let resources = model.resources();

        super::validation::resources(resources)
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        super::validation::model(model)
    }
}
impl SurfacePayload for mdlx::Material<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        key.downcast_ref::<super::properties::Target>()
            .is_some_and(|key| key.supports_material(self))
    }

    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }
}
impl FeaturePayload for mdlx::Node<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        key.downcast_ref::<super::properties::Target>()
            .is_some_and(|key| key.supports_feature(self))
    }

    fn kind(&self) -> FeatureKind {
        match self {
            mdlx::Node::GeometryAnimation(_) => FeatureKind::MdlxGeometryAnimation,
            _ => FeatureKind::Other,
        }
    }
}
impl Extends<Mdlx> for Universal {
    fn property(value: <Mdlx as Profile>::Property) -> Self::Property {
        UniversalProperty::Mdlx(value)
    }
    fn appearance<'a>(value: <Mdlx as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Single(value)
    }
    fn binding<'a>(value: <Mdlx as Profile>::Binding<'a>) -> Self::Binding<'a> {
        UniversalBinding::Identity(value)
    }
    fn geometry<'a>(value: <Mdlx as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn draw(value: <Mdlx as Profile>::Draw) -> Self::Draw {
        let _ = value;
        None
    }
    fn model_metadata<'a>(value: <Mdlx as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        Some(ModelMetadata::Mdlx(value))
    }
    fn system<'a>(value: <Mdlx as Profile>::System<'a>) -> Self::System<'a> {
        match value {}
    }
    fn surface<'a>(value: <Mdlx as Profile>::Surface<'a>) -> Self::Surface<'a> {
        MaterialSurface::Mdlx(Box::new(value))
    }
    fn feature<'a>(value: <Mdlx as Profile>::Feature<'a>) -> Self::Feature<'a> {
        NodeFeature::Mdlx(Box::new(value))
    }
    fn node<'a>(value: <Mdlx as Profile>::Node<'a>) -> Self::Node<'a> {
        Some(NodeControls::Mdlx(value))
    }
    fn texture<'a>(value: <Mdlx as Profile>::Texture<'a>) -> Self::Texture<'a> {
        Some(TextureControls::Mdlx(value))
    }
    fn mesh<'a>(value: <Mdlx as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        Some(MeshControls::Mdlx(value))
    }
    fn instance<'a>(value: <Mdlx as Profile>::Instance<'a>) -> Self::Instance<'a> {
        let _ = value;
        None
    }
    fn clip<'a>(value: <Mdlx as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Some(ClipControls::Mdlx(value))
    }
    fn controller<'a>(value: <Mdlx as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        None
    }
}
