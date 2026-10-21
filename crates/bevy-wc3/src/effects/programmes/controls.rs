//! Programme-scoped attribute bindings and instance-local application overrides.
use bevy::prelude::Component;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use wc3::model::scene::popcorn::{EffectAttributeId, EffectProgram, Value, ValueType};

/// A resolved attribute of one immutable compiled programme.
/// Clones work across actors using that programme; recompilation creates a
/// different domain even when the declarations and numeric indices are equal.
#[derive(Clone)]
pub struct ParticleProgramAttribute {
    program: EffectProgram,
    id: EffectAttributeId,
}
impl Debug for ParticleProgramAttribute {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ParticleProgramAttribute")
            .field("id", &self.id)
            .field("kind", &self.kind())
            .finish()
    }
}
impl ParticleProgramAttribute {
    /// Exact declared layout, without numeric coercion.
    pub fn kind(&self) -> ValueType {
        self.program.attributes()[self.id.0 as usize].default.kind()
    }
}

/// Invalid application control input, separate from programme execution faults.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParticleProgramControlError {
    /// No declaration has this exact exposed name.
    UnknownAttribute(String),
    /// An attribute or moved controls component belongs to another programme.
    WrongProgram,
    /// The supplied value has a different declared layout.
    TypeMismatch {
        /// Required declaration layout.
        expected: ValueType,
        /// Supplied value layout.
        actual: ValueType,
    },
}
impl Display for ParticleProgramControlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::UnknownAttribute(name) => write!(f, "unknown particle attribute {name:?}"),
            Self::WrongProgram => f.write_str("particle controls belong to another programme"),
            Self::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "particle attribute requires {expected:?}, received {actual:?}"
                )
            }
        }
    }
}
impl Error for ParticleProgramControlError {}

/// Live overrides for one external programme instance.
///
/// Created beside `ParticleProgramEmitter`. Live values override prepared
/// values, sampled model bindings and declaration defaults. Removing a value
/// reveals the current underlying value without resetting living particles.
/// Write during `Update`, or before both effect simulation sets in `PostUpdate`.
#[derive(Component)]
pub struct ParticleProgramControls {
    program: EffectProgram,
    values: Vec<(EffectAttributeId, Value)>,
}
impl Debug for ParticleProgramControls {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("ParticleProgramControls")
            .field("attribute_count", &self.program.attributes().len())
            .field("overrides", &self.values)
            .finish()
    }
}
impl Clone for ParticleProgramControls {
    fn clone(&self) -> Self {
        let mut values = Vec::with_capacity(self.program.attributes().len());
        values.extend_from_slice(&self.values);
        Self {
            program: self.program.clone(),
            values,
        }
    }
}
impl ParticleProgramControls {
    pub(super) fn new(program: &EffectProgram) -> Self {
        Self {
            program: program.clone(),
            values: Vec::with_capacity(program.attributes().len()),
        }
    }
    /// Resolve an exposed attribute once, retaining programme ownership.
    pub fn attribute(
        &self,
        name: &str,
    ) -> Result<ParticleProgramAttribute, ParticleProgramControlError> {
        let index = self
            .program
            .attributes()
            .iter()
            .position(|attribute| attribute.name.as_ref() == name.as_bytes())
            .ok_or_else(|| ParticleProgramControlError::UnknownAttribute(name.to_owned()))?;
        Ok(ParticleProgramAttribute {
            program: self.program.clone(),
            id: EffectAttributeId(index as u32),
        })
    }
    fn validate(
        &self,
        attribute: &ParticleProgramAttribute,
    ) -> Result<(), ParticleProgramControlError> {
        if self.program.same_program(&attribute.program) {
            Ok(())
        } else {
            Err(ParticleProgramControlError::WrongProgram)
        }
    }
    /// Set this instance's live value. Equal values leave the table unchanged.
    /// Layout is checked exactly; floating-point domain checks belong to the programme.
    pub fn set(
        &mut self,
        attribute: &ParticleProgramAttribute,
        value: impl Into<Value>,
    ) -> Result<(), ParticleProgramControlError> {
        self.validate(attribute)?;
        let value = value.into();
        if value.kind() != attribute.kind() {
            return Err(ParticleProgramControlError::TypeMismatch {
                expected: attribute.kind(),
                actual: value.kind(),
            });
        }
        match self
            .values
            .binary_search_by_key(&attribute.id.0, |(id, _)| id.0)
        {
            Ok(index) => {
                if self.values[index].1 != value {
                    self.values[index].1 = value;
                }
            }
            Err(index) => self.values.insert(index, (attribute.id, value)),
        }
        Ok(())
    }
    /// Remove an override, revealing the current prepared/model/default baseline.
    /// Returns whether an override was present; foreign handles are errors.
    pub fn remove(
        &mut self,
        attribute: &ParticleProgramAttribute,
    ) -> Result<bool, ParticleProgramControlError> {
        self.validate(attribute)?;
        Ok(
            match self
                .values
                .binary_search_by_key(&attribute.id.0, |(id, _)| id.0)
            {
                Ok(index) => {
                    self.values.remove(index);
                    true
                }
                Err(_) => false,
            },
        )
    }
    /// Remove all explicit values without restarting the simulation.
    pub fn clear(&mut self) {
        self.values.clear();
    }
    /// Read only the explicit live override, not the sampled effective value.
    pub fn get(
        &self,
        attribute: &ParticleProgramAttribute,
    ) -> Result<Option<Value>, ParticleProgramControlError> {
        self.validate(attribute)?;
        Ok(self
            .values
            .binary_search_by_key(&attribute.id.0, |(id, _)| id.0)
            .ok()
            .map(|index| self.values[index].1))
    }
    pub(super) fn matches(&self, program: &EffectProgram) -> bool {
        self.program.same_program(program)
    }
    pub(super) fn contains(&self, id: EffectAttributeId) -> bool {
        self.values
            .binary_search_by_key(&id.0, |(key, _)| key.0)
            .is_ok()
    }
    pub(super) fn values(&self) -> &[(EffectAttributeId, Value)] {
        &self.values
    }
}
