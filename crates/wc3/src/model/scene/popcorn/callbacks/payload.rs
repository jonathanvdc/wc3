//! Invocation-owned payload construction, routing and extraction.
use super::super::graph::{Callback, External};
use super::super::runtime::Spawn;
use super::super::{Value, ValueType};
use super::{BoundElement, Draft, ExecutionHost, PayloadElement, PayloadStorage, Result};
impl ExecutionHost<'_> {
    pub(super) fn payload_call(
        &mut self,
        callback: Callback,
        receiver: Option<u16>,
        a: &[Value],
    ) -> Result<Option<Value>> {
        Ok(match callback {
            Callback::InitPayload => {
                self.context(a[2], false)?;
                if a[0].words()[0] > 2 {
                    return Err("unsupported native payload initialization control");
                }
                let External::Event(event) = self.external(
                    receiver.ok_or("payload initialization requires an event receiver")?,
                )?
                else {
                    return Err("payload initialization receiver is not an event");
                };
                let batch = self
                    .scratch
                    .batches
                    .iter()
                    .position(|batch| batch.key == a[1].words()[2])
                    .ok_or(
                        "generation batch capability is stale or belongs to another invocation",
                    )?;
                if self.scratch.drafts.len() >= self.limits.handles {
                    return Err("effect payload draft capacity exhausted");
                }
                let key = self.scratch.key()?;
                self.scratch.drafts.push(Draft {
                    key,
                    batch,
                    event,
                    committed: false,
                    control: a[0].words()[0],
                    omitted: false,
                });
                let index = self.scratch.drafts.len() - 1;
                let length = self.graph.events[event]
                    .payload
                    .fields
                    .len()
                    .min(self.scratch.payload_width);
                let cost = PayloadStorage::snapshot_words(length);
                if self
                    .payload
                    .charge(cost, self.limits.payload_words_per_update)
                {
                    let row = index * self.scratch.payload_width;
                    self.scratch.payload_fields[row..row + length].fill(None);
                } else {
                    self.scratch.drafts[index].omitted = true;
                }
                Some(Value::from_words(ValueType::Int, [key, 0, 0, 0]))
            }
            Callback::BuildElement => {
                self.context(a[4], false)?;
                let batch = self
                    .scratch
                    .batches
                    .iter()
                    .position(|batch| batch.key == a[0].words()[2])
                    .ok_or("payload element batch capability is stale")?;
                if self.scratch.elements.len() == self.limits.handles {
                    return Err("payload element capability capacity exhausted");
                }
                let element = PayloadElement::new(a[1], a[2], a[3].words()[0])?;
                let key = self.scratch.key()?;
                self.scratch.elements.push(BoundElement {
                    key,
                    batch,
                    element,
                });
                Some(Value::from(key as i32))
            }
            Callback::Append => {
                self.context(a[3], false)?;
                let External::Event(event) =
                    self.external(receiver.ok_or("payload append requires an event receiver")?)?
                else {
                    return Err("payload append receiver is not an event");
                };
                let draft = self.scratch.drafts.iter().position(|draft| draft.key == a[0].words()[0] && draft.event == event && !draft.committed).ok_or("payload append draft capability is stale, committed or owned by another event")?;
                let slot = a[1].words()[0] as usize;
                let declaration = self.graph.events[event]
                    .payload
                    .fields
                    .get(slot)
                    .ok_or("payload append declaration slot is invalid")?;
                let element = self
                    .scratch
                    .elements
                    .iter()
                    .find(|element| {
                        element.key == a[2].words()[0]
                            && element.batch == self.scratch.drafts[draft].batch
                    })
                    .ok_or("payload element capability is stale or belongs to another batch")?
                    .element;
                if declaration.kind != Some(element.first.kind())
                    || (element.semantic >> 8 != 0 && element.semantic >> 8 != declaration.role)
                {
                    return Err(
                        "payload element layout or semantic disagrees with event declaration",
                    );
                }
                if self.scratch.drafts[draft].omitted {
                    return Ok(Some(a[0]));
                }
                let destination =
                    &mut self.scratch.payload_fields[draft * self.scratch.payload_width + slot];
                if destination.is_some() {
                    return Err("payload declaration was appended more than once");
                }
                *destination = Some(element);
                Some(a[0])
            }
            Callback::Extract(kind) => {
                self.context(a[1], false)?;
                let value = self.payload.read(self.slot, a[0].words()[0] as usize)?;
                if value.kind() != kind {
                    return Err("payload extraction layout disagrees with root declaration");
                }
                Some(value)
            }
            Callback::Kick => {
                self.context(a[1], true)?;
                self.context(a[2], false)?;
                let External::Event(event) =
                    self.external(receiver.ok_or("event kick requires a bound receiver")?)?
                else {
                    return Err("event kick receiver is not an event");
                };
                let draft_index = self
                    .scratch
                    .drafts
                    .iter()
                    .position(|draft| draft.key == a[0].words()[0])
                    .ok_or("payload draft capability is stale or belongs to another invocation")?;
                let draft = &mut self.scratch.drafts[draft_index];
                if draft.event != event || draft.committed {
                    return Err("payload draft belongs to another event or was already committed");
                }
                draft.committed = true;
                let batch = self.scratch.batches[draft.batch];
                let _control = draft.control;
                let omitted = draft.omitted;
                if self.running {
                    let route = &self.graph.events[event];
                    let targets = &route.targets;
                    self.stats.omitted_births = self
                        .stats
                        .omitted_births
                        .saturating_add(batch.omitted.saturating_mul(targets.len() as u64));
                    self.random.counter = self.random.counter.wrapping_add(batch.omitted);
                    let start = draft_index * self.scratch.payload_width;
                    let values = &self.scratch.payload_fields
                        [start..start + route.payload.fields.len().min(self.scratch.payload_width)];
                    let words = PayloadStorage::snapshot_words(values.len());
                    if batch.start != batch.end
                        && (omitted
                            || !self
                                .payload
                                .charge(words, self.limits.payload_words_per_update))
                    {
                        self.stats.omitted_births = self.stats.omitted_births.saturating_add(
                            ((batch.end - batch.start) as u64).saturating_mul(targets.len() as u64),
                        );
                        self.random.counter = self
                            .random
                            .counter
                            .wrapping_add((batch.end - batch.start) as u64);
                    } else {
                        let snapshot = if batch.start != batch.end
                            && self.pending.len() < self.limits.pending_events
                            && values.iter().any(Option::is_some)
                        {
                            Some(self.payload.arena.snapshot(values)?)
                        } else {
                            None
                        };
                        for occurrence in &self.scratch.times[batch.start..batch.end] {
                            let seed = self.random.next();
                            for (target, layer) in targets.iter().enumerate() {
                                if !self.emitting && self.graph.layers[layer.0 as usize].rendered {
                                    self.stats.suppressed_births =
                                        self.stats.suppressed_births.saturating_add(1);
                                    continue;
                                }
                                if self.pending.len() >= self.limits.pending_events {
                                    self.stats.omitted_births =
                                        self.stats.omitted_births.saturating_add(1);
                                } else {
                                    if let Some(handle) = snapshot {
                                        self.payload.arena.retain(handle)?;
                                    }
                                    self.pending.push(Spawn {
                                        time: occurrence.time,
                                        layer: *layer,
                                        parent: Some(self.id),
                                        seed,
                                        order: *self.order,
                                        payload: snapshot,
                                        payload_event: event,
                                        payload_target: target,
                                        payload_fraction: occurrence.fraction,
                                        payload_ordinal: occurrence.ordinal,
                                    });
                                    *self.order = self.order.wrapping_add(1);
                                }
                            }
                        }
                        if let Some(handle) = snapshot {
                            self.payload.arena.release(handle)?;
                        }
                    }
                }
                None
            }
            _ => return Err("callback is not part of the payload protocol"),
        })
    }
}
