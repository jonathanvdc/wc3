//! Native animation membership tables represented by editable WEM track sets.
use super::super::effects::metadata as native_metadata;
use super::*;
pub(super) fn attach<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::AnimationState>,
    raw: &mut wem::RawDocument<'static>,
    model: &mut wr::ModelRecord<wr::V9>,
) -> Result<usize, Error> {
    let mut sets: Vec<_> = raw
        .resolve::<wr::TrackSet>(model.track_sets.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let base = sets.len();
    let mut profiles: Vec<_> = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let profile = profiles
        .iter_mut()
        .find(|profile| profile.profile == wr::ProfileId::Sc2)
        .ok_or_else(|| Error::new("wem.profile", "missing SC2 profile"))?;
    let mut entries: Vec<_> = raw
        .resolve::<wr::NativeEntry<wr::V2>>(profile.native.entries.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    for (index, state) in source
        .resolve::<nr::AnimationState>(reference.cast())
        .map_err(read_error)?
        .iter()
        .enumerate()
    {
        let state = state.map_err(read_error)?;
        let ids: Vec<_> = source
            .resolve::<nr::U32>(state.anim_ids.cast())
            .map_err(read_error)?
            .iter()
            .map(|row| row.map(|row| wr::U32 { value: row.value }))
            .collect::<Result<_, _>>()
            .map_err(read_error)?;
        sets.push(wr::TrackSet {
            name: text(raw, &format!("M3 animation state {}", base + index))?,
            channels: put(raw, &ids)?.cast(),
        });
        for (word, bytes) in state.unknown.as_chunks::<4>().0.iter().enumerate() {
            entries.push(wr::NativeEntry::<wr::V2> {
                name: text(raw, &format!("m3.state[{}].unknown[{word}]", base + index))?,
                value: i64::from(u32::from_le_bytes(*bytes)),
                ..Default::default()
            });
        }
    }
    if sets.len() != base {
        entries.push(wr::NativeEntry::<wr::V2> {
            name: text(raw, "m3.states.count")?,
            value: sets.len() as i64,
            ..Default::default()
        });
        profile.native.entries = put(raw, &entries)?.cast();
        model.profile_sets = put(raw, &profiles)?.cast();
        model.track_sets = put(raw, &sets)?.cast();
    }
    Ok(base)
}
pub(super) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<Vec<nr::AnimationState>, Error> {
    let Some(native) = &model.native else {
        return Ok(Vec::new());
    };
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let profiles: Vec<_> = raw
        .resolve::<wr::ProfileMaterialSet>(record.profile_sets.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let Some(profile) = profiles
        .iter()
        .find(|profile| profile.profile == wr::ProfileId::Sc2)
    else {
        return Ok(Vec::new());
    };
    let metadata = native_metadata::read(raw, &profile.native)?;
    if !metadata.contains_key("m3.states.count") {
        return Ok(Vec::new());
    }
    let mut states = Vec::new();
    for (index, set) in raw
        .resolve::<wr::TrackSet>(record.track_sets.cast())?
        .iter()
        .enumerate()
    {
        let set = set?;
        let ids: Vec<_> = raw
            .resolve::<wr::U32>(set.channels.cast())?
            .iter()
            .map(|row| row.map(|row| nr::U32 { value: row.value }))
            .collect::<Result<_, _>>()?;
        let mut unknown = [0; 16];
        for (word, bytes) in unknown.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            *bytes = (metadata
                .get(&format!("m3.state[{index}].unknown[{word}]"))
                .copied()
                .unwrap_or(0) as u32)
                .to_le_bytes();
        }
        states.push(nr::AnimationState {
            anim_ids: append(doc, &ids)?.cast(),
            unknown,
        });
    }
    Ok(states)
}
