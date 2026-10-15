//! Runtime selection of numeric layouts and file envelopes.
use super::codec::{Decoder, Encoder};
use super::resources::{Dependency, FileKey};
use super::{
    Chunked, Document, Flat, ModelLayout, ReadError, V272PreSplit, WriteError, V256, V257, V260,
    V263, V264, V265, V272, V273, V274,
};
use super::{Read, Write};
use crate::model::common::binary::Chunks;
use crate::model::common::Ownership;

fn identity(data: &[u8]) -> Result<(u32, bool), ReadError> {
    let (core, chunked, origin) = if data.starts_with(b"MD20") {
        (data, false, 0)
    } else {
        let mut core = None;
        for chunk in Chunks::new(data, 0) {
            let chunk = chunk.map_err(|error| ReadError::invalid(error.offset, "chunk framing"))?;
            if chunk.header.tag == *b"MD21" {
                core = Some((chunk.data, chunk.offset));
                break;
            }
        }
        let (core, origin) = core.ok_or_else(|| ReadError::invalid(0, "missing MD21 core"))?;
        (core, true, origin)
    };
    let bytes = core
        .get(4..8)
        .ok_or_else(|| ReadError::invalid(origin + 4, "version"))?;
    Ok((u32::from_le_bytes(bytes.try_into().unwrap()), chunked))
}
/// A layout with a corresponding dynamically dispatched model variant.
pub trait DynamicLayout: ModelLayout {
    /// Decodes a file using this layout and its detected native envelope.
    fn decode_dynamic<'data>(data: &'data [u8]) -> Result<DynamicDocument<'data>, ReadError>;
}
impl DynamicLayout for V272PreSplit {
    fn decode_dynamic<'data>(data: &'data [u8]) -> Result<DynamicDocument<'data>, ReadError> {
        let (_, chunked) = identity(data)?;
        if chunked {
            Ok(DynamicDocument::Chunked272PreSplit(Document::decode_m2(
                data,
            )?))
        } else {
            Ok(DynamicDocument::Flat272PreSplit(Document::decode_m2(data)?))
        }
    }
}
macro_rules! dynamic {
    ($($flat:ident, $chunked:ident : $version:ident = $number:literal),* $(,)?) => {
        /// A model whose numeric layout and envelope are selected at runtime.
        ///
        /// Default detection uses the modern sequence-blend interpretation for 272.
        /// Select its earlier interpretation with `decode_m2_with_layout::<V272PreSplit>`.
        #[derive(Clone, Debug, PartialEq, Ownership)]
        pub enum DynamicDocument<'data> {
            $(
                #[doc = concat!("Flat model with core version ", stringify!($number), ".")]
                $flat(Document<'data, Flat<$version>>),
                #[doc = concat!("Chunked model with core version ", stringify!($number), ".")]
                $chunked(Document<'data, Chunked<$version>>),
            )*
            /// Flat version 272 with the earlier single blend-time word.
            Flat272PreSplit(Document<'data, Flat<V272PreSplit>>),
            /// Chunked version 272 with the earlier single blend-time word.
            Chunked272PreSplit(Document<'data, Chunked<V272PreSplit>>),
        }
        $(impl DynamicLayout for $version {
            fn decode_dynamic<'data>(data:&'data [u8])->Result<DynamicDocument<'data>,ReadError>{
                let (_,chunked)=identity(data)?;
                if chunked {Ok(DynamicDocument::$chunked(Document::decode_m2(data)?))} else {Ok(DynamicDocument::$flat(Document::decode_m2(data)?))}
            }
        })*
        impl<'data> DynamicDocument<'data> {
            /// Detects the numeric version and envelope, using default layout profiles.
            pub fn decode_m2(data: &'data [u8]) -> Result<Self, ReadError> {
                match identity(data)? {
                    $(($number, false) => Ok(Self::$flat(Document::decode_m2(data)?)),
                      ($number, true) => Ok(Self::$chunked(Document::decode_m2(data)?)),)*
                    _ => Err(ReadError::invalid(4, "unsupported core version")),
                }
            }
            /// Decodes using an explicit native layout marker and detected envelope.
            pub fn decode_m2_with_layout<V:DynamicLayout>(data:&'data [u8])->Result<Self,ReadError>{V::decode_dynamic(data)}
            /// Returns the numeric core version.
            pub fn version(&self) -> u32 { match self { $(Self::$flat(model) => model.version(), Self::$chunked(model) => model.version(),)* Self::Flat272PreSplit(_) | Self::Chunked272PreSplit(_) => 272 } }
            /// Returns whether the model uses chunk framing.
            pub fn is_chunked(&self) -> bool { match self { $(Self::$flat(_) => false, Self::$chunked(_) => true,)* Self::Flat272PreSplit(_) => false, Self::Chunked272PreSplit(_) => true } }
            /// Enumerates model resource references.
            pub fn dependencies(&self, owner: &FileKey) -> Vec<Dependency> { match self { $(Self::$flat(model) => model.dependencies(owner), Self::$chunked(model) => model.dependencies(owner),)* Self::Flat272PreSplit(model) => model.dependencies(owner), Self::Chunked272PreSplit(model) => model.dependencies(owner) } }
            /// Checks native references using the concrete selected layout.
            pub fn validate(&self)->Result<(),ReadError>{match self {$(Self::$flat(model)=>model.validate(),Self::$chunked(model)=>model.validate(),)*Self::Flat272PreSplit(model)=>model.validate(),Self::Chunked272PreSplit(model)=>model.validate()}}
            /// Encodes the selected layout profile and envelope.
            pub fn encode_m2(&self) -> Result<Vec<u8>, WriteError> { match self { $(Self::$flat(model) => model.encode_m2(), Self::$chunked(model) => model.encode_m2(),)* Self::Flat272PreSplit(model) => model.encode_m2(), Self::Chunked272PreSplit(model) => model.encode_m2() } }
        }
    };
}
dynamic! {
    Flat256, Chunked256: V256 = 256,
    Flat257, Chunked257: V257 = 257,
    Flat260, Chunked260: V260 = 260,
    Flat263, Chunked263: V263 = 263,
    Flat264, Chunked264: V264 = 264,
    Flat265, Chunked265: V265 = 265,
    Flat272, Chunked272: V272 = 272,
    Flat273, Chunked273: V273 = 273,
    Flat274, Chunked274: V274 = 274,
}

impl<'data> Read<'data> for DynamicDocument<'data> {
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError> {
        let base = input.absolute_position();
        let remaining = input.data().len() - input.position();
        Self::decode_m2(input.bytes(remaining)?).map_err(|mut error| {
            error.offset = base.saturating_add(error.offset);
            error
        })
    }
}
impl Write for DynamicDocument<'_> {
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError> {
        output.bytes(&self.encode_m2()?)
    }
}
/// Applies an operation to the concrete typed model selected at runtime.
#[macro_export]
macro_rules! visit_m2_model {
    ($model:expr, |$typed:ident| $body:expr) => {
        match $model {
            $crate::model::formats::m2::DynamicDocument::Flat256($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked256($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat257($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked257($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat260($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked260($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat263($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked263($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat264($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked264($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat265($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked265($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat272($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked272($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat272PreSplit($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked272PreSplit($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat273($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked273($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Flat274($typed) => $body,
            $crate::model::formats::m2::DynamicDocument::Chunked274($typed) => $body,
        }
    };
}
