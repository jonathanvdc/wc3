//! Light records in `LITE` chunks.
use crate::ModelVersion;
use std::fmt::Debug;
crate::animation::track_group! {
    pub enum LightTrack {
        AttenuationStart: LightAttenuationStart,
        AttenuationEnd: LightAttenuationEnd,
        Color: LightColor,
        Intensity: LightIntensity,
        AmbientColor: LightAmbientColor,
        AmbientIntensity: LightAmbientIntensity,
        Visibility: LightVisibility,
    }
}

use crate::Color;
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;
use std::marker::PhantomData;

use crate::{Cursor, LightsChunk};
use crate::{DecodeError, Model, Node};
use crate::{Readable, Writable};

/// Extra fixed words selected by the light record's version.
pub trait LightExtension: Default + Readable + Writable + Clone + Debug + PartialEq {
    fn words(&self) -> Option<[u32; 7]> {
        None
    }
    fn words_mut(&mut self) -> Option<&mut [u32; 7]> {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassicLightExtension;

impl Default for ClassicLightExtension {
    fn default() -> Self {
        Self
    }
}

impl Readable for ClassicLightExtension {
    fn read_from(_: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self)
    }
}

impl Writable for ClassicLightExtension {
    fn write_to(&self, _: &mut Encoder<'_>) -> Result<(), EncodeError> {
        Ok(())
    }
}

impl LightExtension for ClassicLightExtension {}

#[derive(Clone, Debug, PartialEq)]
pub struct ModernLightExtension([u32; 7]);

impl Default for ModernLightExtension {
    fn default() -> Self {
        Self([0; 7])
    }
}

impl Readable for ModernLightExtension {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self(cursor.read()?))
    }
}

impl Writable for ModernLightExtension {
    fn write_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write(&self.0)
    }
}

impl LightExtension for ModernLightExtension {
    fn words(&self) -> Option<[u32; 7]> {
        Some(self.0)
    }
    fn words_mut(&mut self) -> Option<&mut [u32; 7]> {
        Some(&mut self.0)
    }
}

pub trait LightLayout {
    type Extension: LightExtension;
}

use crate::{V1000, V1100, V1200, V1800, V800, V900};
impl LightLayout for V800 {
    type Extension = ClassicLightExtension;
}
impl LightLayout for V900 {
    type Extension = ClassicLightExtension;
}
impl LightLayout for V1000 {
    type Extension = ClassicLightExtension;
}
impl LightLayout for V1100 {
    type Extension = ClassicLightExtension;
}
impl LightLayout for V1200 {
    type Extension = ModernLightExtension;
}
impl LightLayout for V1800 {
    type Extension = ModernLightExtension;
}

/// A light node with decoded lighting values and animation tracks.
#[derive(Clone, Debug, PartialEq, Readable, Writable)]
#[mdx(sized(tag = LightsChunk::<V>::TAG))]
pub struct Light<V: ModelVersion> {
    node: Node,
    light_type: u32,
    attenuation_start: f32,
    attenuation_end: f32,
    color: Color,
    intensity: f32,
    ambient_color: Color,
    ambient_intensity: f32,
    extended_words: V::Extension,
    version: PhantomData<V>,
    tracks: Vec<LightTrack>,
}

impl<V: ModelVersion> Light<V> {
    /// Creates a light with zeroed lighting values.
    pub fn new(node: Node, light_type: u32) -> Self {
        Self {
            node,
            light_type,
            attenuation_start: 0.0,
            attenuation_end: 0.0,
            color: [0.0; 3],
            intensity: 0.0,
            ambient_color: [0.0; 3],
            ambient_intensity: 0.0,
            extended_words: V::Extension::default(),
            tracks: Vec::new(),
            version: PhantomData,
        }
    }

    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows the shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Returns raw light type ID.
    pub fn light_type(&self) -> u32 {
        self.light_type
    }
    /// Sets raw light type ID.
    pub fn set_light_type(&mut self, kind: u32) {
        self.light_type = kind;
    }
    /// Returns attenuation start distance.
    pub fn attenuation_start(&self) -> f32 {
        self.attenuation_start
    }
    /// Sets attenuation start distance.
    pub fn set_attenuation_start(&mut self, value: f32) {
        self.attenuation_start = value;
    }
    /// Returns attenuation end distance.
    pub fn attenuation_end(&self) -> f32 {
        self.attenuation_end
    }
    /// Sets attenuation end distance.
    pub fn set_attenuation_end(&mut self, value: f32) {
        self.attenuation_end = value;
    }
    /// Returns RGB light color.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Sets RGB light color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// Returns light intensity.
    pub fn intensity(&self) -> f32 {
        self.intensity
    }
    /// Sets light intensity.
    pub fn set_intensity(&mut self, value: f32) {
        self.intensity = value;
    }
    /// Returns ambient RGB color.
    pub fn ambient_color(&self) -> Color {
        self.ambient_color
    }
    /// Sets ambient RGB color.
    pub fn set_ambient_color(&mut self, color: Color) {
        self.ambient_color = color;
    }
    /// Returns ambient intensity.
    pub fn ambient_intensity(&self) -> f32 {
        self.ambient_intensity
    }
    /// Sets ambient intensity.
    pub fn set_ambient_intensity(&mut self, value: f32) {
        self.ambient_intensity = value;
    }
    /// Returns the additional seven raw words in newer light records, when present.
    pub fn extended_words(&self) -> Option<[u32; 7]> {
        self.extended_words.words()
    }
    /// Sets the additional seven raw words in a newer light record.
    pub fn set_extended_words(&mut self, words: [u32; 7]) -> Result<(), ValueError> {
        *self
            .extended_words
            .words_mut()
            .ok_or(ValueError::UnavailableField {
                tag: LightsChunk::<V>::TAG,
                field: "extended words",
            })? = words;
        Ok(())
    }
    /// Borrows decoded light animation tracks.
    pub fn tracks(&self) -> &[LightTrack] {
        &self.tracks
    }
    /// Replaces optional light animation tracks.
    pub fn set_tracks(&mut self, tracks: &[LightTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `LITE` records in file order.
    pub fn lights(&self) -> Vec<Light<V>> {
        self.collect_chunk_records::<LightsChunk<V>>()
    }

    /// Replaces lights in the first `LITE` chunk.
    pub fn set_lights(&mut self, lights: &[Light<V>]) {
        self.replace_chunk(LightsChunk::new(lights.to_vec()));
    }
}
