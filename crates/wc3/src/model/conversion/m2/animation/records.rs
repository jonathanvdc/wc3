//! Native rest-value access and constant track construction.
use super::*;
pub(in super::super) fn initial<T: Value, V: ModelLayout>(track: &Track<'_, T, V>) -> Option<T>
where
    [T; 3]: Value,
{
    match track.values.inline(0)? {
        native::records::SequenceValues::Values(values) => values.get(0),
        native::records::SequenceValues::Splines(values) => values.get(0).map(|v| v[0]),
    }
}

pub(in super::super) fn ensure_constant<V: ModelLayout, T: Value>(
    track: &mut Track<'static, T, V>,
    value: T,
    globals: &mut Vec<u32>,
) where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
{
    if (0..track.base.timeline.len()).any(|index| {
        track
            .base
            .timeline
            .inline(index)
            .is_some_and(|times| !times.is_empty())
    }) {
        return;
    }
    let index = globals
        .iter()
        .position(|value| *value == 1000)
        .unwrap_or_else(|| {
            globals.push(1000);
            globals.len() - 1
        });
    track.base.interpolation = NativeInterpolation::Step;
    track.base.global_sequence = index as u16;
    track.base.timeline = TimelineFactory::build(vec![vec![0]], true);
    track.values = ValuesFactory::build(vec![vec![value]], Vec::new(), false);
}
