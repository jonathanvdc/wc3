//! Bridge mappings for native M3 materials.
use super::{body, native, nr, put, read_error, wem, wr, Envelope, Error, PhantomData, Slot};
pub(in crate::model::conversion::m3) fn attach<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
) -> Result<wem::Document<'static>, Error> {
    let mut doc = document.into_raw();
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::new(
            doc.root_reference().entries,
            doc.root_reference().index,
            doc.root_reference().flags,
        ))?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing document root"))??;
    let model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.models", "missing model"))??;
    let mut set = doc
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.materials", "missing profile"))??;
    let mut materials = doc
        .resolve::<wr::MaterialRecord>(set.materials.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let maps = complete_maps(source)?;
    materials.resize_with(maps.len(), wr::MaterialRecord::default);
    for (index, map) in maps.into_iter().enumerate() {
        if matches!(
            map.material_type,
            nr::MaterialKind::Hair | nr::MaterialKind::Unknown(_)
        ) {
            continue;
        }
        let kind = map.material_type;
        let (body, source_version) = body(source, &mut doc, map)?;
        let payload = wr::M3Material::<wr::V1, wr::Abi64> {
            source_version,
            kind: wr::M3MaterialKind::from_raw((kind).raw()),
            body: wr::NativeM3Body {
                body,
                abi: PhantomData,
            },
            ..Default::default()
        };
        let name = body_name(&payload.body.body);
        if let wr::M3Body::Standard(body) = &payload.body.body {
            if let Some(material) = materials.get_mut(index) {
                material.common.blend =
                    super::render_state::blend(nr::BlendMode::from_raw(body.blend_mode.raw()));
                material.common.flags =
                    super::render_state::flags(nr::MaterialFlags(body.flags.bits()));
            }
        }
        let reference = put(&mut doc, &[payload])?.cast();
        if let Some(material) = materials.get_mut(index) {
            material.name = name;
            material.native = wr::NativeMaterial::M3(reference);
            material.sync = wr::NativeSync::NativeAuthoritative;
        }
    }
    if !materials.is_empty() {
        let id = doc
            .section_id(set.materials.index)
            .ok_or_else(|| Error::new("wem.materials", "missing material section"))?;
        doc.replace(id, &materials)?;
        set.materials.entries = materials.len() as u32;
        let profile_id = doc
            .section_id(model.profile_sets.index)
            .ok_or_else(|| Error::new("wem.materials", "missing profile table"))?;
        doc.replace(profile_id, &[set])?;
    }
    Ok(wem::Document::from_raw(doc)?)
}

pub(in crate::model::conversion::m3) fn complete_maps<E: Envelope>(
    source: &native::Model<'_, E>,
) -> Result<Vec<nr::MaterialMap>, Error> {
    let native_root = source.root_record().map_err(read_error)?;
    let (map_ref, _standard_ref) =
        native::visit_m3_root!(native_root, |r| (r.material_maps, r.standard_materials));
    let mut maps = source
        .document()
        .resolve::<nr::MaterialMap>(map_ref.cast())
        .map_err(read_error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    let counts = native::visit_m3_root!(native_root, |r| [
        (1, r.standard_materials.entries),
        (2, r.displacement_materials.entries),
        (3, r.composite_materials.entries),
        (4, r.terrain_materials.entries),
        (5, r.volume_materials.entries),
        (7, r.creep_materials.entries),
        (
            6,
            Slot::<native::Ref<nr::targets::VolumeNoiseMaterial>>::value(&r.volume_noise_materials)
                .entries
        ),
        (
            9,
            Slot::<native::Ref<nr::targets::STBMaterial>>::value(&r.stb_materials).entries
        ),
        (
            10,
            Slot::<native::Ref<nr::targets::ReflectionMaterial>>::value(&r.reflection_materials)
                .entries
        ),
        (
            11,
            Slot::<native::Ref<nr::targets::LensFlare>>::value(&r.lens_flare_materials).entries
        ),
        (
            12,
            Slot::<native::Ref<nr::targets::DataDrivenMaterial>>::value(&r.data_driven_materials)
                .entries
        ),
    ]);
    for (kind, count) in counts {
        for material_index in 0..count {
            if !maps.iter().any(|map| {
                map.material_type == nr::MaterialKind::from_raw(kind)
                    && map.material_index == material_index
            }) {
                maps.push(nr::MaterialMap {
                    material_type: nr::MaterialKind::from_raw(kind),
                    material_index,
                });
            }
        }
    }
    Ok(maps)
}

pub(in crate::model::conversion::m3) fn body_at(
    document: &wem::Document<'_>,
    model_index: u32,
    profile: wr::ProfileId,
    index: usize,
) -> Result<Option<wr::M3Body>, Error> {
    Ok(body_with_revision(document, model_index, profile, index)?.map(|value| value.0))
}

pub(in crate::model::conversion::m3) fn body_with_revision(
    document: &wem::Document<'_>,
    model_index: u32,
    profile: wr::ProfileId,
    index: usize,
) -> Result<Option<(wr::M3Body, u32)>, Error> {
    let doc = document.raw();
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let profile_sets = model_profiles(doc, root.models, model_index as usize)?;
    let sets = doc
        .resolve::<wr::ProfileMaterialSet>(profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let Some(set) = sets.into_iter().find(|set| set.profile == profile) else {
        return Ok(None);
    };
    let material = doc
        .resolve::<wr::MaterialRecord>(set.materials.cast())?
        .get(index)
        .ok_or_else(|| Error::new("wem.materials", "missing native material"))??;
    let wr::NativeMaterial::M3(reference) = material.native else {
        return Ok(None);
    };
    let payload = match doc.options().native_abi {
        Some(wem::NativeAbi::Abi64) => {
            let payload = doc
                .resolve::<wr::M3Material<wr::V1, wr::Abi64>>(reference.cast())?
                .get(0)
                .ok_or_else(|| Error::new("wem.materials.native", "empty M3 body"))??;
            (payload.body.body, payload.source_version)
        }
        Some(wem::NativeAbi::Abi32) => {
            let payload = doc
                .resolve::<wr::M3Material<wr::V1, wr::Abi32>>(reference.cast())?
                .get(0)
                .ok_or_else(|| Error::new("wem.materials.native", "empty M3 body"))??;
            (payload.body.body, payload.source_version)
        }
        None => {
            return Err(Error::new(
                "wem.materials.native",
                "native material ABI must be explicit",
            ));
        }
    };
    Ok(Some(payload))
}

pub(in crate::model::conversion::m3) fn model_profiles(
    doc: &wem::RawDocument<'_>,
    reference: wem::Ref<wr::targets::Model>,
    index: usize,
) -> Result<wem::Ref<wr::targets::ProfileMaterialSet>, Error> {
    let section = doc
        .referenced_section(reference)?
        .ok_or_else(|| Error::new("wem.models", "missing model table"))?;
    macro_rules! read {
        ($v:ty) => {{
            let model = doc
                .resolve::<wr::ModelRecord<$v>>(reference.cast())?
                .get(index)
                .ok_or_else(|| Error::new("wem.models", "missing model index"))??;
            Ok(model.profile_sets)
        }};
    }
    match section.revision() {
        1 => read!(wr::V1),
        2 => read!(wr::V2),
        3 => read!(wr::V3),
        4 => read!(wr::V4),
        5 => read!(wr::V5),
        6 => read!(wr::V6),
        7 => read!(wr::V7),
        8 => read!(wr::V8),
        9 => read!(wr::V9),
        _ => Err(Error::new("wem.models", "unsupported model revision")),
    }
}

pub(in crate::model::conversion::m3) fn body_name(
    body: &wr::M3Body,
) -> wem::Ref<wr::targets::Character> {
    match body {
        wr::M3Body::Standard(b) => b.name,
        wr::M3Body::Displacement(b) => b.name,
        wr::M3Body::Composite(b) => b.name,
        wr::M3Body::Terrain(b) => b.name,
        wr::M3Body::Volume(b) => b.name,
        wr::M3Body::VolumeNoise(b) => b.name,
        wr::M3Body::Creep(b) => b.name,
        wr::M3Body::Stb(b) => b.name,
        wr::M3Body::Reflection(b) => b.name,
        wr::M3Body::LensFlare(b) => b.name,
        wr::M3Body::DataDriven(b) => b.material_name,
    }
}
