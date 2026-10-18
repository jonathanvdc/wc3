# Imported lights

Common authored diffuse lights become ordinary Bevy lights and illuminate other Bevy
assets as well as imported models. `ModelLight` retains the complete supported
light definition, application enablement, and sampled ambient metadata. The model's
node hierarchy controls placement and cleanup.

## Mapping and controls

Portable and MDLX ambient/diffuse lights, WMO world lights, and region point lights map supported color, intensity, range, activation,
and shadow flags to Bevy point, directional, or spot components. Unknown kinds
remain inspectable without an arbitrary substitute.

`ModelLightSettings` on a model root configures power/range conversion and
suppression. Defaults are 1,000 lumens per point intensity unit, 10,000 lux per
directional unit, range scale 1, fallback range 1,000, and shadows permitted. These
are integration defaults, not calibrated physical conversions. Each child model has
its own settings. Configure range conversion explicitly when model units differ from
scene units; Bevy light range does not scale with Transform.

The animation system writes supported light color, power, range, cone, and shadow
permission. Application-specific light radii, biases, render layers, and cascade
settings remain available. Disable `ModelLight.enabled` to suppress an
individual imported light. Removing the interpreter component leaves the ordinary
Bevy light under application control, with hierarchy lifetime unchanged.

## Ambient and native limits

Ambient terms remain metadata; importing a model does not overwrite
`GlobalAmbientLight`. Bevy's distance falloff, PBR response, shadows, exposure, and
tonemapping remain authoritative. Attenuation start is not interpreted as source
radius, and custom falloff/shadow-range controls are not reproduced.

WMO Euler/cone calibration, cookies, flicker, and regional light-set activation
remain uninterpreted. WMO placement appearance retains interior illumination and
linked-light information in the external definition on `ModelAttachment`; it is not multiplied
into child albedo. Applications need region ambient/interior direction policy to
interpret those controls without double-counting Bevy scene lights.
