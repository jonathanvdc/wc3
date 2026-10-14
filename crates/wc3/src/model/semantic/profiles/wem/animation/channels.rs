//! Declared channels own initialization independently from actor payload baselines.
use super::super::materials::ShaderLayerId;
use super::super::physics::{
    BodyId, ClothId, Physics, RigId, SoftBodyId, SoftEffectId, SoftLumpId,
};
use super::super::{AlternativeId, SectionId, SetId};
use super::super::{PosePipeline, StageId, StageSourceId};
use crate::model::semantic::{
    Curve, CurveValue, Error, Id, Link, MaterialSlotId, MeshId, NodeId, Property, Quaternion,
    Scope, SystemId, TcbParameters, TextureId, ValidateLinks, Values,
};
use std::any::TypeId;
use std::borrow::Cow;
use std::marker::PhantomData;

/// Identity marker of a declaration with one exact value domain.
pub struct ChannelObject<T>(PhantomData<fn() -> T>);
/// Stable typed declaration identity owned by one model's animation vocabulary.
pub type ChannelId<T> = Id<ChannelObject<T>>;
/// Authored animation operation; unknown producer choices retain their identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Translation control.
    Translation,
    /// Rotation control.
    Rotation,
    /// Scale control.
    Scale,
    /// Visibility control.
    Visibility,
    /// Color control.
    Color,
    /// Alpha control.
    Alpha,
    /// Intensity control.
    Intensity,
    /// AttenuationStart control.
    AttenuationStart,
    /// AttenuationEnd control.
    AttenuationEnd,
    /// UvTranslate control.
    UvTranslate,
    /// UvRotate control.
    UvRotate,
    /// UvScale control.
    UvScale,
    /// Weight control.
    Weight,
    /// TextureIndex control.
    TextureIndex,
    /// Emissive control.
    Emissive,
    /// EmitterProperty control.
    EmitterProperty,
    /// ShadowCastingStart control.
    ShadowCastingStart,
    /// ShadowCastingEnd control.
    ShadowCastingEnd,
    /// QuadraticFalloff control.
    QuadraticFalloff,
    /// LinearFalloff control.
    LinearFalloff,
    /// Damping control.
    Damping,
    /// StageWeight control.
    StageWeight,
    /// StageSourceWeight control.
    StageSourceWeight,
    /// StageSourceEnabled control.
    StageSourceEnabled,
    /// FocusDistance control.
    FocusDistance,
    /// FocalLength control.
    FocalLength,
    /// FStop control.
    FStop,
    /// Target control.
    Target,
    /// Roll control.
    Roll,
    /// PhysicsDynamic control.
    PhysicsDynamic,
    /// ClothActive control.
    ClothActive,
    /// PhysicsRagdoll control.
    PhysicsRagdoll,
    /// PhysicsBlend control.
    PhysicsBlend,
    /// ClothDriverTranslation control.
    ClothDriverTranslation,
    /// ClothDriverRotation control.
    ClothDriverRotation,
    /// NormalTextureIndex control.
    NormalTextureIndex,
    /// OrmTextureIndex control.
    OrmTextureIndex,
    /// EmissiveTextureIndex control.
    EmissiveTextureIndex,
    /// TeamColorTextureIndex control.
    TeamColorTextureIndex,
    /// EnvironmentTextureIndex control.
    EnvironmentTextureIndex,
    /// SoftAmount control.
    SoftAmount,
    /// SoftLumpTranslation control.
    SoftLumpTranslation,
    /// SoftLumpRotation control.
    SoftLumpRotation,
    /// SoftLumpScale control.
    SoftLumpScale,
    /// Count control.
    Count,
    /// Producer-defined operation.
    Unknown(u8),
}
/// Producer-defined property selection on an actual hierarchy owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeOperation {
    /// Translation.
    Translation,
    /// Quaternion rotation.
    Rotation,
    /// Axis or uniform scaling according to the owner's declaration.
    Scale,
    /// Visibility.
    Visibility,
    /// Color multiplier.
    Color,
    /// Opacity multiplier.
    Alpha,
    /// Light intensity.
    Intensity,
    /// Attenuation start or end.
    Attenuation {
        /// Selects the end distance when true, or start distance when false.
        end: bool,
    },
    /// Unrecognized emitter selector. All producer-defined selector bits are retained;
    /// known emitter controls use the actor field’s typed property destination instead.
    UnknownEmitter(u32),
    /// Producer-defined property selection.
    Other {
        /// Authored control applied to the selected destination.
        operation: Operation,
        /// Producer-defined sub-element identity within this control domain.
        selection: u32,
    },
}
/// A selected material alternative remains an actual dynamic appearance address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialMount {
    /// Owning shader convention set.
    pub set: SetId,
    /// Actual alternative within that set.
    pub alternative: AlternativeId,
    /// Named appearance slot selecting the current material.
    pub slot: MaterialSlotId,
}
/// Component of an animated deformation frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameComponent {
    /// Offset from the authored frame.
    Translation,
    /// Orientation about the authored pivot.
    Rotation,
    /// Scale along the authored axes.
    Scale,
}
/// Control on an actual weighted pipeline input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageSourceControl {
    /// Contribution weight.
    Weight,
    /// Held contribution of this input, expressed as a scalar.
    Enabled,
}
/// Simulation destination, with object identities independent of serialized selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicsControl {
    /// Body simulation switch.
    BodyDynamic(BodyId),
    /// Body animation/simulation contribution.
    BodyBlend(BodyId),
    /// Rig simulation switch.
    RigRagdoll(RigId),
    /// Rig animation/simulation contribution.
    RigBlend(RigId),
    /// Cloth deformation switch.
    ClothActive(ClothId),
    /// Generated free-particle frame, ordered by ascending cage vertex.
    ClothDriver {
        /// Cloth owning the generated frame.
        cloth: ClothId,
        /// Ordinal among free cage particles, in ascending vertex order.
        driver: u16,
        /// Animated frame component.
        component: FrameComponent,
    },
    /// Level of an actual soft-deformation effect.
    SoftEffect {
        /// Soft body owning this destination.
        body: SoftBodyId,
        /// Actual deformation effect within the body.
        effect: SoftEffectId,
    },
    /// Departure of an actual soft lump from its bind frame.
    SoftLump {
        /// Soft body owning this destination.
        body: SoftBodyId,
        /// Actual deformation lump within the body.
        lump: SoftLumpId,
        /// Animated frame component.
        component: FrameComponent,
    },
    /// Unrecognized producer control; its selector has no established object interpretation.
    Unknown {
        /// Unrecognized producer operation.
        operation: Operation,
        /// Unclassified producer selector.
        selection: u32,
    },
}
impl PhysicsControl {
    fn value_type_matches<T: 'static>(self) -> bool {
        let component = match self {
            Self::ClothDriver { component, .. } | Self::SoftLump { component, .. } => {
                Some(component)
            }
            Self::Unknown { .. } => return true,
            _ => None,
        };
        let expected = match component {
            Some(FrameComponent::Rotation) => TypeId::of::<Quaternion>(),
            Some(FrameComponent::Translation | FrameComponent::Scale) => TypeId::of::<[f32; 3]>(),
            None => TypeId::of::<f32>(),
        };
        TypeId::of::<T>() == expected
    }

    fn validate(self, physics: &Physics<'_>) -> bool {
        match self {
            Self::BodyDynamic(id) | Self::BodyBlend(id) => physics.body(id).is_some(),
            Self::RigRagdoll(id) | Self::RigBlend(id) => physics.rig(id).is_some(),
            Self::ClothActive(id) => physics.cloth(id).is_some(),
            Self::ClothDriver {
                cloth, component, ..
            } => physics.cloth(cloth).is_some() && component != FrameComponent::Scale,
            Self::SoftEffect { body, effect } => physics
                .soft_body(body)
                .is_some_and(|value| value.effect(effect).is_some()),
            Self::SoftLump { body, lump, .. } => physics
                .soft_body(body)
                .is_some_and(|value| value.lump(lump).is_some()),
            Self::Unknown {
                operation: Operation::Unknown(_),
                ..
            } => true,
            Self::Unknown { .. } => false,
        }
    }
}
/// Authored property destination with all known ownership references lifted to handles.
#[derive(Clone, Debug)]
pub enum Target<T> {
    /// A directly understood field on its actual graph owner.
    Property(Property<T>),
    /// Node-local control, including producer-defined emitter selections.
    Node {
        /// Actual hierarchy owner.
        node: NodeId,
        /// Node-local control to evaluate.
        operation: NodeOperation,
    },
    /// A control multiplying the entire shader selected through an appearance mount.
    Material {
        /// Appearance address resolving the material dynamically.
        mount: MaterialMount,
        /// Authored control applied to all shader contributions.
        operation: Operation,
    },
    /// Shader contribution selected through an appearance mount.
    MaterialLayer {
        /// Appearance address resolving the material dynamically.
        mount: MaterialMount,
        /// Actual shader contribution within the selected material.
        layer: ShaderLayerId,
        /// Authored control applied to the selected destination.
        operation: Operation,
    },
    /// Authored shader feature selected through an appearance mount.
    MaterialFeature {
        /// Appearance address resolving the material dynamically.
        mount: MaterialMount,
        /// Authored shader-feature identity, independent of its array position.
        feature: u32,
        /// Authored control applied to the selected destination.
        operation: Operation,
    },
    /// Polygon-group control.
    Section {
        /// Actual mesh owning the polygon group.
        mesh: MeshId,
        /// Polygon group within this mesh.
        section: SectionId,
        /// Authored control applied to the selected destination.
        operation: Operation,
    },
    /// Weight of an actual pose operation.
    Stage {
        /// Operation whose contribution is animated.
        stage: StageId,
    },
    /// Weight or activation of an actual operation input.
    StageSource {
        /// Operation owning the weighted input.
        stage: StageId,
        /// Actual weighted input within the operation.
        source: StageSourceId,
        /// Animated input property.
        control: StageSourceControl,
    },
    /// Control of an actual object within a simulation system.
    Physics {
        /// Simulation system owning this destination.
        system: SystemId,
        /// Typed object and control selection.
        control: PhysicsControl,
    },
}
impl<T: 'static> ValidateLinks for Target<T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Property(property) => property.visit_links(visitor),
            Self::Node { node, .. } => node.visit_links(visitor),
            Self::Material { mount, .. }
            | Self::MaterialLayer { mount, .. }
            | Self::MaterialFeature { mount, .. } => visitor(Link::AppearanceSlot(mount.slot)),
            Self::Section { mesh, .. } => mesh.visit_links(visitor),
            Self::Physics { system, .. } => system.visit_links(visitor),
            Self::Stage { .. } | Self::StageSource { .. } => {}
        }
    }
}
impl<T: 'static> Target<T> {
    fn validate_simulation<'p, 'data: 'p>(
        &self,
        pipeline: &PosePipeline<'_>,
        physics: &impl Fn(SystemId) -> Option<&'p Physics<'data>>,
    ) -> Result<(), Error> {
        let valid = match self {
            Self::Stage { stage } => {
                TypeId::of::<T>() == TypeId::of::<f32>()
                    && pipeline
                        .stage(*stage)
                        .is_some_and(|value| !value.driven.is_empty())
            }
            Self::StageSource { stage, source, .. } => {
                pipeline.stage(*stage).is_some_and(|value| {
                    !value.driven.is_empty() && value.source(*source).is_some()
                }) && TypeId::of::<T>() == TypeId::of::<f32>()
            }
            Self::Physics { system, control } => {
                physics(*system).is_some_and(|value| control.validate(value))
                    && control.value_type_matches::<T>()
            }
            _ => true,
        };
        if valid {
            Ok(())
        } else {
            Err(Error::new(
                "wem.animation.target",
                "destination belongs to another pose operation or simulation object",
            ))
        }
    }
}
/// Actual channel declaration; initialization does not replace the target's payload value.
#[derive(Clone, Debug)]
pub struct Declaration<T> {
    /// Actual property destination.
    pub target: Target<T>,
    /// Separately authored animation initialization, or no initialization declaration.
    pub initial: Option<T>,
}
impl<T: CurveValue + 'static> ValidateLinks for Declaration<T> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.target.visit_links(visitor);
        if let Some(value) = &self.initial {
            value.visit_value_links(visitor);
            if !value.is_valid() {
                visitor(Link::InvalidValue);
            }
        }
    }
}
/// Exact typed curve selecting a declaration rather than copying its initialization.
#[derive(Clone, Debug)]
pub struct Track<'a, T> {
    /// Declaration whose initialization and destination this track uses.
    pub channel: ChannelId<T>,
    /// Ordered keys and interpolation controls.
    pub curve: Curve<'a, T>,
    /// Authored auxiliary tension/continuity/bias controls; no alternate sampling convention is inferred.
    pub auxiliary_tcb: Values<'a, TcbParameters>,
}

macro_rules! domains {
    ($(($variant:ident,$type:ty)),* $(,)?) => {
        /// Exact value domains of model-owned declarations.
        #[derive(Clone,Debug)]
        pub enum AnyDeclaration {$(
            #[doc=concat!(stringify!($variant)," initialization and destination.")]
            $variant(ChannelId<$type>,Declaration<$type>),
        )*}
        /// Typed declaration identity without changing its value domain.
        #[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
        pub enum AnyChannelId {$(
            #[doc=concat!(stringify!($variant)," declaration.")]
            $variant(ChannelId<$type>),
        )*}
        impl AnyDeclaration {
            /// Actual typed identity of this declaration.
            pub fn id(&self)->AnyChannelId {match self {$(Self::$variant(id,_)=>AnyChannelId::$variant(*id),)*}}
            fn validate_simulation<'p, 'data: 'p>(&self, pipeline: &PosePipeline<'_>, physics: &impl Fn(SystemId) -> Option<&'p Physics<'data>>) -> Result<(), Error> {match self {$(Self::$variant(_,value)=>value.target.validate_simulation(pipeline,physics),)*}}
        }
        impl ValidateLinks for AnyDeclaration {
            fn visit_links(&self,visitor:&mut dyn FnMut(Link)){match self {$(Self::$variant(_,value)=>value.visit_links(visitor),)*}}
        }
        /// A typed controller program whose entries share no duplicated payload baseline.
        #[derive(Clone,Debug)]
        pub enum AnyTrack<'a> {$(
            #[doc=concat!(stringify!($variant)," keys.")]
            $variant(Track<'a,$type>),
        )*}
        impl AnyTrack<'_> {
            /// Actual typed declaration selected by these keys.
            pub fn channel(&self)->AnyChannelId {match self {$(Self::$variant(track)=>AnyChannelId::$variant(track.channel),)*}}
            /// Checks the declaration's actual owner and complete typed key controls.
            pub fn validate(&self,channels:&Channels)->Result<(),Error>{
                if channels.declaration(self.channel()).is_none(){return Err(Error::new("wem.animation.channel","declaration belongs to another model"));}
                let mut valid=true;
                match self {$(Self::$variant(track)=>{
                    visit_curve(&track.curve,&mut |link| {if matches!(link,Link::InvalidValue){valid=false;}});
                    if !track.auxiliary_tcb.is_empty() && (track.auxiliary_tcb.len()!=track.curve.times().len()||track.auxiliary_tcb.iter().any(|v|!v.tension.is_finite()||!v.continuity.is_finite()||!v.bias.is_finite())){valid=false;}
                },)*}
                if valid {Ok(())}else{Err(Error::new("wem.animation.keys","invalid curve or auxiliary TCB controls"))}
            }
        }
        impl ValidateLinks for AnyTrack<'_> {
            fn visit_links(&self,visitor:&mut dyn FnMut(Link)){match self {$(Self::$variant(track)=>visit_curve(&track.curve,visitor),)*}}
        }
        $(impl ChannelValue for $type {
            fn wrap(id:ChannelId<Self>,value:Declaration<Self>)->AnyDeclaration {AnyDeclaration::$variant(id,value)}
        })*
    };
}
/// Value domain supported by actual editable-model animation declarations.
pub trait ChannelValue: CurveValue + Sized + 'static {
    /// Stores the declaration while retaining its exact value domain.
    fn wrap(id: ChannelId<Self>, value: Declaration<Self>) -> AnyDeclaration;
}
domains!(
    (Scalar, f32),
    (Vector2, [f32; 2]),
    (Vector3, [f32; 3]),
    (Vector4, [f32; 4]),
    (Rotation, Quaternion),
    (Boolean, bool),
    (Unsigned16, u16),
    (Unsigned, u32),
    (Signed, i32),
    (Bytes4, [u8; 4]),
    (Unsigned4, [u32; 4]),
    (Texture, Option<TextureId>)
);
/// Actual declaration arena, including aliases and declarations unused by every clip.
#[derive(Clone, Debug, Default)]
pub struct Channels {
    scope: Scope,
    declarations: Vec<AnyDeclaration>,
}
impl Channels {
    /// Adds a declaration without altering the targeted payload baseline.
    pub fn add<T: ChannelValue>(&mut self, value: Declaration<T>) -> ChannelId<T> {
        let id = self.scope.allocate();
        self.declarations.push(T::wrap(id, value));
        id
    }
    /// Checks operation, weighted-input, and simulation destinations against their actual owners.
    pub fn validate_simulations<'p, 'data: 'p>(
        &self,
        pipeline: &PosePipeline<'_>,
        physics: impl Fn(SystemId) -> Option<&'p Physics<'data>>,
    ) -> Result<(), Error> {
        for value in &self.declarations {
            value.validate_simulation(pipeline, &physics)?;
        }
        Ok(())
    }
    /// Iterates complete declarations in authored order.
    pub fn declarations(&self) -> impl ExactSizeIterator<Item = &AnyDeclaration> {
        self.declarations.iter()
    }
    /// Borrows a declaration only when this arena owns its exact typed identity.
    pub fn declaration(&self, id: AnyChannelId) -> Option<&AnyDeclaration> {
        self.declarations.iter().find(|value| value.id() == id)
    }
}
impl ValidateLinks for Channels {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for value in &self.declarations {
            value.visit_links(visitor);
        }
    }
}

fn visit_curve<T: CurveValue>(curve: &Curve<'_, T>, visitor: &mut dyn FnMut(Link)) {
    let mut previous = None;
    for time in curve.times().iter() {
        if previous.is_some_and(|old| old > time) {
            visitor(Link::InvalidValue);
        }
        previous = Some(time);
    }
    for value in curve.values().iter() {
        value.visit_value_links(visitor);
        if !value.is_valid() {
            visitor(Link::InvalidValue);
        }
    }
    for value in curve
        .incoming()
        .into_iter()
        .chain(curve.outgoing())
        .flat_map(|values| values.iter())
    {
        value.visit_value_links(visitor);
        if !value.is_valid_tangent() {
            visitor(Link::InvalidValue);
        }
    }
}

/// Identity marker of a model-owned named channel selection.
pub enum SelectionObject {}
/// Stable identity of a named selection of animation declarations.
pub type SelectionId = Id<SelectionObject>;
/// Independently authored selection, including declarations unused by every program.
#[derive(Clone, Debug)]
pub struct Selection<'a> {
    /// Authoring name.
    pub name: Cow<'a, str>,
    /// Ordered selected declarations; repeated identities retain the authored order.
    pub channels: Vec<AnyChannelId>,
}
/// Model-owned channel selections with checked declaration membership.
#[derive(Clone, Debug, Default)]
pub struct Selections<'a> {
    scope: Scope,
    values: Vec<(SelectionId, Selection<'a>)>,
}
impl<'a> Selections<'a> {
    /// Adds a selection whose declarations belong to the supplied model arena.
    pub fn add(&mut self, value: Selection<'a>, channels: &Channels) -> Result<SelectionId, Error> {
        if value
            .channels
            .iter()
            .any(|id| channels.declaration(*id).is_none())
        {
            return Err(Error::new(
                "wem.animation.selection",
                "declaration belongs to another model",
            ));
        }
        let id = self.scope.allocate();
        self.values.push((id, value));
        Ok(id)
    }
    /// Borrows a selection owned by this model.
    pub fn get(&self, id: SelectionId) -> Option<&Selection<'a>> {
        self.values
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates selections in authored order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (SelectionId, &Selection<'a>)> {
        self.values.iter().map(|(id, value)| (*id, value))
    }
    /// Checks membership after edits to the declaration arena.
    pub fn validate(&self, channels: &Channels) -> Result<(), Error> {
        if self.values.iter().any(|(_, value)| {
            value
                .channels
                .iter()
                .any(|id| channels.declaration(*id).is_none())
        }) {
            return Err(Error::new(
                "wem.animation.selection",
                "declaration belongs to another model",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::semantic::{Interpolation, Time};

    fn declaration(scope: &Scope) -> Declaration<u32> {
        Declaration {
            target: Target::Node {
                node: scope.allocate(),
                operation: NodeOperation::UnknownEmitter(17),
            },
            initial: Some(u32::MAX),
        }
    }

    #[test]
    fn exact_initialization_and_selection_membership_are_independent() {
        let scope = Scope::default();
        let mut channels = Channels::default();
        let id = channels.add(declaration(&scope));
        let mut foreign = Channels::default();
        let other = foreign.add(declaration(&scope));
        let mut selections = Selections::default();
        let selection = selections
            .add(
                Selection {
                    name: "unused controls".into(),
                    channels: vec![AnyChannelId::Unsigned(id), AnyChannelId::Unsigned(id)],
                },
                &channels,
            )
            .unwrap();
        assert_eq!(selections.get(selection).unwrap().channels.len(), 2);
        assert!(selections
            .add(
                Selection {
                    name: "foreign".into(),
                    channels: vec![AnyChannelId::Unsigned(other)],
                },
                &channels
            )
            .is_err());
        let Some(AnyDeclaration::Unsigned(_, value)) =
            channels.declaration(AnyChannelId::Unsigned(id))
        else {
            panic!("typed declaration missing")
        };
        assert_eq!(value.initial, Some(u32::MAX));
    }

    #[test]
    fn operation_targets_require_matching_input_owners_and_scalar_values() {
        use crate::model::semantic::profiles::wem::{PoseStage, StageKind, StageSource};
        let scope = Scope::default();
        let mut pipeline = PosePipeline::default();
        let mut stage = PoseStage::new(StageKind::Link);
        stage.driven.push(scope.allocate());
        let source = stage.add_source(StageSource::new(None, 1.0)).unwrap();
        let stage = pipeline.add_stage(stage);
        let mut channels = Channels::default();
        channels.add(Declaration {
            target: Target::StageSource {
                stage,
                source,
                control: StageSourceControl::Enabled,
            },
            initial: Some(0.5f32),
        });
        assert!(channels.validate_simulations(&pipeline, |_| None).is_ok());
        assert!(channels
            .validate_simulations(&PosePipeline::default(), |_| None)
            .is_err());
        let mut wrong_type = Channels::default();
        wrong_type.add(Declaration {
            target: Target::Stage { stage },
            initial: Some(true),
        });
        assert!(wrong_type
            .validate_simulations(&pipeline, |_| None)
            .is_err());
    }

    #[test]
    fn controller_tracks_require_the_actual_typed_declaration_owner() {
        let scope = Scope::default();
        let mut channels = Channels::default();
        let id = channels.add(declaration(&scope));
        let track = AnyTrack::Unsigned(Track {
            channel: id,
            curve: Curve::new(
                vec![Time::seconds(-0.25).unwrap()],
                vec![u32::MAX],
                Interpolation::Step,
            )
            .unwrap(),
            auxiliary_tcb: Values::owned(Vec::new()),
        });
        assert!(track.validate(&channels).is_ok());
        assert!(track.validate(&Channels::default()).is_err());
    }
}
