//! Dense private register identities for an effect layer's selected programmes.
use super::{Address, Destination, Instruction, Program, Source};
use std::array::from_fn;
use std::collections::{BTreeMap, BTreeSet};

// Initialization and evolution share persistent scopes, so their aliases must
// be renamed together. Public scalar programmes retain their original layout.
pub(crate) fn compact_registers<'a>(programs: impl IntoIterator<Item = &'a mut Program>) {
    let mut programs: Vec<_> = programs.into_iter().collect();
    let mut used: [BTreeSet<usize>; 4] = from_fn(|_| BTreeSet::new());
    for program in &mut programs {
        for operation in &mut program.operations {
            visit_registers(&mut operation.instruction, &mut |scope, slot| {
                used[scope].insert(*slot);
            });
        }
    }
    let mapping: [BTreeMap<usize, usize>; 4] = used.map(|slots| {
        slots
            .into_iter()
            .enumerate()
            .map(|(dense, native)| (native, dense))
            .collect()
    });
    let counts = from_fn(|scope| mapping[scope].len());
    for program in programs {
        for operation in &mut program.operations {
            visit_registers(&mut operation.instruction, &mut |scope, slot| {
                *slot = mapping[scope][slot];
            });
        }
        program.counts = counts;
    }
}

fn source(source: &mut Source, visit: &mut impl FnMut(usize, &mut usize)) {
    if let Address::Slot { scope, slot } = &mut source.address {
        visit(*scope, slot);
    }
}
fn destination(destination: &mut Destination, visit: &mut impl FnMut(usize, &mut usize)) {
    visit(destination.scope, &mut destination.slot);
}
fn visit_registers(instruction: &mut Instruction, visit: &mut impl FnMut(usize, &mut usize)) {
    match instruction {
        Instruction::Load {
            destination: output,
            ..
        } => destination(output, visit),
        Instruction::Store { source: input, .. } => source(input, visit),
        Instruction::Reinterpret {
            destination: output,
            source: input,
        }
        | Instruction::Convert {
            destination: output,
            source: input,
        }
        | Instruction::Swizzle {
            destination: output,
            source: input,
            ..
        }
        | Instruction::UnaryFunction {
            destination: output,
            source: input,
            ..
        } => {
            destination(output, visit);
            source(input, visit);
        }
        Instruction::Construct {
            destination: output,
            sources,
        } => {
            destination(output, visit);
            for input in sources {
                source(input, visit);
            }
        }
        Instruction::Binary {
            destination: output,
            left,
            right,
            ..
        }
        | Instruction::BinaryFunction {
            destination: output,
            left,
            right,
            ..
        } => {
            destination(output, visit);
            source(left, visit);
            source(right, visit);
        }
        Instruction::Select {
            destination: output,
            if_false,
            if_true,
            condition,
        } => {
            destination(output, visit);
            source(if_false, visit);
            source(if_true, visit);
            source(condition, visit);
        }
        Instruction::Call {
            destination: output,
            arguments,
            ..
        } => {
            if let Some(output) = output {
                destination(output, visit);
            }
            for input in arguments {
                source(input, visit);
            }
        }
    }
}

#[cfg(test)]
#[path = "layout_tests.rs"]
mod tests;
