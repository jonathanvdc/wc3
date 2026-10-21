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
children (`RPID`) use the separate [pooled sprite path](pre2.md#m2-adopted-emitter-templates),
including when their source is a geometry-model particle. Resolving their model
does not make them ordinary geometry actors.

## Emission and motion

Continuous rates integrate in bounded 60 Hz intervals. The shared occurrence
sampler evaluates activation, parameters, and hierarchy at subframe births.
Application root motion interpolates observed update endpoints. MDLX PREM retains
world-space origins, velocities, sampled scale, and random headings. Analytic
constant-gravity motion follows the authored model-up axis after coordinate
conversion. Latitude bounds the signed polar angle and longitude bounds the
signed azimuth, both sampled in raw radians at each occurrence. Zero longitude
confines initial velocity to the emitter's XZ plane; zero latitude launches along
positive Z regardless of longitude. Native game angular distributions remain
unverified. Each scheduled PREM occurrence owns a deterministic yaw, pitch, and
spin packet. Per-birth rejection for pose visibility, invalid values,
population limits or missing resources does not shift later packets. Emitter
activation and hierarchy visibility gate scheduling first. Reset restarts the
source sequence; native game random-sequence fidelity remains unverified.
Missing child assets consume the current authored interval without queuing
historical births.

M2 geometry particles share births, motion, attachment, follow response, and
retirement with [M2 sprites](pre2.md). Their child scale samples the lifetime size
curve: X and Y use the authored components, and Z uses their mean. Twinkle scales
and hides the child; sprite atlas cells and size variation do not alter model
geometry. Detached orientation freezes the normalized emitter frame at birth;
attached orientation follows the live emitter. A birth samples angular velocity
once, in radians per second, and integrates that tumble by particle age.

The child retains its own material colors and opacity. Parent particle color and
alpha curves are not material overrides. Model coordinate conversion applies
once, separately from authored particle size. Collapsed birth frames omit the
birth. A temporarily collapsed attached frame hides its existing actor while
retaining its identity, clock, and last finite
Transform; valid projection makes that same actor visible again. Signed scale
is retained, while shear is approximated by Bevy's rotation/scale Transform.
Native tumble distributions, unit-scale conventions, and attached model-particle
fidelity remain unverified.

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

Ready children seed their sprite, ribbon, model-emitter, and external programme
histories at the captured birth frame before advancing over their initial
particle-age interval. Their occurrence poses use the shared evaluator without a
second physics step. Paused warmup frames do not repeat births. A pending child
can retain this initial interval while its source age stays unchanged; once that
age advances, later readiness starts effects at the current time. Asset reloads
also start at the current time rather than replaying old emissions.

Pending asset requests have the same birth identity and lifetime as ready
children. A late load cannot resurrect a dead or reset particle. World-space
children use a linked ownership relationship independent of transform parenting,
so renderer-owner despawn also removes detached requests and populated rigs.
Cyclic asset dependencies are rejected. Automatic attachments and rigid mounts
inside an owned actor use the same current-frame evaluation; unrelated
application models retain ordinary scheduling.

Fresh rigid descendants resolve their birth placement through their owner's
birth pose. Constant-active or socketless automatic attachments advance their
independent clock over the accepted particle-age interval, applying their own
playback speed once; mapped ancestor speeds do not multiply it again. Explicit
nonzero or paused child clocks retain their policy. Animated attachment activation
starts at readiness and reports an approximation: initial activation crossings
are not reconstructed. Pending automatic attachments retain their captured
interval while the owner's observed clock stays unchanged. Once that clock
advances, later readiness starts at the endpoint without replaying the owner's
startup interval.

Current hierarchy visibility hides living particles and suppresses new births.
An explicit `Visibility::Visible` override follows Bevy ancestry semantics.
Emitter activation stops births without destroying existing particles.

## Limits

Each emitter retains its own live-particle and per-update birth limits.
`ModelEmitter` exposes population, omitted births, and skipped simulation time.
PREM counts authored occurrences discarded by its per-update cap, including those
skipped without evaluating each birth. An omitted occurrence is counted once.
Unexamined simulation time is reported separately as skipped seconds.

Nested geometry-model particles share one ownership tree with their static rigid
parts and automatic model attachments. Add `ModelParticleLimits` to the outer
application model root to configure the tree; defaults allow 4,096 pending or
ready actors, 512 birth/admission operations, and 16,384 examined emission steps
per application frame. Preexisting application-owned assembly parts are outside
these actor limits. Accepted pending requests reserve actor capacity immediately;
despawn, reset, reload, failed loading and rejected ownership release it.
Reducing limits preserves already accepted actors and constrains future work.
At capacity, an update that retires children can still omit replacement births
until those children despawn.

Readiness retries precede fresh births and rotate through a stable queue. Ready
actors always receive their pose, material and effect update; this does not spend
birth/admission work. Owned effect sources share the emission-step allowance,
including programme simulation steps and examined keyed-burst keys or
occurrences. Current owner order determines which sources receive the remaining work during a
long frame. Excess admissions and unexamined time are omitted without a catch-up
queue, rather than creating an unbounded recursive hierarchy. `ModelParticleTreeStats`
on the application root reports pending/ready actors, current-frame work and
cumulative exclusive omission reasons. Its skipped seconds sum across effect
sources, rather than measuring elapsed scene wall time.

Cyclic resource dependencies remain rejected. Adopted sprite-template pools have
separate particle and source-work limits; their children do not reserve geometry
actor slots.

This is a CPU hierarchy/model simulation and can cost substantially more than
sprite particles at comparable populations. Synthetic tests cover lifecycle and
scheduling; successful loading or nonempty captures do not establish exact game
appearance.
