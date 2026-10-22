//! Prepared application contact admission, separate from authored semantic grants.
use super::{BodyDefinition, JointDefinition, DEFAULT_CONTACT_PAIR_BUDGET};
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Range;
use wc3::model::values::model::PhysicsBodyKind;

/// Application interpretation of contact admission within each source physics system.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PhysicsContactPolicy {
    /// Use only explicitly authored semantic collision pairs. Empty grants no contacts.
    #[default]
    AuthoredPairs,
    /// Permit retained fixture-bearing bodies when at least one is dynamic and no
    /// successfully compiled joint directly connects them. Replaces authored grants.
    /// Native game collision filtering remains unverified.
    UnconnectedDynamics,
}

/// Requested inferred contacts are unavailable; bodies and constraints still integrate.
/// No authored fallback or truncated inferred list is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhysicsContactError {
    /// Actual retained body/fixture controls require an unresolved contact interpretation.
    UnsupportedControls,
    /// Conservative candidate count exceeds the configured compiled-definition preparation bound.
    PairBudgetExceeded {
        /// Required source candidates before joint exclusions, or remapped grants
        /// before composition deduplication.
        candidates: usize,
        /// Maximum configured preparation pairs for the compiled definition.
        limit: usize,
    },
    /// Checked candidate-count arithmetic could not represent the bound.
    PairCountOverflow,
}
impl Display for PhysicsContactError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::UnsupportedControls => f.write_str(
                "unconnected-dynamic admission requires resolved body-parent, attachment-host, ragdoll and fixture filtering controls",
            ),
            Self::PairBudgetExceeded { candidates, limit } => write!(
                f, "unconnected-dynamic admission has {candidates} candidates before exclusions; compiled-definition limit is {limit}",
            ),
            Self::PairCountOverflow => f.write_str("unconnected-dynamic candidate count overflow"),
        }
    }
}
impl Error for PhysicsContactError {}

#[derive(Clone)]
pub(super) struct Admission {
    pub pairs: Vec<(usize, usize)>,
    pub error: Option<PhysicsContactError>,
    budget: usize,
    candidates: usize,
}
impl Default for Admission {
    fn default() -> Self {
        Self::new(DEFAULT_CONTACT_PAIR_BUDGET)
    }
}
fn unordered(a: usize, b: usize) -> (usize, usize) {
    (a.min(b), a.max(b))
}
fn combinations(count: usize) -> Option<usize> {
    if count < 2 {
        return Some(0);
    }
    if count.is_multiple_of(2) {
        (count / 2).checked_mul(count - 1)
    } else {
        count.checked_mul((count - 1) / 2)
    }
}
fn candidates(dynamic: usize, other: usize) -> Option<usize> {
    // Dynamic/dynamic plus dynamic/non-dynamic. This avoids the large all-pairs
    // subtraction and never enumerates kinematic/kinematic combinations.
    combinations(dynamic)?.checked_add(dynamic.checked_mul(other)?)
}
fn usable(body: &BodyDefinition) -> bool {
    body.fixtures.iter().any(|fixture| {
        fixture.center.is_finite() && fixture.radius.is_finite() && fixture.radius >= 0.
    })
}
fn unresolved(body: &BodyDefinition) -> bool {
    let body = &body.info.body;
    body.parent_body.is_some()
        || body.attachment_host
        || body.ragdoll_root
        || body
            .shapes
            .iter()
            .any(|shape| shape.game_flags.bits() != 0 || shape.collision_mask.bits() != 0)
}
fn joint_pairs(joints: &[JointDefinition]) -> BTreeSet<(usize, usize)> {
    joints
        .iter()
        .map(|joint| unordered(joint.a, joint.b))
        .collect()
}
impl Admission {
    #[cfg(test)]
    pub fn derive(
        bodies: &[BodyDefinition],
        joints: &[JointDefinition],
        source: Range<usize>,
        limit: usize,
    ) -> Result<Vec<(usize, usize)>, PhysicsContactError> {
        Self::derive_counted(bodies, joints, source, limit).map(|(pairs, _)| pairs)
    }
    fn derive_counted(
        bodies: &[BodyDefinition],
        joints: &[JointDefinition],
        source: Range<usize>,
        limit: usize,
    ) -> Result<(Vec<(usize, usize)>, usize), PhysicsContactError> {
        let (dynamic, other): (Vec<_>, Vec<_>) = source
            .filter(|&index| usable(&bodies[index]))
            .partition(|&index| bodies[index].info.body.kind == PhysicsBodyKind::Dynamic);
        let count =
            candidates(dynamic.len(), other.len()).ok_or(PhysicsContactError::PairCountOverflow)?;
        // Controls on bodies that cannot be contact endpoints do not require a
        // contact-filter interpretation. Their motion limitations remain diagnosed
        // independently by physics compilation.
        if count == 0 {
            return Ok((Vec::new(), 0));
        }
        if dynamic
            .iter()
            .chain(&other)
            .any(|&index| unresolved(&bodies[index]))
        {
            return Err(PhysicsContactError::UnsupportedControls);
        }
        if count > limit {
            return Err(PhysicsContactError::PairBudgetExceeded {
                candidates: count,
                limit,
            });
        }
        // Bound the ACTUAL nested iteration count, not merely accepted results.
        // Enumerating all non-dynamic pairs would escape this work bound.
        let connected = joint_pairs(joints);
        let mut pairs = Vec::with_capacity(count);
        for (offset, &a) in dynamic.iter().enumerate() {
            for &b in dynamic[offset + 1..].iter().chain(&other) {
                let pair = unordered(a, b);
                if !connected.contains(&pair) {
                    pairs.push(pair);
                }
            }
        }
        pairs.sort_unstable();
        Ok((pairs, count))
    }
    pub fn new(budget: usize) -> Self {
        Self {
            pairs: Vec::new(),
            error: None,
            budget,
            candidates: 0,
        }
    }
    pub fn prepare_source(
        &mut self,
        bodies: &[BodyDefinition],
        joints: &[JointDefinition],
        source: Range<usize>,
    ) {
        if self.error.is_some() {
            return;
        }
        let remaining = self.budget - self.candidates;
        match Self::derive_counted(bodies, joints, source, remaining) {
            Ok((pairs, cost)) => {
                self.candidates += cost; // cost was checked against remaining
                self.pairs.extend(pairs);
            }
            Err(PhysicsContactError::PairBudgetExceeded { candidates, .. }) => {
                self.error = Some(match self.candidates.checked_add(candidates) {
                    Some(candidates) => PhysicsContactError::PairBudgetExceeded {
                        candidates,
                        limit: self.budget,
                    },
                    None => PhysicsContactError::PairCountOverflow,
                });
                self.pairs.clear();
            }
            Err(error) => {
                self.error = Some(error);
                self.pairs.clear();
            }
        }
    }
    pub fn append_remapped(&mut self, source: &Self, remap: &HashMap<usize, usize>) {
        if self.error.is_some() {
            return;
        }
        if let Some(error) = source.error {
            self.error = Some(error);
            self.pairs.clear();
            return;
        }
        // Composition only visits already prepared source grants. Never enumerate
        // a fresh body cross-product, and bound retained output before every push.
        for (a, b) in &source.pairs {
            let (Some(&a), Some(&b)) = (remap.get(a), remap.get(b)) else {
                continue;
            };
            if a == b {
                continue;
            }
            if self.pairs.len() >= self.budget {
                self.error = Some(PhysicsContactError::PairBudgetExceeded {
                    candidates: self.pairs.len().saturating_add(1),
                    limit: self.budget,
                });
                self.pairs.clear();
                return;
            }
            self.pairs.push(unordered(a, b));
        }
    }
    pub fn retain_compiled(&mut self, bodies: &[BodyDefinition], joints: &[JointDefinition]) {
        let connected = joint_pairs(joints);
        self.pairs.retain(|&(a, b)| {
            usable(&bodies[a])
                && usable(&bodies[b])
                && (bodies[a].info.body.kind == PhysicsBodyKind::Dynamic
                    || bodies[b].info.body.kind == PhysicsBodyKind::Dynamic)
                && !connected.contains(&(a, b))
        });
        self.pairs.sort_unstable();
        self.pairs.dedup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidate_bound_is_checked_without_spurious_intermediate_overflow() {
        assert_eq!(combinations(0), Some(0));
        assert_eq!(combinations(1), Some(0));
        assert_eq!(candidates(3, 2), Some(9));
        assert_eq!(candidates(0, usize::MAX), Some(0));
        assert_eq!(candidates(usize::MAX, 0), None);
        assert_eq!(candidates(2, usize::MAX), None);
    }
}
