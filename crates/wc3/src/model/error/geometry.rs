//! Numeric, correspondence and operation failures in semantic geometry.
use crate::model::{
    AttributeValues, ConnectivityId, DrawSubsetId, GeometryInfluences, NodeId, PrimitiveRange,
    TangentBinding, Topology,
};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// The connectivity family, without retaining its index buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologyKind {
    /// Mixed representation.
    Mixed,
    /// Triangles representation.
    Triangles,
    /// Polygons representation.
    Polygons,
    /// Lines representation.
    Lines,
    /// TriangleStrip representation.
    TriangleStrip,
    /// Points representation.
    Points,
}
impl From<&Topology<'_>> for TopologyKind {
    fn from(value: &Topology<'_>) -> Self {
        match value {
            Topology::Mixed(_) => Self::Mixed,
            Topology::Triangles(_) => Self::Triangles,
            Topology::Polygons { .. } => Self::Polygons,
            Topology::Lines(_) => Self::Lines,
            Topology::TriangleStrip(_) => Self::TriangleStrip,
            Topology::Points(_) => Self::Points,
        }
    }
}
/// The encoded value type of one geometry attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributeValueKind {
    /// TriangleProperties representation.
    TriangleProperties,
    /// Boolean representation.
    Boolean,
    /// Byte3 representation.
    Byte3,
    /// Byte4 representation.
    Byte4,
    /// U16 representation.
    U16,
    /// Unsigned4 representation.
    Unsigned4,
    /// Quaternion representation.
    Quaternion,
    /// Scalar representation.
    Scalar,
    /// Vector2 representation.
    Vector2,
    /// Vector3 representation.
    Vector3,
    /// Vector4 representation.
    Vector4,
    /// Unsigned representation.
    Unsigned,
    /// Signed representation.
    Signed,
}
impl From<&AttributeValues<'_>> for AttributeValueKind {
    fn from(value: &AttributeValues<'_>) -> Self {
        match value {
            AttributeValues::TriangleProperties(_) => Self::TriangleProperties,
            AttributeValues::Boolean(_) => Self::Boolean,
            AttributeValues::Byte3(_) => Self::Byte3,
            AttributeValues::Byte4(_) => Self::Byte4,
            AttributeValues::U16(_) => Self::U16,
            AttributeValues::Unsigned4(_) => Self::Unsigned4,
            AttributeValues::Quaternion(_) => Self::Quaternion,
            AttributeValues::Scalar(_) => Self::Scalar,
            AttributeValues::Vector2(_) => Self::Vector2,
            AttributeValues::Vector3(_) => Self::Vector3,
            AttributeValues::Vector4(_) => Self::Vector4,
            AttributeValues::Unsigned(_) => Self::Unsigned,
            AttributeValues::Signed(_) => Self::Signed,
        }
    }
}
/// Authored influence interpretation, independent of its backing storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InfluenceKind {
    /// None representation.
    None,
    /// Variable representation.
    Variable,
    /// Three representation.
    Three,
    /// Four representation.
    Four,
    /// MatrixGroups representation.
    MatrixGroups,
}
impl From<&GeometryInfluences<'_>> for InfluenceKind {
    fn from(value: &GeometryInfluences<'_>) -> Self {
        match value {
            GeometryInfluences::None => Self::None,
            GeometryInfluences::Variable(_) => Self::Variable,
            GeometryInfluences::Three(_) => Self::Three,
            GeometryInfluences::Four(_) => Self::Four,
            GeometryInfluences::MatrixGroups(_) => Self::MatrixGroups,
        }
    }
}
/// An operation the selected geometry representation cannot retain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GeometryOperation {
    /// Edit representation.
    Edit,
    /// AddConnectivity representation.
    AddConnectivity,
    /// ReplacePositions representation.
    ReplacePositions,
    /// ReplaceNormals representation.
    ReplaceNormals,
    /// ReplaceTangents representation.
    ReplaceTangents,
    /// ReplaceColors representation.
    ReplaceColors,
    /// VertexRemap representation.
    VertexRemap,
    /// RemapSelectionBindings representation.
    RemapSelectionBindings,
    /// AlternateSelection representation.
    AlternateSelection,
    /// SupplementalStrips representation.
    SupplementalStrips,
    /// VertexWindows representation.
    VertexWindows,
    /// TangentBindings representation.
    TangentBindings,
    /// PrimitiveSelection representation.
    PrimitiveSelection,
    /// PrimaryConnectivity representation.
    PrimaryConnectivity,
    /// The number of additional topology sets exceeds the target limit.
    AdditionalConnectivity,
    /// IndexedPools representation.
    IndexedPools,
    /// TriangleProperties representation.
    TriangleProperties,
    /// AdditionalAttributes representation.
    AdditionalAttributes,
}
/// Why polygon boundaries are structurally invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonBoundaryRule {
    /// StartsAtZero representation.
    StartsAtZero,
    /// EndsAtCornerCount representation.
    EndsAtCornerCount,
    /// OrderedWithThreeCorners representation.
    OrderedWithThreeCorners,
}
/// Why an otherwise selected polygon cannot be triangulated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangulationFailure {
    /// ZeroArea representation.
    ZeroArea,
    /// NoEar representation.
    NoEar,
}
/// A concrete geometry constraint with the evidence needed to inspect it.
#[derive(Clone, Debug)]
pub enum GeometryError {
    /// The number of values differs from the required correspondence.
    CountMismatch {
        /// Required value or correspondence count.
        expected: usize,
        /// Observed value or representation.
        actual: usize,
    },
    /// An index does not address an existing element.
    IndexOutOfBounds {
        /// Rejected zero-based element index.
        index: usize,
        /// Number of available elements.
        len: usize,
    },
    /// A scalar component is not finite.
    NonFinite {
        /// Rejected scalar component.
        value: f32,
    },
    /// A minimum coordinate exceeds the corresponding maximum.
    InvertedBounds {
        /// Lower bound or minimum supported count.
        minimum: f32,
        /// Upper bound or maximum supported count, when bounded.
        maximum: f32,
    },
    /// A bounding radius is negative.
    NegativeRadius {
        /// Computed or authored radius.
        radius: f32,
    },
    /// A computed radius cannot be represented as finite f32.
    RadiusOverflow {
        /// Computed or authored radius.
        radius: f64,
    },
    /// A weight is negative or nonfinite.
    InvalidWeight {
        /// Rejected authored influence weight.
        weight: f32,
    },
    /// Nonzero weights do not sum to one within the accepted tolerance.
    UnnormalizedWeights {
        /// Sum of the authored positive contributions.
        sum: f32,
        /// Maximum accepted absolute difference from one.
        tolerance: f32,
    },
    /// A vertex has no finite positive joint contribution.
    NoPositiveWeights {
        /// Sum of the authored positive contributions.
        sum: f32,
    },
    /// An attribute duplicates an existing semantic/domain identity.
    DuplicateAttribute,
    /// The attribute value representation does not match its semantic/domain.
    AttributeType {
        /// Observed value or representation.
        actual: AttributeValueKind,
    },
    /// Connectivity cannot be resolved in the owning mesh.
    MissingConnectivity {
        /// Explicit mesh-owned connectivity identity.
        connectivity: ConnectivityId,
    },
    /// A topology set cannot be resolved by its schema index.
    MissingConnectivityIndex {
        /// Rejected topology-set index.
        index: usize,
    },
    /// A subset cannot be resolved in the owning mesh.
    MissingSubset {
        /// Explicit mesh-owned subset identity.
        subset: DrawSubsetId,
    },
    /// The selected representation does not support the requested operation.
    UnsupportedOperation {
        /// Requested unsupported operation.
        operation: GeometryOperation,
    },
    /// The ordered UV stream count is outside the supported range.
    TextureCoordinateCount {
        /// Lower bound or minimum supported count.
        minimum: usize,
        /// Upper bound or maximum supported count, when bounded.
        maximum: Option<usize>,
        /// Observed value or representation.
        actual: usize,
    },
    /// A vertex count cannot be represented by the index type.
    VertexCountOverflow {
        /// Observed value or representation.
        actual: usize,
        /// Upper bound or maximum supported count, when bounded.
        maximum: usize,
    },
    /// A remap changes primitive grouping or uses an unsupported topology.
    RemapTopology {
        /// Required value or correspondence count.
        expected: TopologyKind,
        /// Observed value or representation.
        actual: TopologyKind,
    },
    /// A remapped corner does not select the original vertex.
    RemapCorrespondence {
        /// Required value or correspondence count.
        expected: u32,
        /// Observed value or representation.
        actual: Option<u32>,
    },
    /// A polygon offset violates its boundary invariant.
    PolygonBoundary {
        /// Boundary invariant that failed.
        rule: PolygonBoundaryRule,
        /// Preceding offset, when one exists.
        previous: Option<u32>,
        /// Observed value or representation.
        actual: Option<u32>,
        /// Number of authored polygon corners.
        corners: usize,
    },
    /// The range counting domain does not match connectivity.
    SelectionDomain {
        /// Connectivity family being selected.
        topology: TopologyKind,
        /// Authored range and its counting domain.
        range: PrimitiveRange,
    },
    /// A selection exceeds its connectivity capacity.
    SelectionRange {
        /// Explicit mesh-owned connectivity identity.
        connectivity: ConnectivityId,
        /// Authored range and its counting domain.
        range: PrimitiveRange,
        /// Available elements in the selected counting domain.
        capacity: usize,
    },
    /// An authored inclusive vertex window is inverted or outside geometry.
    VertexWindow {
        /// First authored vertex or primitive.
        first: u32,
        /// Inclusive last authored vertex.
        last: u32,
        /// Number of available geometry vertices.
        vertices: usize,
    },
    /// A tangent binding cannot address its vertex window or pool.
    TangentBinding {
        /// Authored tangent correspondence.
        binding: TangentBinding,
        /// Number of available geometry vertices.
        vertices: usize,
        /// Number of available pool values, or no pool.
        pool: Option<usize>,
    },
    /// The end of a primitive range overflows.
    RangeOverflow {
        /// First authored vertex or primitive.
        first: u32,
        /// Authored number of elements.
        count: u32,
    },
    /// A polygon cannot be triangulated by the projection algorithm.
    Triangulation {
        /// Triangulation condition that prevented progress.
        reason: TriangulationFailure,
        /// Number of authored polygon corners.
        corners: usize,
    },
    /// A pool attribute has no explicit correspondence binding.
    MissingPoolBinding,
    /// A requested UV slot is absent.
    MissingTextureCoordinates {
        /// Requested texture-coordinate slot.
        slot: u32,
    },
    /// A tangent binding has no associated pool.
    MissingTangentPool,
    /// A projected vertex lies outside its tangent binding window.
    TangentVertexWindow {
        /// Selected geometry vertex index.
        vertex: usize,
        /// First authored vertex or primitive.
        first: u32,
        /// Authored number of elements.
        count: u32,
    },
    /// A node influence is absent from the skin palette.
    UnboundNode {
        /// Referenced influence node identity.
        node: NodeId,
    },
    /// Projected corners do not have a consistent attribute value type.
    InconsistentAttribute {
        /// Required value or correspondence count.
        expected: AttributeValueKind,
        /// Observed value or representation.
        actual: Option<AttributeValueKind>,
    },
    /// Triangle properties were attached to another topology family.
    TrianglePropertiesTopology {
        /// Observed value or representation.
        actual: TopologyKind,
    },
}
impl Display for GeometryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountMismatch { expected, actual } => write!(f, "expected {expected} values, found {actual}"),
            Self::IndexOutOfBounds { index, len } => write!(f, "index {index} is outside {len} values"),
            Self::NonFinite { value } => write!(f, "value must be finite, found {value}"),
            Self::InvertedBounds { minimum, maximum } => write!(f, "minimum {minimum} exceeds maximum {maximum}"),
            Self::NegativeRadius { radius } => write!(f, "radius must be nonnegative, found {radius}"),
            Self::RadiusOverflow { radius } => write!(f, "bounding radius {radius} exceeds f32"),
            Self::InvalidWeight { weight } => write!(f, "weight must be finite and nonnegative, found {weight}"),
            Self::UnnormalizedWeights { sum, tolerance } => write!(f, "weight sum {sum} differs from one by more than {tolerance}"),
            Self::NoPositiveWeights { sum } => write!(f, "vertex has no finite positive joint contribution (sum {sum})"),
            Self::DuplicateAttribute => f.write_str("attribute semantic and domain are already present"),
            Self::AttributeType { actual } => write!(f, "{actual:?} values do not match the attribute semantic and domain"),
            Self::MissingConnectivity { connectivity } => write!(f, "connectivity {connectivity:?} is absent or belongs to another mesh"),
            Self::MissingConnectivityIndex { index } => write!(f, "connectivity set {index} is absent"),
            Self::MissingSubset { subset } => write!(f, "subset {subset:?} is absent or belongs to another mesh"),
            Self::UnsupportedOperation { operation } => write!(f, "geometry representation does not support {operation:?}"),
            Self::TextureCoordinateCount { minimum, maximum, actual } => write!(f, "expected {minimum}..={maximum:?} UV streams, found {actual}"),
            Self::VertexCountOverflow { actual, maximum } => write!(f, "{actual} vertices exceed maximum {maximum}"),
            Self::RemapTopology { expected, actual } => write!(f, "remap requires {expected:?} primitive grouping, found {actual:?}"),
            Self::RemapCorrespondence { expected, actual } => write!(f, "remapped corner must select old vertex {expected}, found {actual:?}"),
            Self::PolygonBoundary { rule, previous, actual, corners } => write!(f, "polygon boundary violates {rule:?}: previous {previous:?}, actual {actual:?}, {corners} corners"),
            Self::SelectionDomain { topology, range } => write!(f, "{range:?} counting domain does not match {topology:?} connectivity"),
            Self::SelectionRange { connectivity, range, capacity } => write!(f, "{range:?} exceeds {capacity} elements in {connectivity:?}"),
            Self::VertexWindow { first, last, vertices } => write!(f, "vertex window {first}..={last} exceeds {vertices} vertices"),
            Self::TangentBinding { binding, vertices, pool } => write!(f, "{binding:?} exceeds {vertices} vertices or tangent pool {pool:?}"),
            Self::RangeOverflow { first, count } => write!(f, "primitive range {first}+{count} overflows"),
            Self::Triangulation { reason, corners } => write!(f, "cannot triangulate {corners}-corner polygon: {reason:?}"),
            Self::MissingPoolBinding => f.write_str("pool attribute requires an explicit selection binding"),
            Self::MissingTextureCoordinates { slot } => write!(f, "texture-coordinate slot {slot} is absent"),
            Self::MissingTangentPool => f.write_str("tangent binding requires a tangent pool"),
            Self::TangentVertexWindow { vertex, first, count } => write!(f, "vertex {vertex} is outside tangent window {first}+{count}"),
            Self::UnboundNode { node } => write!(f, "influence node {node:?} is absent from the skin palette"),
            Self::InconsistentAttribute { expected, actual } => write!(f, "projected attribute requires {expected:?} values, found {actual:?}"),
            Self::TrianglePropertiesTopology { actual } => write!(f, "triangle properties require triangle connectivity, found {actual:?}"),
        }
    }
}
impl StdError for GeometryError {}
