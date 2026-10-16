# Characters and equipment

A geared character combines several kinds of content. Clothing can replace regions
of the body's skin texture, equipment can select different body geosets, separate
armor meshes can deform with the body skeleton, and helmets, shoulders and weapons
can follow attachment points. The `wow` module resolves these choices before the
ordinary model compiler prepares meshes and Bevy PBR materials. An assembled
character uses the same lighting, shadows and skinning as other models.

## Resolve an appearance

`CharacterRecipe` identifies a race, model sex, explicit customization choices and
item display appearances by equipment slot. Its equipment identifiers are
`ItemDisplayInfo` identities, rather than item IDs. `CharacterDatabase` separates
the game-data lookup from rendering. An adapter must resolve one coherent client
build's model resources, customization choices, helmet visibility rules, clothing
layers and equipment precedence.

`CharacterTables` provides a concrete adapter for applications that have already
resolved those table rows. Bodies are keyed by race and sex, customization patches
by option and choice, and equipment patches by slot and display. An
`AppearancePatch` carries explicit ordering, controlled geoset alternatives,
texture replacements, additional atlas layers and model parts. Patches apply in
increasing order, with stable table keys resolving ties; changing the order of
selections in a request does not change the appearance. Missing exact rows are
errors. There is no implicit substitution of another race or sex, and this module
does not decode DB2 files or infer an item display from an item ID.

The resolved `CharacterDefinition` contains the root model, exact visible geoset
identifiers, replacement texture slots and equipment parts. Applications may
construct it directly when their content pipeline already supplies those facts.
`CharacterCatalog::inspect` extracts the model's actual geoset draws, replacement
categories, native bone identities and attachment roles. Geoset selection retains
all passes and authored LODs of each selected identifier; unknown selected values
are rejected. Neither geoset category arithmetic nor race-specific defaults are
built into the generic compiler.

## Compose body textures

`TextureReplacement::Image` selects a complete replacement image.
`TextureReplacement::Atlas` supplies final dimensions, a clear color and ordered
`TextureLayer` contributions. Each layer identifies an image, an optional source
crop, a destination rectangle and an opacity. The database adapter supplies atlas
rectangles from the selected character layout; historical fixed rectangles are
not assumed to fit every model.

`compose_atlas` accepts checked, tightly packed `RgbaImage` pixels. It applies
back-to-front source-over composition in the decoded sRGB byte domain, including
both source and destination alpha, with deterministic nearest-neighbor scaling.
Empty or out-of-bounds rectangles, dimension overflow and invalid pixel storage
are errors. `RgbaImage::into_image` creates a Bevy sRGB image. Character compilation
keeps role-dependent color-space and sampler variants separate, just as ordinary
model compilation does. Texture composition happens during character setup; it
is not repeated by the animation update.

## Bind equipment

`wow::ModelPart::new` creates a part with its authored geometry and strict native
joint matching. `EquipmentBinding::Attachment` addresses an authored attachment
identity, such as an M2 role. Its model follows the attachment's pivot-local frame;
the part retains its own small rig and animation clock. Authored attachment
activation gates both live rendering and baked visibility. Repeated occurrences of
the same rigid model share source geometry and retain independent state.

`EquipmentBinding::SharedSkeleton` binds weighted armor joints to the character's
joint entities. Automatic matching uses native nonzero name checksums first and
key-bone categories when no checksum matches. Ambiguous or missing identities
require a unique matching checksum and model-space pivot within 0.00001 authored
units. Joint order and generated bone names are not correspondence rules. An
explicit source-to-character map in `ModelPart::joints` can resolve cases known by
the application. Only joints actively weighted by selected compiled geometry
require automatic matching. Other unambiguous native matches also reuse the body
skeleton, avoiding duplicate unused bones in equipment files that carry a full
character skeleton.

Mapped joints reuse the character's evaluated skeleton. Extra equipment joints
can retain their own local hierarchy beneath a mapped ancestor when
`retain_unmatched_joints` is enabled explicitly. A weighted joint without a valid
mapped ancestor is rejected. Optional host-to-part clip mappings sample equipment
animation in host seconds; they do not silently stretch a clip. Native authored
bind spaces remain part of the resulting skin palette. Native zero-duration
global sequences remain typed frozen clocks, preserving their time-zero pose
without inventing a positive animation period.

`compile_character` takes the resolved definition, a `CharacterModels` resource map,
Bevy image assets and a decoded-image callback. It selects geometry, resolves
replacement images, composes atlases, compiles the body and equipment, validates
joint bindings and returns a reusable `CompiledModel`. Skinned subassemblies retain
their own rigid children and source effects. Explicit source clip-controller
selection requires a matching authored host timeline origin. Missing required models,
images or attachment identities fail setup. Prepare and spawn the result through
the ordinary model APIs; applications can reuse the compiled appearance across
many character instances. Shared-skeleton parts that select the same collection
resource coalesce into one rig when their texture, clip and joint policies agree;
their geoset selections are united. Conflicting policies for that same resource
are rejected before compilation. Distinct independently imported resources can
carry separate appearances; rigid instances remain independent.

Character animation and grip policy remain application choices. Select an authored
weapon stance through `Playback`, and use `PoseOverrides` for a finger-pose layer,
off-hand IK or gameplay pose controls. Equipping a weapon installs its authored
socket placement; assembly does not select combat clips or close the fingers.

## Secondary motion

Armor deformation from skeletal animation does not require a physics solver.
`PhysicsSimulation` is an optional component for authored rigid-body secondary
motion, and `PoseOverrides` remains available to application physics or IK
systems. Simulation uses authored model coordinates before the model's conversion
into Bevy coordinates. Choose gravity and other accelerations in that frame.
Inherited forces are converted using the current carrier pose.
`PhysicsFrame::World` is the default and preserves inertial motion when the character or a mount moves or rotates;
`PhysicsFrame::Model` retains simulation in the moving model frame. Repeated
zero-delta render warmups do not advance simulation. Changes in frame scale, shear
or reflection reset simulation.

The built-in solver supports worn-item, ragdoll and private-world rigid-body
systems under explicit application opt-in. Ragdoll systems retain their authored
domain; local bodies and constraints do not supply external-world contacts or
gameplay ragdoll arbitration. Vegetation and unknown domains remain external.
Simulation uses bounded fixed steps and positional constraint projection, including shoulder swing/twist and hinge limits, distance and weld
constraints, and explicitly permitted body contacts using body-centered bounding
spheres. Capsule, box and hull contacts use conservative bounds. Shape centers of
mass let gravity rotate anchored pendants. The renderer preserves authored body
information for external adapters and
reports unsupported native controls, including motors, friction and slider limits,
through rendering diagnostics. This is an approximation of native rigid-body behavior, not a cloth simulation or a claim of
WoW physics parity. Removing simulation restores authored animation; pausing it
retains the simulated pose, and resetting starts from the next authored pose.
Lights and mounted models follow the final simulated skeleton. Retimed effect
births and event poses sample authored animation and application overrides; they
do not reconstruct a history of secondary-motion states.

## Fidelity boundaries

The assembly layer supports explicit client-data rules and renders their resolved
result. It does not supply a complete WoW game database, automatic item inventory,
or a universally correct set of default customization choices. Correct armor
textures and geometry require a matching client build's models, layouts and table
rows. Native material and physics limitations remain observable in compiled
rendering diagnostics; see the [renderer overview](renderer.md) and
[semantic compiler guide](universal.md) for those interpretations.
