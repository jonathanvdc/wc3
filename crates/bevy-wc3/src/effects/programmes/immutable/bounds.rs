//! Conservative birth admission over the complete finite appearance domain.
use super::super::prepared::{ProgrammeRenderer, RendererGeometry};
use super::super::MODEL_UNITS_PER_EFFECT_UNIT;
use bevy::math::Mat4;
use wc3::model::scene::popcorn::{
    EffectFieldId, ImmutableExpression, ImmutableNodeId, ImmutableParticleView, ImmutableRecipe,
    ValueType,
};

#[derive(Clone, Copy, Default)]
struct Range {
    lo: f32,
    hi: f32,
}
impl Range {
    fn point(value: f32) -> Option<Self> {
        value.is_finite().then_some(Self {
            lo: value,
            hi: value,
        })
    }
    fn add(self, other: Self) -> Option<Self> {
        Self::rounded(
            f64::from(self.lo) + f64::from(other.lo),
            f64::from(self.hi) + f64::from(other.hi),
        )
    }
    fn sub(self, other: Self) -> Option<Self> {
        Self::rounded(
            f64::from(self.lo) - f64::from(other.hi),
            f64::from(self.hi) - f64::from(other.lo),
        )
    }
    fn mul(self, other: Self) -> Option<Self> {
        let values = [
            f64::from(self.lo) * f64::from(other.lo),
            f64::from(self.lo) * f64::from(other.hi),
            f64::from(self.hi) * f64::from(other.lo),
            f64::from(self.hi) * f64::from(other.hi),
        ];
        Self::rounded(
            values.into_iter().fold(f64::INFINITY, f64::min),
            values.into_iter().fold(f64::NEG_INFINITY, f64::max),
        )
    }
    fn rounded(lo: f64, hi: f64) -> Option<Self> {
        // Outward rounding also covers an f64 intermediate rounding before
        // conversion. Exact authored/capture points bypass this operation.
        let result = Self {
            lo: (lo as f32).next_down(),
            hi: (hi as f32).next_up(),
        };
        (result.lo.is_finite() && result.hi.is_finite()).then_some(result)
    }
    fn union(self, other: Self) -> Self {
        Self {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }
}

pub(super) fn stationary(recipe: ImmutableRecipe<'_>, field: EffectFieldId) -> bool {
    let mut fixed = [false; 128];
    if recipe.node_count() > fixed.len() {
        return false;
    }
    for i in 0..recipe.node_count() {
        let Some(node) = recipe.node(i) else {
            return false;
        };
        fixed[i] = match node.expression {
            ImmutableExpression::Constant(_) | ImmutableExpression::Capture(_) => true,
            ImmutableExpression::Time | ImmutableExpression::Life => false,
            ImmutableExpression::Words { input, .. } | ImmutableExpression::Curve { input, .. } => {
                fixed[input.index()]
            }
            ImmutableExpression::Add { left, right }
            | ImmutableExpression::Subtract { left, right }
            | ImmutableExpression::Multiply { left, right } => {
                fixed[left.index()] && fixed[right.index()]
            }
        };
    }
    (0..recipe.output_count())
        .filter_map(|i| recipe.output(i))
        .find(|output| output.field == field)
        .is_none_or(|output| fixed[output.value.index()])
}
fn input(
    recipe: ImmutableRecipe<'_>,
    values: &[[Range; 4]; 128],
    id: ImmutableNodeId,
    component: usize,
) -> Option<Range> {
    let width = recipe.node(id.index())?.kind.components();
    Some(values[id.index()][component.min(width - 1)])
}

/// Every arithmetic result must stay finite; size must remain strictly positive.
/// This is conservative admission, not a new runtime failure for legal CPU values.
pub(super) fn admit(
    particle: ImmutableParticleView<'_>,
    renderer: &ProgrammeRenderer,
    world_from_space: Mat4,
    time_end: f32,
) -> Option<()> {
    let recipe = particle.recipe();
    let mut values = [[Range::default(); 4]; 128];
    if recipe.node_count() > values.len() {
        return None;
    }
    for i in 0..recipe.node_count() {
        let node = recipe.node(i)?;
        if matches!(node.kind, ValueType::Bool | ValueType::Bool3) {
            continue;
        }
        let mut result = [Range::default(); 4];
        for (c, result) in result.iter_mut().enumerate().take(node.kind.components()) {
            *result = match node.expression {
                ImmutableExpression::Constant(value) => {
                    Range::point(f32::from_bits(value.words()[c]))?
                }
                ImmutableExpression::Capture(index) => {
                    Range::point(f32::from_bits(particle.capture(index)?.words()[c]))?
                }
                ImmutableExpression::Life => Range { lo: 0., hi: 1. },
                ImmutableExpression::Time => Range {
                    lo: 0.,
                    hi: time_end,
                },
                ImmutableExpression::Words { input, .. } => {
                    if matches!(
                        recipe.node(input.index())?.kind,
                        ValueType::Bool | ValueType::Bool3
                    ) {
                        return None;
                    }
                    values[input.index()][c]
                }
                ImmutableExpression::Add { left, right } => {
                    input(recipe, &values, left, c)?.add(input(recipe, &values, right, c)?)?
                }
                ImmutableExpression::Subtract { left, right } => {
                    input(recipe, &values, left, c)?.sub(input(recipe, &values, right, c)?)?
                }
                ImmutableExpression::Multiply { left, right } => {
                    input(recipe, &values, left, c)?.mul(input(recipe, &values, right, c)?)?
                }
                ImmutableExpression::Curve { sampler, .. } => {
                    let curve = recipe.curve(sampler)?;
                    let mut range = Range::point(curve.values.first()?[c])?;
                    for value in curve.values {
                        range = range.union(Range::point(value[c])?);
                    }
                    if curve.tangents.is_empty() {
                        range = range
                            .union(range.add(range.sub(range)?.mul(Range { lo: 0., hi: 1. })?)?);
                    } else {
                        let a = range;
                        let mut m = Range::point(curve.tangents.first()?[c])?;
                        for value in curve.tangents {
                            m = m.union(Range::point(value[c])?);
                        }
                        // Interval extensions of the exact four Hermite basis terms.
                        let h0 = Range { lo: -2., hi: 3. };
                        let h1 = Range { lo: -2., hi: 2. };
                        let h2 = Range { lo: -2., hi: 3. };
                        let h3 = Range { lo: -1., hi: 1. };
                        range = range.union(
                            h0.mul(a)?
                                .add(h1.mul(m)?)?
                                .add(h2.mul(a)?)?
                                .add(h3.mul(m)?)?,
                        );
                    }
                    range
                }
            };
        }
        values[i] = result;
    }
    if let Some(output) = (0..recipe.output_count())
        .filter_map(|i| recipe.output(i))
        .find(|out| out.field == renderer.size)
    {
        let range = values[output.value.index()][0];
        if range.lo <= 0. {
            return None;
        }
        range
            .mul(Range::point(MODEL_UNITS_PER_EFFECT_UNIT)?)?
            .mul(Range::point(2.)?)?;
    }
    // Bound transformed geometry too: finite authored values can overflow when
    // converted into model units or projected through an affine basis.
    let rows = world_from_space.transpose().to_cols_array_2d();
    let longitudinal = matches!(renderer.geometry, RendererGeometry::Billboard { facing, .. } if facing.longitudinal_axis());
    for (field, scale, point) in [
        (Some(renderer.position), MODEL_UNITS_PER_EFFECT_UNIT, true),
        (
            renderer.geometry.axis_fields()[0],
            if longitudinal {
                MODEL_UNITS_PER_EFFECT_UNIT
            } else {
                1.
            },
            false,
        ),
        (renderer.geometry.axis_fields()[1], 1., false),
    ] {
        let Some(value) = field.and_then(|field| {
            (0..recipe.output_count())
                .filter_map(|i| recipe.output(i))
                .find(|output| output.field == field)
                .map(|output| output.value)
        }) else {
            continue;
        };
        for row in rows.iter().take(3) {
            let mut result = Range::point(if point { row[3] } else { 0. })?;
            for (component, coefficient) in row.iter().take(3).enumerate() {
                result = result.add(
                    values[value.index()][component]
                        .mul(Range::point(scale)?)?
                        .mul(Range::point(*coefficient)?)?,
                )?;
            }
        }
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_bounds_round_outward_while_authored_points_stay_exact() {
        let point = Range::point(1.).unwrap();
        assert_eq!(point.lo.to_bits(), 1f32.to_bits());
        assert_eq!(point.hi.to_bits(), 1f32.to_bits());
        let computed = point.mul(point).unwrap();
        assert_eq!(computed.lo, 1f32.next_down());
        assert_eq!(computed.hi, 1f32.next_up());
        let cancelled = point.sub(point).unwrap();
        assert!(cancelled.lo < 0. && cancelled.hi > 0.);
    }

    #[test]
    fn finite_edge_that_cannot_be_bounded_finitely_uses_sampled_route() {
        assert!(Range::point(f32::MAX).is_some());
        assert!(Range::point(f32::NAN).is_none());
        assert!(Range::point(f32::MAX)
            .unwrap()
            .mul(Range::point(1.).unwrap())
            .is_none());
        assert!(Range::point(f32::MIN_POSITIVE)
            .unwrap()
            .mul(Range::point(0.5).unwrap())
            .is_some());
    }
}
