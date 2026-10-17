//! Raw selector fixtures for exercising public material interpretation.
pub mod m2 {
    use std::borrow::Cow;
    pub use wc3::model::materials::m2::*;
    use wc3::model::materials::state::PassState;
    use wc3::model::values::model::{BatchFlags, BlendMode};
    use wc3::model::{Animated, Error, MaterialSurface};
    #[derive(Clone, Debug, wc3::model::Construct)]
    pub struct MaterialFixture<'a> {
        /// Shader family identifier interpreted by the game shader implementation.
        pub shader: u16,
        /// Established draw behavior and retained extension bits.
        pub batch_flags: BatchFlags,
        /// Color composition, rasterization, and authored pass ordering.
        pub state: PassState<BlendMode, ModelRasterization, i8>,
        /// Animated surface color multiplier.
        pub color: Animated<[f32; 3]>,
        /// Animated surface opacity multiplier.
        pub opacity: Animated<f32>,
        /// External colours used by legacy guild shader programmes.
        pub guild_palette: GuildPalette,
        /// Ordered shader texture inputs.
        pub textures: Vec<TextureUnit>,
        /// Optional user-authored shader label.
        pub label: Cow<'a, str>,
    }

    pub type Material<'a> = MaterialFixture<'a>;
    impl<'a> From<MaterialFixture<'a>> for MaterialStep<'a> {
        fn from(v: MaterialFixture<'a>) -> Self {
            MaterialStep::from_native(
                v.shader,
                MaterialDeclaration {
                    batch_flags: v.batch_flags,
                    state: v.state,
                    color: v.color,
                    opacity: v.opacity,
                    label: v.label,
                },
                v.textures,
                v.guild_palette,
            )
        }
    }
    impl<'a> From<MaterialFixture<'a>> for MaterialSurface<'a> {
        fn from(v: MaterialFixture<'a>) -> Self {
            MaterialStep::from(v).into()
        }
    }
    impl<'a> TryFrom<MaterialSurface<'a>> for MaterialFixture<'a> {
        type Error = Error;
        fn try_from(v: MaterialSurface<'a>) -> Result<Self, Self::Error> {
            let v = MaterialStep::try_from(v)?;
            let h = v.declaration().clone();
            Ok(Self {
                shader: v.native_shader(),
                textures: v.inputs().into_iter().cloned().collect(),
                guild_palette: v.guild_palette().unwrap_or_default(),

                batch_flags: h.batch_flags,
                state: h.state,
                color: h.color,
                opacity: h.opacity,
                label: h.label,
            })
        }
    }
}
pub mod wmo {
    use std::borrow::Cow;
    use wc3::model::materials::bindings::TextureBinding;
    use wc3::model::materials::state::PassState;
    pub use wc3::model::materials::wmo::*;
    use wc3::model::{Error, MaterialSurface, Texture};
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct MaterialFixture<'a> {
        /// Optional authoring name.
        pub name: Cow<'a, str>,
        /// External client shader-table selector, absent for fixed-function surfaces.
        pub shader: Option<u32>,
        /// Color composition and shared rasterization controls.
        pub state: PassState<Blend, MaterialRasterization>,
        /// Texture bindings in shader-slot order.
        pub textures: Vec<TextureBinding<Option<Texture>>>,
        /// Self-illumination color in RGBA order.
        pub self_illumination: [u8; 4],
        /// Serialized runtime self-illumination color in RGBA order.
        pub runtime_self_illumination: [u8; 4],
        /// Diffuse color, also used for interior liquid shading.
        pub diffuse: [u8; 4],
        /// External terrain-type identifier.
        pub ground_type: u32,
        /// UV scroll velocity for the first two texture slots.
        pub uv_scroll: [[f32; 2]; 2],
        /// Optional extended shader overrides.
        pub shader_overrides: Option<ShaderOverrides>,
        /// Historical additional material color in RGBA order.
        pub legacy_color: Option<[u8; 4]>,
        /// Historical additional material controls with unresolved meanings.
        pub legacy_controls: Option<u32>,
        /// Historical runtime control values with unresolved meanings.
        pub legacy_runtime: Option<[u32; 4]>,
    }

    pub type Material<'a> = MaterialFixture<'a>;
    impl<'a> From<MaterialFixture<'a>> for MaterialStep<'a> {
        fn from(v: MaterialFixture<'a>) -> Self {
            MaterialStep::from_native(
                v.shader,
                MaterialDeclaration {
                    name: v.name,
                    state: v.state,
                    self_illumination: v.self_illumination,
                    runtime_self_illumination: v.runtime_self_illumination,
                    diffuse: v.diffuse,
                    ground_type: v.ground_type,
                    uv_scroll: v.uv_scroll,
                    shader_overrides: v.shader_overrides,
                    legacy_color: v.legacy_color,
                    legacy_controls: v.legacy_controls,
                    legacy_runtime: v.legacy_runtime,
                },
                v.textures
                    .into_iter()
                    .map(|binding| TextureBinding {
                        image: binding.image.map(|image| image.resource),
                        sampling: binding.image.map(|image| image.sampler).unwrap_or_default(),
                        ..Default::default()
                    })
                    .collect(),
            )
        }
    }
    impl<'a> From<MaterialFixture<'a>> for MaterialSurface<'a> {
        fn from(v: MaterialFixture<'a>) -> Self {
            MaterialStep::from(v).into()
        }
    }
    impl<'a> TryFrom<MaterialSurface<'a>> for MaterialFixture<'a> {
        type Error = Error;
        fn try_from(v: MaterialSurface<'a>) -> Result<Self, Self::Error> {
            let v = MaterialStep::try_from(v)?;
            let h = v.declaration().clone();
            Ok(Self {
                shader: v.native_shader(),
                textures: v
                    .inputs()
                    .into_iter()
                    .map(|binding| TextureBinding {
                        image: binding.image.map(|resource| {
                            let mut image = Texture::new(resource);
                            image.sampler = binding.sampling;
                            image
                        }),
                        ..Default::default()
                    })
                    .collect(),
                name: h.name,
                state: h.state,
                self_illumination: h.self_illumination,
                runtime_self_illumination: h.runtime_self_illumination,
                diffuse: h.diffuse,
                ground_type: h.ground_type,
                uv_scroll: h.uv_scroll,
                shader_overrides: h.shader_overrides,
                legacy_color: h.legacy_color,
                legacy_controls: h.legacy_controls,
                legacy_runtime: h.legacy_runtime,
            })
        }
    }
}
