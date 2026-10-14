use crate::model::common::FileKey;
use crate::model::conversion::{Error, ExportOptions, Format};
use crate::model::m3::records::{ReferenceFlags, VertexFlags};
use crate::model::semantic::profiles::wem::ShaderFamily;
use crate::model::semantic::profiles::{Extends, Narrows, Profile, Universal};
use crate::model::semantic::{
    ClipId, ContainsCardinality, Error as SemanticError, MaterialId, MeshId, Model, ModelBuilder,
    ModelCollection, OwnerId, Resources, ResourcesBuilder, SingleModel,
};
use std::borrow::Cow;

/// An uninterpreted source payload retained only when explicitly requested.
#[derive(Clone, Debug)]
pub struct OpaquePayload<'a> {
    /// Source format identifying the payload vocabulary.
    pub format: Format,
    /// Source member containing this payload.
    pub file: FileKey,
    /// Native chunk or section tag.
    pub tag: [u8; 4],
    /// Native record revision when the source declares one.
    pub revision: Option<u32>,
    /// Declared native record count, when applicable.
    pub count: Option<u32>,
    /// Uninterpreted payload bytes; borrowed unless explicitly detached.
    pub bytes: Cow<'a, [u8]>,
}
/// Optional physical encoding information that does not define model behavior.
#[derive(Clone, Debug)]
pub enum EncodingHint<'a> {
    /// Producer cache fingerprints for saved WEM bake results; they do not validate edited programmes.
    WemBakeFingerprints {
        /// Actual clip containing the recorded bake.
        clip: ClipId,
        /// Fingerprint of original bake inputs.
        inputs: u64,
        /// Fingerprint of original generated keys.
        written: u64,
    },
    /// Uninterpreted qualifiers of the packed M3 vertex declaration.
    M3VertexQualifiers {
        /// Only qualifier bits 2–4; logical geometry owns all actual attribute selections.
        flags: VertexFlags,
    },
    /// Producer revision of a native shader declaration embedded in an editable document.
    NativeShaderRevision {
        /// Actual logical material whose native declaration has this provenance.
        material: MaterialId,
        /// Native-game family supplying the declaration.
        source: ShaderFamily,
        /// Producer-reported native revision, retained outside authored shading behavior.
        revision: u32,
    },
    /// Affine native UV quantization. Current logical coordinates remain authoritative.
    M3TextureCoordinates {
        /// Geometry whose texture coordinates used this quantization.
        mesh: MeshId,
        /// Scale applied while decoding quantized coordinates.
        scale: f32,
        /// Offset applied after scaling quantized coordinates.
        offset: f32,
    },
    /// Uninterpreted qualifiers of a logical field's native reference.
    M3ReferenceQualifier {
        /// Logical object owning the native reference.
        owner: OwnerId,
        /// Field meaning distinguishing references on the same owner.
        role: Cow<'a, str>,
        /// Uninterpreted reference qualifiers retained outside model behavior.
        flags: ReferenceFlags,
    },
}
/// Logical models and optional inactive source attachments.
#[derive(Clone, Debug)]
pub struct Asset<'a, P: Profile = Universal> {
    /// Checked models in source asset order.
    pub models: ModelCollection<'a, P, P::Models>,
    /// Explicitly archived unknown source payloads.
    pub opaque: Vec<OpaquePayload<'a>>,
    /// Optional physical encoding information for rebuilding unchanged values.
    pub encoding_hints: Vec<EncodingHint<'a>>,
}
impl<'a, P: Profile> Asset<'a, P> {
    /// Widens each graph while preserving attachments and graph identities.
    pub fn widen<Q: Extends<P>>(self) -> Asset<'a, Q>
    where
        Q::Models: ContainsCardinality<P::Models>,
    {
        Asset {
            models: self.models.widen::<Q>().widen_cardinality(),
            opaque: self.opaque,
            encoding_hints: self.encoding_hints,
        }
    }
    /// Wraps one checked model without copying its values.
    pub fn new(model: SingleModel<'a, P>) -> Self {
        Self {
            models: ModelCollection::single(model),
            opaque: Vec::new(),
            encoding_hints: Vec::new(),
        }
    }
    /// Joins models according to this profile's native asset cardinality.
    pub fn from_models(
        models: Vec<SingleModel<'a, P>>,
        controls: P::Asset<'a>,
    ) -> Result<Self, Error> {
        Ok(Self {
            models: ModelCollection::from_models(models, controls)
                .map_err(|error| Error::new("models", error.to_string()))?,
            opaque: Vec::new(),
            encoding_hints: Vec::new(),
        })
    }
    /// Collection-wide authoring controls selected by this profile.
    pub fn controls(&self) -> &P::Asset<'a> {
        self.models.collection_controls()
    }
    /// Replaces collection controls after checking all their relationships.
    pub fn set_controls(&mut self, controls: P::Asset<'a>) -> Result<(), Error> {
        self.models
            .set_collection_controls(controls)
            .map_err(|error| Error::new("asset.controls", error.to_string()))
    }
    /// The single authoritative resource arena shared by the models.
    pub fn resources(&self) -> &Resources<'a, P> {
        self.models.resources()
    }
    /// Validates every model and the profile's asset-wide authoring relationships.
    pub fn validate(&self) -> Result<(), Error> {
        self.models
            .validate()
            .map_err(|error| Error::new("asset.models", error.to_string()))
    }
    /// Edits resource definitions atomically and checks the complete asset before publication.
    pub fn edit_resources<R>(
        &mut self,
        edit: impl FnOnce(&mut ResourcesBuilder<'a, P>) -> Result<R, SemanticError>,
    ) -> Result<R, Error> {
        self.models
            .edit_resources(edit)
            .map_err(|error| Error::new("resources", error.to_string()))
    }
    /// Edits one model and its dependencies atomically within the complete asset.
    pub fn edit_model<R>(
        &mut self,
        index: usize,
        edit: impl FnOnce(&mut ModelBuilder<'a, P>) -> Result<R, SemanticError>,
    ) -> Result<R, Error> {
        self.models
            .edit_model(index, edit)
            .map_err(|error| Error::new("model", error.to_string()))
    }
    /// Selects the model requested by an export plan.
    pub fn model(&self, options: &ExportOptions) -> Result<&Model<'a, P>, Error> {
        self.validate()?;
        self.models
            .get(options.model as usize)
            .ok_or_else(|| Error::new("model", format!("model {} is absent", options.model)))
    }
}

impl<'a, P: Profile> Asset<'a, P> {
    /// Narrows every graph exactly, preserving identities, resource storage, and inactive attachments.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<Asset<'a, Q>, Error> {
        Ok(Asset {
            models: self.models.try_narrow::<Q>()?,
            opaque: self.opaque,
            encoding_hints: self.encoding_hints,
        })
    }
}
