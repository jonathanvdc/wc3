//! Built-in M3 authoring vocabulary and its lossless embeddings.
use super::properties::Target as PropertySelector;
use super::properties::{Field, FieldSegment};
use crate::model::materials::m3::properties::MaterialProperties;
use crate::model::profiles::m3;
use crate::model::profiles::Unavailable;
use crate::model::profiles::UniversalProperty;
use crate::model::profiles::{self, Extends, Profile, SurfacePayload, Universal};
use crate::model::resources::PathResource;
use crate::model::PropertyValues;
use crate::model::{
    Basis, Billboard, Channel, ClipControls, DrawControls, Error, ExplicitBinding, FeatureKind,
    GeometrySchema as _, Inheritance, InstanceControls, Interpolation, LengthUnit, Link,
    M3Geometry, MeshControls, Model, ModelMetadata, NodeControls, SingleLook, SkinId, System,
    TimeSource, Topology, Transform, UniversalAppearance, UniversalBinding, ValidateLinks,
    VertexInfluences,
};
use crate::model::{FeatureSet, FeatureView, FeatureVocabulary};
use crate::model::{PropertyIdentityKey, PropertyPayload};
use std::collections::HashSet;

use std::str::from_utf8;
/// Vocabulary that maps mechanically to M3 logical features.
#[derive(Clone, Debug, Default)]
pub struct M3;
impl Profile for M3 {
    type ImageResource<'a> = PathResource<'a>;
    type ModelResource<'a> = PathResource<'a>;
    type EffectResource<'a> = Unavailable;
    type PhysicsResource<'a> = Unavailable;
    type BoneOverrideResource<'a> = Unavailable;

    type Property = PropertySelector;
    fn default_bounds() -> Option<crate::model::Bounds> {
        Some(Default::default())
    }
    type Appearance<'a> = SingleLook<'a>;
    type Binding<'a> = ExplicitBinding<'a>;
    type Geometry<'a> = M3Geometry<'a>;
    type Draw = Option<profiles::m3::BatchControls>;
    type ModelMetadata<'a> = profiles::m3::ModelMetadata;
    type System<'a> = profiles::m3::System<'a>;
    type Surface<'a> = profiles::m3::MaterialStep<'a>;
    type Feature<'a> = profiles::m3::Feature<'a>;
    type Node<'a> = profiles::m3::NodeRole;
    type Mesh<'a> = Option<profiles::m3::MeshControls>;
    fn mesh_skin(controls: &Self::Mesh<'_>) -> Option<SkinId> {
        controls.as_ref().and_then(|value| value.skin)
    }
    type Instance<'a> = Option<profiles::m3::InstanceControls>;
    type Clip<'a> = profiles::m3::SequenceControls<'a>;
    type Controller<'a> = profiles::m3::ControllerControls<'a>;
    fn validate_model(model: &Model<'_, Self>) -> Result<(), Error> {
        model.validate_animation_consumers()?;
        if model.bounds().is_none() {
            return Err(Error::new(
                "model.bounds",
                "this profile requires an authored bounding volume",
            ));
        }

        if model.basis() != Basis::default() {
            return Err(Error::new(
                "m3.basis",
                "M3 uses its fixed authored axes; a basis translation must precede native construction",
            ));
        }
        if model.units() != LengthUnit::Unspecified {
            return Err(Error::new(
                "m3.units",
                "M3 has no independently authored physical-unit declaration",
            ));
        }
        if model
            .systems()
            .iter()
            .filter(|(_, system)| matches!(system, profiles::m3::System::Physics(_)))
            .count()
            > 1
        {
            return Err(Error::new(
                "m3.physics.worlds",
                "native M3 declares one model-wide simulation domain",
            ));
        }
        let mut missing_selection = false;
        let mut legacy_presence = None;
        for (_, clip) in model.clips() {
            if let Some(previous) = legacy_presence {
                if previous != clip.controls.legacy_control.is_some() {
                    return Err(Error::new(
                        "m3.clips.legacy_control",
                        "sequence declarations must agree on legacy control availability",
                    ));
                }
            } else {
                legacy_presence = Some(clip.controls.legacy_control.is_some());
            }
            if clip.controls.selection.is_none() {
                missing_selection = true;
            } else if missing_selection {
                return Err(Error::new(
                    "m3.clips.selection",
                    "declared positional selections cannot follow a missing selection",
                ));
            }
        }
        if missing_selection && model.systems().iter().any(|(_,system)|matches!(system,profiles::m3::System::ControllerSelections(groups) if !groups.is_empty())) {
            return Err(Error::new("m3.controller_selections", "unassociated selections require a declaration for every positional playback selection"));
        }
        for (_, node) in model.nodes() {
            if matches!(node.controls, profiles::m3::NodeRole::ModelSpace)
                && (!node.name.is_empty()
                    || node.parent.is_some()
                    || node.transform.defaults(model.values()) != Transform::default()
                    || node.inheritance != Inheritance::default()
                    || node.billboard != Billboard::None)
            {
                return Err(Error::new(
                    "m3.node.model_space",
                    "the implicit model frame is unnamed, fixed, and independent of the skeleton",
                ));
            }
            if let profiles::m3::NodeRole::Bone(controls) = &node.controls {
                if controls.extra_flags.bits() & 0x10 != 0 {
                    return Err(Error::new(
                        "m3.node.flags",
                        "the interpreted billboard control belongs to the node field",
                    ));
                }
            }
            if node.transform.defaults(model.values()).pivot != [0.; 3] {
                return Err(Error::new(
                    "m3.node.pivot",
                    "M3 transforms have no separate pivot",
                ));
            }
            if model.values().get(node.visible) != Some(&true) || node.camera_anchored {
                return Err(Error::new(
                    "m3.node",
                    "M3 has no generic visibility or camera-origin control",
                ));
            }
            if matches!(node.billboard, Billboard::Locked { .. }) {
                return Err(Error::new(
                    "m3.node.billboard",
                    "axis-locked billboard behavior has no established M3 bone mapping",
                ));
            }
        }
        if model
            .clips()
            .iter()
            .any(|(_, clip)| !clip.animations.is_empty())
        {
            return Err(Error::new(
                "m3.clip.program",
                "M3 sequence selectors use controller-owned programs",
            ));
        }
        if model.global_channels().next().is_some() || model.global_events().next().is_some() {
            return Err(Error::new(
                "m3.animation",
                "M3 animation programs require native controller selections",
            ));
        }
        for (_, controller) in model.controllers() {
            for channel in model.animation_channels(&controller.animations) {
                let mode = match channel {
                    Channel::Scalar(t) => t.curve.interpolation(),
                    Channel::Vector2(t) => t.curve.interpolation(),
                    Channel::Vector3(t) => t.curve.interpolation(),
                    Channel::Rotation(t) => t.curve.interpolation(),
                    Channel::Bytes4(t) => t.curve.interpolation(),
                    Channel::Byte(t) => t.curve.interpolation(),
                    Channel::Unsigned16(t) => t.curve.interpolation(),
                    Channel::Unsigned(t) => t.curve.interpolation(),
                    Channel::Signed(t) => t.curve.interpolation(),
                    Channel::Vector4(t) => t.curve.interpolation(),
                    _ => {
                        return Err(Error::new(
                            "m3.controller.value",
                            "channel value domain has no matching native M3 key family",
                        ));
                    }
                };
                if !matches!(mode, Interpolation::Step | Interpolation::Linear) {
                    return Err(Error::new(
                        "m3.controller.interpolation",
                        "M3 property headers encode held or linear keys without cubic tangent controls",
                    ));
                }
            }

            if model
                .animation_channels(&controller.animations)
                .any(|channel| channel.clock() != TimeSource::Timeline)
                || model
                    .animation_events(&controller.animations)
                    .any(|event| event.clock != TimeSource::Timeline)
            {
                return Err(Error::new(
                    "m3.controller.clock",
                    "M3 controller programs use the absolute authored timeline",
                ));
            }
            if controller.controls.blocks.iter().any(|block| {
                block.events.iter().any(|id| {
                    !controller.animations.iter().any(|animation| {
                        model.animation(*animation).unwrap().events.contains_key(id)
                    })
                })
            }) {
                return Err(Error::new(
                    "m3.controller.events",
                    "a block occurrence must belong to its owning controller program",
                ));
            }
        }
        if model.skins().iter().any(|(_, skin)| {
            skin.joints.iter().any(|id| {
                !model
                    .node(*id)
                    .is_some_and(|node| matches!(node.controls, profiles::m3::NodeRole::Bone(_)))
            })
        }) {
            return Err(Error::new(
                "m3.skin.joints",
                "M3 joint palettes must reference bone definitions",
            ));
        }
        let bone = |id| {
            model
                .node(id)
                .is_some_and(|node| matches!(node.controls, profiles::m3::NodeRole::Bone(_)))
        };
        for (id, mesh) in model.meshes() {
            if !model
                .instances()
                .iter()
                .any(|(_, instance)| instance.mesh == id)
                && mesh.skin().is_none()
                && mesh
                    .influences()
                    .iter()
                    .filter_map(VertexInfluences::into_joint_weights)
                    .flatten()
                    .any(|influence| influence.weight > 0.)
            {
                return Err(Error::new(
                    "m3.region.skin",
                    "uninstanced skinned definitions require a definition-level binding",
                ));
            }
            if let Some(control) = &mesh.controls {
                if control.undrawn.is_some()
                    && model
                        .instances()
                        .iter()
                        .any(|(_, instance)| instance.mesh == id)
                {
                    return Err(Error::new(
                        "m3.region.placement",
                        "undrawn definition controls cannot coexist with draw instances",
                    ));
                }
                if let Some(skin_id) = control.skin {
                    if model
                        .instances()
                        .iter()
                        .any(|(_, instance)| instance.mesh == id && instance.skin.is_some())
                    {
                        return Err(Error::new(
                            "m3.region.skin",
                            "region and instance cannot both define a skeleton binding",
                        ));
                    }
                    let skin = model.skin(skin_id).expect("checked definition binding");
                    if mesh
                        .influences()
                        .iter()
                        .filter_map(VertexInfluences::into_joint_weights)
                        .flatten()
                        .any(|influence| {
                            influence.weight > 0. && influence.joint as usize >= skin.joints.len()
                        })
                    {
                        return Err(Error::new(
                            "m3.region.skin",
                            "influence references a joint outside the definition binding",
                        ));
                    }
                } else if control.undrawn.is_some()
                    && mesh
                        .influences()
                        .iter()
                        .filter_map(VertexInfluences::into_joint_weights)
                        .flatten()
                        .any(|influence| influence.weight > 0.)
                {
                    return Err(Error::new(
                        "m3.region.skin",
                        "undrawn skinned definitions require an authored binding",
                    ));
                }
            }
        }
        let mut precise_volume = false;
        for (_, attached) in model.features() {
            if let profiles::m3::Feature::Extent(bounds) = &attached.payload {
                if bounds.detail_level.is_none() || !bone(attached.node) {
                    return Err(Error::new(
                        "m3.bounds",
                        "active M3 bounds require a bone anchor and an authored detail group",
                    ));
                }
            }
            if !matches!(
                attached.payload,
                profiles::m3::Feature::Event(_) | profiles::m3::Feature::Extent(_)
            ) && !bone(attached.node)
            {
                return Err(Error::new(
                    "m3.feature.node",
                    "this M3 feature requires an actual bone anchor",
                ));
            }
            let volume_mesh = match &attached.payload {
                profiles::m3::Feature::HitTest(shape) => {
                    if shape.role == profiles::m3::HitTestRole::Tight {
                        if precise_volume {
                            return Err(Error::new(
                                "m3.hit_shape.tight",
                                "M3 permits one precise selection volume",
                            ));
                        }
                        precise_volume = true;
                    }
                    shape.mesh
                }
                profiles::m3::Feature::AttachmentVolume(volume) => {
                    if !bone(volume.bone1) || !bone(volume.bone2) {
                        return Err(Error::new(
                            "m3.attachment_volume.bones",
                            "attachment volume joints require bone definitions",
                        ));
                    }
                    volume.mesh
                }
                _ => None,
            };
            if let Some(mesh) = volume_mesh {
                let mesh = model.mesh(mesh).expect("checked volume mesh");
                let M3Geometry::Collision(geometry) = &mesh.geometry else {
                    return Err(Error::new(
                        "m3.volume.mesh",
                        "selection volumes require collision geometry",
                    ));
                };
                if geometry.triangle_controls().is_some() || geometry.face_normals().is_some() {
                    return Err(Error::new(
                        "m3.volume.mesh",
                        "selection volumes have no triangle controls or face-normal storage",
                    ));
                }
                match geometry.topology() {
                    Topology::Triangles(rows) => {
                        if rows
                            .iter()
                            .flatten()
                            .any(|index| index > u32::from(u16::MAX))
                        {
                            return Err(Error::new(
                                "m3.volume.mesh",
                                "selection volume indices exceed native 16-bit capacity",
                            ));
                        }
                    }
                    Topology::Points(_) => {}
                    _ => {
                        return Err(Error::new(
                            "m3.volume.mesh",
                            "selection volumes support triangle or point geometry",
                        ));
                    }
                }
            }
        }
        for (_, instance) in model.instances() {
            for draw in &instance.draws {
                if let Some(id) = draw.controls.as_ref().and_then(|controls| controls.bounds) {
                    let feature = model.feature(id).expect("checked draw bounds");
                    let profiles::m3::Feature::Extent(bounds) = &feature.payload else {
                        continue;
                    };
                    if bounds.detail_level != Some(instance.lod.unwrap_or(0)) {
                        return Err(Error::new(
                            "m3.batch.bounds",
                            "draw and bounds definition belong to different detail groups",
                        ));
                    }
                }
            }
        }
        let mut body_program = None;
        let mut body_ids = HashSet::new();
        for (_, system) in model.systems() {
            if let profiles::m3::System::Physics(system) = system {
                for body in &system.bodies {
                    if !body_ids.insert(body.id()) {
                        return Err(Error::new(
                            "m3.physics.body",
                            "body definitions cannot be duplicated between physics worlds",
                        ));
                    }
                    let declaration = (body.parameters.is_some(), body.dynamic.is_some());
                    if body_program.is_some_and(|previous| previous != declaration)
                        || (body.parameters.is_none()
                            && (body.dynamic.is_some() || body.simulation_type != 0))
                    {
                        return Err(Error::new(
                            "m3.physics.body",
                            "the native body table requires consistent authored parameter and state declarations",
                        ));
                    }
                    body_program = Some(declaration);
                }
                if system.bodies.iter().any(|body| !bone(body.node))
                    || system
                        .joints
                        .iter()
                        .any(|joint| !bone(joint.node_a) || !bone(joint.node_b))
                {
                    return Err(Error::new(
                        "m3.physics.node",
                        "M3 physics attachments require bone definitions",
                    ));
                }
            }
            if let profiles::m3::System::Cloth(system) = system {
                for cloth in &system.cloths {
                    let count = model
                        .mesh(cloth.cage)
                        .expect("checked cloth cage")
                        .positions()
                        .len();
                    if cloth.anchors.len() != count {
                        return Err(Error::new(
                            "m3.cloth.anchors",
                            "each cage vertex requires one anchor row",
                        ));
                    }
                    for proxy in &cloth.proxies {
                        let output_count = model
                            .mesh(proxy.mesh)
                            .expect("checked cloth proxy")
                            .positions()
                            .len();
                        let cage_count = model
                            .mesh(system.cloths[proxy.cloth.index()].cage)
                            .expect("checked target cage")
                            .positions()
                            .len();
                        if proxy.influences.len() != output_count
                            || proxy.influences.iter().any(|row| {
                                row.iter().any(|influence| {
                                    influence.weight != 0.
                                        && influence.vertex as usize >= cage_count
                                })
                            })
                        {
                            return Err(Error::new(
                                "m3.cloth.proxy",
                                "proxy contributions must cover its vertices and identify cage vertices",
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_resources(model: &Model<'_, Self>) -> Result<(), Error> {
        for (_, resource) in model.dependencies().images().iter() {
            match resource {
                PathResource::Path(path)
                    if !path.is_empty() && from_utf8(path).is_ok() && !path.contains(&0) => {}
                _ => {
                    return Err(Error::new(
                        "m3.image.resource",
                        "M3 images require nonempty UTF-8 paths without embedded NULs",
                    ))
                }
            }
        }
        if !model.sampled_images().is_empty() {
            return Err(Error::new(
                "m3.image",
                "M3 image sampling must be declared by its material inputs",
            ));
        }
        Ok(())
    }
}
impl SurfacePayload for profiles::m3::MaterialStep<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        PropertySelector::from_identity(key).is_some_and(|key| match key {
            PropertySelector::Field(path) => match path.segments().split_first() {
                Some((FieldSegment::Field(Field::Material), rest)) => self.supports_path(rest),
                _ => false,
            },
            _ => false,
        })
    }

    fn validate(&self) -> Result<(), Error> {
        let mut invalid = false;
        self.visit_links(&mut |link| invalid |= matches!(link, Link::InvalidValue));
        if invalid {
            Err(Error::new(
                "m3.material",
                "invalid M3 shader input or rendering state",
            ))
        } else {
            Ok(())
        }
    }
}
impl profiles::m3::Feature<'_> {
    pub(crate) fn profile_supports_property(&self, key: &PropertyIdentityKey) -> bool {
        super::properties::Target::from_identity(key).is_some_and(|key| key.supports_feature(self))
    }

    pub(crate) fn profile_payload_kind(&self) -> FeatureKind {
        match self {
            profiles::m3::Feature::Particle(_) => FeatureKind::M3Particle,
            profiles::m3::Feature::ParticleCopy(_) => FeatureKind::M3ParticleCopy,
            profiles::m3::Feature::Ribbon(_) => FeatureKind::M3Ribbon,
            profiles::m3::Feature::Projector(_) => FeatureKind::M3Projection,
            profiles::m3::Feature::Extent(_) => FeatureKind::M3Extent,
            profiles::m3::Feature::Turret(_) => FeatureKind::M3Turret,
            _ => FeatureKind::Other,
        }
    }
}
impl Extends<M3> for Universal {
    fn image_resource<'a>(value: <M3 as Profile>::ImageResource<'a>) -> Self::ImageResource<'a> {
        value.into()
    }
    fn model_resource<'a>(value: <M3 as Profile>::ModelResource<'a>) -> Self::ModelResource<'a> {
        value.into()
    }
    fn effect_resource<'a>(value: <M3 as Profile>::EffectResource<'a>) -> Self::EffectResource<'a> {
        match value {}
    }
    fn physics_resource<'a>(
        value: <M3 as Profile>::PhysicsResource<'a>,
    ) -> Self::PhysicsResource<'a> {
        match value {}
    }
    fn bone_override_resource<'a>(
        value: <M3 as Profile>::BoneOverrideResource<'a>,
    ) -> Self::BoneOverrideResource<'a> {
        match value {}
    }

    fn property(value: <M3 as Profile>::Property) -> Self::Property {
        UniversalProperty::M3(value)
    }
    fn appearance<'a>(value: <M3 as Profile>::Appearance<'a>) -> Self::Appearance<'a> {
        UniversalAppearance::Single(value)
    }
    fn binding<'a>(value: <M3 as Profile>::Binding<'a>) -> Self::Binding<'a> {
        UniversalBinding::Explicit(value)
    }
    fn geometry<'a>(value: <M3 as Profile>::Geometry<'a>) -> Self::Geometry<'a> {
        value.into()
    }
    fn draw(value: <M3 as Profile>::Draw) -> Self::Draw {
        value.map(DrawControls::M3)
    }
    fn model_metadata<'a>(value: <M3 as Profile>::ModelMetadata<'a>) -> Self::ModelMetadata<'a> {
        Some(ModelMetadata::M3(value))
    }
    fn system<'a>(value: <M3 as Profile>::System<'a>) -> Self::System<'a> {
        System::M3(value)
    }
    fn surface<'a>(value: <M3 as Profile>::Surface<'a>) -> Self::Surface<'a> {
        value.into()
    }
    fn feature<'a>(
        value: <M3 as Profile>::Feature<'a>,
        values: &mut PropertyValues,
    ) -> Self::Feature<'a> {
        crate::model::features::FromFeature::from_feature(value, values)
    }
    fn node<'a>(value: <M3 as Profile>::Node<'a>) -> Self::Node<'a> {
        Some(NodeControls::M3(value))
    }
    fn mesh<'a>(value: <M3 as Profile>::Mesh<'a>) -> Self::Mesh<'a> {
        value.map(MeshControls::M3)
    }
    fn instance<'a>(value: <M3 as Profile>::Instance<'a>) -> Self::Instance<'a> {
        value.map(InstanceControls::M3)
    }
    fn clip<'a>(value: <M3 as Profile>::Clip<'a>) -> Self::Clip<'a> {
        Some(ClipControls::M3(value))
    }
    fn controller<'a>(value: <M3 as Profile>::Controller<'a>) -> Self::Controller<'a> {
        Some(value)
    }
}

impl<'a> FeatureVocabulary<'a> for M3 {
    type Light = Box<m3::Light>;
    type Camera = Box<m3::Camera<'a>>;
    type Attachment = Box<m3::Attachment<'a>>;
    type Particle = Box<m3::Particle<'a>>;
    type Ribbon = Box<m3::Ribbon<'a>>;
    type Event = Box<m3::EventCommand<'a>>;
    type GeometryAnimation = Unavailable;
    type CollisionShape = Unavailable;
    type ParticleProgram = Unavailable;
    type FaceActor = Unavailable;
    type LightProgram = Unavailable;
    type Extent = Box<m3::ExtentBinding>;
    type ParticleCopy = Box<m3::ParticleCopy>;
    type Projector = Box<m3::Projector>;
    type Force = Box<m3::Force>;
    type Warp = Box<m3::Warp>;
    type HitTest = Box<m3::HitTestShape>;
    type AttachmentVolume = Box<m3::AttachmentVolume>;
    type Billboard = Box<m3::BillboardBehavior>;
    type Turret = Box<m3::TurretBehavior>;
    type IkJoint = Box<m3::IkJoint>;
    type IkTwoJoint = Box<m3::IkTwoJoint>;
    type IkCcd = Box<m3::IkCcd>;
    type OneBoneSolver = Box<m3::OneBoneSolver>;
    type ShadowBox = Box<m3::ShadowBox>;
    type ViewVolume = Box<m3::ViewVolume>;
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
