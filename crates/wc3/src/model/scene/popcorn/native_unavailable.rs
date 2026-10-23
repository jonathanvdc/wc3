//! Native preparation fallback on builds without executable-code support.
use super::callbacks::ExecutionHost;
use super::graph::EffectGraph;
use super::packet::{PureEvolution, Scratch as PacketScratch};
use super::{ExecutionError, ExecutionFallback, ExecutionStats, Program, State};

#[derive(Debug)]
pub(super) enum NativeCode {}
impl NativeCode {
    pub(super) fn prepare(_: &EffectGraph, _: bool) -> Result<Self, ExecutionFallback> {
        #[cfg(not(feature = "popcorn-jit"))]
        {
            Err(ExecutionFallback::FeatureDisabled)
        }
        #[cfg(feature = "popcorn-jit")]
        {
            Err(ExecutionFallback::UnsupportedPlatform)
        }
    }
    pub(super) fn has_simd(&self, _: usize) -> bool {
        match *self {}
    }
    pub(super) fn execute_simd(
        &self,
        _: usize,
        _: &PureEvolution,
        _: &mut PacketScratch,
        _: f64,
        _: &mut ExecutionStats,
    ) -> Option<Result<(), ()>> {
        match *self {}
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
