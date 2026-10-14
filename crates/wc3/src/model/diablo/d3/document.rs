use super::codec::{Decoder, Encoder};
use super::records::{
    ActorRecord, AnimationRecord, AnimationSetRecord, AppearanceRecord, MaterialRecord, ShaderMap,
    Shaders,
};
use super::{Read, ReadError, RecordLayout, Write, WriteError};
use crate::model::common::{Borrowed, Detached};
use std::marker::PhantomData;

mod sealed {
    pub trait Sealed {}
}
/// Exact native root revision and SNO group.
pub trait RootLayout: sealed::Sealed + Clone + std::fmt::Debug + PartialEq {
    /// Header revision belonging to this root family.
    const VERSION: u32;
    /// Native SNO group identifier.
    const GROUP: i32;
    /// Decoded record selected by this layout.
    type Record<'data>: Read<'data> + Write + Clone + std::fmt::Debug + PartialEq + Default;
}
/// A typed SNO document whose revision cannot contradict its root record.
#[derive(Clone, Debug, PartialEq)]
pub struct Document<'data, L: RootLayout> {
    /// Header words at bytes 8 and 12, retained verbatim.
    pub reserved: [u32; 2],
    /// Editable decoded root with borrowed encoded leaves.
    pub root: L::Record<'data>,
    marker: PhantomData<L>,
}
impl<L: RootLayout> Default for Document<'_, L> {
    fn default() -> Self {
        Self {
            reserved: [0; 2],
            root: Default::default(),
            marker: PhantomData,
        }
    }
}
impl<'data, L: RootLayout> Document<'data, L> {
    /// Constructs a document with the exact revision selected by `L`.
    pub fn new(root: L::Record<'data>) -> Self {
        Self {
            reserved: [0; 2],
            root,
            marker: PhantomData,
        }
    }
    /// Checks the header and resolves the selected native root.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        let head = Decoder::new(bytes);
        if head.value::<u32>(0)? != 0xDEADBEEF {
            return Err(ReadError::Magic);
        }
        let actual = head.value(4)?;
        if actual != L::VERSION {
            return Err(ReadError::Revision {
                expected: L::VERSION,
                actual,
            });
        }
        let reserved = [head.value(8)?, head.value(12)?];
        let decoder = Decoder::new(bytes.get(16..).ok_or(ReadError::Range {
            offset: 16,
            size: 0,
        })?);
        Ok(Self {
            reserved,
            root: L::Record::read(&decoder, 0)?,
            marker: PhantomData,
        })
    }
    /// Rebuilds every offset and byte-length pair in the supported root graph.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        let mut encoder = Encoder::new(L::Record::SIZE);
        self.root.write(&mut encoder, 0)?;
        let payload = encoder.finish();
        let mut bytes = Vec::with_capacity(16 + payload.len());
        bytes.extend(0xDEADBEEFu32.to_le_bytes());
        bytes.extend(L::VERSION.to_le_bytes());
        for word in self.reserved {
            bytes.extend(word.to_le_bytes());
        }
        bytes.extend(payload);
        Ok(bytes)
    }
}
macro_rules! family {
    ($trait:ident,$alias:ident,$marker:ident,$layout:ident,$revision:literal,$group:literal,$record:ident) => {
        #[doc=concat!("Native revision ",stringify!($revision)," marker.")]
        #[derive(Clone,Copy,Debug,Default,Eq,PartialEq)] pub struct $marker;
        impl sealed::Sealed for $marker {}
        /// Sealed layout selector for this native document family.
        pub trait $trait: sealed::Sealed { /// Concrete root layout.
            type Layout: RootLayout; }
        #[doc=concat!("Exact ",stringify!($alias)," root layout for revision ",stringify!($revision),".")]
        #[derive(Clone,Copy,Debug,Default,Eq,PartialEq)] pub struct $layout;
        impl sealed::Sealed for $layout {}
        impl RootLayout for $layout {const VERSION:u32=$revision;const GROUP:i32=$group;type Record<'data>=$record<'data>;}
        impl $trait for $marker {type Layout=$layout;}
        #[doc=concat!("Typed ",stringify!($alias)," SNO document.")]
        pub type $alias<'data,V=$marker> = Document<'data,<V as $trait>::Layout>;
    };
}
family!(
    AppearanceLayout,
    Appearance,
    V260,
    Appearance260,
    260,
    9,
    AppearanceRecord
);
family!(
    AnimationLayout,
    Animation,
    V118,
    Animation118,
    118,
    6,
    AnimationRecord
);
family!(ActorLayout, Actor, V282, Actor282, 282, 1, ActorRecord);
family!(
    AnimationSetLayout,
    AnimationSet,
    V24,
    AnimationSet24,
    24,
    8,
    AnimationSetRecord
);
family!(
    MaterialLayout,
    Material,
    V25,
    Material25,
    25,
    57,
    MaterialRecord
);
family!(
    ShaderMapLayout,
    ShaderMapDocument,
    V26,
    ShaderMap26,
    26,
    36,
    ShaderMap
);
family!(ShaderLayout, Shader, V150, Shader150, 150, 37, Shaders);

impl<'view, 'data, L: RootLayout> Borrowed<'view> for Document<'data, L>
where
    L::Record<'data>: Borrowed<'view, View = L::Record<'view>>,
{
    type View = Document<'view, L>;
    fn as_borrowed(&'view self) -> Self::View {
        Document {
            reserved: self.reserved,
            root: self.root.as_borrowed(),
            marker: PhantomData,
        }
    }
}
impl<'owned, 'data, L: RootLayout> Detached<'owned> for Document<'data, L>
where
    L::Record<'data>: Detached<'owned, Detached = L::Record<'owned>>,
{
    type Detached = Document<'owned, L>;
    fn into_detached(self) -> Self::Detached {
        Document {
            reserved: self.reserved,
            root: self.root.into_detached(),
            marker: PhantomData,
        }
    }
}
