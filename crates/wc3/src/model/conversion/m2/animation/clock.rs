//! Clock components for native animation conversion.
use super::*;

/// Decodes the two native animation clock representations.
pub trait SequenceClock {
    /// Start and duration in milliseconds.
    fn window(&self) -> (u32, u32);
    /// Constructs the native clock from a start and duration.
    fn from_window(start: u32, duration: u32) -> Self;
}

impl SequenceClock for u32 {
    fn window(&self) -> (u32, u32) {
        (0, *self)
    }
    fn from_window(_: u32, duration: u32) -> Self {
        duration
    }
}

impl SequenceClock for LegacyTiming {
    fn window(&self) -> (u32, u32) {
        (self.start, self.end.saturating_sub(self.start))
    }
    fn from_window(start: u32, duration: u32) -> Self {
        Self {
            start,
            end: start.saturating_add(duration),
        }
    }
}

pub(in super::super) fn milliseconds(
    seconds: f32,
    context: &mut Context,
    path: &str,
) -> Result<u32, Error> {
    if !seconds.is_finite() || seconds < 0. || f64::from(seconds) * 1000. > f64::from(u32::MAX) {
        return Err(Error::new(
            path,
            "time exceeds the unsigned millisecond clock",
        ));
    }
    let result = (seconds * 1000.).round() as u32;
    context.approximate(
        path,
        (result as f32 / 1000. - seconds).abs(),
        "animation time quantized to milliseconds",
    )?;
    Ok(result)
}

pub(in super::super) struct SequenceTables<'view, V: ModelLayout> {
    pub(in super::super) sequences: &'view [native::records::Sequence<V>],
    pub(in super::super) lookup: &'view zerovec::ZeroSlice<u16>,
    pub(in super::super) globals: &'view zerovec::ZeroSlice<u32>,
}
pub(in super::super) fn sequence_tables<'view, 'data, D: Dialect>(
    source: &'view Bundle<'data, D>,
    owner: &native::FileKey,
) -> Result<SequenceTables<'view, D::Layout>, Error> {
    let mut key = owner.clone();
    let mut seen = std::collections::BTreeSet::new();
    while key != source.key {
        if !seen.insert(key.clone()) {
            return Err(Error::new("skeleton.parent", "parent skeleton cycle"));
        }
        let Some(File::Skeleton(skeleton)) = source.file(&key) else {
            break;
        };
        let typed = skeleton
            .as_typed::<D::Layout>()
            .ok_or_else(|| Error::new("skeleton.layout", "skeleton layout differs from model"))?;
        let mut parent = None;
        for chunk in &typed.chunks {
            match chunk {
                native::companions::SkeletonChunk::Sequences {
                    global_loops,
                    sequences,
                    lookup,
                    ..
                } => {
                    return Ok(SequenceTables {
                        sequences,
                        lookup: lookup.as_slice(),
                        globals: global_loops.as_slice(),
                    });
                }
                native::companions::SkeletonChunk::Parent { file_id, .. } if *file_id != 0 => {
                    parent = Some(native::FileKey::FileId(*file_id))
                }
                _ => {}
            }
        }
        let Some(parent) = parent else {
            break;
        };
        key = parent;
    }
    Ok(SequenceTables {
        sequences: &source.model.core().sequences,
        lookup: source.model.core().sequence_lookup.as_slice(),
        globals: source.model.core().global_sequences.as_slice(),
    })
}
