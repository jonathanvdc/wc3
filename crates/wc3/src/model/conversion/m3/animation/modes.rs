//! Native interpolation headers reconciled with the active editable curves.
use super::*;
pub(in super::super) fn modes(
    model: &sem::Model<'_>,
    context: &mut Context,
) -> Result<BTreeMap<u32, u16>, Error> {
    let mut result = BTreeMap::new();
    let Some(native) = &model.native else {
        return Ok(result);
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let channels: Vec<_> = raw
        .resolve::<wr::AnimChannel>(record.anim_channels.channels.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    for clip in raw.resolve::<wr::Clip<wr::V7>>(root.clips.cast())?.iter() {
        let clip = clip?;
        if clip.model != native.model {
            continue;
        }
        for container in raw
            .resolve::<wr::SubTrackContainer>(clip.containers.cast())?
            .iter()
        {
            for track in raw
                .resolve::<wr::SubTrack<wr::V2>>(container?.sub_tracks.cast())?
                .iter()
            {
                let track = track?;
                let channel = channels
                    .iter()
                    .find(|channel| channel.id == track.channel)
                    .ok_or_else(|| Error::new("animation.channel", "missing channel"))?;
                if channel.target.channel != wr::Channel(255) {
                    continue;
                }
                let mode = if track.interp == wr::Interpolation::STEP {
                    0
                } else {
                    1
                };
                let id = channel.target.sub;
                if result
                    .insert(id, mode)
                    .is_some_and(|previous| previous != mode)
                {
                    context.unsupported(
                        format!("animation[{id}].interpolation"),
                        "native interpolation header must agree across all clips",
                    )?;
                }
            }
        }
    }
    Ok(result)
}
pub(in super::super) fn reconcile_controls(
    values: &mut BTreeMap<String, i64>,
    modes: &BTreeMap<u32, u16>,
) {
    let updates: Vec<_> = values
        .iter()
        .filter_map(|(key, &id)| {
            key.strip_suffix(".animation_id").and_then(|prefix| {
                modes
                    .get(&(id as u32))
                    .map(|&mode| (format!("{prefix}.interpolation"), i64::from(mode)))
            })
        })
        .collect();
    values.extend(updates);
}
trait HeadersMut {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>);
}
impl<T> HeadersMut for wr::M3AnimRef<T> {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        if let Some(&mode) = modes.get(&self.anim_id) {
            self.interp_type = mode;
        }
    }
}
impl<T: HeadersMut> HeadersMut for wem::InlineOption<T> {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        if let Some(value) = self.value.as_mut() {
            value.update_headers(modes);
        }
    }
}
impl<T: HeadersMut> HeadersMut for wem::InlineList<T> {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        for value in &mut self.0 {
            value.update_headers(modes);
        }
    }
}
impl HeadersMut for wr::M3Composite {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.sections.update_headers(modes);
    }
}
impl HeadersMut for wr::M3CompositeSection {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.map_multiplier.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Creep {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.mask_map.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Displacement {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.strength.update_headers(modes);
        self.normal_map.update_headers(modes);
        self.strength_map.update_headers(modes);
    }
}
impl HeadersMut for wr::M3LensFlare {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.flare_map.update_headers(modes);
        self.mask_map.update_headers(modes);
        self.intensity.update_headers(modes);
        self.color.update_headers(modes);
        self.hdr.update_headers(modes);
        self.size.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Reflection {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.reflection_strength.update_headers(modes);
        self.displacement_strength.update_headers(modes);
        self.reflection_offset.update_headers(modes);
        self.blur_angle.update_headers(modes);
        self.blur_distance_max.update_headers(modes);
        self.reflection_map.update_headers(modes);
        self.displacement_map.update_headers(modes);
        self.blur_map.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Standard {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.diffuse_layer.update_headers(modes);
        self.decal_layer.update_headers(modes);
        self.specular_layer.update_headers(modes);
        self.gloss_layer.update_headers(modes);
        self.emissive_layer1.update_headers(modes);
        self.emissive_layer2.update_headers(modes);
        self.environment_layer.update_headers(modes);
        self.environment_mask_layer.update_headers(modes);
        self.alpha_layer1.update_headers(modes);
        self.alpha_layer2.update_headers(modes);
        self.normal_layer.update_headers(modes);
        self.height_layer.update_headers(modes);
        self.light_map_layer.update_headers(modes);
        self.ambient_occlusion_layer.update_headers(modes);
        self.normal_blend1_mask_layer.update_headers(modes);
        self.normal_blend2_mask_layer.update_headers(modes);
        self.normal_blend1_layer.update_headers(modes);
        self.normal_blend2_layer.update_headers(modes);
        self.parallax_height.update_headers(modes);
        self.motion_blur_amount.update_headers(modes);
        self.normal_blend_factors.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Stb {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.diffuse_map.update_headers(modes);
        self.normal_map.update_headers(modes);
        self.specular_map.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Terrain {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.terrain_map.update_headers(modes);
    }
}
impl HeadersMut for wr::M3TextureLayer {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.color.update_headers(modes);
        self.rgb_multiply.update_headers(modes);
        self.rgb_add.update_headers(modes);
        self.avi_play.update_headers(modes);
        self.avi_restart.update_headers(modes);
        self.current_frame.update_headers(modes);
        self.uv_offset.update_headers(modes);
        self.uv_angle.update_headers(modes);
        self.uv_tiling.update_headers(modes);
        self.w_offset.update_headers(modes);
        self.w_tiling.update_headers(modes);
        self.map_alpha.update_headers(modes);
        self.triplanar_offset.update_headers(modes);
        self.triplanar_scale.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Volume {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.density.update_headers(modes);
        self.color_map.update_headers(modes);
        self.noise_map1.update_headers(modes);
        self.noise_map2.update_headers(modes);
    }
}
impl HeadersMut for wr::M3VolumeNoise {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        self.density.update_headers(modes);
        self.near_plane.update_headers(modes);
        self.falloff.update_headers(modes);
        self.color_map.update_headers(modes);
        self.noise_map1.update_headers(modes);
        self.noise_map2.update_headers(modes);
        self.scroll_rate.update_headers(modes);
        self.position.update_headers(modes);
        self.scale.update_headers(modes);
        self.rotation.update_headers(modes);
    }
}
impl HeadersMut for wr::M3Body {
    fn update_headers(&mut self, modes: &BTreeMap<u32, u16>) {
        match self {
            Self::Standard(body) => body.update_headers(modes),
            Self::Displacement(body) => body.update_headers(modes),
            Self::Composite(body) => body.update_headers(modes),
            Self::Terrain(body) => body.update_headers(modes),
            Self::Volume(body) => body.update_headers(modes),
            Self::Creep(body) => body.update_headers(modes),
            Self::VolumeNoise(body) => body.update_headers(modes),
            Self::Stb(body) => body.update_headers(modes),
            Self::Reflection(body) => body.update_headers(modes),
            Self::LensFlare(body) => body.update_headers(modes),
            Self::DataDriven(_) => {}
        }
    }
}
pub(in super::super) fn reconcile_material(body: &mut wr::M3Body, modes: &BTreeMap<u32, u16>) {
    body.update_headers(modes);
}
