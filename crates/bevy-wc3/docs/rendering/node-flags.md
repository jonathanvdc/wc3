# Node transforms and camera controls

Nodes combine animated local transforms with pivots, inheritance, billboarding,
and camera anchoring. Live rigs, effect births, event poses, and offline baking use
the same `wc3` evaluator. Ordinary hierarchy preserves affine shear; special
inheritance and facing operations also retain signed rotation and scale factors.
Set a driving camera explicitly when a model uses camera-sensitive nodes.

## Driving camera

Set `Playback::driving_camera` explicitly on each camera-sensitive application
model root. There is no automatic main-camera or window-camera selection. Missing
camera input leaves camera-dependent adjustments unavailable while ordinary
animation can still evaluate. One instance has one resolved skeleton for all views;
use separate instances for portrait and world billboards.

Authored attachments, composed mounts and external model particles inherit their
ultimate owning model's current driver before pose evaluation. Nested ownership
resolves in the same frame, including changes and removal of the driver, while
child animation clocks remain independent. Application-owned model children keep
their own camera choice.

Pose evaluation runs in `PostUpdate` under `Wc3Systems::EvaluateNodePoses`, before
transform propagation. Move roots and driving cameras before that set. Authored
camera bindings run earlier in `AnimateCameras`; ordinary application hierarchy
motion is resolved before camera projection updates.

## Inheritance and facing behavior

Translation, rotation, and scale inheritance remain independently selectable.
`Node::transform_semantics` explicitly selects affine composition or pivot-local
bone-frame compensation. MDLX import selects `TransformSemantics::PivotLocal`;
M2 import and newly authored nodes use `Affine`. This choice applies even without
billboarding. Native controls and model metadata do not select evaluation rules.
Ordinary full inheritance preserves the parent affine transform in either mode.
Pivot-local compensation removes accumulated authored motion, rotation, or scale
inside the parent frame when inheritance is disabled.

`Billboard` describes the effective facing operation. `None` leaves rotation alone.
`Spherical` selects an explicit geometry frame and scale treatment, replacing
inherited rotation before applying authored local rotation. `Euler` retains a
nonempty proper subset of the composed rotation's XYZ Euler components and takes
the remaining components from the camera.

`Axial` uses the camera's +Z facing direction expressed in the authored rotation
frame. X rotates local +Y toward that direction's YZ projection and reflects local
Z. Y rotates local +X toward its XZ projection; Z rotates local +X toward its XY
projection. A vanishing projection leaves rotation unchanged. Camera translation
does not affect this operation. Euler component retention and axial rotation are
different operations.

MDLX full facing uses `BillboardFrame::Yz` and
`BillboardScale::SignedFactors` within pivot-local composition. Native full facing
takes precedence over all lock declarations; otherwise import selects X, then Y,
then Z. M2 full facing uses the same YZ frame with
`BillboardScale::BasisLengths` within affine composition. Authored geometry in
the YZ plane faces the camera along +X, with +Y running across the image and +Z
following camera up. The transformed pivot stays fixed, and the composed basis
lengths retain their authored signs, including mirrored and collapsed scales.
The `Camera` frame instead maps authored +X/+Y/+Z to camera right/up/facing.

M2 axis locks remain an approximation expressed as Euler component retention and
produce a rendering diagnostic. The remaining M2 axis-lock mirroring and
camera-frame reconstruction are not implemented. Disabled native locks normalize
to `None`; retaining all three Euler components also normalizes to `None` and
leaves affine shear intact.
Native declarations remain in profile controls as `billboard_flags`; runtime
behavior follows the effective semantic mode.

Camera anchoring adds the driving camera position once along an inherited chain.
Repeated anchored descendants account for the already inherited offset. It does not
itself force a node to face the camera.

Signed factors retain mirrored scales. A collapsed parent axis cannot be inverted
exactly; finite fallbacks keep it collapsed. Exception nodes that need affine
compensation cannot preserve arbitrary shear in a single Bevy Transform. Exact
native billboard appearance beyond the established facing and pivot rules remains
unverified. M2 spherical orientation follows the driving camera frame.

## Occurrence-time evaluation

Emitter and event sampling evaluates authored hierarchy at each occurrence,
including nonlinear parent animation and transition progress. Camera pose is held at
current update input. World-space births retain their sampled frame; local particles
follow the live emitter. Application root motion for effects interpolates observed
update endpoints rather than reconstructing unobserved movement.
