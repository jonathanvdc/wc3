# Model particles

MDLX model-based PREM and M2 geometry-model emitters create ordinary animated
model instances. Their geometry and bind palettes share prepared assets, while
clocks, animated materials, lights, and scene features remain independent. The
Universal emitter keeps its authored family and resource role; applications
resolve both families through `ChildModelKey::particle_geometry`. Attachment
models and pooled emitter-template models have separate dependency roles.

Image-based PREM and external effect-only resources need other interpreters.
The resource variant distinguishes a model from an image; an absent resource
retains the authored mode without declaring a dependency. M2 emitter-template
children (`RPID`) are a separate dependency role and do not yet have a runtime
interpreter. Resolving their model does not make them ordinary geometry actors.

## Emission and motion

Continuous rates integrate in bounded 60 Hz intervals. The shared occurrence
sampler evaluates activation, parameters, and hierarchy at subframe births.
Application root motion interpolates observed update endpoints. MDLX PREM retains
world-space origins, velocities, sampled scale, and random headings. Analytic
constant-gravity motion follows the authored model-up axis after coordinate
conversion. Its established latitude interpretation remains unverified against
the native game.

M2 geometry particles share births, motion, attachment, follow response, and
retirement with [M2 sprites](pre2.md). Their child scale samples the lifetime size
curve: X and Y use the authored components, and Z uses their mean. Twinkle scales
and hides the child; sprite atlas cells and size variation do not alter model
geometry. Detached orientation freezes the normalized emitter frame at birth;
attached orientation follows the live emitter. A birth samples angular velocity
once, in radians per second, and integrates that tumble by particle age.

The child retains its own material colors and opacity. Parent particle color and
alpha curves are not material overrides. Model coordinate conversion applies
once, separately from authored particle size. Collapsed frames omit the affected
projection; signed scale is retained, while shear is approximated by Bevy's
rotation/scale Transform. Native tumble distributions, unit-scale conventions,
and attached model-particle fidelity remain unverified.

## Playback, timing, and ownership

Children start their first clip and sample it by particle age, following their
own looping/clamping policy. Parent pause and playback speed control effect age.
Immediate programme changes and backwards traversal clear particles; blended
changes can retain living particles. Each child retains its own animation root
and authored-coordinate placement policy.

Roots with model-particle emitters simulate in `PostUpdate` at
`Wc3Systems::SimulateModelParticles`, before Bevy transform propagation. Set
application root and camera transforms before that stage. Existing children use
the current source projection, then sample their pose, materials, and features in
owner order. Ready newborn models populate and sample in that same stage, so a
particle shorter than one frame can still appear. Roots containing both model
and sprite emitters observe the shared effect clock once; their other effects
publish after propagation. Sprite-only roots retain the ordinary late path.

Pending asset requests have the same birth identity and lifetime as ready
children. A late load cannot resurrect a dead or reset particle. World-space
children use a linked ownership relationship independent of transform parenting,
so renderer-owner despawn also removes detached requests and populated rigs.
Cyclic asset dependencies are rejected. Automatic attachments and rigid mounts
inside an owned actor use the same current-frame evaluation; unrelated
application models retain ordinary scheduling.

Current hierarchy visibility hides living particles and suppresses new births.
An explicit `Visibility::Visible` override follows Bevy ancestry semantics.
Emitter activation stops births without destroying existing particles.

## Limits

Each emitter is limited to 1,024 live child models and a bounded per-update birth
budget. `ModelEmitter` exposes population, omitted births, and skipped simulation
time. Excess work is discarded rather than queued after a stall. The initial
adapter permits one geometry-model particle generation: child models retain
ordinary mesh, sprite, ribbon, light, and attachment behavior, but further
model-particle births are omitted and diagnosed. Recursive geometry actors and
pooled M2 emitter-template trails require additional bounded ownership support.

This is a CPU hierarchy/model simulation and can cost substantially more than
sprite particles at comparable populations. Synthetic tests cover lifecycle and
scheduling; successful loading or nonempty captures do not establish exact game
appearance.
