//! Logical M3 lights, cameras, forces, and geometry-bound effects.
use crate::model::semantic::materials::programmed::Property;
use crate::model::semantic::values::programmed::*;
use crate::model::semantic::{MaterialId, MeshId, NodeId};
use std::borrow::Cow;

/// Editable M3 Projector values.
#[derive(Clone, Debug, PartialEq)]
pub struct Projector {
    /// Selects orthographic or perspective projection.
    pub projection_type: ProjectionKind,
    /// Material whose layers are projected onto receiving surfaces.
    pub material: MaterialId,
    /// Animates the position offset.
    pub offset: Property<[f32; 3]>,
    /// Animates the pitch angle.
    pub pitch: Property<f32>,
    /// Animates the yaw angle.
    pub yaw: Property<f32>,
    /// Animates the roll angle.
    pub roll: Property<f32>,
    /// Animates the field of view.
    pub field_of_view: Property<f32>,
    /// Animates the aspect ratio.
    pub aspect_ratio: Property<f32>,
    /// Animates the near clip plane.
    pub near: Property<f32>,
    /// Animates the far clip plane.
    pub far: Property<f32>,
    /// Animates the box Z bottom offset.
    pub box_offset_z_bottom: Property<f32>,
    /// Animates the box Z top offset.
    pub box_offset_z_top: Property<f32>,
    /// Animates the box X left offset.
    pub box_offset_x_left: Property<f32>,
    /// Animates the box X right offset.
    pub box_offset_x_right: Property<f32>,
    /// Animates the box Y front offset.
    pub box_offset_y_front: Property<f32>,
    /// Animates the box Y back offset.
    pub box_offset_y_back: Property<f32>,
    /// Sets the projection falloff distance.
    pub falloff: f32,
    /// Sets opacity at the envelope’s creation.
    pub alpha_init: f32,
    /// Sets opacity at the envelope’s midpoint.
    pub alpha_mid: f32,
    /// Sets opacity at the envelope’s end.
    pub alpha_end: f32,
    /// Sets the phase duration for the attack portion of the projector’s envelope.
    pub lifetime_attack: f32,
    /// Sets the target time for the attack portion of the projector’s envelope.
    pub lifetime_attack_to: f32,
    /// Sets the phase duration for the hold portion of the projector’s envelope.
    pub lifetime_hold: f32,
    /// Sets the target time for the hold portion of the projector’s envelope.
    pub lifetime_hold_to: f32,
    /// Sets the phase duration for the decay portion of the projector’s envelope.
    pub lifetime_decay: f32,
    /// Sets the target time for the decay portion of the projector’s envelope.
    pub lifetime_decay_to: f32,
    /// Sets the distance-based attenuation.
    pub attenuation_distance: f32,
    /// Animates the active state.
    pub active: Property<u32>,
    /// Sets the render layer.
    pub layer: u32,
    /// Selects the detail level at which projector detail is reduced.
    pub lod_reduce: u32,
    /// Selects the detail level beyond which this effect is omitted.
    pub lod_cut: u32,
    /// Stores the bit mask controlling projector behavior.
    pub flags: ProjectorFlags,
}

/// Selection volume role in the authored model.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HitTestRole {
    /// A member of the fuzzy selection volume collection.
    #[default]
    Fuzzy,
    /// The model's single precise selection volume.
    Tight,
}

/// A bone-attached selection volume with optional checked mesh geometry.
#[derive(Clone, Debug, PartialEq)]
pub struct HitTestShape {
    /// Selects the primitive or mesh geometry used for selection testing.
    pub shape_type: VolumeShape,
    /// Distinguishes the precise volume from fuzzy selection volumes.
    pub role: HitTestRole,
    /// Transforms the shape using a four-by-four matrix.
    pub transform: [[f32; 4]; 4],
    /// Checked collision mesh defining a mesh-shaped volume.
    pub mesh: Option<MeshId>,
    /// Sets the shape’s X extent, interpreted as radius for spheres and capsules.
    pub size_x: f32,
    /// Sets the shape’s Y extent, interpreted as height for capsules and cylinders.
    pub size_y: f32,
    /// Sets the z dimension.
    pub size_z: f32,
}

/// Editable M3 AttachmentVolume values.
#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentVolume {
    /// First additional joint influencing the attachment volume.
    pub bone1: NodeId,
    /// Second additional joint influencing the attachment volume.
    pub bone2: NodeId,
    /// Sets the shape type.
    pub shape_type: VolumeShape,
    /// Transforms the volume using a four-by-four matrix.
    pub transform: [[f32; 4]; 4],
    /// Checked collision mesh defining a mesh-shaped volume.
    pub mesh: Option<MeshId>,
    /// Sets the x dimension.
    pub size_x: f32,
    /// Sets the y dimension.
    pub size_y: f32,
    /// Sets the z dimension.
    pub size_z: f32,
}

/// Editable M3 TurretBehavior values.
#[derive(Clone, Debug, PartialEq)]
pub struct TurretBehavior {
    /// Transforms the turret using a four-by-four matrix.
    pub transform: [[f32; 4]; 4],
    /// Stores an uninterpreted vector whose role is not established.
    pub unknown1: [f32; 4],
    /// Stores a second uninterpreted vector whose role is not established.
    pub unknown2: [f32; 4],
    /// Marks the turret as the primary turret when nonzero.
    pub use_as_main_turret: TurretPartFlags,
    /// Sets the turret group identifier.
    pub turret_group_id: u8,
    /// Enables enforcement of yaw limits.
    pub yaw_limited: TurretAxisFlags,
    /// Limits yaw rotation to its minimum angle, measured in radians.
    pub yaw_min: f32,
    /// Limits yaw rotation to its maximum angle, measured in radians.
    pub yaw_max: f32,
    /// Sets the yaw rotation weight.
    pub yaw_weight: f32,
    /// Enables enforcement of pitch limits.
    pub pitch_limited: TurretAxisFlags,
    /// Limits pitch rotation to its minimum angle, measured in radians.
    pub pitch_min: f32,
    /// Limits pitch rotation to its maximum angle, measured in radians.
    pub pitch_max: f32,
    /// Sets the pitch rotation weight.
    pub pitch_weight: f32,
    /// The purpose of this field has not been established.
    pub unknown3: f32,
    /// The purpose of this field has not been established.
    pub unknown4: f32,
    /// Sets the offset from main bone.
    pub main_bone_offset: [f32; 3],
}

/// Editable M3 BillboardBehavior values.
#[derive(Clone, Debug, PartialEq)]
pub struct BillboardBehavior {
    /// Nodes declared dependent on this billboard behavior; runtime use of this list is unresolved.
    pub dependents: Vec<NodeId>,
    /// Selects the billboard mode and its allowed rotational axes.
    pub billboard_type: BillboardKind,
    /// Nonzero aims the bone toward the camera position; zero aligns it with the camera’s viewing direction.
    pub camera_look_at: u8,
    /// Supplies a quaternion pre-rotation for axis-constrained billboard modes 0, 1, and 2; this is not an up-direction vector.
    pub up: [f32; 4],
    /// Supplies a quaternion pre-rotation for billboard mode 6 when the bone has a bone parent; modes 3, 4, and 5 ignore it.
    pub forward: [f32; 4],
}

/// Editable M3 OneBoneSolver values.
#[derive(Clone, Debug, PartialEq)]
pub struct OneBoneSolver {
    /// Nodes declared dependent on this solver.
    pub dependents: Vec<NodeId>,
    /// Alternate joint used by the solver when declared.
    pub bone_fallback: Option<NodeId>,
    /// Stores the bit mask controlling solver behavior.
    pub flags: OneBoneFlags,
    /// Sets the maximum rotation angle.
    pub max_angle: f32,
}

/// Editable M3 Force values.
#[derive(Clone, Debug, PartialEq)]
pub struct Force {
    /// Sets the force kind.
    pub force_type: ForceKind,
    /// Sets the influence volume shape.
    pub force_shape: ForceShape,
    /// Stores a possible local-versus-world scope selector; this interpretation has not been confirmed by a traced reader.
    pub unknown: u32,
    /// Encodes force options for attenuation, vertical gradient, unbounded influence, and affected targets.
    pub flags: ForceFlags,
    /// Selects force channels; a matching bit connects this force with a body or emitter.
    pub local_channels: ForceChannels,
    /// Animates the force strength.
    pub strength: Property<f32>,
    /// Animates the influence width.
    pub width: Property<f32>,
    /// Animates the influence height.
    pub height: Property<f32>,
    /// Animates the influence length.
    pub length: Property<f32>,
}

/// Editable M3 Warp values.
#[derive(Clone, Debug, PartialEq)]
pub struct Warp {
    /// Sets the warp type.
    pub warp_type: u32,
    /// The purpose of this field has not been established.
    pub unknown: u32,
    /// Animates the warp radius.
    pub radius: Property<f32>,
    /// Animates the warp height.
    pub height: Property<f32>,
    /// Animates the warp strength.
    pub strength: Property<f32>,
    /// Animates the angular component.
    pub angular: Property<f32>,
    /// Animates the axial component.
    pub axial: Property<f32>,
    /// Animates the radial component.
    pub radial: Property<f32>,
}

/// Editable M3 Light values.
#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    /// Selects omnidirectional, spot, or directional illumination.
    pub light_type: LightKind,
    /// Controls lighting features, including shadowing, specular response, and ambient occlusion, through individual bits.
    pub flags: LightFlags,
    /// Selects the detail level beyond which this effect is omitted.
    pub lod_cut: u32,
    /// Sets the shadow LOD cut-off level.
    pub shadow_lod_cut: u32,
    /// Animates the diffuse color using red, green, and blue components.
    pub diffuse_color: Property<[f32; 3]>,
    /// Animates the intensity multiplier.
    pub intensity_multiplier: Property<f32>,
    /// Animates the specular color using red, green, and blue components.
    pub specular_color: Property<[f32; 3]>,
    /// Animates the specular multiplier.
    pub specular_multiplier: Property<f32>,
    /// Animates the distance decay exponent.
    pub decay: Property<f32>,
    /// Sets the attenuation end distance.
    pub attenuation_end: f32,
    /// Animates the attenuation start distance.
    pub attenuation_start: Property<f32>,
    /// Animates the spot inner cone angle.
    pub hot_spot: Property<f32>,
    /// Animates the spot outer cone falloff.
    pub falloff: Property<f32>,
}

/// Editable M3 Camera values.
#[derive(Clone, Debug, PartialEq)]
pub struct Camera<'a> {
    /// Camera name.
    pub name: Cow<'a, str>,
    /// Animates the camera’s field-of-view angle in radians.
    pub field_of_view: Property<f32>,
    /// Selects vertical rather than horizontal field-of-view measurement; the control uses a zero-or-one value.
    pub use_vertical_f_o_v: u32,
    /// Complete optical controls authored for this camera.
    pub optics: CameraOptics,
    /// Animates the shadow clip distance.
    pub shadow_clip_distance: Property<f32>,
    /// Animates the depth-of-field focal point distance.
    pub focus_distance: Property<f32>,
    /// Animates the depth-of-field far focus range.
    pub far_focus_range: Property<f32>,
    /// Animates the depth-of-field near focus range.
    pub near_focus_range: Property<f32>,
    /// Animates the depth-of-field strength.
    pub dof_amount: Property<f32>,
}

/// Near and far planes measured from the camera.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraClipping {
    /// Distance of the far clipping plane from the camera.
    pub far: Property<f32>,
    /// Distance of the near clipping plane from the camera.
    pub near: Property<f32>,
}
/// Near-field depth-of-field falloff interval.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraFalloff {
    /// Beginning of the near-field blur falloff interval.
    pub start: Property<f32>,
    /// End of the near-field blur falloff interval.
    pub end: Property<f32>,
}
/// Bokeh controls, including the unresolved numeric depth-of-field mode.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraBokeh {
    /// Authored depth-of-field selector whose numeric meanings are unresolved.
    pub mode: u32,
    /// Aperture f-number controlling bokeh.
    pub f_stop: Property<f32>,
    /// Maximum circle-of-confusion diameter; its native measurement convention is unresolved.
    pub maximum_circle: Property<f32>,
}
/// Complete optical shapes supported by native M3 cameras.
#[derive(Clone, Debug, PartialEq)]
pub enum CameraOptics {
    /// Legacy optical controls without separately authored clipping planes.
    Legacy {
        /// Two unresolved optical parameters kept independently editable.
        controls: [f32; 2],
    },
    /// Independently animated near and far clipping planes.
    Clipping(CameraClipping),
    /// Clipping planes with an animated near-field blur interval.
    Falloff {
        /// Near and far clipping planes.
        clipping: CameraClipping,
        /// Near-field blur falloff.
        falloff: CameraFalloff,
    },
    /// Clipping, blur falloff, and aperture-dependent bokeh controls.
    Bokeh {
        /// Near and far clipping planes.
        clipping: CameraClipping,
        /// Near-field blur falloff.
        falloff: CameraFalloff,
        /// Aperture and circle-of-confusion controls.
        bokeh: CameraBokeh,
    },
}
