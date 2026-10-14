//! Closed-profile playback declarations retain one mechanically encodable authority.
use super::ClipControls;
use crate::model::semantic::{ControllerId, Error};
use std::collections::BTreeSet;

pub(crate) fn playback<'a, 'data: 'a>(
    clips: impl IntoIterator<Item = &'a ClipControls<'data>>,
) -> Result<(), Error> {
    let mut presence = None;
    let mut report_family = None;
    for clip in clips {
        let fields = (clip.bounds.is_some(), clip.read_rule.is_some());
        if presence.is_some_and(|previous| previous != fields) {
            return Err(Error::new("wem.animation.clip","one editable-model clip table cannot mix bounding or read-rule declaration presence"));
        }
        presence = Some(fields);
        if !fields.0
            && (!clip.selections.is_empty() || clip.read_rule.is_some() || clip.physics.is_some())
        {
            return Err(Error::new(
                "wem.animation.clip",
                "playback selections and recorded physics require explicitly declared clip bounds",
            ));
        }
        if clip.physics.is_some() && !fields.1 {
            return Err(Error::new(
                "wem.physics.clip",
                "recorded simulation settings require an authored playback read rule",
            ));
        }
        if let Some(report) = clip
            .physics
            .as_ref()
            .and_then(|recipe| recipe.baked.as_ref())
        {
            let family = (report.cloth.is_some(), report.soft.is_some());
            if report_family.is_some_and(|previous| previous != family) {
                return Err(Error::new(
                    "wem.physics.bake",
                    "one clip table cannot mix distinct recorded bake measurement declarations",
                ));
            }
            report_family = Some(family);
        }
    }
    Ok(())
}
pub(crate) fn programmes<'a, 'data: 'a>(
    clips: impl IntoIterator<Item = &'a ClipControls<'data>>,
    controllers: impl IntoIterator<Item = ControllerId>,
) -> Result<(), Error> {
    let mut lists = Vec::new();
    for clip in clips {
        lists.push(clip.programs.clone());
        if let Some(report) = clip
            .physics
            .as_ref()
            .and_then(|recipe| recipe.baked.as_ref())
        {
            lists.push(report.source.iter().map(|saved| saved.program).collect());
            if let Some(source) = &report.soft_source {
                lists.push(source.clone());
            }
        }
    }
    let mut owned = BTreeSet::new();
    for (index, list) in lists.iter().enumerate() {
        let members: BTreeSet<_> = list.iter().copied().collect();
        if members.len() != list.len() {
            return Err(Error::new(
                "wem.animation.controller",
                "one playback reference cannot repeat a layer identity",
            ));
        }
        owned.extend(members.iter().copied());
        for other in &lists[index + 1..] {
            if other.iter().any(|id| members.contains(id))
                && !other.starts_with(list)
                && !list.starts_with(other)
            {
                return Err(Error::new(
                    "wem.animation.controller",
                    "shared playback layers require ordered prefix programme references",
                ));
            }
        }
    }
    if controllers.into_iter().any(|id| !owned.contains(&id)) {
        return Err(Error::new(
            "wem.animation.controller",
            "an authored layer requires a playback or restoration owner",
        ));
    }
    Ok(())
}
