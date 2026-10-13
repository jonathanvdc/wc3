//! Spline components for native animation conversion.
use super::*;

/// Builds the native camera tangent-triplet storage family.
pub trait SplineFactory<T: Value>: Sized
where
    [T; 3]: Value,
{
    /// Constructs complete per-sequence tangent-triplet leaves.
    fn build(leaves: Vec<Vec<[T; 3]>>) -> Self;
}

impl<T: Value> SplineFactory<T> for native::records::LegacySplineValues<'static, T>
where
    [T; 3]: Value,
{
    fn build(leaves: Vec<Vec<[T; 3]>>) -> Self {
        Self {
            values: ZeroVec::alloc_from_slice(&leaves.into_iter().flatten().collect::<Vec<_>>()),
        }
    }
}

impl<T: Value> SplineFactory<T> for native::records::ModernSplineValues<'static, T>
where
    [T; 3]: Value,
{
    fn build(leaves: Vec<Vec<[T; 3]>>) -> Self {
        Self {
            values: leaves
                .into_iter()
                .map(|v| Array::Inline(ZeroVec::alloc_from_slice(&v)))
                .collect(),
        }
    }
}

pub(in super::super) fn resolve_array<D: Dialect, T: Value>(
    source: &Bundle<'_, D>,
    array: Array<'_, T>,
    sequence: usize,
) -> Result<Vec<T>, Error> {
    match array {
        Array::Inline(values) => Ok(values.iter().collect()),
        Array::External(reference) if reference.count == 0 => Ok(Vec::new()),
        Array::External(reference) => {
            let sequence = source
                .model
                .core()
                .sequences
                .get(sequence)
                .ok_or_else(|| Error::new("cameras.animation", "missing sequence"))?;
            let dependency=source.model.dependencies(&source.key).into_iter().find(|d|matches!(d.kind,native::DependencyKind::Animation{animation_id,variation_id}if animation_id==sequence.id&&variation_id==sequence.variation_index)).ok_or_else(||Error::new("cameras.animation","missing external animation identity"))?;
            let Some(File::Animation(animation)) = source.file(&dependency.key) else {
                return Err(Error::new(
                    "cameras.animation",
                    "missing external animation file",
                ));
            };
            let bytes = animation
                .stream(*b"AFM2")
                .ok_or_else(|| Error::new("cameras.animation", "missing model animation stream"))?;
            Ok(Decoder::new(bytes, D::VERSION, 0)
                .array::<T>(reference)
                .map_err(failure)?
                .iter()
                .collect())
        }
    }
}

pub(in super::super) fn import_spline<D: Dialect, T: Value>(
    source: &Bundle<'_, D>,
    track: &native::records::SplineTrack<'_, T, D::Layout>,
    sequence: usize,
    start: u32,
    target: sm::Target,
    values: impl Fn(T) -> Vec<f32>,
) -> Result<Option<sm::Track>, Error>
where
    [T; 3]: Value,
{
    use native::records::SplineStorage as _;
    let global = track.base.global_sequence != u16::MAX;
    let leaf = if global || D::VERSION < 264 {
        0
    } else {
        sequence
    };
    let Some(times) = track.base.timeline.leaf(leaf) else {
        return Ok(None);
    };
    let times = resolve_array(source, times, sequence)?;
    if times.is_empty() {
        return Ok(None);
    }
    let triplets = resolve_array(
        source,
        track
            .values
            .leaf(leaf)
            .ok_or_else(|| Error::new("cameras.keys", "missing spline values"))?,
        sequence,
    )?;
    if times.len() != triplets.len() {
        return Err(Error::new("cameras.keys", "spline time/key count differs"));
    }
    let range = (!global)
        .then(|| track.base.timeline.ranges().get(sequence))
        .flatten()
        .map(|[start, end]| start as usize..=end as usize);
    let mut keys = Vec::new();
    for (index, (time, [value, incoming, outgoing])) in times.into_iter().zip(triplets).enumerate()
    {
        if range.as_ref().is_some_and(|range| !range.contains(&index)) {
            continue;
        }
        keys.push(sm::Key {
            time: if global {
                time
            } else {
                time.checked_sub(start)
                    .ok_or_else(|| Error::new("cameras.time", "key precedes sequence start"))?
            } as f32
                / 1000.,
            value: values(value),
            incoming: Some(values(incoming)),
            outgoing: Some(values(outgoing)),
        });
    }
    let global_duration = if global {
        Some(
            source
                .model
                .core()
                .global_sequences
                .get(usize::from(track.base.global_sequence))
                .ok_or_else(|| Error::new("cameras.global_clock", "missing duration"))?
                as f32
                / 1000.,
        )
    } else {
        None
    };
    Ok(Some(sm::Track {
        layer: Default::default(),
        target,
        interpolation: interpolation(track.base.interpolation)?,
        global_duration,
        keys,
    }))
}

pub(in super::super) fn export_spline<V: ModelLayout, T: Value>(
    model: &sm::Model<'_>,
    target: sm::Target,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
    values: impl Fn(&[f32], &mut Context, &str) -> Result<T, Error>,
) -> Result<native::records::SplineTrack<'static, T, V>, Error>
where
    [T; 3]: Value,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, T>: ValuesFactory<T>,
    V::SplineValues<'static, T>: SplineFactory<T>,
{
    let track = export_track::<V, T>(model, target, starts, globals, context, values)?;
    let mut leaves = Vec::new();
    for index in 0..track.values.len() {
        leaves.push(match track.values.inline(index) {
            Some(native::records::SequenceValues::Values(values)) => {
                values.iter().map(|value| [value; 3]).collect()
            }
            Some(native::records::SequenceValues::Splines(values)) => values.iter().collect(),
            None => Vec::new(),
        });
    }
    Ok(native::records::SplineTrack {
        base: track.base,
        values: SplineFactory::build(leaves),
    })
}
