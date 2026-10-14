//! Construction-only native selections disappear before a graph is published.
use super::codec::error;
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::semantic::{profiles::Wem, Clip, ClipId, ModelCollectionBuilder, Time};
use crate::model::wem::{records, Bool, InlineOption, RawDocument, Ref, Target};
impl Field<Bool> for () {
    fn get(&self) -> Option<Bool> {
        None
    }
}
impl Field<Bool> for Bool {
    fn get(&self) -> Option<Bool> {
        Some(*self)
    }
}
pub(super) trait Field<T> {
    fn get(&self) -> Option<T>;
}
impl Field<records::Extent> for () {
    fn get(&self) -> Option<records::Extent> {
        None
    }
}
impl Field<records::Extent> for records::Extent {
    fn get(&self) -> Option<records::Extent> {
        Some(self.clone())
    }
}
impl Field<records::ReadRule> for () {
    fn get(&self) -> Option<records::ReadRule> {
        None
    }
}
impl Field<records::ReadRule> for records::ReadRule {
    fn get(&self) -> Option<records::ReadRule> {
        Some(*self)
    }
}
impl<T: Target> Field<Ref<T>> for () {
    fn get(&self) -> Option<Ref<T>> {
        None
    }
}
impl<T: Target> Field<Ref<T>> for Ref<T> {
    fn get(&self) -> Option<Ref<T>> {
        Some(*self)
    }
}
pub(in crate::model::conversion::wem) struct Prepared {
    pub clip_ids: Vec<ClipId>,
    pub(super) clips: Vec<NativeClip>,
}
pub(super) struct NativeClip {
    pub id: ClipId,
    pub model: usize,
    pub flags: records::ClipFlags,
    pub containers: Ref<records::targets::SubTrackContainer>,
    pub events: Ref<records::targets::ClipEvent>,
    pub metadata: records::NativeBag,
    pub bounds: Option<records::Extent>,
    pub selections: Ref<records::targets::ClipTrackSet>,
    pub read_rule: Option<records::ReadRule>,
    pub physics: Option<NativePhysics>,
}

/// Private parsing inventory; record dialects never enter the published graph.
pub(super) enum NativePhysics {
    Legacy(records::ClipPhysics<records::V5>),
    Cloth(records::ClipPhysics<records::V6>),
    Soft(records::ClipPhysics<records::V7>),
}
pub(super) trait PhysicsField {
    fn presence(&self) -> u8;
    fn physics(&self) -> Option<NativePhysics>;
}
impl PhysicsField for () {
    fn presence(&self) -> u8 {
        0
    }
    fn physics(&self) -> Option<NativePhysics> {
        None
    }
}
macro_rules! physics_field {
    ($dialect:ident,$variant:ident) => {
        impl PhysicsField for InlineOption<records::ClipPhysics<records::$dialect>> {
            fn presence(&self) -> u8 {
                self.presence
            }
            fn physics(&self) -> Option<NativePhysics> {
                self.value.clone().map(NativePhysics::$variant)
            }
        }
    };
}
physics_field!(V5, Legacy);
physics_field!(V6, Cloth);
physics_field!(V7, Soft);

pub(in crate::model::conversion::wem) fn prepare_clips<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Clip>,
    builder: &mut ModelCollectionBuilder<'a, Wem>,
    context: &mut Context,
) -> Result<Prepared, Error> {
    let mut prepared = Prepared {
        clip_ids: Vec::new(),
        clips: Vec::new(),
    };
    let Some(section) = document.referenced_section(reference).map_err(error)? else {
        return Ok(prepared);
    };
    macro_rules! read {
        ($dialect:ident) => {
            for row in document
                .resolve::<records::Clip<records::$dialect>>(reference.cast())
                .map_err(error)?
                .iter()
            {
                let row = row.map_err(error)?;
                let name = document.string(row.name).map_err(error)?.unwrap_or("");
                let duration = Time::seconds(f64::from(row.duration))?;
                if row.duration < 0.0 {
                    return Err(error("clip duration is negative"));
                }
                let model = usize::try_from(row.model).map_err(error)?;
                let target = builder
                    .model_mut(model)
                    .ok_or_else(|| error("clip names a missing model"))?;
                let mut clip = Clip::named(name).with_duration(duration);
                if row.looping.0 == 0 {
                    clip = clip.once();
                }
                if row.looping.0 > 1 {
                    context.issue(
                        "wem.clip.looping",
                        IssueKind::Normalized,
                        "nonzero looping byte is represented as true",
                    );
                }
                if row.physics.presence() > 1 {
                    context.issue(
                        "wem.clip.physics.presence",
                        IssueKind::Normalized,
                        "nonzero recorded-physics presence is represented as present",
                    );
                }
                let id = target.add_clip(clip)?;
                prepared.clip_ids.push(id);
                prepared.clips.push(NativeClip {
                    id,
                    model,
                    flags: row.flags,
                    containers: row.containers,
                    events: row.events,
                    metadata: row.native,
                    bounds: row.bounds.get(),
                    selections: row.track_sets.get().unwrap_or_default(),
                    read_rule: row.read_rule.get(),
                    physics: row.physics.physics(),
                });
            }
        };
    }
    match section.revision() {
        1 => read!(V1),
        2 => read!(V2),
        3 => read!(V3),
        4 => read!(V4),
        5 => read!(V5),
        6 => read!(V6),
        7 => read!(V7),
        _ => return Err(error("unsupported CLIP record revision")),
    }
    Ok(prepared)
}
