//! Diablo IV authoring vocabulary is independent of native SNO revisions.
use super::{Appearance, Geometry, Material, MeshControls, ModelMetadata, NodeRole};
use crate::model::profiles::{Extends, Narrows, NoControls, Profile, Unavailable, Universal};
use crate::model::resources::SnoResource;
use crate::model::PropertyValues;
use crate::model::TimeSource;
use crate::model::{
    Basis, Billboard, Bounds, Error, ExplicitBinding, Inheritance, LengthUnit, Model, NodeControls,
    SnoNamespace, Transform, UniversalAppearance, UniversalBinding,
};
use crate::model::{MeshControls as AnyMeshControls, ModelMetadata as AnyModelMetadata};

/// Diablo IV appearance, skeleton, and material authoring vocabulary.
#[derive(Clone, Debug, Default)]
pub struct D4;
impl Profile for D4 {
    type ImageResource<'a> = SnoResource;
    type ModelResource<'a> = SnoResource;
    type EffectResource<'a> = SnoResource;
    type PhysicsResource<'a> = Unavailable;
    type BoneOverrideResource<'a> = Unavailable;

    type Property = Unavailable;
    type Appearance<'a> = Appearance;
    type Binding<'a> = ExplicitBinding<'a>;
    type Geometry<'a> = Geometry<'a>;
    type ModelMetadata<'a> = ModelMetadata;
    type System<'a> = Unavailable;
    type Surface<'a> = Material;
    type Feature<'a> = Unavailable;
    type Node<'a> = NodeRole;
    type Mesh<'a> = MeshControls;
    type Instance<'a> = NoControls;
    type Draw = NoControls;
    type Clip<'a> = NoControls;
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
                "d4.model.coordinates",
                "expected authored bounds, the fixed Diablo basis, and unspecified physical units",
            ));
        }
        for (_, node) in model.nodes() {
            if node.transform.defaults(model.values()).pivot != [0.; 3]
                || node.billboard != Billboard::None
                || node.inheritance != Inheritance::default()
                || model.values().get(node.visible) != Some(&true)
                || node.camera_anchored
            {
                return Err(Error::new("d4.node", "generic pivots, visibility, billboard, and inheritance overrides have no native mapping"));
            }
            if matches!(node.controls, NodeRole::ModelSpace)
                && (node.parent.is_some()
                    || node.transform.defaults(model.values()) != Transform::default()
                    || !node.name.is_empty())
            {
                return Err(Error::new(
                    "d4.node.model_space",
                    "implicit model frame must be unnamed and fixed",
                ));
            }
        }
        let is_bone = |id| {
            model
                .node(id)
                .is_some_and(|node| matches!(node.controls, NodeRole::Bone(_)))
        };
        for (_, skin) in model.skins() {
            if skin.joints.iter().any(|id| !is_bone(*id)) {
                return Err(Error::new(
                    "d4.skin.joints",
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
                    "d4.mesh.material_slot",
                    "subobject must select a material slot owned by this appearance",
                ));
            }
            if mesh
                .controls
                .chunk
                .is_none_or(|id| model.metadata().chunk(id).is_none())
            {
                return Err(Error::new(
                    "d4.mesh.chunk",
                    "geometry must select a chunk owned by this model",
                ));
            }
            for segment in &mesh.controls.segments {
                if segment.first_vertex as usize + segment.vertex_count as usize
                    > mesh.positions().len()
                    || segment.first_triangle as usize + segment.triangle_count as usize
                        > mesh.topology().primitive_count()
                    || segment.bones.iter().any(|id| !is_bone(*id))
                {
                    return Err(Error::new(
                        "d4.mesh.segments",
                        "segment window or bone palette is outside its owning geometry or skeleton",
                    ));
                }
            }
            if mesh.controls.base_lod_subobject.is_some_and(|id| {
                model
                    .mesh(id)
                    .is_none_or(|base| base.controls.chunk != mesh.controls.chunk)
            }) {
                return Err(Error::new(
                    "d4.mesh.base_lod",
                    "base-detail subobject must belong to the same geometry chunk",
                ));
            }
        }
        for (_, instance) in model.instances() {
            if !instance.draws.is_empty() {
                return Err(Error::new(
                    "d4.instance.draws",
                    "Diablo IV material assignments belong to actual appearance slots",
                ));
            }
            let mesh = model
                .mesh(instance.mesh)
                .expect("checked mesh relationship");
            let chunk = model
                .metadata()
                .chunk(mesh.controls.chunk.expect("checked chunk"))
                .expect("checked chunk ownership");
            if instance
                .lod
                .is_none_or(|lod| lod as usize >= chunk.lod_distances.len())
            {
                return Err(Error::new(
                    "d4.instance.lod",
                    "placement must select a detail level declared by its chunk",
                ));
            }
        }
        if !model.clips().is_empty()
            || model
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
                "d4.animation",
                "compressed Diablo IV animation programs require an established semantic decoder",
            ));
        }
        Ok(())
    }
    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        if !model.sampled_images().is_empty() {
            return Err(Error::new(
                "d4.texture",
                "Diablo IV sampling belongs to shader inputs",
            ));
        }
        for (_, resource) in model.dependencies().images().iter() {
            if !matches!(
                resource,
                SnoResource::Sno {
                    namespace: SnoNamespace::Diablo4,
                    ..
                }
            ) {
                return Err(Error::new(
                    "d4.texture.resource",
                    "Diablo IV textures require a game-qualified SNO identity",
                ));
            }
        }
        Ok(())
    }
}
impl Extends<D4> for Universal {
    fn image_resource<'a>(value: <D4 as Profile>::ImageResource<'a>) -> Self::ImageResource<'a> {
        value.into()
    }
    fn model_resource<'a>(value: <D4 as Profile>::ModelResource<'a>) -> Self::ModelResource<'a> {
        value.into()
    }
    fn effect_resource<'a>(value: <D4 as Profile>::EffectResource<'a>) -> Self::EffectResource<'a> {
        value.into()
    }
    fn physics_resource<'a>(
        value: <D4 as Profile>::PhysicsResource<'a>,
    ) -> Self::PhysicsResource<'a> {
        match value {}
    }
    fn bone_override_resource<'a>(
        value: <D4 as Profile>::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        match value {}
    }

    fn property(value: <D4 as Profile>::Property) -> Self::Property {
        match value {}
    }
    fn appearance<'a>(value: <D4 as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::D4(value)
    }
    fn binding<'a>(value: <D4 as Profile>::Binding<'a>) -> Self::Binding<'a> {
        UniversalBinding::Explicit(value)
    }
    fn geometry<'a>(value: <D4 as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn model_metadata<'a>(value: <D4 as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        Some(AnyModelMetadata::D4(value))
    }
    fn system<'a>(value: <D4 as Profile>::System<'a>) -> Self::System<'a> {
        match value {}
    }
    fn surface<'a>(value: <D4 as Profile>::Surface<'a>) -> Self::Surface<'a> {
        value.into()
    }
    fn feature<'a>(
        value: <D4 as Profile>::Feature<'a>,
        _values: &mut PropertyValues,
    ) -> Self::Feature<'a> {
        match value {}
    }
    fn node<'a>(value: <D4 as Profile>::Node<'a>) -> Self::Node<'a> {
        Some(NodeControls::D4(value))
    }
    fn mesh<'a>(value: <D4 as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        Some(AnyMeshControls::D4(value))
    }
    fn instance<'a>(value: <D4 as Profile>::Instance<'a>) -> Self::Instance<'a> {
        let _ = value;
        None
    }
    fn draw(value: <D4 as Profile>::Draw) -> Self::Draw {
        let _ = value;
        None
    }
    fn clip<'a>(value: <D4 as Profile>::Clip<'a>) -> Self::Clip<'a> {
        let _ = value;
        None
    }
    fn controller<'a>(value: <D4 as Profile>::Controller<'a>) -> Self::Controller<'a> {
        let _ = value;
        None
    }
}
impl Narrows<Universal> for D4 {
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
            UniversalAppearance::D4(value) => Ok(value),
            _ => Err(Error::new(
                "d4.narrow.appearance",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn binding<'a>(value: <Universal as Profile>::Binding<'a>) -> Result<Self::Binding<'a>, Error> {
        match value {
            UniversalBinding::Explicit(value) => Ok(value),
            _ => Err(Error::new(
                "d4.narrow.binding",
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
            Some(AnyModelMetadata::D4(value)) => Ok(value),
            _ => Err(Error::new(
                "d4.narrow.model",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn system<'a>(value: <Universal as Profile>::System<'a>) -> Result<Self::System<'a>, Error> {
        let _ = value;
        Err(Error::new(
            "d4.narrow.system",
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
            "d4.narrow.feature",
            "object family is unavailable in this vocabulary",
        ))
    }
    fn node<'a>(value: <Universal as Profile>::Node<'a>) -> Result<Self::Node<'a>, Error> {
        match value {
            Some(NodeControls::D4(value)) => Ok(value),
            _ => Err(Error::new(
                "d4.narrow.node",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn mesh<'a>(value: <Universal as Profile>::Mesh<'a>) -> Result<Self::Mesh<'a>, Error> {
        match value {
            Some(AnyMeshControls::D4(value)) => Ok(value),
            _ => Err(Error::new(
                "d4.narrow.mesh",
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
                "d4.narrow.instance",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn draw(value: <Universal as Profile>::Draw) -> Result<Self::Draw, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(Error::new(
                "d4.narrow.draw",
                "payload belongs to another authoring vocabulary",
            )),
        }
    }
    fn clip<'a>(value: <Universal as Profile>::Clip<'a>) -> Result<Self::Clip<'a>, Error> {
        match value {
            None => Ok(NoControls),
            _ => Err(Error::new(
                "d4.narrow.clip",
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
                "d4.narrow.controller",
                "controller behavior belongs to another vocabulary",
            )),
        }
    }
}
