//! Native preparation fallback on builds without executable-code support.
use super::callbacks::ExecutionHost;
use super::graph::EffectGraph;
use super::{ExecutionError, ExecutionFallback, Program, State};

#[derive(Debug)]
pub(super) enum NativeCode {}
impl NativeCode {
    pub(super) fn prepare(_: &EffectGraph) -> Result<Self, ExecutionFallback> {
        #[cfg(not(feature = "popcorn-jit"))]
        {
            Err(ExecutionFallback::FeatureDisabled)
        }
        #[cfg(feature = "popcorn-jit")]
        {
            Err(ExecutionFallback::UnsupportedPlatform)
        }
    }
    pub(super) fn execute_bound(
        &self,
        _: usize,
        _: &Program,
        _: &mut State,
        _: &mut ExecutionHost<'_>,
    ) -> Result<(), ExecutionError<&'static str>> {
        match *self {}
    }
}
