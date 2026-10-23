//! One requested physics update owns contact quotas, across all solver passes.
/// Per-update limits for optional secondary-motion contacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicsContactLimits {
    /// Body-pair examinations before weight/broad-phase rejection, shared across
    /// every fixed step, projection iteration and velocity pass. Zero disables contacts.
    pub pair_checks_per_update: usize,
    /// Narrow-phase queries across every fixed step, projection and velocity pass.
    /// Zero disables contacts while integration and joints continue normally.
    pub queries_per_update: usize,
    /// Hull vertices examined and closed-form primitive support samples.
    /// This quota is shared across all support-map queries in the same update.
    pub support_vertices_per_update: usize,
}
impl Default for PhysicsContactLimits {
    fn default() -> Self {
        Self {
            pair_checks_per_update: 16_384,
            queries_per_update: 16_384,
            support_vertices_per_update: 262_144,
        }
    }
}
/// Most recent contact omission/failure; successful later queries do not erase it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PhysicsContactIssue {
    /// The shared per-update body-pair examination quota was exhausted.
    PairLimit,
    /// The shared per-update narrow-phase query quota was exhausted.
    QueryLimit,
    /// A support-map query exhausted the shared support-sample quota.
    SupportLimit,
    /// A contact input or transformed support was nonfinite or singular.
    InvalidInput,
    /// The analytical query does not support this primitive combination.
    UnsupportedPair,
    /// The support search could not establish a nondegenerate simplex or polytope.
    Degenerate,
    /// The bounded iterative search did not converge.
    IterationLimit,
    /// A prepared hull or fixed polytope exceeded its supported capacity.
    CapacityLimit,
    /// The instance has no prepared hull-query scratch storage.
    ScratchUnavailable,
}
/// Saturating cumulative accounting since rig creation or an explicit reset generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PhysicsContactStats {
    /// Actual body-pair examinations, including weight/broad-phase rejections.
    pub pair_checks: u64,
    /// Updates whose remaining contact passes were truncated by their pair quota.
    pub pair_budget_exhaustions: u64,
    /// Admitted narrow-phase queries, including unsuccessful/inconclusive ones.
    pub queries: u64,
    /// Actual hull vertices examined and closed-form support samples evaluated.
    pub support_vertices: u64,
    /// Non-budget queries omitted because inputs or numerical convergence failed.
    pub inconclusive_queries: u64,
    /// Updates whose remaining contact passes were truncated by their query quota.
    pub query_budget_exhaustions: u64,
    /// Support-map queries omitted after exhausting their shared support quota.
    pub support_budget_exhaustions: u64,
    /// Latest retained issue, independently of native contact-admission errors.
    pub last_issue: Option<PhysicsContactIssue>,
}
impl PhysicsContactStats {
    pub(super) fn accumulate(&mut self, other: Self) {
        self.pair_checks = self.pair_checks.saturating_add(other.pair_checks);
        self.pair_budget_exhaustions = self
            .pair_budget_exhaustions
            .saturating_add(other.pair_budget_exhaustions);
        self.queries = self.queries.saturating_add(other.queries);
        self.support_vertices = self.support_vertices.saturating_add(other.support_vertices);
        self.inconclusive_queries = self
            .inconclusive_queries
            .saturating_add(other.inconclusive_queries);
        self.query_budget_exhaustions = self
            .query_budget_exhaustions
            .saturating_add(other.query_budget_exhaustions);
        self.support_budget_exhaustions = self
            .support_budget_exhaustions
            .saturating_add(other.support_budget_exhaustions);
        self.last_issue = other.last_issue.or(self.last_issue);
    }
}
pub(super) struct ContactWork {
    pairs: usize,
    pair_exhausted: bool,
    queries: usize,
    vertices: usize,
    query_exhausted: bool,
    pub stats: PhysicsContactStats,
}
impl ContactWork {
    pub fn new(limits: PhysicsContactLimits) -> Self {
        Self {
            pairs: limits.pair_checks_per_update,
            pair_exhausted: false,
            queries: limits.queries_per_update,
            vertices: limits.support_vertices_per_update,
            query_exhausted: false,
            stats: PhysicsContactStats::default(),
        }
    }
    /// Check before Cartesian fixture loops and before later contact passes.
    pub fn available(&mut self) -> bool {
        if self.queries > 0 {
            return true;
        }
        if !self.query_exhausted {
            self.query_exhausted = true;
            self.stats.query_budget_exhaustions = 1;
            self.stats.last_issue = Some(PhysicsContactIssue::QueryLimit);
        }
        false
    }
    /// Admit one outer-loop visit before inspecting weights, bounds or fixtures.
    /// Remaining queries within an admitted pair keep their independent allowance.
    pub fn begin_pair(&mut self) -> bool {
        if !self.available() {
            return false;
        }
        if self.pairs == 0 {
            if !self.pair_exhausted {
                self.pair_exhausted = true;
                self.stats.pair_budget_exhaustions = 1;
                self.stats.last_issue = Some(PhysicsContactIssue::PairLimit);
            }
            return false;
        }
        self.pairs -= 1;
        self.stats.pair_checks = self.stats.pair_checks.saturating_add(1);
        true
    }
    pub fn begin_query(&mut self) -> bool {
        if !self.available() {
            return false;
        }
        self.queries -= 1;
        self.stats.queries = self.stats.queries.saturating_add(1);
        true
    }
    pub fn vertex(&mut self) -> Result<(), PhysicsContactIssue> {
        if self.vertices == 0 {
            return Err(PhysicsContactIssue::SupportLimit);
        }
        self.vertices -= 1;
        self.stats.support_vertices = self.stats.support_vertices.saturating_add(1);
        Ok(())
    }
    pub fn issue(&mut self, issue: PhysicsContactIssue) {
        self.stats.last_issue = Some(issue);
        if issue == PhysicsContactIssue::SupportLimit {
            self.stats.support_budget_exhaustions =
                self.stats.support_budget_exhaustions.saturating_add(1);
        } else if !matches!(
            issue,
            PhysicsContactIssue::QueryLimit | PhysicsContactIssue::PairLimit
        ) {
            self.stats.inconclusive_queries = self.stats.inconclusive_queries.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits_account_actual_work_and_retained_issue_without_invented_omissions() {
        let mut work = ContactWork::new(PhysicsContactLimits {
            queries_per_update: 2,
            support_vertices_per_update: 3,
            ..Default::default()
        });
        assert!(work.begin_query());
        for _ in 0..3 {
            work.vertex().unwrap();
        }
        assert_eq!(work.vertex(), Err(PhysicsContactIssue::SupportLimit));
        work.issue(PhysicsContactIssue::SupportLimit);
        // Exhausted support does not consume the remaining analytical query.
        assert!(work.begin_query());
        work.issue(PhysicsContactIssue::InvalidInput);
        for _ in 0..10 {
            assert!(!work.available());
        }
        assert_eq!(work.stats.queries, 2);
        assert_eq!(work.stats.support_vertices, 3);
        assert_eq!(work.stats.query_budget_exhaustions, 1);
        assert_eq!(work.stats.support_budget_exhaustions, 1);
        assert_eq!(work.stats.inconclusive_queries, 1);
        let mut total = work.stats;
        total.accumulate(PhysicsContactStats::default());
        assert_eq!(total.last_issue, Some(PhysicsContactIssue::QueryLimit));
        total.queries = u64::MAX;
        total.accumulate(work.stats);
        assert_eq!(total.queries, u64::MAX);
    }

    #[test]
    fn pair_admission_is_independent_exact_and_once_per_truncated_update() {
        let mut work = ContactWork::new(PhysicsContactLimits {
            pair_checks_per_update: 2,
            queries_per_update: 3,
            support_vertices_per_update: 4,
        });
        assert!(work.begin_pair());
        assert!(work.begin_pair());
        assert_eq!(work.stats.pair_checks, 2);
        assert_eq!(work.stats.pair_budget_exhaustions, 0);
        assert_eq!(work.stats.queries, 0);
        // Finishing compound queries within the last admitted body pair must
        // remain possible even though a new pair cannot be admitted.
        assert!(work.begin_query());
        work.vertex().unwrap();
        for _ in 0..8 {
            assert!(!work.begin_pair());
        }
        assert_eq!(work.stats.pair_budget_exhaustions, 1);
        assert_eq!(work.stats.last_issue, Some(PhysicsContactIssue::PairLimit));
        assert_eq!(work.stats.queries, 1);
        assert_eq!(work.stats.support_vertices, 1);
        assert_eq!(work.stats.inconclusive_queries, 0);
        assert_eq!(work.stats.query_budget_exhaustions, 0);
        assert_eq!(work.stats.support_budget_exhaustions, 0);
        let mut total = PhysicsContactStats {
            pair_checks: u64::MAX,
            pair_budget_exhaustions: u64::MAX,
            ..Default::default()
        };
        total.accumulate(work.stats);
        assert_eq!(total.pair_checks, u64::MAX);
        assert_eq!(total.pair_budget_exhaustions, u64::MAX);
        assert_eq!(total.last_issue, Some(PhysicsContactIssue::PairLimit));
    }
    #[test]
    fn zero_pair_allowance_performs_no_pair_query_or_support_work() {
        let mut work = ContactWork::new(PhysicsContactLimits {
            pair_checks_per_update: 0,
            ..Default::default()
        });
        assert!(!work.begin_pair());
        assert_eq!(work.stats.pair_checks, 0);
        assert_eq!(work.stats.queries, 0);
        assert_eq!(work.stats.support_vertices, 0);
        assert_eq!(work.stats.pair_budget_exhaustions, 1);
        assert_eq!(work.stats.last_issue, Some(PhysicsContactIssue::PairLimit));
        let mut no_queries = ContactWork::new(PhysicsContactLimits {
            pair_checks_per_update: 5,
            queries_per_update: 0,
            ..Default::default()
        });
        assert!(!no_queries.begin_pair());
        assert_eq!(no_queries.stats.pair_checks, 0);
        assert_eq!(no_queries.stats.pair_budget_exhaustions, 0);
        assert_eq!(no_queries.stats.query_budget_exhaustions, 1);
    }
}
