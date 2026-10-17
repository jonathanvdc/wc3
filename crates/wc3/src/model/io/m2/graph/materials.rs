//! Shader-control tables rebuilt from logical animated surface fields.
use super::animation::{animated, fixed_alpha};
use super::*;
#[derive(Default)]
pub(super) struct Controls {
    pub color: i16,
    pub units: Vec<(u16, u16)>,
}
pub(super) fn export<V: ModelLayout>(
    model: &Graph<'_>,
    core: &mut Core<'static, V>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<BTreeMap<(semantic::MaterialId, usize), Controls>, Error>
where
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, [f32; 3]>: ValuesFactory<[f32; 3]>,
    V::TrackValues<'static, [f32; 4]>: ValuesFactory<[f32; 4]>,
    V::TrackValues<'static, i16>: ValuesFactory<i16>,
{
    let mut controls = BTreeMap::new();
    for (id, material) in model.materials() {
        for (step_index, surface) in material.surface.steps.iter().enumerate() {
            let declaration = surface.declaration();
            if surface.guild_palette().unwrap_or_default() != profile::GuildPalette::default() {
                context.unsupported(
                    "m2.material.guild_palette",
                    "application guild colours have no native M2 encoding",
                )?;
            }
            let color = i16::try_from(core.colors.len())
                .map_err(|_| Error::new("m2.material.color", "too many color controls"))?;
            core.colors.push(native::records::ColorAnimation {
                color: animated(
                    model,
                    &declaration.color,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?,
                alpha: animated(
                    model,
                    &declaration.opacity,
                    starts,
                    globals,
                    context,
                    fixed_alpha,
                )?,
            });
            let mut units = Vec::new();
            for unit in surface.inputs() {
                let weight = u16::try_from(core.texture_weights.len())
                    .map_err(|_| Error::new("m2.material.weight", "too many weight controls"))?;
                core.texture_weights.push(native::records::TextureWeight {
                    weight: animated(
                        model,
                        &unit.contribution,
                        starts,
                        globals,
                        context,
                        fixed_alpha,
                    )?,
                });
                let encoded = native::records::TextureTransform {
                    translation: animated(
                        model,
                        &unit.coordinates.transform.translation,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    rotation: animated(
                        model,
                        &unit.coordinates.transform.rotation,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v.components()),
                    )?,
                    scaling: animated(
                        model,
                        &unit.coordinates.transform.scale,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                };
                let index = core
                    .texture_transforms
                    .iter()
                    .position(|value| value == &encoded)
                    .unwrap_or_else(|| {
                        core.texture_transforms.push(encoded);
                        core.texture_transforms.len() - 1
                    });
                let transform = u16::try_from(index)
                    .map_err(|_| Error::new("m2.material.transform", "too many transforms"))?;
                units.push((weight, transform))
            }
            controls.insert((id, step_index), Controls { color, units });
        }
    }
    Ok(controls)
}
