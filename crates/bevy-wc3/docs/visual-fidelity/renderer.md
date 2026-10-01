# General renderer visual fidelity

[Documentation index](../README.md)

These gaps cover model rendering beyond [PRE2 particle emitters](pre2.md).

| Area | Current behavior and remaining work |
| --- | --- |
| Reforged materials | Reforged normal and ORM texture slots are not implemented. Models use Bevy PBR shading with WC3 layer blend and depth states. |
| Geoset animation | Geoset alpha is sampled and combined with layer alpha, including visibility changes. Geoset color animation is not applied. |
| UV animation | Layer alpha and texture selection animate, but texture-animation transforms are not applied to UVs. |
| Node billboards | Node transforms animate, but camera-facing node billboard flags are not implemented. |
| Classic PREM particles | Node entities and resolved model-resource handles exist. Child instances support independent animation and lifetime ownership, but Classic particle emitters do not yet spawn or simulate particles. |
| Attachments | Resolved paths spawn independently animated child models. Visibility tracks gate separate mounts; children follow node transforms, loop their first sequence, restart on show/parent sequence changes, and clean up with their owner. Points are exposed by ID/name for consumer-supplied models. |
| Ribbons | Node entities exist, but ribbon emitters do not yet render ribbon geometry. |
| WC3 pass ordering | Geometry uses Bevy render phases and WC3 layer blend/depth states. Warcraft's ordering across material layers and effects is not implemented as a dedicated ordering scheme. |
| Skinning limits | GPU skinning supports up to eight influences per vertex. Larger Classic matrix groups are rejected; they need a rendering strategy before they can be displayed. |

Geoset alpha and eight-influence skinning are already implemented. They should
not be listed as entirely missing features; the remaining gaps are geoset color
animation and support for larger matrix groups.
