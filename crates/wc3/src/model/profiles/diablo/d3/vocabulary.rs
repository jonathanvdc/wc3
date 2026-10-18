//! Diablo III authoring vocabulary is independent of native SNO revisions.
use super::{
    uniform_transform, Appearance, ClipControls, Geometry, LookAtTarget, Material, MeshControls,
    ModelMetadata, NodeRole,
};
use crate::model::profiles::{
    Extends, Narrows, NoControls, Profile, SurfacePayload, Unavailable, Universal,
};
use crate::model::resources::SnoResource;
use crate::model::PropertyValues;
use crate::model::TimeSource;
use crate::model::{
    Basis, Billboard, Bounds, Error, ExplicitBinding, Inheritance, LengthUnit, Model, NodeControls,
    SnoNamespace, Transform, UniversalAppearance, UniversalBinding,
};
use crate::model::{
    ClipControls as AnyClipControls, MeshControls as AnyMeshControls,
    ModelMetadata as AnyModelMetadata,
};

/// Diablo III appearance, skeleton, material, and animation authoring vocabulary.
#[derive(Clone, Debug, Default)]
pub struct D3;
impl Profile for D3 {
    type ImageResource<'a> = SnoResource;
    type ModelResource<'a> = SnoResource;
    type EffectResource<'a> = SnoResource;
    type PhysicsResource<'a> = Unavailable;
    type BoneOverrideResource<'a> = Unavailable;

    type Property = Unavailable;
    type Appearance<'a> = Appearance<'a>;
    type Binding<'a> = ExplicitBinding<'a>;
    type Geometry<'a> = Geometry<'a>;
    type ModelMetadata<'a> = ModelMetadata;
    type System<'a> = Unavailable;
    type Surface<'a> = Material<'a>;
    type Feature<'a> = Unavailable;
    type Node<'a> = NodeRole<'a>;
    type Mesh<'a> = MeshControls;
    type Instance<'a> = NoControls;
    type Draw = NoControls;
    type Clip<'a> = ClipControls;
    type Controller<'a> = NoControls;
    fn default_bounds() -> Option<Bounds> {
        Some(Bounds::default())
    }
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        model.validate_animation_consumers()?;
        if model.bounds().is_none()
            || model.basis() != Basis::default()
            || model.units() != LengthUnit::Unspecified
        {
            return Err(Error::new(
                "d3.model.coordinates",
                "expected authored bounds, the fixed Diablo basis, and unspecified physical units",
            ));
        }
        if let LookAtTarget::Bone(id) = model.metadata().look_at.target {
            if !model
                .node(id)
                .is_some_and(|node| matches!(node.controls, NodeRole::Bone(_)))
            {
                return Err(Error::new(
                    "d3.look_at.target",
                    "look-at target must reference an actual skeleton bone",
                ));
            }
        }
        for (_, node) in model.nodes() {
            uniform_transform(&node.transform.defaults(model.values()))?;
            if node.billboard != Billboard::None
                || node.inheritance != Inheritance::default()
                || model.values().get(node.visible) != Some(&true)
                || node.camera_anchored
            {
                return Err(Error::new("d3.node", "generic visibility, billboard, and inheritance overrides have no native Diablo III mapping"));
            }
            match &node.controls {
                NodeRole::ModelSpace
                    if node.parent.is_some()
                        || node.transform.defaults(model.values()) != Transform::default()
                        || !node.name.is_empty() =>
                {
                    return Err(Error::new(
                        "d3.node.model_space",
                        "implicit model frame must be unnamed and fixed",
                    ))
                }
                NodeRole::Bone(controls) => {
                    for pose in [
                        controls.bind_pose,
                        controls.inverse_bind_pose,
                        controls.alternate_bind_pose,
                        controls.inverse_alternate_bind_pose,
                    ] {
                        uniform_transform(&pose)?;
                    }
                }
                _ => {}
            }
        }
        for (_, skin) in model.skins() {
            if skin.joints.iter().any(|id| {
                !model
                    .node(*id)
                    .is_some_and(|node| matches!(node.controls, NodeRole::Bone(_)))
            }) {
                return Err(Error::new(
                    "d3.skin.joints",
                    "skin joints must reference actual skeleton bones",
                ));
            }
        }
        for (_, mesh) in model.meshes() {
            if mesh
                .controls
                .material_slot
                .is_none_or(|id| model.appearance().slot(id).is_none())
            {
                return Err(Error::new(
                    "d3.mesh.material_slot",
                    "geometry must select an actual appearance-owned material slot",
                ));
            }
            if mesh.controls.bone.is_some_and(|id| {
                !model
                    .node(id)
                    .is_some_and(|node| matches!(node.controls, NodeRole::Bone(_)))
            }) {
                return Err(Error::new(
                    "d3.mesh.bone",
                    "subobject bone must reference an actual skeleton bone",
                ));
            }
        }
        if model
            .instances()
            .iter()
            .any(|(_, instance)| !instance.draws.is_empty())
        {
            return Err(Error::new(
                "d3.instance.draws",
                "material assignments belong to actual appearance slots",
            ));
        }
        if model
            .global_channels()
            .find(|v| v.clock() == TimeSource::Timeline)
            .is_some()
            || model
                .global_channels()
                .find(|v| matches!(v.clock(), TimeSource::Shared(_)))
                .is_some()
            || !model.controllers().is_empty()
        {
            return Err(Error::new(
                "d3.animation",
                "Diablo III animations use permutation-local programs",
            ));
        }
        Ok(())
    }
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        for resource in model
            .dependencies()
            .images()
            .iter()
            .map(|(_, value)| value)
            .chain(model.dependencies().models().iter().map(|(_, value)| value))
            .chain(
                model
                    .dependencies()
                    .effects()
                    .iter()
                    .map(|(_, value)| value),
            )
        {
            if !matches!(
                resource,
                SnoResource::Sno {
                    namespace: SnoNamespace::Diablo3,
                    ..
                }
            ) {
                return Err(Error::new(
                    "d3.texture.resource",
                    "Diablo III textures require a game-qualified SNO identity",
                ));
            }
        }
        Ok(())
    }
}
impl SurfacePayload for Material<'_> {
    fn validate(&self) -> Result<(), Error> {
        Material::validate(self)
    }
}
impl Extends<D3> for Universal {
    fn image_resource<'a>(value: <D3 as Profile>::ImageResource<'a>) -> Self::ImageResource<'a> {
        value.into()
    }
    fn model_resource<'a>(value: <D3 as Profile>::ModelResource<'a>) -> Self::ModelResource<'a> {
        value.into()
    }
    fn effect_resource<'a>(value: <D3 as Profile>::EffectResource<'a>) -> Self::EffectResource<'a> {
        value.into()
    }
    fn physics_resource<'a>(
        value: <D3 as Profile>::PhysicsResource<'a>,
    ) -> Self::PhysicsResource<'a> {
        match value {}
    }
    fn bone_override_resource<'a>(
        value: <D3 as Profile>::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        match value {}
    }

    fn property(value: <D3 as Profile>::Property) -> Self::Property {
        match value {}
    }
    fn appearance<'a>(value: <D3 as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::D3(value)
    }
    fn binding<'a>(value: <D3 as Profile>::Binding<'a>) -> Self::Binding<'a> {
        UniversalBinding::Explicit(value)
    }
    fn geometry<'a>(value: <D3 as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn model_metadata<'a>(value: <D3 as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        Some(AnyModelMetadata::D3(value))
    }
    fn system<'a>(value: <D3 as Profile>::System<'a>) -> Self::System<'a> {
        match value {}
    }
    fn surface<'a>(value: <D3 as Profile>::Surface<'a>) -> Self::Surface<'a> {
        value.into()
    }
    fn feature<'a>(
        value: <D3 as Profile>::Feature<'a>,
        _values: &mut PropertyValues,
    ) -> Self::Feature<'a> {
        match value {}
    }
    fn node<'a>(value: <D3 as Profile>::Node<'a>) -> Self::Node<'a> {
        Some(NodeControls::D3(value))
    }
    fn mesh<'a>(value: <D3 as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        Some(AnyMeshControls::D3(value))
    }
    fn instance<'a>(value: <D3 as Profile>::Instance<'a>) -> Self::Instance<'a> {
        let _ = value;
        None
    }
    fn draw(value: <D3 as Profile>::Draw) -> Self::Draw {
        let _ = value;
        None
    }
    fn clip<'a>(value: <D3 as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Some(AnyClipControls::D3(value))
    }
    fn controller<'a>(value: <D3 as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        None
    }
}
impl Narrows<Universal> for D3 {
    fn image_resource<'a>(
        value: <Universal as Profile>::ImageResource<'a>,
    ) -> Result<Self::ImageResource<'a>, Error> {
        value.try_into()
    }
    fn model_resource<'a>(
        value: <Universal as Profile>::ModelResource<'a>,
    ) -> Result<Self::ModelResource<'a>, Error> {
        value.try_into()
    }
    fn effect_resource<'a>(
        value: <Universal as Profile>::EffectResource<'a>,
    ) -> Result<Self::EffectResource<'a>, Error> {
        value.try_into()
    }
    fn physics_resource<'a>(
        _value: <Universal as Profile>::PhysicsResource<'a>,
    ) -> Result<Self::PhysicsResource<'a>, Error> {
        Err(Error::new(
            "resources",
            "resource kind is unavailable in this profile",
        ))
    }
    fn bone_override_resource<'a>(
        _value: <Universal as Profile>::BoneOverrideResource<'a>,
    ) -> Result<Self::BoneOverrideResource<'a>, Error> {
        Err(Error::new(
            "resources",
            "resource kind is unavailable in this profile",
        ))
    }
    fn appearance<'a>(
        value: <Universal as Profile>::Appearance<'a>,
    ) -> Result<Self::Appearance<'a>, Error> {
        match value {
            UniversalAppearance::D3(value) => Ok(value),
            _ => Err(Error::new(
                "d3.narrow.appearance",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn binding<'a>(value: <Universal as Profile>::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        match value {
            UniversalBinding::Explicit(value) => Ok(value),
            _ => Err(Error::new(
                "d3.narrow.binding",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn geometry<'a>(
        value: <Universal as Profile>::Geometry<'a>,
    ) -> Result<Self::Geometry<'a>, Error> {
        value.try_into()
    }
    fn model_metadata<'a>(
        value: <Universal as Profile>::ModelMetadata<'a>,
    ) -> Result<Self::ModelMetadata<'a>, Error> {
        match value {
            Some(AnyModelMetadata::D3(value)) => Ok(value),
            _ => Err(Error::new(
                "d3.narrow.model",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        let _ = value;
        Err(Error::new(
            "d3.narrow.system",
            "object family is unavailable in this vocabulary",
        ))
    }
    fn surface<'a>(value: <Universal as Profile>::Surface<'a>) -> Result<Self::Surface<'a>, Error> {
        value.try_into()
    }
    fn feature<'a>(
        value: <Universal as Profile>::Feature<'a>,
        _values: &PropertyValues,
    ) -> Result<Self::Feature<'a>, Error> {
        let _ = value;
        Err(Error::new(
            "d3.narrow.feature",
            "object family is unavailable in this vocabulary",
        ))
    }
    fn node<'a>(value: <Universal as Profile>::Node<'a>) -> Result<Self::Node<'a>, Error> {
        match value {
            Some(NodeControls::D3(value)) => Ok(value),
            _ => Err(Error::new(
                "d3.narrow.node",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn mesh<'a>(value: <Universal as Profile>::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        match value {
            Some(AnyMeshControls::D3(value)) => Ok(value),
            _ => Err(Error::new(
                "d3.narrow.mesh",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn instance<'a>(
        value: <Universal as Profile>::Instance<'a>,
    ) -> Result<Self::Instance<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(Error::new(
                "d3.narrow.instance",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn draw(value: <Universal as Profile>::Draw) -> Result<Self::Draw, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(Error::new(
                "d3.narrow.draw",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn clip<'a>(value: <Universal as Profile>::Clip<'a>) -> Result<Self::Clip<'a>, Error> {
        match value {
            Some(AnyClipControls::D3(value)) => Ok(value),
            _ => Err(Error::new(
                "d3.narrow.clip",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn controller<'a>(
        value: <Universal as Profile>::Controller<'a>,
    ) -> Result<Self::Controller<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(Error::new(
                "d3.narrow.controller",
                "controller behavior belongs to another vocabulary",
            )),
        }
    }
}
