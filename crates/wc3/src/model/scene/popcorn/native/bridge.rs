//! Synchronous private ABI between generated scalar code and checked host services.
use super::super::callbacks::ExecutionHost;
use super::super::execution::{ExecutionError, Failure, Host, State};
use super::super::runtime::FieldLayout;
use super::super::Program;
use std::any::Any;
use std::array::from_fn;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::ptr::null_mut;

/// Generated code returns zero on success and nonzero after a failed service.
/// The entry must belong to this exact checked programme and use this ABI.
use super::Entry;

/// Borrowed only during a synchronous entry call; never retained by generated code.
#[repr(C)]
pub(crate) struct Invocation {
    pub(crate) scopes: [*mut u32; 4],
    pub(crate) fields: *mut u32,
    pub(crate) stride: usize,
    pub(crate) slot: usize,
    pub(crate) time: u32,
    pub(crate) delta: u32,
    pub(crate) context: *mut c_void,
    pub(crate) service: unsafe extern "C" fn(*mut c_void, u32) -> u32,
}

struct Frame<'a, H: Host> {
    program: &'a Program,
    state: *mut State,
    host: *mut H,
    error: Option<ExecutionError<H::Error>>,
    panic: Option<Box<dyn Any + Send>>,
    // No mutable Rust references span generated raw-buffer accesses. The frame
    // nevertheless owns the exclusive caller borrows for the entire invocation.
    exclusive: PhantomData<(&'a mut State, &'a mut H)>,
}

unsafe extern "C" fn service<H: Host>(context: *mut c_void, index: u32) -> u32 {
    // SAFETY: run supplies a live, uniquely accessed matching Frame<H>. Generated
    // code calls synchronously, and may neither retain nor fabricate this context.
    let frame = unsafe { &mut *context.cast::<Frame<'_, H>>() };
    if frame.error.is_some() || frame.panic.is_some() {
        return 1;
    }
    let Some(operation) = frame.program.operations.get(index as usize) else {
        // A defensive ABI check; correct generated code emits only checked indices.
        frame.error = Some(ExecutionError {
            offset: 0,
            kind: Failure::IncompatibleState,
        });
        return 1;
    };
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: generated execution is suspended for the service call. No scope
        // access overlaps these temporary exclusive Rust reborrows. execute_instruction
        // uses preallocated State buffers and does not replace/reallocate scope storage.
        unsafe {
            frame.program.execute_instruction(
                &operation.instruction,
                &mut *frame.state,
                &mut *frame.host,
            )
        }
    }));
    match outcome {
        Ok(Ok(())) => 0,
        Ok(Err(kind)) => {
            frame.error = Some(ExecutionError {
                offset: operation.offset,
                kind,
            });
            1
        }
        Err(payload) => {
            frame.panic = Some(payload);
            1
        }
    }
}

/// Run an internally prepared entry for this exact programme and ABI.
///
/// This safe crate-private boundary relies on the enclosing JIT module keeping
/// Entry construction private, binding entries to their immutable programme and
/// retaining executable memory until return. It is not a public raw-code API.
/// Host errors stop generated execution; panics resume only after the C call ends.
#[cfg(test)]
pub(super) fn run<H: Host>(
    entry: Entry,
    program: &Program,
    state: &mut State,
    host: &mut H,
) -> Result<(), ExecutionError<H::Error>> {
    run_with_inputs(entry, program, state, host, DirectInputs::default())
}

#[derive(Default)]
struct DirectInputs {
    fields: *mut u32,
    stride: usize,
    slot: usize,
    time: u32,
    delta: u32,
}
/// Concrete runtime entry. No raw pointer can escape this synchronous call.
/// Mismatched storage falls back to the generic checked services, never an
/// unchecked pointer interpretation. The runtime owns exact programme identity.
pub(super) fn run_bound(
    entry: Entry,
    program: &Program,
    state: &mut State,
    host: &mut ExecutionHost<'_>,
    layout: &FieldLayout,
) -> Result<(), ExecutionError<&'static str>> {
    let stride = host.fields.plane_stride();
    let inputs = if host.fields.layout() == layout && host.slot < stride {
        DirectInputs {
            fields: host.fields.data_mut_ptr(),
            stride,
            slot: host.slot,
            time: (host.time as f32).to_bits(),
            delta: (host.delta as f32).to_bits(),
        }
    } else {
        DirectInputs {
            fields: null_mut(),
            ..Default::default()
        }
    };
    run_with_inputs(entry, program, state, host, inputs)
}
fn run_with_inputs<H: Host>(
    entry: Entry,
    program: &Program,
    state: &mut State,
    host: &mut H,
    inputs: DirectInputs,
) -> Result<(), ExecutionError<H::Error>> {
    // Match Program::execute before any local reset, host operation or native entry.
    if state.arguments.capacity() < program.max_arguments
        || state
            .scopes
            .iter()
            .zip(program.counts)
            .any(|(storage, count)| storage.len() < count)
    {
        return Err(ExecutionError {
            offset: 0,
            kind: Failure::IncompatibleState,
        });
    }
    state.scopes[0][..program.counts[0]].fill([0; 4]);
    state.scopes[1][..program.counts[1]].fill([0; 4]);
    // Do not clear arguments: the scalar interpreter preserves its existing
    // call-site scratch behavior, including on failure before the next Call.
    let scopes = from_fn(|scope| state.scopes[scope].as_mut_ptr().cast::<u32>());
    let mut frame = Frame {
        program,
        state: state as *mut State,
        host: host as *mut H,
        error: None,
        panic: None,
        exclusive: PhantomData,
    };
    let mut invocation = Invocation {
        scopes,
        fields: inputs.fields,
        stride: inputs.stride,
        slot: inputs.slot,
        time: inputs.time,
        delta: inputs.delta,
        context: (&mut frame as *mut Frame<'_, H>).cast::<c_void>(),
        service: service::<H>,
    };
    // SAFETY: private prepared code uses only validated scope addresses and this
    // invocation ABI. All backing buffers and borrowed host/context remain live;
    // generated code and services never unwind across this extern-C boundary.
    let status = unsafe { entry(&mut invocation) };
    if let Some(payload) = frame.panic.take() {
        resume_unwind(payload);
    }
    if let Some(error) = frame.error.take() {
        return Err(error);
    }
    if status != 0 {
        return Err(ExecutionError {
            offset: 0,
            kind: Failure::IncompatibleState,
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "bridge_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "bound_tests.rs"]
mod bound_tests;
