//! Mesh-owned connectivity selections retain identities across instances and profile changes.
use super::ids::Scope;
use super::{Error, GeometrySchema, Id, Topology};
use crate::model::error::{GeometryError, GeometryOperation, Location, TopologyKind};
use std::collections::BTreeMap;

/// ConnectivitySets identity marker, scoped to its owning mesh.
pub enum ConnectivityObject {}
/// Stable identity of one connectivity set on a mesh.
pub type ConnectivityId = Id<ConnectivityObject>;
/// Draw-subset identity marker, scoped to its owning mesh.
pub enum DrawSubsetObject {}
/// Stable identity of one reusable geometry selection.
pub type DrawSubsetId = Id<DrawSubsetObject>;

/// Counts either logical primitives or an authored strip-index window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveRange {
    /// Ordered primitives in triangle, polygon, point, line, or mixed topology.
    Primitives {
        /// First logical primitive.
        first: u32,
        /// Number of logical primitives.
        count: u32,
    },
    /// Indices in triangle-strip connectivity, independently of triangle counts.
    StripIndices {
        /// First authored strip index.
        first: u32,
        /// Number of strip indices.
        count: u32,
    },
}
impl PrimitiveRange {
    /// First element in this range's explicit counting domain.
    pub fn first(self) -> u32 {
        match self {
            Self::Primitives { first, .. } | Self::StripIndices { first, .. } => first,
        }
    }
    /// Number of elements in this range's explicit counting domain.
    pub fn count(self) -> u32 {
        match self {
            Self::Primitives { count, .. } | Self::StripIndices { count, .. } => count,
        }
    }
}
/// A range in an explicitly identified mesh connectivity set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveSelection {
    /// ConnectivitySets set in the owning mesh.
    pub connectivity: ConnectivityId,
    /// Range with an explicit counting domain.
    pub range: PrimitiveRange,
}
/// Supplemental strip declaration retained alongside an ordinary primitive selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StripSelection {
    /// Selected geometry in the owning mesh.
    pub selection: PrimitiveSelection,
    /// Unresolved supplemental strip control.
    pub control: u16,
}
/// An independently indexed tangent association with a consecutive vertex window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TangentBinding {
    /// First vertex in the associated consecutive window.
    pub first_vertex: u32,
    /// Number of vertices in the associated window.
    pub vertex_count: u32,
    /// First value in the owning mesh tangent pool.
    pub first_tangent: u32,
}
/// A reusable mesh-owned geometry selection; instance draws supply rendering behavior.
#[derive(Clone, Debug)]
pub struct DrawSubset {
    /// Selected geometry in the owning mesh.
    pub selection: PrimitiveSelection,
    /// Supplemental strip selection retained with the primary selection.
    pub strip: Option<StripSelection>,
    /// Inclusive authored vertex window, when declared.
    pub vertex_window: Option<[u32; 2]>,
    /// Association with an independently indexed tangent pool.
    pub tangents: Option<TangentBinding>,
}
impl DrawSubset {
    /// Selects a range with no additional pool bindings.
    pub fn new(selection: PrimitiveSelection) -> Self {
        Self {
            selection,
            strip: None,
            vertex_window: None,
            tangents: None,
        }
    }
}
/// Stable mesh-owned identities and draw subsets, with connectivity resolved through its schema.
#[derive(Clone, Debug)]
pub struct DrawSubsets {
    scope: Scope,
    primary: ConnectivityId,
    connectivity: BTreeMap<ConnectivityId, usize>,
    subsets: BTreeMap<DrawSubsetId, DrawSubset>,
}
impl Default for DrawSubsets {
    fn default() -> Self {
        let scope = Scope::default();
        let primary = scope.allocate();
        Self {
            scope,
            primary,
            connectivity: BTreeMap::from([(primary, 0)]),
            subsets: BTreeMap::new(),
        }
    }
}
impl DrawSubsets {
    /// Identity of the owning mesh's primary topology.
    pub fn primary(&self) -> ConnectivityId {
        self.primary
    }
    pub(crate) fn connectivity_id(&mut self, index: usize) -> ConnectivityId {
        if let Some((id, _)) = self.connectivity.iter().find(|(_, value)| **value == index) {
            return *id;
        }
        let id = self.scope.allocate();
        self.connectivity.insert(id, index);
        id
    }
    /// Inspects a subset by its scoped identity.
    pub fn get(&self, id: DrawSubsetId) -> Option<&DrawSubset> {
        self.subsets.get(&id)
    }
    /// Iterates stable subset identities and their selections.
    pub fn iter(&self) -> impl Iterator<Item = (DrawSubsetId, &DrawSubset)> {
        self.subsets.iter().map(|(id, value)| (*id, value))
    }
    pub(crate) fn insert(&mut self, value: DrawSubset) -> DrawSubsetId {
        let id = self.scope.allocate();
        self.subsets.insert(id, value);
        id
    }
    /// Updates a subset while retaining its identity; mesh publication validates its selections.
    pub fn set(&mut self, id: DrawSubsetId, value: DrawSubset) -> Result<(), Error> {
        let slot = self.subsets.get_mut(&id).ok_or_else(|| {
            Error::at(
                Location::for_owner(id),
                GeometryError::MissingSubset { subset: id },
            )
        })?;
        *slot = value;
        Ok(())
    }
    pub(crate) fn resolve<'data>(
        &self,
        id: ConnectivityId,
        geometry: &impl GeometrySchema<'data>,
    ) -> Option<Topology<'data>> {
        geometry.connectivity(*self.connectivity.get(&id)?)
    }
    pub(crate) fn validate<'data>(
        &self,
        geometry: &impl GeometrySchema<'data>,
    ) -> Result<(), Error> {
        for (id, subset) in self.iter() {
            let capabilities = geometry.subset_capabilities();
            for (requested, operation) in [
                (
                    !capabilities.alternate_selection
                        && subset.selection.connectivity != self.primary,
                    GeometryOperation::AlternateSelection,
                ),
                (
                    !capabilities.supplemental_strips && subset.strip.is_some(),
                    GeometryOperation::SupplementalStrips,
                ),
                (
                    !capabilities.vertex_windows && subset.vertex_window.is_some(),
                    GeometryOperation::VertexWindows,
                ),
                (
                    !capabilities.tangent_bindings && subset.tangents.is_some(),
                    GeometryOperation::TangentBindings,
                ),
            ] {
                if requested {
                    return Err(Error::at(
                        Location::for_owner(id),
                        GeometryError::UnsupportedOperation { operation },
                    ));
                }
            }

            self.check_selection(subset.selection, geometry)
                .map_err(|error| error.with_context(Location::for_owner(id).field("selection")))?;
            if let Some(strip) = subset.strip {
                if !matches!(strip.selection.range, PrimitiveRange::StripIndices { .. }) {
                    return Err(Error::at(
                        Location::for_owner(id)
                            .field("strip")
                            .field("selection")
                            .field("range"),
                        GeometryError::SelectionDomain {
                            topology: TopologyKind::TriangleStrip,
                            range: strip.selection.range,
                        },
                    ));
                }
                self.check_selection(strip.selection, geometry)
                    .map_err(|error| {
                        error
                            .with_context(Location::for_owner(id).field("strip").field("selection"))
                    })?;
            }
            let count = subset.selection.range.count();
            if count != 0
                && subset.vertex_window.is_some_and(|[first, last]| {
                    first > last || last as usize >= geometry.positions().len()
                })
            {
                return Err(Error::at(
                    Location::for_owner(id).field("vertex_window"),
                    GeometryError::VertexWindow {
                        first: subset.vertex_window.expect("checked window")[0],
                        last: subset.vertex_window.expect("checked window")[1],
                        vertices: geometry.positions().len(),
                    },
                ));
            }
            if let Some(binding) = subset.tangents {
                if (count != 0
                    && (binding.vertex_count == 0
                        || u64::from(binding.first_vertex) + u64::from(binding.vertex_count)
                            > geometry.positions().len() as u64))
                    || geometry
                        .tangent_pool()
                        .is_none_or(|values| binding.first_tangent as usize >= values.len())
                {
                    return Err(Error::at(
                        Location::for_owner(id).field("tangents"),
                        GeometryError::TangentBinding {
                            binding,
                            vertices: geometry.positions().len(),
                            pool: geometry.tangent_pool().map(|values| values.len()),
                        },
                    ));
                }
            }
        }
        Ok(())
    }
    fn check_selection<'data>(
        &self,
        selection: PrimitiveSelection,
        geometry: &impl GeometrySchema<'data>,
    ) -> Result<(), Error> {
        let topology = self
            .resolve(selection.connectivity, geometry)
            .ok_or_else(|| {
                Error::at(
                    Location::new().field("connectivity"),
                    GeometryError::MissingConnectivity {
                        connectivity: selection.connectivity,
                    },
                )
            })?;
        let capacity = match (&topology, selection.range) {
            (Topology::TriangleStrip(indices), PrimitiveRange::StripIndices { .. }) => {
                indices.len()
            }
            (Topology::TriangleStrip(_), PrimitiveRange::Primitives { .. })
            | (_, PrimitiveRange::StripIndices { .. }) => {
                return Err(Error::at(
                    Location::new().field("range"),
                    GeometryError::SelectionDomain {
                        topology: TopologyKind::from(&topology),
                        range: selection.range,
                    },
                ))
            }
            (_, PrimitiveRange::Primitives { .. }) => topology.primitive_count(),
        };
        if u64::from(selection.range.first()) + u64::from(selection.range.count()) > capacity as u64
        {
            return Err(Error::at(
                Location::new().field("range"),
                GeometryError::SelectionRange {
                    connectivity: selection.connectivity,
                    range: selection.range,
                    capacity,
                },
            ));
        }
        Ok(())
    }
}

/// Geometry-selection operations supported by an authoring schema.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawSubsetCapabilities {
    /// A draw can choose connectivity other than the primary topology.
    pub alternate_selection: bool,
    /// A subset can retain an independent supplemental strip declaration.
    pub supplemental_strips: bool,
    /// A subset can retain an explicit vertex window.
    pub vertex_windows: bool,
    /// A subset can associate vertices with an independent tangent pool.
    pub tangent_bindings: bool,
}
