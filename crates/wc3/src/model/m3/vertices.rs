//! Storage-, layout-, and shape-selected views into native packed vertices.
//!
//! A single `Vertices<S, L, Shape>` type represents immutable borrowed slices,
//! mutable slices, and owned buffers. Layout markers determine byte meaning;
//! shape markers distinguish one vertex from a complete buffer. Raw layouts do
//! not expose position, normal, or UV methods.
use super::records::VertexFlags;
use super::ReadError;
use crate::visit_m3_vertices;
use std::array::from_fn;
use std::borrow::Cow;
use std::marker::PhantomData;

mod writable_sealed {
    pub trait Storage {}
}
/// Native byte storage whose mutable view preserves its length.
pub trait WritableStorage: AsRef<[u8]> + writable_sealed::Storage {
    /// Mutable native bytes; borrowed Cow storage copies on its first edit.
    fn bytes_mut(&mut self) -> &mut [u8];
}
impl writable_sealed::Storage for Vec<u8> {}
impl WritableStorage for Vec<u8> {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}
impl writable_sealed::Storage for &mut [u8] {}
impl WritableStorage for &mut [u8] {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self
    }
}
impl writable_sealed::Storage for Cow<'_, [u8]> {}
impl WritableStorage for Cow<'_, [u8]> {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self.to_mut()
    }
}
impl<const N: usize> writable_sealed::Storage for [u8; N] {}
impl<const N: usize> WritableStorage for [u8; N] {
    fn bytes_mut(&mut self) -> &mut [u8] {
        self
    }
}

/// One complete vertex rather than an array of vertices.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct One;
/// A complete array of vertices with a statically selected stride.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Many;
/// Color storage selected by a packed layout.
pub trait ColorLayout {
    /// Number of native color bytes.
    const WIDTH: usize;
}
/// Packed vertices without BGRA color bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoColor;
/// Packed vertices with four BGRA color bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Color;
impl ColorLayout for NoColor {
    const WIDTH: usize = 0;
}
impl ColorLayout for Color {
    const WIDTH: usize = 4;
}
/// Extra serialized slot selected independently of color and UV channels.
pub trait ExtraLayout {
    /// Number of extra native bytes.
    const WIDTH: usize;
}
/// No additional uninterpreted slot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoExtra;
/// Four raw bytes before the tangent; their interpretation must be selected explicitly.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExtraRaw;
impl ExtraLayout for NoExtra {
    const WIDTH: usize = 0;
}
impl ExtraLayout for ExtraRaw {
    const WIDTH: usize = 4;
}
/// Native layout with a fixed per-element byte width.
pub trait Layout {
    /// Encoded bytes in one element.
    const STRIDE: usize;
    /// Checks that marker parameters describe a supported native layout.
    fn validate_layout() -> Result<(), ReadError> {
        Ok(())
    }
}
/// Verified packed prefix, sparse UV channels, optional color, and tangent bytes.
///
/// UV_MASK bits 0–4 select the named native UV channels. Channel four uses
/// vertex flag 0x40000000. Flag 0x20000000 has no automatic typed interpretation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Packed<const UV_MASK: u8, C = NoColor, X = NoExtra>(PhantomData<(C, X)>);
impl<const M: u8, C: ColorLayout, X: ExtraLayout> Layout for Packed<M, C, X> {
    const STRIDE: usize = 28 + C::WIDTH + (M.count_ones() as usize) * 4 + X::WIDTH;
    fn validate_layout() -> Result<(), ReadError> {
        if M & !31 != 0 {
            return Err(ReadError::invalid(0, "unsupported UV mask"));
        }
        Ok(())
    }
}
/// Explicit uninterpreted elements; callers supply their verified stride.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Raw<const STRIDE: usize>;
impl<const STRIDE: usize> Layout for Raw<STRIDE> {
    const STRIDE: usize = STRIDE;
}
/// Packed layouts that contain one particular named UV channel.
pub trait UvChannel<const CHANNEL: usize>: Layout {
    /// Byte offset of that channel.
    const OFFSET: usize;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<1, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<2, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<3, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<3, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<4, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<5, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<5, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<6, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<6, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<7, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<7, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<7, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<8, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<9, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<9, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<10, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<10, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<11, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<11, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<11, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<12, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<12, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<13, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<13, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<13, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<14, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<14, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<14, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<15, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<15, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<15, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<15, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 12;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<16, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<17, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<17, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<18, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<18, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<19, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<19, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<19, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<20, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<20, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<21, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<21, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<21, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<22, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<22, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<22, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<23, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<23, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<23, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<23, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 12;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<24, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<24, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<25, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<25, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<25, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<26, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<26, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<26, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<27, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<27, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<27, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<27, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 12;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<28, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<28, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<28, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<29, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<29, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<29, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<29, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 12;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<30, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<30, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<30, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<30, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 12;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<0> for Packed<31, C, X> {
    const OFFSET: usize = 24 + C::WIDTH;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<1> for Packed<31, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 4;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<2> for Packed<31, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 8;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<3> for Packed<31, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 12;
}
impl<C: ColorLayout, X: ExtraLayout> UvChannel<4> for Packed<31, C, X> {
    const OFFSET: usize = 24 + C::WIDTH + 16;
}
/// One native data structure whose storage, byte interpretation, and shape are types.
#[derive(Clone, Debug, PartialEq)]
pub struct Vertices<S, L, Shape = Many> {
    storage: S,
    layout: PhantomData<L>,
    shape: PhantomData<Shape>,
}
impl<S: AsRef<[u8]>, L: Layout> Vertices<S, L, Many> {
    /// Validates complete element boundaries without copying storage.
    pub fn new(storage: S) -> Result<Self, ReadError> {
        L::validate_layout()?;
        if L::STRIDE == 0 || !storage.as_ref().len().is_multiple_of(L::STRIDE) {
            return Err(ReadError::invalid(0, "packed vertex stride"));
        }
        Ok(Self {
            storage,
            layout: PhantomData,
            shape: PhantomData,
        })
    }
    /// Complete element count under the selected layout.
    pub fn len(&self) -> usize {
        self.storage.as_ref().len() / L::STRIDE
    }
    /// Whether no complete elements are present.
    pub fn is_empty(&self) -> bool {
        self.storage.as_ref().is_empty()
    }
    /// Borrows a one-element view with the same byte interpretation.
    pub fn vertex(&self, index: usize) -> Option<Vertices<&[u8], L, One>> {
        if index >= self.len() {
            return None;
        }
        let at = index * L::STRIDE;
        Some(Vertices {
            storage: &self.storage.as_ref()[at..at + L::STRIDE],
            layout: PhantomData,
            shape: PhantomData,
        })
    }
    /// Iterates immutable one-element views in native order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = Vertices<&[u8], L, One>> {
        (0..self.len()).map(|index| self.vertex(index).unwrap())
    }
}
impl<S: WritableStorage, L: Layout> Vertices<S, L, Many> {
    /// Borrows a mutable one-element view without copying its storage.
    pub fn vertex_mut(&mut self, index: usize) -> Option<Vertices<&mut [u8], L, One>> {
        if index >= self.len() {
            return None;
        }
        let at = index * L::STRIDE;
        Some(Vertices {
            storage: &mut self.storage.bytes_mut()[at..at + L::STRIDE],
            layout: PhantomData,
            shape: PhantomData,
        })
    }
}
impl<S: AsRef<[u8]>, L, Sh> Vertices<S, L, Sh> {
    /// Exact original or edited encoded bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.storage.as_ref()
    }
    /// Consumes the view and returns its storage without conversion.
    pub fn into_storage(self) -> S {
        self.storage
    }
    /// Copies bytes into independently owned storage while preserving layout and shape.
    pub fn to_owned(&self) -> Vertices<Vec<u8>, L, Sh> {
        Vertices {
            storage: self.storage.as_ref().to_vec(),
            layout: PhantomData,
            shape: PhantomData,
        }
    }
}
impl<S: AsRef<[u8]>, const M: u8, C: ColorLayout, X: ExtraLayout>
    Vertices<S, Packed<M, C, X>, One>
{
    /// Position in model coordinates.
    pub fn position(&self) -> [f32; 3] {
        let bytes = self.as_bytes();
        from_fn(|i| f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap()))
    }
    /// Four encoded weight bytes, interpreted as weight / 255.
    pub fn weights(&self) -> [u8; 4] {
        self.as_bytes()[12..16].try_into().unwrap()
    }
    /// Four region-local bone lookup slots.
    pub fn bone_indices(&self) -> [u8; 4] {
        self.as_bytes()[16..20].try_into().unwrap()
    }
    /// Packed normal XYZ and its handedness byte, without shader conversion.
    pub fn normal_bytes(&self) -> [u8; 4] {
        self.as_bytes()[20..24].try_into().unwrap()
    }
    /// Packed tangent XYZ and its stored trailing fourth byte.
    pub fn tangent_bytes(&self) -> [u8; 4] {
        let at = Packed::<M, C, X>::STRIDE - 4;
        self.as_bytes()[at..].try_into().unwrap()
    }
    /// Explicit unsigned-normalized normal reading: byte / 255 × 2 − 1.
    pub fn normal_unorm(&self) -> [f32; 3] {
        let bytes = self.normal_bytes();
        from_fn(|i| f32::from(bytes[i]) / 255.0 * 2.0 - 1.0)
    }
    /// Explicit unsigned-normalized tangent reading, without orthogonalization.
    pub fn tangent_unorm(&self) -> [f32; 3] {
        let bytes = self.tangent_bytes();
        from_fn(|i| f32::from(bytes[i]) / 255.0 * 2.0 - 1.0)
    }
    /// Raw signed coordinates of a channel present in this layout.
    pub fn uv<const CHANNEL: usize>(&self) -> [i16; 2]
    where
        Packed<M, C, X>: UvChannel<CHANNEL>,
    {
        let at = <Packed<M, C, X> as UvChannel<CHANNEL>>::OFFSET;
        let bytes = self.as_bytes();
        [
            i16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()),
            i16::from_le_bytes(bytes[at + 2..at + 4].try_into().unwrap()),
        ]
    }
    /// Historical UV interpretation, dividing native coordinates by 2048.
    pub fn uv_legacy<const CHANNEL: usize>(&self) -> [f32; 2]
    where
        Packed<M, C, X>: UvChannel<CHANNEL>,
    {
        self.uv::<CHANNEL>().map(|value| f32::from(value) / 2048.0)
    }
    /// REGN v5+ interpretation with caller-selected multiply and offset.
    pub fn uv_scaled<const CHANNEL: usize>(&self, multiply: f32, offset: f32) -> [f32; 2]
    where
        Packed<M, C, X>: UvChannel<CHANNEL>,
    {
        self.uv::<CHANNEL>()
            .map(|value| f32::from(value) * multiply + offset)
    }
}
impl<S: AsRef<[u8]>, const M: u8, X: ExtraLayout> Vertices<S, Packed<M, Color, X>, One> {
    /// Native BGRA color; unavailable on a NoColor layout.
    pub fn color(&self) -> [u8; 4] {
        self.as_bytes()[24..28].try_into().unwrap()
    }
}
impl<S: AsRef<[u8]>, const M: u8, C: ColorLayout> Vertices<S, Packed<M, C, ExtraRaw>, One> {
    /// Four unresolved bytes explicitly selected before the packed tangent.
    pub fn extra_bytes(&self) -> [u8; 4] {
        let at = Packed::<M, C, ExtraRaw>::STRIDE - 8;
        self.as_bytes()[at..at + 4].try_into().unwrap()
    }
}
impl<S: WritableStorage, const M: u8, C: ColorLayout, X: ExtraLayout>
    Vertices<S, Packed<M, C, X>, One>
{
    /// Replaces native little-endian model-space position floats.
    pub fn set_position(&mut self, value: [f32; 3]) {
        for (i, value) in value.into_iter().enumerate() {
            self.storage.bytes_mut()[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
    /// Replaces four native weight bytes without normalization.
    pub fn set_weights(&mut self, value: [u8; 4]) {
        self.storage.bytes_mut()[12..16].copy_from_slice(&value);
    }
    /// Replaces region-local bone lookup slots.
    pub fn set_bone_indices(&mut self, value: [u8; 4]) {
        self.storage.bytes_mut()[16..20].copy_from_slice(&value);
    }
    /// Replaces normal XYZ and handedness bytes exactly.
    pub fn set_normal_bytes(&mut self, value: [u8; 4]) {
        self.storage.bytes_mut()[20..24].copy_from_slice(&value);
    }
    /// Replaces tangent XYZ and the trailing fourth byte exactly.
    pub fn set_tangent_bytes(&mut self, value: [u8; 4]) {
        let at = Packed::<M, C, X>::STRIDE - 4;
        self.storage.bytes_mut()[at..].copy_from_slice(&value);
    }
    /// Replaces a UV channel defined by this layout.
    pub fn set_uv<const CHANNEL: usize>(&mut self, value: [i16; 2])
    where
        Packed<M, C, X>: UvChannel<CHANNEL>,
    {
        let at = <Packed<M, C, X> as UvChannel<CHANNEL>>::OFFSET;
        for (i, value) in value.into_iter().enumerate() {
            self.storage.bytes_mut()[at + i * 2..at + i * 2 + 2]
                .copy_from_slice(&value.to_le_bytes());
        }
    }
}
impl<S: WritableStorage, const M: u8, X: ExtraLayout> Vertices<S, Packed<M, Color, X>, One> {
    /// Replaces the native BGRA color bytes.
    pub fn set_color(&mut self, value: [u8; 4]) {
        self.storage.bytes_mut()[24..28].copy_from_slice(&value);
    }
}
impl<S: WritableStorage, const M: u8, C: ColorLayout> Vertices<S, Packed<M, C, ExtraRaw>, One> {
    /// Replaces the explicitly selected uninterpreted extra slot.
    pub fn set_extra_bytes(&mut self, value: [u8; 4]) {
        let at = Packed::<M, C, ExtraRaw>::STRIDE - 8;
        self.storage.bytes_mut()[at..at + 4].copy_from_slice(&value);
    }
}
/// Runtime-selected typed layout, retaining raw bytes when flags are ambiguous.
#[derive(Clone, Debug, PartialEq)]
pub enum DynamicVertices<S> {
    /// Uninterpreted bytes; no vertex boundaries or field meanings are inferred.
    Raw(Vertices<S, Raw<1>, Many>),
    /// Packed UV mask 0x00 with no color.
    NoColorUv00(Vertices<S, Packed<0, NoColor>, Many>),
    /// Packed UV mask 0x01 with no color.
    NoColorUv01(Vertices<S, Packed<1, NoColor>, Many>),
    /// Packed UV mask 0x02 with no color.
    NoColorUv02(Vertices<S, Packed<2, NoColor>, Many>),
    /// Packed UV mask 0x03 with no color.
    NoColorUv03(Vertices<S, Packed<3, NoColor>, Many>),
    /// Packed UV mask 0x04 with no color.
    NoColorUv04(Vertices<S, Packed<4, NoColor>, Many>),
    /// Packed UV mask 0x05 with no color.
    NoColorUv05(Vertices<S, Packed<5, NoColor>, Many>),
    /// Packed UV mask 0x06 with no color.
    NoColorUv06(Vertices<S, Packed<6, NoColor>, Many>),
    /// Packed UV mask 0x07 with no color.
    NoColorUv07(Vertices<S, Packed<7, NoColor>, Many>),
    /// Packed UV mask 0x08 with no color.
    NoColorUv08(Vertices<S, Packed<8, NoColor>, Many>),
    /// Packed UV mask 0x09 with no color.
    NoColorUv09(Vertices<S, Packed<9, NoColor>, Many>),
    /// Packed UV mask 0x0a with no color.
    NoColorUv10(Vertices<S, Packed<10, NoColor>, Many>),
    /// Packed UV mask 0x0b with no color.
    NoColorUv11(Vertices<S, Packed<11, NoColor>, Many>),
    /// Packed UV mask 0x0c with no color.
    NoColorUv12(Vertices<S, Packed<12, NoColor>, Many>),
    /// Packed UV mask 0x0d with no color.
    NoColorUv13(Vertices<S, Packed<13, NoColor>, Many>),
    /// Packed UV mask 0x0e with no color.
    NoColorUv14(Vertices<S, Packed<14, NoColor>, Many>),
    /// Packed UV mask 0x0f with no color.
    NoColorUv15(Vertices<S, Packed<15, NoColor>, Many>),
    /// Packed UV mask 0x10 with no color.
    NoColorUv16(Vertices<S, Packed<16, NoColor>, Many>),
    /// Packed UV mask 0x11 with no color.
    NoColorUv17(Vertices<S, Packed<17, NoColor>, Many>),
    /// Packed UV mask 0x12 with no color.
    NoColorUv18(Vertices<S, Packed<18, NoColor>, Many>),
    /// Packed UV mask 0x13 with no color.
    NoColorUv19(Vertices<S, Packed<19, NoColor>, Many>),
    /// Packed UV mask 0x14 with no color.
    NoColorUv20(Vertices<S, Packed<20, NoColor>, Many>),
    /// Packed UV mask 0x15 with no color.
    NoColorUv21(Vertices<S, Packed<21, NoColor>, Many>),
    /// Packed UV mask 0x16 with no color.
    NoColorUv22(Vertices<S, Packed<22, NoColor>, Many>),
    /// Packed UV mask 0x17 with no color.
    NoColorUv23(Vertices<S, Packed<23, NoColor>, Many>),
    /// Packed UV mask 0x18 with no color.
    NoColorUv24(Vertices<S, Packed<24, NoColor>, Many>),
    /// Packed UV mask 0x19 with no color.
    NoColorUv25(Vertices<S, Packed<25, NoColor>, Many>),
    /// Packed UV mask 0x1a with no color.
    NoColorUv26(Vertices<S, Packed<26, NoColor>, Many>),
    /// Packed UV mask 0x1b with no color.
    NoColorUv27(Vertices<S, Packed<27, NoColor>, Many>),
    /// Packed UV mask 0x1c with no color.
    NoColorUv28(Vertices<S, Packed<28, NoColor>, Many>),
    /// Packed UV mask 0x1d with no color.
    NoColorUv29(Vertices<S, Packed<29, NoColor>, Many>),
    /// Packed UV mask 0x1e with no color.
    NoColorUv30(Vertices<S, Packed<30, NoColor>, Many>),
    /// Packed UV mask 0x1f with no color.
    NoColorUv31(Vertices<S, Packed<31, NoColor>, Many>),
    /// Packed UV mask 0x00 with BGRA color.
    ColorUv00(Vertices<S, Packed<0, Color>, Many>),
    /// Packed UV mask 0x01 with BGRA color.
    ColorUv01(Vertices<S, Packed<1, Color>, Many>),
    /// Packed UV mask 0x02 with BGRA color.
    ColorUv02(Vertices<S, Packed<2, Color>, Many>),
    /// Packed UV mask 0x03 with BGRA color.
    ColorUv03(Vertices<S, Packed<3, Color>, Many>),
    /// Packed UV mask 0x04 with BGRA color.
    ColorUv04(Vertices<S, Packed<4, Color>, Many>),
    /// Packed UV mask 0x05 with BGRA color.
    ColorUv05(Vertices<S, Packed<5, Color>, Many>),
    /// Packed UV mask 0x06 with BGRA color.
    ColorUv06(Vertices<S, Packed<6, Color>, Many>),
    /// Packed UV mask 0x07 with BGRA color.
    ColorUv07(Vertices<S, Packed<7, Color>, Many>),
    /// Packed UV mask 0x08 with BGRA color.
    ColorUv08(Vertices<S, Packed<8, Color>, Many>),
    /// Packed UV mask 0x09 with BGRA color.
    ColorUv09(Vertices<S, Packed<9, Color>, Many>),
    /// Packed UV mask 0x0a with BGRA color.
    ColorUv10(Vertices<S, Packed<10, Color>, Many>),
    /// Packed UV mask 0x0b with BGRA color.
    ColorUv11(Vertices<S, Packed<11, Color>, Many>),
    /// Packed UV mask 0x0c with BGRA color.
    ColorUv12(Vertices<S, Packed<12, Color>, Many>),
    /// Packed UV mask 0x0d with BGRA color.
    ColorUv13(Vertices<S, Packed<13, Color>, Many>),
    /// Packed UV mask 0x0e with BGRA color.
    ColorUv14(Vertices<S, Packed<14, Color>, Many>),
    /// Packed UV mask 0x0f with BGRA color.
    ColorUv15(Vertices<S, Packed<15, Color>, Many>),
    /// Packed UV mask 0x10 with BGRA color.
    ColorUv16(Vertices<S, Packed<16, Color>, Many>),
    /// Packed UV mask 0x11 with BGRA color.
    ColorUv17(Vertices<S, Packed<17, Color>, Many>),
    /// Packed UV mask 0x12 with BGRA color.
    ColorUv18(Vertices<S, Packed<18, Color>, Many>),
    /// Packed UV mask 0x13 with BGRA color.
    ColorUv19(Vertices<S, Packed<19, Color>, Many>),
    /// Packed UV mask 0x14 with BGRA color.
    ColorUv20(Vertices<S, Packed<20, Color>, Many>),
    /// Packed UV mask 0x15 with BGRA color.
    ColorUv21(Vertices<S, Packed<21, Color>, Many>),
    /// Packed UV mask 0x16 with BGRA color.
    ColorUv22(Vertices<S, Packed<22, Color>, Many>),
    /// Packed UV mask 0x17 with BGRA color.
    ColorUv23(Vertices<S, Packed<23, Color>, Many>),
    /// Packed UV mask 0x18 with BGRA color.
    ColorUv24(Vertices<S, Packed<24, Color>, Many>),
    /// Packed UV mask 0x19 with BGRA color.
    ColorUv25(Vertices<S, Packed<25, Color>, Many>),
    /// Packed UV mask 0x1a with BGRA color.
    ColorUv26(Vertices<S, Packed<26, Color>, Many>),
    /// Packed UV mask 0x1b with BGRA color.
    ColorUv27(Vertices<S, Packed<27, Color>, Many>),
    /// Packed UV mask 0x1c with BGRA color.
    ColorUv28(Vertices<S, Packed<28, Color>, Many>),
    /// Packed UV mask 0x1d with BGRA color.
    ColorUv29(Vertices<S, Packed<29, Color>, Many>),
    /// Packed UV mask 0x1e with BGRA color.
    ColorUv30(Vertices<S, Packed<30, Color>, Many>),
    /// Packed UV mask 0x1f with BGRA color.
    ColorUv31(Vertices<S, Packed<31, Color>, Many>),
}
impl<S: AsRef<[u8]>> DynamicVertices<S> {
    /// Selects known packed layouts, or preserves an ambiguous/unknown blob as Raw.
    ///
    /// Unknown flags never select a guessed stride. The original flag word
    /// remains in the owning MODL record rather than being rewritten by this view.
    pub fn from_flags(storage: S, flags: VertexFlags) -> Result<Self, ReadError> {
        let flags = flags.bits();
        const REQUIRED: u32 = 0x0180_0061;
        const OPTIONAL: u32 = 0x1c | 0x200 | 0x001e_0000 | 0x4000_0000;
        if flags & REQUIRED != REQUIRED || flags & !(REQUIRED | OPTIONAL) != 0 {
            return Ok(Self::Raw(Vertices::new(storage)?));
        }
        let mask = ((flags >> 17) & 15) as u8 | (((flags >> 30) & 1) as u8) << 4;
        Ok(match (flags & 0x200 != 0, mask) {
            (false, 0) => Self::NoColorUv00(Vertices::new(storage)?),
            (false, 1) => Self::NoColorUv01(Vertices::new(storage)?),
            (false, 2) => Self::NoColorUv02(Vertices::new(storage)?),
            (false, 3) => Self::NoColorUv03(Vertices::new(storage)?),
            (false, 4) => Self::NoColorUv04(Vertices::new(storage)?),
            (false, 5) => Self::NoColorUv05(Vertices::new(storage)?),
            (false, 6) => Self::NoColorUv06(Vertices::new(storage)?),
            (false, 7) => Self::NoColorUv07(Vertices::new(storage)?),
            (false, 8) => Self::NoColorUv08(Vertices::new(storage)?),
            (false, 9) => Self::NoColorUv09(Vertices::new(storage)?),
            (false, 10) => Self::NoColorUv10(Vertices::new(storage)?),
            (false, 11) => Self::NoColorUv11(Vertices::new(storage)?),
            (false, 12) => Self::NoColorUv12(Vertices::new(storage)?),
            (false, 13) => Self::NoColorUv13(Vertices::new(storage)?),
            (false, 14) => Self::NoColorUv14(Vertices::new(storage)?),
            (false, 15) => Self::NoColorUv15(Vertices::new(storage)?),
            (false, 16) => Self::NoColorUv16(Vertices::new(storage)?),
            (false, 17) => Self::NoColorUv17(Vertices::new(storage)?),
            (false, 18) => Self::NoColorUv18(Vertices::new(storage)?),
            (false, 19) => Self::NoColorUv19(Vertices::new(storage)?),
            (false, 20) => Self::NoColorUv20(Vertices::new(storage)?),
            (false, 21) => Self::NoColorUv21(Vertices::new(storage)?),
            (false, 22) => Self::NoColorUv22(Vertices::new(storage)?),
            (false, 23) => Self::NoColorUv23(Vertices::new(storage)?),
            (false, 24) => Self::NoColorUv24(Vertices::new(storage)?),
            (false, 25) => Self::NoColorUv25(Vertices::new(storage)?),
            (false, 26) => Self::NoColorUv26(Vertices::new(storage)?),
            (false, 27) => Self::NoColorUv27(Vertices::new(storage)?),
            (false, 28) => Self::NoColorUv28(Vertices::new(storage)?),
            (false, 29) => Self::NoColorUv29(Vertices::new(storage)?),
            (false, 30) => Self::NoColorUv30(Vertices::new(storage)?),
            (false, 31) => Self::NoColorUv31(Vertices::new(storage)?),
            (true, 0) => Self::ColorUv00(Vertices::new(storage)?),
            (true, 1) => Self::ColorUv01(Vertices::new(storage)?),
            (true, 2) => Self::ColorUv02(Vertices::new(storage)?),
            (true, 3) => Self::ColorUv03(Vertices::new(storage)?),
            (true, 4) => Self::ColorUv04(Vertices::new(storage)?),
            (true, 5) => Self::ColorUv05(Vertices::new(storage)?),
            (true, 6) => Self::ColorUv06(Vertices::new(storage)?),
            (true, 7) => Self::ColorUv07(Vertices::new(storage)?),
            (true, 8) => Self::ColorUv08(Vertices::new(storage)?),
            (true, 9) => Self::ColorUv09(Vertices::new(storage)?),
            (true, 10) => Self::ColorUv10(Vertices::new(storage)?),
            (true, 11) => Self::ColorUv11(Vertices::new(storage)?),
            (true, 12) => Self::ColorUv12(Vertices::new(storage)?),
            (true, 13) => Self::ColorUv13(Vertices::new(storage)?),
            (true, 14) => Self::ColorUv14(Vertices::new(storage)?),
            (true, 15) => Self::ColorUv15(Vertices::new(storage)?),
            (true, 16) => Self::ColorUv16(Vertices::new(storage)?),
            (true, 17) => Self::ColorUv17(Vertices::new(storage)?),
            (true, 18) => Self::ColorUv18(Vertices::new(storage)?),
            (true, 19) => Self::ColorUv19(Vertices::new(storage)?),
            (true, 20) => Self::ColorUv20(Vertices::new(storage)?),
            (true, 21) => Self::ColorUv21(Vertices::new(storage)?),
            (true, 22) => Self::ColorUv22(Vertices::new(storage)?),
            (true, 23) => Self::ColorUv23(Vertices::new(storage)?),
            (true, 24) => Self::ColorUv24(Vertices::new(storage)?),
            (true, 25) => Self::ColorUv25(Vertices::new(storage)?),
            (true, 26) => Self::ColorUv26(Vertices::new(storage)?),
            (true, 27) => Self::ColorUv27(Vertices::new(storage)?),
            (true, 28) => Self::ColorUv28(Vertices::new(storage)?),
            (true, 29) => Self::ColorUv29(Vertices::new(storage)?),
            (true, 30) => Self::ColorUv30(Vertices::new(storage)?),
            (true, 31) => Self::ColorUv31(Vertices::new(storage)?),
            _ => unreachable!("masked UV channels"),
        })
    }
    /// Exact native bytes, independent of selected byte interpretation.
    pub fn as_bytes(&self) -> &[u8] {
        visit_m3_vertices!(self, |vertices| vertices.as_bytes())
    }
    /// Known vertex count, or None when the layout remains raw.
    pub fn vertex_count(&self) -> Option<usize> {
        if matches!(self, Self::Raw(_)) {
            None
        } else {
            Some(visit_m3_vertices!(self, |vertices| vertices.len()))
        }
    }
}
/// Dispatches over the concrete storage-preserving packed layout or Raw variant.
#[macro_export]
macro_rules! visit_m3_vertices {
    ($vertices:expr,|$typed:ident|$body:expr) => {
        match $vertices {
            $crate::model::m3::vertices::DynamicVertices::Raw($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv00($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv01($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv02($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv03($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv04($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv05($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv06($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv07($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv08($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv09($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv10($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv11($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv12($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv13($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv14($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv15($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv16($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv17($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv18($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv19($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv20($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv21($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv22($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv23($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv24($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv25($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv26($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv27($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv28($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv29($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv30($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::NoColorUv31($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv00($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv01($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv02($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv03($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv04($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv05($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv06($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv07($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv08($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv09($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv10($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv11($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv12($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv13($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv14($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv15($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv16($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv17($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv18($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv19($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv20($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv21($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv22($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv23($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv24($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv25($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv26($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv27($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv28($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv29($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv30($typed) => $body,
            $crate::model::m3::vertices::DynamicVertices::ColorUv31($typed) => $body,
        }
    };
}
