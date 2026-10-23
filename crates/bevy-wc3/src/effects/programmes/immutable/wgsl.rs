//! Emit the core's closed numeric recipe; never decode particle instructions here.
use super::super::MODEL_UNITS_PER_EFFECT_UNIT;
use super::AppearancePhase;
use std::fmt::Write;
use wc3::model::scene::popcorn::{
    EffectFieldId, ImmutableExpression, ImmutableNodeId, ImmutableRecipe, ValueType,
};

/// Static row: the existing sampled Particle, lifecycle anchor, then typed captures.
pub(super) const HEADER_WORDS: usize = 32;
pub(super) const PHASE_WORD: usize = 31;
const EXPANSION_LIMIT: usize = 4096;

#[derive(Clone, Copy)]
pub(super) struct AppearanceFields {
    pub position: Option<EffectFieldId>,
    pub axes: [Option<EffectFieldId>; 2],
    pub longitudinal_axis: bool,
    pub size: Option<EffectFieldId>,
    pub color: Option<EffectFieldId>,
    pub rotation: Option<EffectFieldId>,
    pub texture: Option<EffectFieldId>,
}
fn output(recipe: ImmutableRecipe<'_>, field: EffectFieldId) -> Option<ImmutableNodeId> {
    (0..recipe.output_count())
        .filter_map(|i| recipe.output(i))
        .find(|out| out.field == field)
        .map(|out| out.value)
}
fn padded(expression: &str, width: usize) -> String {
    let lanes: Vec<_> = (0..4)
        .map(|i| {
            if i < width {
                format!("({expression})[{}u]", i)
            } else {
                "0.0".to_owned()
            }
        })
        .collect();
    format!("vec4<f32>({})", lanes.join(", "))
}
fn literal(values: [f32; 4]) -> String {
    format!(
        "bitcast<vec4<f32>>(vec4<u32>({}u, {}u, {}u, {}u))",
        values[0].to_bits(),
        values[1].to_bits(),
        values[2].to_bits(),
        values[3].to_bits()
    )
}
fn scalar(value: f32) -> String {
    format!("bitcast<f32>({}u)", value.to_bits())
}
fn operand(recipe: ImmutableRecipe<'_>, id: ImmutableNodeId) -> Option<String> {
    let node = recipe.node(id.index())?;
    Some(if node.kind.components() == 1 {
        format!("vec4<f32>(v{}.x)", id.index())
    } else {
        format!("v{}", id.index())
    })
}
fn interval(
    text: &mut String,
    recipe: ImmutableRecipe<'_>,
    sampler: usize,
    start: usize,
    end: usize,
    indent: &str,
) -> Option<()> {
    let curve = recipe.curve(sampler)?;
    if end - start != 1 {
        let mid = (start + end) / 2;
        writeln!(
            text,
            "{indent}if coordinate < {} {{",
            scalar(curve.times[mid])
        )
        .ok()?;
        interval(text, recipe, sampler, start, mid, &format!("{indent}    "))?;
        writeln!(text, "{indent}}} else {{").ok()?;
        interval(text, recipe, sampler, mid, end, &format!("{indent}    "))?;
        writeln!(text, "{indent}}}").ok()?;
        return Some(());
    }
    writeln!(
        text,
        "{indent}let t = (coordinate - {}) / ({} - {});",
        scalar(curve.times[start]),
        scalar(curve.times[end]),
        scalar(curve.times[start])
    )
    .ok()?;
    let a = literal(curve.values[start]);
    let b = literal(curve.values[end]);
    let value = if curve.tangents.is_empty() {
        format!("{a} + ({b} - {a}) * t")
    } else {
        writeln!(text, "{indent}let t2 = t * t;\n{indent}let t3 = t2 * t;").ok()?;
        format!("(((2.0 * t3 - 3.0 * t2 + 1.0) * {a} + (t3 - 2.0 * t2 + t) * {}) + (-2.0 * t3 + 3.0 * t2) * {b}) + (t3 - t2) * {}",
            literal(curve.tangents[start * 2 + 1]), literal(curve.tangents[end * 2]))
    };
    writeln!(text, "{indent}let result = {value};").ok()?;
    writeln!(
        text,
        "{indent}return {};",
        padded("result", curve.kind.components())
    )
    .ok()?;
    Some(())
}

/// A complete shader prefix. Particle/Emitter and geometry/fragment stay in the shared shader.
pub(super) fn emit(recipe: ImmutableRecipe<'_>, fields: AppearanceFields) -> Option<String> {
    let stride = HEADER_WORDS.checked_add(recipe.capture_count().checked_mul(4)?)?;
    let mut text =
        String::from("@group(2) @binding(3) var<storage, read> immutable_words: array<u32>;\n");
    writeln!(
        text,
        "const MODEL_UNITS_PER_EFFECT_UNIT: f32 = {MODEL_UNITS_PER_EFFECT_UNIT:?};"
    )
    .ok()?;
    let mut budget = EXPANSION_LIMIT;
    for i in 0..recipe.node_count() {
        if let ImmutableExpression::Curve { sampler, .. } = recipe.node(i)?.expression {
            let curve = recipe.curve(sampler)?;
            if curve.times.is_empty() || curve.times.len() > 16 {
                return None;
            }
            budget = budget.checked_sub(curve.times.len() * curve.kind.components() * 42)?;
            writeln!(
                text,
                "fn immutable_curve_{i}(coordinate: f32) -> vec4<f32> {{"
            )
            .ok()?;
            writeln!(
                text,
                "    if coordinate <= {} {{ return {}; }}",
                scalar(curve.times[0]),
                padded(&literal(curve.values[0]), curve.kind.components())
            )
            .ok()?;
            if curve.times.len() == 1 {
                writeln!(
                    text,
                    "    return {};",
                    padded(&literal(curve.values[0]), curve.kind.components())
                )
                .ok()?;
            } else {
                let last = curve.times.len() - 1;
                writeln!(
                    text,
                    "    if coordinate >= {} {{ return {}; }}",
                    scalar(curve.times[last]),
                    padded(&literal(curve.values[last]), curve.kind.components())
                )
                .ok()?;
                interval(&mut text, recipe, sampler, 0, last, "    ")?;
            }
            writeln!(text, "}}").ok()?;
        }
    }
    text.push_str("fn immutable_vec4(base: u32) -> vec4<u32> {\n    return vec4<u32>(immutable_words[base], immutable_words[base + 1u], immutable_words[base + 2u], immutable_words[base + 3u]);\n}\n");
    writeln!(
        text,
        "fn immutable_particle(index: u32) -> Particle {{\n    let row = index * {stride}u;"
    )
    .ok()?;
    text.push_str("    var particle = Particle(bitcast<vec4<f32>>(immutable_vec4(row)), bitcast<vec4<f32>>(immutable_vec4(row + 4u)), bitcast<vec4<f32>>(immutable_vec4(row + 8u)), bitcast<vec4<f32>>(immutable_vec4(row + 12u)), bitcast<vec4<f32>>(immutable_vec4(row + 16u)), bitcast<vec4<f32>>(immutable_vec4(row + 20u)), immutable_vec4(row + 24u));\n");
    writeln!(
        text,
        "    if immutable_words[row + {PHASE_WORD}u] == {}u {{ return particle; }}",
        AppearancePhase::Initialized as u32
    )
    .ok()?;
    text.push_str("    let birth = bitcast<vec4<f32>>(immutable_vec4(row + 28u));\n    let authorized = immutable_authorized_time();\n    let life = min(birth.y + (authorized - birth.x) * birth.z, 1.0);\n");
    for i in 0..recipe.node_count() {
        let node = recipe.node(i)?;
        if !matches!(
            node.kind,
            ValueType::Float
                | ValueType::Float3
                | ValueType::Float4
                | ValueType::Bool
                | ValueType::Bool3
        ) {
            return None;
        }
        let boolean = matches!(node.kind, ValueType::Bool | ValueType::Bool3);
        if boolean {
            let expression = match node.expression {
                ImmutableExpression::Constant(value) => {
                    let w = value.words();
                    format!("vec4<u32>({}u, {}u, {}u, {}u)", w[0], w[1], w[2], w[3])
                }
                ImmutableExpression::Capture(capture) => {
                    format!("immutable_vec4(row + {}u)", HEADER_WORDS + capture * 4)
                }
                ImmutableExpression::Words { input, .. } => {
                    if matches!(
                        recipe.node(input.index())?.kind,
                        ValueType::Bool | ValueType::Bool3
                    ) {
                        format!("v{}", input.index())
                    } else {
                        format!("bitcast<vec4<u32>>(v{})", input.index())
                    }
                }
                _ => return None,
            };
            writeln!(text, "    let result_{i} = {expression};").ok()?;
            let lanes: Vec<_> = (0..4)
                .map(|c| {
                    if c < node.kind.components() {
                        format!("result_{i}[{c}u]")
                    } else {
                        "0u".to_owned()
                    }
                })
                .collect();
            writeln!(text, "    let v{i} = vec4<u32>({});", lanes.join(", ")).ok()?;
            continue;
        }
        let expression = match node.expression {
            ImmutableExpression::Constant(value) => literal(value.words().map(f32::from_bits)),
            ImmutableExpression::Capture(capture) => {
                if capture >= recipe.capture_count() {
                    return None;
                }
                format!(
                    "bitcast<vec4<f32>>(immutable_vec4(row + {}u))",
                    HEADER_WORDS + capture * 4
                )
            }
            ImmutableExpression::Life => "vec4<f32>(life, 0.0, 0.0, 0.0)".to_owned(),
            ImmutableExpression::Time => "vec4<f32>(authorized, 0.0, 0.0, 0.0)".to_owned(),
            ImmutableExpression::IntegratedDelta => return None,
            ImmutableExpression::Words { input, kind } => {
                let value = if matches!(
                    recipe.node(input.index())?.kind,
                    ValueType::Bool | ValueType::Bool3
                ) {
                    format!("bitcast<vec4<f32>>(v{})", input.index())
                } else {
                    format!("v{}", input.index())
                };
                padded(&value, kind.components())
            }
            ImmutableExpression::Add { left, right } => {
                format!("{} + {}", operand(recipe, left)?, operand(recipe, right)?)
            }
            ImmutableExpression::Subtract { left, right } => {
                format!("{} - {}", operand(recipe, left)?, operand(recipe, right)?)
            }
            ImmutableExpression::Multiply { left, right } => {
                format!("{} * {}", operand(recipe, left)?, operand(recipe, right)?)
            }
            ImmutableExpression::Curve { input, .. } => {
                format!("immutable_curve_{i}(v{}.x)", input.index())
            }
        };
        writeln!(text, "    let result_{i} = {expression};").ok()?;
        writeln!(
            text,
            "    let v{i} = {};",
            padded(&format!("result_{i}"), node.kind.components())
        )
        .ok()?;
    }
    if let Some(value) = fields.position.and_then(|field| output(recipe, field)) {
        writeln!(text, "    let position = vec4<f32>(v{}.xyz * MODEL_UNITS_PER_EFFECT_UNIT, 1.0);\n    particle.position = vec4<f32>(dot(emitter.world_from_space[0], position), dot(emitter.world_from_space[1], position), dot(emitter.world_from_space[2], position), 1.0);", value.index()).ok()?;
    }
    for (index, field) in fields.axes.into_iter().enumerate() {
        if let Some(value) = field.and_then(|field| output(recipe, field)) {
            let target = if index == 0 {
                "velocity_tail"
            } else {
                "offsets"
            };
            let scale = if index == 0 && fields.longitudinal_axis {
                " * MODEL_UNITS_PER_EFFECT_UNIT"
            } else {
                ""
            };
            writeln!(text, "    let axis_{index} = vec4<f32>(v{}.xyz, 0.0);\n    particle.{target} = vec4<f32>(vec3<f32>(dot(emitter.world_from_space[0], axis_{index}), dot(emitter.world_from_space[1], axis_{index}), dot(emitter.world_from_space[2], axis_{index})){scale}, 0.0);", value.index()).ok()?;
        }
    }
    if let Some(value) = fields.size.and_then(|field| output(recipe, field)) {
        writeln!(text, "    particle.size_spin.x = (v{}.x * MODEL_UNITS_PER_EFFECT_UNIT) * 2.0;\n    particle.size_spin.y = particle.size_spin.x;", value.index()).ok()?;
    }
    if let Some(value) = fields.color.and_then(|field| output(recipe, field)) {
        writeln!(
            text,
            "    particle.color = max(v{}, vec4<f32>(0.0));",
            value.index()
        )
        .ok()?;
    }
    if let Some(value) = fields.rotation.and_then(|field| output(recipe, field)) {
        writeln!(
            text,
            "    particle.size_spin.z = cos(v{}.x);\n    particle.size_spin.w = sin(v{}.x);",
            value.index(),
            value.index()
        )
        .ok()?;
    }
    if let Some(value) = fields.texture.and_then(|field| output(recipe, field)) {
        writeln!(text, "    let cursor = min(abs(v{}.x), f32(max(emitter.atlas.x, 1u) * max(emitter.atlas.y, 1u) - 1u));\n    particle.cells.x = u32(floor(cursor));\n    particle.cells.z = min(particle.cells.x + 1u, max(emitter.atlas.x, 1u) * max(emitter.atlas.y, 1u) - 1u);\n    particle.scales.z = fract(cursor);", value.index()).ok()?;
    }
    text.push_str("    return particle;\n}\n");
    Some(text)
}
