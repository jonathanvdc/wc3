# Ribbon trails

Ribbon emitters leave connected trails of birth-time cross sections. MDLX and M2
families use the same GPU geometry transport, while retaining separate authored
semantics. Material passes share section records but keep their own blend, depth,
and texture state. `bevy_wc3::RibbonEmitter` exposes source identity, population,
and omitted-work statistics. Connected ribbons produced by external baked graphs
use living particle strands; see [external programme ribbons](popcorn.md#connected-ribbons).

## Cross sections and motion

`TrailGeometry.axis` explicitly selects the local section direction. MDLX ribbons
use local +Y. M2 imports currently use explicit +Z; that native convention remains
unverified rather than being inferred from the model's up axis. Gravity independently follows model up and
produces ballistic displacement without moving the retained birth origin.

Births sample animated hierarchy, section heights, activation, and appearance at
occurrence times. Existing world-space sections do not follow later emitter motion.
Retired/omitted sections and sampled visibility gaps break connectivity, preventing
joins between unrelated spans. Atlas cells map over connected chains.

## Materials and playback

Every supported compiled material pass receives shared trail geometry. Primary
texture/color/opacity and UV transforms sample the semantic animation programme.
Authored alpha testing, additive/modulate factors, culling, depth controls, unlit
state, priority bias, and supported segment sorting remain explicit. HD normal, ORM,
reflection and secondary compositing do not have full ribbon transport
equivalents and produce limitations.

M2 ribbons resolve their first authored texture through a feature-scoped
`TextureInput::Ribbon { feature, slot: 0 }` request, independently of material
inputs. Native material entries start with neutral white tint and full opacity;
emitter color and alpha supply their animated response. An explicit ribbon image
continues to take precedence over a material image animation. Additional textures
remain unsupported. Ordered semantic material entries
still produce Bevy passes; native game use beyond the first material remains
unverified. A ribbon's animated transform applies once to generated atlas UVs,
around `(0.5, 0.5)`, with translation before scale and rotation. Explicit material
UV transforms apply first. Native transform lookup slot zero is valid; negative
selectors and `0xffff` combo entries mean absence. The native global-flag gate
and game fidelity remain unverified and produce a typed diagnostic when a
transform is used.

Trails follow the application camera’s Bevy distance fog and output processing,
including unlit passes. Unfogged materials bypass fog independently of lighting.
Additive passes fade toward black, modulation toward white, and doubled modulation
toward half-white so a fully fogged trail contributes the blend’s neutral value.

Pause and speed control effect age. Programme changes and backwards traversal break
connectivity; living sections retain their lifetime policy. Root hierarchy
visibility gates rendering, and despawn removes emitter/render state. Simulation
bounds living records and within-update births rather than retaining an unbounded
backlog.

## Limits

The trail transport uses Bevy's transparent phase even for cutout or depth-writing
passes. Dedicated native ordering across meshes and effects is absent. Shaded trails
use Bevy scene lighting. Native M2 cross-section, multi-material, and transform
flag conventions remain unverified. Hidden intervals entirely between sampled
occurrences can remain undetected.
