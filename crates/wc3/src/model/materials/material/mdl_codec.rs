//! Material directives, including historical flags and version-selected shaders.
use super::{Material, MaterialRenderFlags, NoShader, ShaderText};
use crate::model::materials::Layer;
use crate::model::mdl;
use crate::model::mdl::{
    Dialect, Field, Parser, ReadErrorKind, ReadProperty, Span, WriteProperty, Writer,
};
use crate::model::{FixedText, ModelVersion};
use std::io::{sink, Write as IoWrite};
use std::marker::PhantomData;

impl ReadProperty for NoShader {
    fn read_mdl_property(_: &mut Parser<'_>, field: Field<'_>) -> Result<Self, mdl::ReadError> {
        Err(mdl::ReadError::new(
            field.span,
            ReadErrorKind::UnsupportedField,
        ))
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self)
    }
}
impl WriteProperty for NoShader {
    fn write_mdl_property<W: IoWrite>(
        &self,
        _: &'static str,
        _: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        Ok(())
    }
}
impl ReadProperty for ShaderText {
    fn read_mdl_property(parser: &mut Parser<'_>, _: Field<'_>) -> Result<Self, mdl::ReadError> {
        parser.read_property().map(Self)
    }
    fn missing_mdl_property(_: &'static str, _: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self::default())
    }
}
impl WriteProperty for ShaderText {
    fn validate_mdl_property(&self, _: &'static str, _: Dialect) -> Result<(), mdl::WriteError> {
        Writer::new(sink()).write(&self.0)
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut Writer<W>,
    ) -> Result<(), mdl::WriteError> {
        self.validate_mdl_property(name, writer.dialect())?;
        if self.0 != FixedText::default() {
            writer.property(name, &self.0)?;
        }
        Ok(())
    }
}

fn zero_priority(value: &i32) -> bool {
    *value == 0
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(
    block = "Material",
    after_read = "Self::finish",
    validate_write = "Self::validate"
)]
struct MaterialMdl<V: ModelVersion> {
    #[mdl(skip, default)]
    version: PhantomData<V>,
    #[mdl(property = "PriorityPlane", default, skip_if = "zero_priority")]
    priority: i32,
    #[mdl(flags(
        ConstantColor = 1,
        TwoSided = 2,
        SortPrimsNearZ = 8,
        SortPrimsFarZ = 16,
        FullResolution = 32
    ))]
    #[mdl(hive_flags(SortPrimitives = 16))]
    flags: u32,
    #[mdl(flag = "Unfogged", default)]
    unfogged: bool,
    #[mdl(property = "Shader", delegate)]
    shader: V::Shader,
    #[mdl(repeated = "Layer")]
    layers: Vec<Layer<V>>,
}
impl<V: ModelVersion> MaterialMdl<V> {
    fn finish(&mut self, _: Span) -> Result<(), mdl::ReadError> {
        self.unfogged = false;
        if self.flags & 2 != 0 {
            for layer in &mut self.layers {
                let mut flags = layer.shading_flags;
                flags.set_two_sided(true);
                layer.shading_flags = flags;
            }
        }
        Ok(())
    }
    fn validate(&self) -> Result<(), mdl::WriteError> {
        if self.flags & 2 != 0
            && self
                .layers
                .iter()
                .any(|layer| !layer.shading_flags.two_sided())
        {
            return Err(mdl::WriteError::Unsupported(
                "material TwoSided with one-sided layers",
            ));
        }
        Ok(())
    }
}
impl<V: ModelVersion> mdl::Read for Material<V> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<MaterialMdl<V>>()?;
        Ok(Self {
            version: PhantomData,
            priority_plane: value.priority,
            render_mode: MaterialRenderFlags(value.flags),
            shader: value.shader,
            layers: value.layers,
        })
    }
}
impl<V: ModelVersion> mdl::Write for Material<V> {
    fn write_mdl<W: IoWrite>(&self, writer: &mut Writer<W>) -> Result<(), mdl::WriteError> {
        writer.write(&MaterialMdl::<V> {
            version: PhantomData,
            priority: self.priority_plane,
            flags: self.render_mode.bits(),
            unfogged: false,
            shader: self.shader.clone(),
            layers: self.layers.clone(),
        })
    }
}
