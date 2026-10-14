//! Shared prefix programme references preserve actual layer identity.
use super::super::{
    super::{metadata, text},
    write_curves,
};
use super::{
    codec::{append, error},
    declarations::Written,
};
use crate::model::conversion::{Context, Error};
use crate::model::semantic::{profiles::Wem, ControllerId, Model, TextureId};
use crate::model::wem::{records, Bool, RawDocument, Ref};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct ProgramLists {
    groups: Vec<(Vec<ControllerId>, Ref<records::targets::SubTrackContainer>)>,
}
impl ProgramLists {
    pub fn reference(
        &self,
        ids: &[ControllerId],
    ) -> Result<Ref<records::targets::SubTrackContainer>, Error> {
        if ids.is_empty() {
            return Ok(Ref::default());
        }
        let (_, reference) = self
            .groups
            .iter()
            .find(|(group, _)| group.starts_with(ids))
            .ok_or_else(|| error("programme list has no shared native prefix"))?;
        Ok(Ref::from_section(
            u32::try_from(ids.len()).map_err(error)?,
            reference.index,
        ))
    }
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    model: &Model<'_, Wem>,
    declarations: &Written,
    textures: &BTreeMap<TextureId, u32>,
    context: &mut Context,
) -> Result<ProgramLists, Error> {
    let mut lists = Vec::<Vec<ControllerId>>::new();
    for (_, clip) in model.clips() {
        lists.push(clip.controls.programs.clone());
        if let Some(report) = clip
            .controls
            .physics
            .as_ref()
            .and_then(|physics| physics.baked.as_ref())
        {
            lists.push(report.source.iter().map(|source| source.program).collect());
            if let Some(source) = &report.soft_source {
                lists.push(source.clone());
            }
        }
    }
    lists.retain(|list| !list.is_empty());
    for list in &lists {
        if list.iter().copied().collect::<BTreeSet<_>>().len() != list.len() {
            return Err(error(
                "one native programme list cannot repeat the same layer identity",
            ));
        }
    }
    lists.sort_by_key(|list| Reverse(list.len()));
    let mut groups = Vec::<Vec<ControllerId>>::new();
    for list in lists {
        if groups.iter().any(|group| group.starts_with(&list)) {
            continue;
        }
        if groups
            .iter()
            .any(|group| group.iter().any(|id| list.contains(id)))
        {
            return Err(error(
                "overlapping programme lists must be ordered prefixes to preserve layer identity",
            ));
        }
        groups.push(list);
    }
    let mut used = BTreeSet::new();
    let mut output = Vec::new();
    for group in groups {
        let mut rows = Vec::new();
        for id in &group {
            if !used.insert(*id) {
                return Err(error(
                    "programme identity occurs in more than one native section",
                ));
            }
            let controller = model
                .controller(*id)
                .ok_or_else(|| error("programme belongs to another model"))?;
            let mut tracks = Vec::new();
            for track in &controller.controls.tracks {
                let uid = *declarations
                    .ids
                    .get(&track.channel())
                    .ok_or_else(|| error("programme names a foreign declaration"))?;
                tracks.push(write_curves::write(document, track, uid, textures)?);
            }
            rows.push(records::SubTrackContainer {
                name: text::write_bytes(document, controller.name.as_bytes())?,
                priority: controller.priority,
                concurrent: Bool(u8::from(controller.concurrent)),
                sub_tracks: append(document, &tracks)?.cast(),
                native: metadata::write(document, &controller.controls.metadata)?,
            });
        }
        output.push((group, append(document, &rows)?.cast()));
    }
    for (id, _) in model.controllers() {
        if !used.contains(&id) {
            context.unsupported(
                "wem.controller.owner",
                "unselected layer has no native root or playback owner",
            )?;
        }
    }
    Ok(ProgramLists { groups: output })
}
