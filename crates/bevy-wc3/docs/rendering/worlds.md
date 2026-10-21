# Static worlds

Prepared models expose world groups, portals, placement sets, regions, liquids, and
authored fog and ambient definitions. `WorldSelection` can hide groups or
select placement sets. The default is conservative; an explicit `ReachableFrom`
policy follows portal connectivity. `WorldPortalPolicy::Clipped(PortalView)` adds
opt-in portal-window traversal. Applications supply the containing group, eye and
inward-facing frustum planes in semantic model coordinates, and update that view
when their chosen camera moves. The renderer clips convex portal polygons against
the current window and narrows each neighboring view through the visible edges.
Different paths into the same room retain their separate windows; cycles are
bounded without discarding other entrance paths.

Exterior and unclassified groups remain visible. Unknown or exterior starting
groups, authored portal adjustments, malformed polygons and traversal work limits
fall back conservatively. Native portal-side conventions, supplemental plane
adjustments and occluder planes are not interpreted; this is geometric window
culling rather than a complete native visibility implementation. Applications own
camera choice and containment rather than relying on guessed room bounds. Visibility
gates intersect these selections with authored animation, material opacity and LOD.

Liquid grids preserve holes, native spacing, and absolute vertex heights. An
authored material is interpreted through the standard material compiler. A liquid
without a resolved material exposes its mesh and `WorldLiquidSurface`
identity for an application material provider; the renderer does not infer a shader
from an external liquid-type database. Environment sampling exposes regional fog and
ambient weights without changing global Bevy lighting or fog. [WMO surfaces](wmo-materials.md)
interpret baked vertex lighting, draw lighting classes and root/region ambient
contributions independently of those application-owned scene resources. Regional
transition lighting uses the authored transition draw ranges. Unclassified native
flag bits are discarded by semantic import and do not affect portal traversal or
expand those ranges. Native regional lighting fidelity remains unverified.

Native dedicated shadow batches participate only in light-shadow views. Their
`ModelDraw::purpose` is also retained by prepared parts and baked geometry; custom
spawn paths should apply `DrawPurpose::configure_visibility` after inserting
`Mesh3d`. This removes its camera visibility class while retaining ordinary
inherited visibility, render layers and light-shadow culling. It does not change
which normal surfaces cast shadows. Keep ordinary camera visibility selection for
these casters: Bevy's opt-in `NoCpuCulling` bypasses this class-based routing and is
not supported for shadow-only geometry. GPU mesh preprocessing can remain enabled.

See [WMO materials](wmo-materials.md) for surface and vertex-light interpretation,
and [application integration](../usage.md) for model loading and resource lookup.
