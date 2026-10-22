# Secondary motion

`PhysicsSimulation` enables authored rigid-body secondary motion on a model
instance. Insert it on the model root; rigid mounted parts inherit its settings
unless overridden. All sources use the same `Model<Universal>` physics systems.
Applications integrating another solver can inspect
`PreparedModel::physics_bodies()` and `physics_joints()` and supply local poses
through `PoseOverrides`.

## Frames and playback

Forces use authored model coordinates and units before conversion to Bevy
coordinates. Choose gravity and other accelerations in that frame. Inherited
forces convert using the current carrier pose. `PhysicsFrame::World` preserves
inertial motion when the character or a mount moves or rotates;
`PhysicsFrame::Model` carries simulation with the model frame. Changes in frame
scale, shear or reflection reset simulation.

The solver uses bounded fixed steps and positional constraint projection. Configure
`timestep`, `max_steps` and `iterations` through `PhysicsSimulation`. Removing the
component restores authored animation; pausing retains the simulated pose, and
`reset()` starts from the next authored pose. Repeated zero-delta render warmups
do not advance simulation. Kinematic targets interpolate over the available fixed
steps and supply point velocities to contacts.

Lights and mounted models follow the final simulated skeleton. Retimed effect
births and event poses sample authored animation and application overrides; they
do not reconstruct a history of secondary-motion states.

## Choose contact admission

`PhysicsContactPolicy::AuthoredPairs` is the default. It uses only
`Physics::collision_pairs` in semantic systems. Endpoints identify `Body::node`
placements, and an empty list grants no contacts. Native M2 imports leave that
list empty, so adding `PhysicsSimulation` or capture's `--physics` alone leaves
native body contacts disabled.

Applications can select an interpretation of retained body and joint topology
without modifying the model:

```rust
use bevy_wc3::{PhysicsContactPolicy, PhysicsSimulation};
let settings = PhysicsSimulation {
    contact_policy: PhysicsContactPolicy::UnconnectedDynamics,
    ..Default::default()
};
```

This policy replaces authored grants with pairs from the same source physics
system. Each body must carry a finite fixture, and at least one must be dynamic.
Zero-radius point spheres and degenerate capsules remain eligible. Directly
connected compiled joints exclude contact; connectivity is not transitive, so the
first and last links of a three-body chain can contact. The currently approximated
prismatic joint also excludes its connected bodies.

Assembly remaps prepared lists without generating cross-source pairs; mounted
rigs keep independent simulation. No ground, terrain or cross-actor contacts are
inserted. Live policy is application state and is never exported as native or
semantic collision grants. Native filtering and game fidelity of this
interpretation remain unverified.

## Bound preparation and inspect failures

`CompileOptions::physics_contact_pair_budget` defaults to 16,384 potential body
pairs across the compiled definition. Source systems consume that allowance
before enumeration and direct-joint exclusions. It bounds dynamic/dynamic and
dynamic/non-dynamic candidates; a dense joint graph can exceed the allowance even
if few pairs would survive. Zero disables preparation when any source has
potential pairs.

Composition bounds the retained remapped list using the destination definition's
budget. Repeated remapped grants can exhaust that bound before deduplication.
Actual nonzero fixture flags/masks and body-parent, attachment-host or ragdoll
controls on candidate contact participants make inference unavailable. Fixtureless
bodies cannot be endpoints, so their controls do not block admission. A source with
no possible dynamic pair contributes an empty available list even if its bodies
carry unsupported controls; their motion limitations remain independently diagnosed.
A fixture-bearing kinematic anchor with a dynamic partner remains a candidate and
still requires resolved controls. Rendering diagnostics describe omitted bodies and
unsupported controls; the selected policy does not restore omitted bodies.

Requesting an unavailable policy disables contacts for the whole rig while bodies
and constraints continue integrating. It supplies neither a partial list nor an
authored fallback. `ModelRig::physics_contact_error()` exposes the typed cause
after sampling, with a warning only when that failure changes. Selecting an
available policy or removing simulation clears the error. Changing contact policy
resets velocities and placements to the current authored pose, including while
paused or at zero delta.

Use the existing [capture tool](../tools.md) with
`--physics --physics-contact-policy unconnected-dynamics` to check this policy on
an imported model. Capture terminates when the requested admission is unavailable.
Candidate pair counts establish possible work; they do not establish touching
contacts or native pose fidelity.

## Bound per-update contact work

`PhysicsSimulation::contact_limits` defaults to 16,384 body-pair examinations,
16,384 narrow-phase queries and 262,144 hull-vertex/primitive-support samples per
requested update. These are independent allowances shared across fixed steps,
projection iterations and velocity passes. `pair_checks_per_update` charges each
body-pair examination before weight or broad-phase rejection, including repeated
visits to a separated pair. Compound fixture queries then consume the separate
query and support allowances. A zero pair or query allowance disables contacts
while integration and joints continue. Pair exhaustion stops remaining body-pair
traversal; an already admitted pair can finish its compound queries. Query
exhaustion truncates the remaining contact passes; support exhaustion omits that
hull query while analytical contacts can use remaining query allowance. Omitted
work is never queued for a later frame.

`ModelRig::physics_contact_stats()` reports saturating cumulative actual body-pair
examinations (`pair_checks`), queries, support samples and typed numerical/budget
issues since rig creation or an explicit reset generation. Pair and query
exhaustions are each counted once per truncated update; using exactly an allowance
without further requested work is not an exhaustion. Support exhaustion is counted
per failed hull query. The latest issue remains visible after successful queries.
This runtime observation is independent of
`physics_contact_error()`, which reports contact-policy preparation failure.
Hull scratch is allocated once when its instance initializes and reused across
contacts; source hulls above 4096 vertices are rejected before a private vertex copy.

## Geometry and response limits

The built-in solver supports worn-item, ragdoll and private-world rigid systems
under explicit application opt-in. Local ragdoll bodies and constraints supply
neither external-world contacts nor gameplay arbitration. Vegetation and unknown
domains remain application-owned. Supported constraints include shoulder
swing/twist and hinge limits, distance and weld constraints. Motors, angular joint
friction and slider limits retain diagnostics.

Each compound fixture retains its own geometry and material response.
Sphere/capsule pairs use exact segment distances; sphere/orthogonal-box contacts
use the actual box. Pairs containing a convex hull use bounded support-map distance
and penetration queries over the original prepared vertices. Affine support retains
nonuniform scale, reflection and shear. Invalid or degenerate hulls, numerical
nonconvergence and exhausted work quotas omit the contact rather than substitute
a sphere. These iterative contacts approximate penetration; native game fidelity
remains unverified. Other box-box and capsule-box pairs retain conservative fixture
bounds, as do analytical-only rounded pairs under nonuniform scale or shear.
Compound centers and full tensor inertia retain fixture offsets and orientation;
contacts produce angular impulses and bounded Coulomb friction. Restitution uses
the maximum and friction the geometric mean of the two fixtures' values.

One representative point is used per fixture pair; stable multipoint manifolds
are absent. Hull mass uses declared volume and centroid with a diagnosed spherical
tensor approximation. Body scale changes retain compiled mass and transform its
centered second moments; native scale/mass fidelity remains unverified. This
solver approximates rigid-body secondary motion and does not implement cloth
simulation or establish native physics parity.
