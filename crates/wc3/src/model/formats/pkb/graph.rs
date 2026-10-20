//! Checked graph-slot domains, independent of native object-table links.
use super::{Cursor, Document, ReadError};

/// Zero-based layer position within one effect graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerIndex(#[doc = "Index into the owning graph's layers."] pub u32);
/// Zero-based event position within one effect graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EventIndex(#[doc = "Index into the owning graph's events."] pub u32);
/// Native effect resource roots; links remain one-based document identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Effect {
    /// Compiled graph object link.
    pub graph: u32,
    /// Authored attribute-list object link.
    pub attributes: u32,
}
/// Fully resolved graph slots with checked index domains.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph<'a> {
    /// Default entry position, distinct from the entry's event index.
    pub default_entry: u32,
    /// Uninterpreted authored coordinate-system tuple.
    pub coordinates: [u32; 4],
    /// Named and anonymous graph entry events.
    pub entries: Vec<GraphEntry<'a>>,
    /// Ordered layer slots referring to native compiled caches.
    pub layers: Vec<GraphLayer>,
    /// Ordered routes between layers.
    pub events: Vec<GraphEvent<'a>>,
    /// Native broadcast objects, preserved without claiming broadcast execution.
    pub broadcasts: Vec<u32>,
}
/// An entry name selects a graph event, rather than a bytecode position.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEntry<'a> {
    /// Native name; sparse omission denotes an anonymous entry.
    pub name: &'a [u8],
    /// Entry event within the owning graph.
    pub event: EventIndex,
}
/// One layer's native programme cache and event ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphLayer {
    /// One-based native compiled-layer object link.
    pub cache: u32,
    /// Incoming graph events.
    pub parents: Vec<EventIndex>,
    /// Events owned by this layer.
    pub events: Vec<EventIndex>,
}
/// Event routing; parent absence is the explicit native minus-one sentinel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEvent<'a> {
    /// Native name; an omitted name remains empty.
    pub name: &'a [u8],
    /// Owning layer; sparse omission denotes layer zero.
    pub parent: Option<LayerIndex>,
    /// Target layers in native order.
    pub targets: Vec<LayerIndex>,
    /// Zero-based native broadcast slots; execution remains unsupported.
    pub broadcasts: Vec<u32>,
}

impl<'a> Document<'a> {
    /// Discover native effect roots in document order.
    pub fn effects(&self) -> impl Iterator<Item = Result<Effect, ReadError>> + '_ {
        self.objects
            .iter()
            .enumerate()
            .filter(|(_, object)| {
                self.strings[self.types[object.type_index as usize].name as usize]
                    == b"CParticleEffect"
            })
            .map(|(index, _)| self.effect((index + 1) as u32))
    }
    /// Read the graph and authored-attribute links of one native effect root.
    pub fn effect(&self, link: u32) -> Result<Effect, ReadError> {
        let object = self.declaration(link, b"CParticleEffect")?;
        let (mut graph, mut attributes) = (None, None);
        self.fields(object, |field, input| {
            match field {
                1 => graph = Some(self.required_link(input)?),
                3 => attributes = Some(self.required_link(input)?),
                _ => return Err(input.error("unsupported effect-root field")),
            }
            Ok(())
        })?;
        Ok(Effect {
            graph: self.required(object, graph)?,
            attributes: self.required(object, attributes)?,
        })
    }
    pub(super) fn required_link(&self, input: &mut Cursor<'_>) -> Result<u32, ReadError> {
        let link = input.word()?;
        if self.object(link).is_none() {
            return Err(input.error("required object link is null or outside the object table"));
        }
        Ok(link)
    }
    pub(super) fn words(&self, input: &mut Cursor<'_>) -> Result<[u32; 4], ReadError> {
        Ok([input.word()?, input.word()?, input.word()?, input.word()?])
    }
    fn indices(&self, input: &mut Cursor<'_>) -> Result<Vec<u32>, ReadError> {
        let count = input.count(4)?;
        let mut indices = Vec::with_capacity(count);
        for _ in 0..count {
            let index = input.word()?;
            if index > i32::MAX as u32 {
                return Err(input.error("negative graph slot index is unsupported"));
            }
            indices.push(index);
        }
        Ok(indices)
    }
    /// Resolve graph links and check all layer, event and broadcast slot indices.
    /// Native coordinate controls remain data rather than an inferred transform.
    pub fn layer_graph(&self, link: u32) -> Result<Graph<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerGraphCompileCache")?;
        let (mut entries, mut layers, mut events, mut coordinates) = (None, None, None, None);
        let mut default_entry = 0;
        let mut broadcasts = Vec::new();
        self.fields(object, |field, input| {
            match field {
                0 => default_entry = input.word()?,
                1 => entries = Some(self.links(input)?),
                2 => layers = Some(self.links(input)?),
                3 => events = Some(self.links(input)?),
                4 => broadcasts = self.links(input)?,
                5 => coordinates = Some(self.words(input)?),
                _ => return Err(input.error("unsupported layer-graph field")),
            }
            Ok(())
        })?;
        let entries = self
            .required(object, entries)?
            .into_iter()
            .map(|link| self.graph_entry(link))
            .collect::<Result<Vec<_>, _>>()?;
        let layers = self
            .required(object, layers)?
            .into_iter()
            .map(|link| self.graph_layer(link))
            .collect::<Result<Vec<_>, _>>()?;
        let events = self
            .required(object, events)?
            .into_iter()
            .map(|link| self.graph_event(link))
            .collect::<Result<Vec<_>, _>>()?;
        let graph = Graph {
            default_entry,
            coordinates: self.required(object, coordinates)?,
            entries,
            layers,
            events,
            broadcasts,
        };
        if graph.default_entry as usize >= graph.entries.len() {
            return Err(ReadError {
                offset: self.object_origin(object),
                message: "default graph entry is outside the entry table",
            });
        }
        let event_valid = |index: EventIndex| (index.0 as usize) < graph.events.len();
        let layer_valid = |index: LayerIndex| (index.0 as usize) < graph.layers.len();
        if graph.entries.iter().any(|entry| !event_valid(entry.event))
            || graph.layers.iter().any(|layer| {
                layer
                    .parents
                    .iter()
                    .chain(&layer.events)
                    .any(|index| !event_valid(*index))
            })
            || graph.events.iter().any(|event| {
                event.parent.is_some_and(|index| !layer_valid(index))
                    || event.targets.iter().any(|index| !layer_valid(*index))
                    || event
                        .broadcasts
                        .iter()
                        .any(|index| *index as usize >= graph.broadcasts.len())
            })
        {
            return Err(ReadError {
                offset: self.object_origin(object),
                message: "graph route refers to a slot outside its owning table",
            });
        }
        for (index, layer) in graph.layers.iter().enumerate() {
            self.declaration(layer.cache, b"CLayerCompileCache")?;
            if layer.events.iter().any(|event| {
                graph.events[event.0 as usize].parent != Some(LayerIndex(index as u32))
            }) || layer.parents.iter().any(|event| {
                !graph.events[event.0 as usize]
                    .targets
                    .contains(&LayerIndex(index as u32))
            }) {
                return Err(ReadError {
                    offset: self.object_origin(object),
                    message: "layer event ownership disagrees with graph routing",
                });
            }
        }
        Ok(graph)
    }
    fn graph_entry(&self, link: u32) -> Result<GraphEntry<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerGraphCompileCache_EntrySlot")?;
        let mut name = &b""[..];
        let mut event = 0;
        self.fields(object, |field, input| {
            match field {
                0 => name = self.string(input)?,
                1 => {
                    event = input.word()?;
                    if event > i32::MAX as u32 {
                        return Err(input.error("negative graph entry event is unsupported"));
                    }
                }
                _ => return Err(input.error("unsupported graph-entry field")),
            }
            Ok(())
        })?;
        Ok(GraphEntry {
            name,
            event: EventIndex(event),
        })
    }
    fn graph_layer(&self, link: u32) -> Result<GraphLayer, ReadError> {
        let object = self.declaration(link, b"CLayerGraphCompileCache_LayerSlot")?;
        let mut cache = None;
        let mut parents = Vec::new();
        let mut events = Vec::new();
        self.fields(object, |field, input| {
            match field {
                0 => cache = Some(self.required_link(input)?),
                1 => parents = self.indices(input)?.into_iter().map(EventIndex).collect(),
                2 => events = self.indices(input)?.into_iter().map(EventIndex).collect(),
                _ => return Err(input.error("unsupported graph-layer field")),
            }
            Ok(())
        })?;
        Ok(GraphLayer {
            cache: self.required(object, cache)?,
            parents,
            events,
        })
    }
    fn graph_event(&self, link: u32) -> Result<GraphEvent<'a>, ReadError> {
        let object = self.declaration(link, b"CLayerGraphCompileCache_EventSlot")?;
        let mut name = &b""[..];
        let mut parent = Some(LayerIndex(0));
        let mut targets = Vec::new();
        let mut broadcasts = Vec::new();
        self.fields(object, |field, input| {
            match field {
                0 => name = self.string(input)?,
                1 => {
                    parent = match input.word()? as i32 {
                        -1 => None,
                        value if value >= 0 => Some(LayerIndex(value as u32)),
                        _ => return Err(input.error("unsupported graph parent-layer sentinel")),
                    }
                }
                2 => targets = self.indices(input)?.into_iter().map(LayerIndex).collect(),
                3 => broadcasts = self.indices(input)?,
                _ => return Err(input.error("unsupported graph-event field")),
            }
            Ok(())
        })?;
        Ok(GraphEvent {
            name,
            parent,
            targets,
            broadcasts,
        })
    }
}
