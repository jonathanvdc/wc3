//! Numeric and correspondence failures in semantic geometry.
//!
//! These errors retain the rejected counts, values and identities. The enclosing
//! [`super::Error::location`] identifies the stream, attribute and failing element;
//! diagnostics never retain a mesh or copy its buffers.
use crate::model::{
    AttributeDomain, AttributeSemantic, AttributeValues, ConnectivityId, DrawSubsetId,
    GeometryInfluences, NodeId, PrimitiveRange, TangentBinding, Topology,
};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// The connectivity family, without retaining its index buffers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologyKind {
    /// Ordered groups containing independently typed primitive families.
    Mixed,
    /// Independent indexed triangles.
    Triangles,
    /// Polygon corner indices and face-offset boundaries.
    Polygons,
    /// Independent indexed line pairs.
    Lines,
    /// An authored strip-index sequence.
    TriangleStrip,
    /// Independent indexed points.
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
    /// Typed per-triangle surface and collision behavior.
    TriangleProperties,
    /// Boolean values.
    Boolean,
    /// Three-component unsigned byte records.
    Byte3,
    /// Four-component unsigned byte records.
    Byte4,
    /// Unsigned 16-bit integers.
    U16,
    /// Four-component unsigned 32-bit integer records.
    Unsigned4,
    /// Quaternion values.
    Quaternion,
    /// Scalar 32-bit floats.
    Scalar,
    /// Two-component float vectors.
    Vector2,
    /// Three-component float vectors.
    Vector3,
    /// Four-component float vectors.
    Vector4,
    /// Unsigned 32-bit integers.
    Unsigned,
    /// Signed 32-bit integers.
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
    /// No authored joint distribution.
    None,
    /// Variable-length rows of weighted joint indices.
    Variable,
    /// Three authored weighted slots per vertex.
    Three,
    /// Four authored weighted slots per vertex.
    Four,
    /// Equal blending among ordered matrix-group joints.
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
    /// Apply a common geometry edit.
    Edit,
    /// Add another independently selectable topology set.
    AddConnectivity,
    /// Replace vertex positions.
    ReplacePositions,
    /// Replace vertex normals.
    ReplaceNormals,
    /// Replace vertex tangent frames.
    ReplaceTangents,
    /// Replace primary vertex colors.
    ReplaceColors,
    /// Change vertex correspondence while retaining primitive order.
    VertexRemap,
    /// Remap geometry with authored windows or independent pool bindings.
    RemapSelectionBindings,
    /// Select a topology other than the primary connectivity.
    AlternateSelection,
    /// Retain a supplemental strip-index selection.
    SupplementalStrips,
    /// Retain an authored inclusive vertex window.
    VertexWindows,
    /// Bind a tangent pool to a consecutive vertex window.
    TangentBindings,
    /// Use logical primitive counts at a native adapter boundary.
    PrimitiveSelection,
    /// Use primary connectivity at a native adapter boundary.
    PrimaryConnectivity,
}
/// Why polygon boundaries are structurally invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolygonBoundaryRule {
    /// The first boundary must be zero.
    StartsAtZero,
    /// The final boundary must equal the corner-array length.
    EndsAtCornerCount,
    /// Consecutive boundaries must increase by at least three corners.
    OrderedWithThreeCorners,
}
/// Why an otherwise selected polygon cannot be triangulated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriangulationFailure {
    /// The projected polygon has zero signed area.
    ZeroArea,
    /// Ear clipping cannot find another valid triangle.
    NoEar,
}
/// A concrete geometry constraint with the evidence needed to inspect it.
#[derive(Clone, Debug)]
pub enum GeometryError {
    /// The number of values differs from the required correspondence.
    CountMismatch {
        /// Number of values required by the correspondence domain.
        expected: usize,
        /// Number of authored values.
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
        /// Authored lower coordinate or radius.
        minimum: f32,
        /// Authored upper coordinate or radius.
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
        /// Sum of all authored influence weights.
        sum: f32,
        /// Maximum accepted absolute difference from one.
        tolerance: f32,
    },
    /// Positive influence weights overflowed to a nonfinite sum.
    NonFiniteWeightSum {
        /// Sum before normalization.
        sum: f32,
    },
    /// A vertex has no positive joint contribution.
    NoPositiveWeights {
        /// Sum of the authored positive contributions.
        sum: f32,
    },
    /// An attribute duplicates an existing semantic/domain identity.
    DuplicateAttribute,
    /// The attribute value representation does not match its semantic.
    AttributeType {
        /// Value type required by the attribute semantic.
        expected: AttributeValueKind,
        /// Authored value type.
        actual: AttributeValueKind,
    },
    /// An attribute requires another correspondence domain.
    AttributeDomain {
        /// Domain required by the attribute semantic.
        expected: AttributeDomain,
        /// Authored correspondence domain.
        actual: AttributeDomain,
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
        /// Minimum number of ordered UV streams.
        minimum: usize,
        /// Maximum number of UV streams, or no upper bound.
        maximum: Option<usize>,
        /// Number of authored UV streams.
        actual: usize,
    },
    /// A vertex or primitive count cannot be represented by the index type.
    CountOverflow {
        /// Authored vertex or primitive count.
        actual: usize,
        /// Largest count representable by the target index type.
        maximum: usize,
    },
    /// A remap changes the original primitive family.
    RemapTopology {
        /// Original primitive family whose grouping must be preserved.
        expected: TopologyKind,
        /// Replacement primitive family.
        actual: TopologyKind,
    },
    /// Vertex remapping is unavailable for this topology family.
    UnsupportedRemapTopology {
        /// Original connectivity family.
        topology: TopologyKind,
    },
    /// A polygon remap changed an authored offset boundary.
    RemapBoundary {
        /// Original corner offset.
        expected: u32,
        /// Replacement corner offset.
        actual: u32,
    },
    /// A remapped corner does not select the original vertex.
    RemapCorrespondence {
        /// Original vertex selected by this corner.
        expected: u32,
        /// Old vertex selected by the replacement corner, or an absent remap entry.
        actual: Option<u32>,
    },
    /// A polygon offset violates its boundary invariant.
    PolygonBoundary {
        /// Boundary invariant that failed.
        rule: PolygonBoundaryRule,
        /// Preceding offset, when one exists.
        previous: Option<u32>,
        /// Rejected boundary offset, or a missing boundary.
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
        /// First vertex in the authored inclusive window.
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
        /// First logical primitive in the range.
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
        /// First vertex addressed by the tangent binding.
        first: u32,
        /// Number of vertices addressed by the tangent binding.
        count: u32,
    },
    /// A node influence is absent from the skin palette.
    UnboundNode {
        /// Referenced influence node identity.
        node: NodeId,
    },
    /// Projected corners do not have a consistent attribute value type.
    InconsistentAttribute {
        /// Owned meaning of the requested attribute, including a custom name.
        semantic: AttributeSemantic<'static>,
        /// Value type present at the first projected source corner.
        expected: AttributeValueKind,
        /// Value type at the rejected corner, or an absent attribute.
        actual: Option<AttributeValueKind>,
    },
    /// Triangle properties were attached to another topology family.
    TrianglePropertiesTopology {
        /// Authored primary connectivity family.
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
            Self::NonFiniteWeightSum { sum } => write!(f, "positive influence weights have nonfinite sum {sum}"),
            Self::NoPositiveWeights { sum } => write!(f, "vertex has no positive joint contribution (sum {sum})"),
            Self::DuplicateAttribute => f.write_str("attribute semantic and domain are already present"),
            Self::AttributeType { expected, actual } => write!(f, "attribute requires {expected:?} values, found {actual:?}"),
            Self::AttributeDomain { expected, actual } => write!(f, "attribute requires {expected:?} correspondence, found {actual:?}"),
            Self::MissingConnectivity { connectivity } => write!(f, "connectivity {connectivity:?} is absent or belongs to another mesh"),
            Self::MissingConnectivityIndex { index } => write!(f, "connectivity set {index} is absent"),
            Self::MissingSubset { subset } => write!(f, "subset {subset:?} is absent or belongs to another mesh"),
            Self::UnsupportedOperation { operation } => write!(f, "geometry representation does not support {operation:?}"),
            Self::TextureCoordinateCount { minimum, maximum: Some(maximum), actual } => write!(f, "expected {minimum}..={maximum} UV streams, found {actual}"),
            Self::TextureCoordinateCount { minimum, maximum: None, actual } => write!(f, "expected at least {minimum} UV streams, found {actual}"),
            Self::CountOverflow { actual, maximum } => write!(f, "{actual} elements exceed maximum {maximum}"),
            Self::RemapTopology { expected, actual } => write!(f, "remap requires {expected:?} primitive grouping, found {actual:?}"),
            Self::UnsupportedRemapTopology { topology } => write!(f, "vertex remapping does not support {topology:?} connectivity"),
            Self::RemapBoundary { expected, actual } => write!(f, "remap must retain polygon offset {expected}, found {actual}"),
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
            Self::InconsistentAttribute { semantic, expected, actual } => write!(f, "projected {semantic:?} attribute requires {expected:?} values, found {actual:?}"),
            Self::TrianglePropertiesTopology { actual } => write!(f, "triangle properties require triangle connectivity, found {actual:?}"),
        }
    }
}
impl StdError for GeometryError {}
