//! Property identities from active native controls and node metadata.
use super::super::effects::metadata as native_metadata;
use super::super::materials;
use super::*;
#[derive(Clone)]
pub(super) struct Property {
    pub target: wr::TrackTarget,
    pub mode: u16,
}
pub(super) type Properties = BTreeMap<u32, Property>;
trait Headers {
    fn headers(&self, owner: u32, values: &mut Properties);
}
impl<T> Headers for wr::M3AnimRef<T> {
    fn headers(&self, owner: u32, values: &mut Properties) {
        if self.anim_id != 0 {
            values.insert(
                self.anim_id,
                Property {
                    target: wr::TrackTarget {
                        kind: wr::TrackTargetKind::MaterialFeature,
                        node: owner,
                        material: wr::MaterialChannelRef {
                            profile: wr::ProfileId::Sc2,
                            slot: owner & 0x7fffffff,
                            look: 0,
                        },
                        sub: self.anim_id,
                        channel: wr::Channel::from_raw(255),
                        ..Default::default()
                    },
                    mode: (self.interp_type).raw(),
                },
            );
        }
    }
}
impl<T: Headers> Headers for wem::InlineOption<T> {
    fn headers(&self, owner: u32, values: &mut Properties) {
        if let Some(value) = &self.value {
            value.headers(owner, values);
        }
    }
}
impl<T: Headers> Headers for wem::InlineList<T> {
    fn headers(&self, owner: u32, values: &mut Properties) {
        for value in &self.0 {
            value.headers(owner, values);
        }
    }
}
impl Headers for wr::M3Composite {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.sections.headers(owner, values);
    }
}
impl Headers for wr::M3CompositeSection {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.map_multiplier.headers(owner, values);
    }
}
impl Headers for wr::M3Creep {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.mask_map.headers(owner, values);
    }
}
impl Headers for wr::M3Displacement {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.strength.headers(owner, values);
        self.normal_map.headers(owner, values);
        self.strength_map.headers(owner, values);
    }
}
impl Headers for wr::M3LensFlare {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.flare_map.headers(owner, values);
        self.mask_map.headers(owner, values);
        self.intensity.headers(owner, values);
        self.color.headers(owner, values);
        self.hdr.headers(owner, values);
        self.size.headers(owner, values);
    }
}
impl Headers for wr::M3Reflection {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.reflection_strength.headers(owner, values);
        self.displacement_strength.headers(owner, values);
        self.reflection_offset.headers(owner, values);
        self.blur_angle.headers(owner, values);
        self.blur_distance_max.headers(owner, values);
        self.reflection_map.headers(owner, values);
        self.displacement_map.headers(owner, values);
        self.blur_map.headers(owner, values);
    }
}
impl Headers for wr::M3Standard {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.diffuse_layer.headers(owner, values);
        self.decal_layer.headers(owner, values);
        self.specular_layer.headers(owner, values);
        self.gloss_layer.headers(owner, values);
        self.emissive_layer1.headers(owner, values);
        self.emissive_layer2.headers(owner, values);
        self.environment_layer.headers(owner, values);
        self.environment_mask_layer.headers(owner, values);
        self.alpha_layer1.headers(owner, values);
        self.alpha_layer2.headers(owner, values);
        self.normal_layer.headers(owner, values);
        self.height_layer.headers(owner, values);
        self.light_map_layer.headers(owner, values);
        self.ambient_occlusion_layer.headers(owner, values);
        self.normal_blend1_mask_layer.headers(owner, values);
        self.normal_blend2_mask_layer.headers(owner, values);
        self.normal_blend1_layer.headers(owner, values);
        self.normal_blend2_layer.headers(owner, values);
        self.parallax_height.headers(owner, values);
        self.motion_blur_amount.headers(owner, values);
        self.normal_blend_factors.headers(owner, values);
    }
}
impl Headers for wr::M3Stb {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.diffuse_map.headers(owner, values);
        self.normal_map.headers(owner, values);
        self.specular_map.headers(owner, values);
    }
}
impl Headers for wr::M3Terrain {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.terrain_map.headers(owner, values);
    }
}
impl Headers for wr::M3TextureLayer {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.color.headers(owner, values);
        self.rgb_multiply.headers(owner, values);
        self.rgb_add.headers(owner, values);
        self.avi_play.headers(owner, values);
        self.avi_restart.headers(owner, values);
        self.current_frame.headers(owner, values);
        self.uv_offset.headers(owner, values);
        self.uv_angle.headers(owner, values);
        self.uv_tiling.headers(owner, values);
        self.w_offset.headers(owner, values);
        self.w_tiling.headers(owner, values);
        self.map_alpha.headers(owner, values);
        self.triplanar_offset.headers(owner, values);
        self.triplanar_scale.headers(owner, values);
    }
}
impl Headers for wr::M3Volume {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.density.headers(owner, values);
        self.color_map.headers(owner, values);
        self.noise_map1.headers(owner, values);
        self.noise_map2.headers(owner, values);
    }
}
impl Headers for wr::M3VolumeNoise {
    fn headers(&self, owner: u32, values: &mut Properties) {
        self.density.headers(owner, values);
        self.near_plane.headers(owner, values);
        self.falloff.headers(owner, values);
        self.color_map.headers(owner, values);
        self.noise_map1.headers(owner, values);
        self.noise_map2.headers(owner, values);
        self.scroll_rate.headers(owner, values);
        self.position.headers(owner, values);
        self.scale.headers(owner, values);
        self.rotation.headers(owner, values);
    }
}
impl Headers for wr::M3Body {
    fn headers(&self, owner: u32, values: &mut Properties) {
        match self {
            Self::Standard(body) => body.headers(owner, values),
            Self::Displacement(body) => body.headers(owner, values),
            Self::Composite(body) => body.headers(owner, values),
            Self::Terrain(body) => body.headers(owner, values),
            Self::Volume(body) => body.headers(owner, values),
            Self::Creep(body) => body.headers(owner, values),
            Self::VolumeNoise(body) => body.headers(owner, values),
            Self::Stb(body) => body.headers(owner, values),
            Self::Reflection(body) => body.headers(owner, values),
            Self::LensFlare(body) => body.headers(owner, values),
            Self::DataDriven(_) => {}
        }
    }
}
pub(super) fn properties(document: &wem::Document<'_>) -> Result<Properties, Error> {
    let raw = document.raw();
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut values = Properties::new();
    for channel in raw
        .resolve::<wr::AnimChannel>(model.anim_channels.channels.cast())?
        .iter()
    {
        let channel = channel?;
        values.insert(
            channel.id,
            Property {
                target: channel.target,
                mode: 1,
            },
        );
    }

    for (owner, node) in raw
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .enumerate()
    {
        let node = node?;
        let metadata = native_metadata::read(raw, &node.native)?;
        for (key, &id) in &metadata {
            if let Some(prefix) = key.strip_suffix(".animation_id") {
                if id != 0 {
                    values.insert(
                        id as u32,
                        Property {
                            target: wr::TrackTarget {
                                kind: wr::TrackTargetKind::Node,
                                node: owner as u32,
                                sub: id as u32,
                                channel: match prefix {
                                    "m3.bone.position" => wr::Channel::Translation,
                                    "m3.bone.rotation" => wr::Channel::Rotation,
                                    "m3.bone.scale" => wr::Channel::Scale,
                                    _ => wr::Channel::from_raw(255),
                                },
                                ..Default::default()
                            },
                            mode: metadata
                                .get(&format!("{prefix}.interpolation"))
                                .copied()
                                .unwrap_or(0) as u16,
                        },
                    );
                }
            }
        }
    }
    for profile in raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
    {
        let profile = profile?;
        if profile.profile != wr::ProfileId::Sc2 {
            continue;
        }
        let metadata = native_metadata::read(raw, &profile.native)?;
        for (key, &id) in &metadata {
            if let Some(prefix) = key.strip_suffix(".animation_id") {
                if id != 0 {
                    let base = prefix.rsplit_once('.').map_or(prefix, |(base, _)| base);
                    let owner = metadata
                        .get(&format!("{base}.node_index"))
                        .or_else(|| metadata.get(&format!("{base}.bone_index")))
                        .copied()
                        .unwrap_or(i64::from(u32::MAX)) as u32;
                    values.insert(
                        id as u32,
                        Property {
                            target: wr::TrackTarget {
                                kind: wr::TrackTargetKind::Node,
                                node: owner,
                                sub: id as u32,
                                channel: wr::Channel::from_raw(255),
                                ..Default::default()
                            },
                            mode: metadata
                                .get(&format!("{prefix}.interpolation"))
                                .copied()
                                .unwrap_or(0) as u16,
                        },
                    );
                }
            }
        }
        for index in 0..profile.materials.entries as usize {
            if let Some(body) = materials::body_at(document, 0, profile.profile, index)? {
                body.headers(0x80000000 | index as u32, &mut values);
            }
        }
    }
    Ok(values)
}
