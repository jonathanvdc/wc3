//! Interval traversal for discrete event tracks.
use super::AnimationTime;
use crate::model::scene::EventObject;

/// One occurrence of an event key, including repeated loop occurrences.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventOccurrence {
    /// Index into the event object's sorted frame slice.
    pub key_index: usize,
    /// Authored timestamp on the model or global-sequence timeline.
    pub frame: i32,
    /// Unwrapped elapsed playback time of this occurrence, in milliseconds.
    pub elapsed_ms: f64,
}

impl AnimationTime<'_> {
    /// Collects all event keys crossed between `from_ms` and `self.elapsed_ms`.
    ///
    /// The interval is `(from_ms, elapsed_ms]`, or `[from_ms, elapsed_ms]` when
    /// `include_from` is true. Results are ordered by occurrence time and sorted
    /// key index. Duplicate keys each produce an occurrence. Traversal starts at
    /// elapsed zero; reverse or nonfinite intervals produce no occurrences.
    ///
    /// Model keys are restricted to the selected sequence's inclusive interval.
    /// Global keys are restricted to `0..=duration`. Looping tracks enumerate every
    /// crossed loop; a key at the end and a key at the next start are distinct
    /// occurrences at the same elapsed time. Zero-duration tracks fire zero-offset
    /// keys once. Missing clocks and reversed sequence intervals produce no events.
    pub fn event_occurrences(
        &self,
        event: &EventObject,
        from_ms: f64,
        include_from: bool,
    ) -> Vec<EventOccurrence> {
        let to_ms = self.elapsed_ms;
        if !from_ms.is_finite() || !to_ms.is_finite() || to_ms < from_ms || to_ms < 0.0 {
            return Vec::new();
        }
        let (start, end, looping) = if event.global_sequence_id != u32::MAX {
            let Some(&duration) = self.global_sequences.get(event.global_sequence_id as usize)
            else {
                return Vec::new();
            };
            (0.0, f64::from(duration), true)
        } else {
            let Some(sequence) = self.sequence else {
                return Vec::new();
            };
            (
                f64::from(sequence.interval[0]),
                f64::from(sequence.interval[1]),
                !sequence.flags.non_looping(),
            )
        };
        if end < start {
            return Vec::new();
        }
        let period = end - start;
        let mut occurrences = Vec::new();
        let frames = event.frames();
        let first = frames.partition_point(|&frame| f64::from(frame) < start);
        let last = frames.partition_point(|&frame| f64::from(frame) <= end);
        for (index, &frame) in frames[first..last].iter().enumerate() {
            let offset = f64::from(frame) - start;
            let mut cycle = if looping && period > 0.0 {
                ((from_ms - offset) / period).floor().max(0.0)
            } else {
                0.0
            };
            loop {
                let elapsed_ms = offset + cycle * period;
                if elapsed_ms > to_ms {
                    break;
                }
                if elapsed_ms > from_ms || (include_from && elapsed_ms == from_ms) {
                    occurrences.push(EventOccurrence {
                        key_index: first + index,
                        frame,
                        elapsed_ms,
                    });
                }
                if !looping || period == 0.0 {
                    break;
                }
                // Avoid a non-progressing loop at floating-point precision limits.
                let next = cycle + 1.0;
                if next == cycle || offset + next * period <= elapsed_ms {
                    break;
                }
                cycle = next;
            }
        }
        occurrences.sort_by(|a, b| {
            a.elapsed_ms
                .total_cmp(&b.elapsed_ms)
                .then(a.key_index.cmp(&b.key_index))
        });
        occurrences
    }
}
