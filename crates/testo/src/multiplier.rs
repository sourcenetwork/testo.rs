use crate::action::{Action, ActionAsync, ActionInstance};

/// Multiplier represents a complexity multiplier of the system under test.
///
/// It represents a concept that multiplies the surface area and complexity
/// of other proximal features, for example, database transactions are
/// complexity modifiers, as when adding many new database [actions](crate::action::Action) such as
/// a new filter operation, the new action must be tested both with, and without
/// a transaction - the transaction concept multiplies the complexity of the system.
///
/// The identification and automatic representation of complexity multipiers are critical
/// to the long term maintainability and scalability of a codebase. Without them, introducing
/// fairly simple features becomes high risk, and requires a forever growing number of manually
/// constructed tests, when the feature developer needs to remember, and write tests for every
/// multiplier that affects the feature under test.  This is error prone, very tedious, and
/// distracts from the feature itself - often degrading the quality of both test and production code.
///
/// Complexity multipliers defined using this type may redefine the set of test [actions](crate::action::Action)
/// to perform within a test, allowing a single test to scale with the complexity
/// multiplier.  For example, applying a 'namespace' multplier may add an action that
/// namespaces the store under test, reducing, but not removing, testing cost of the
/// namespace multiplier.
///
/// Multipliers do not act upon test state, and should not be given access to it.
pub trait Multiplier<TActionAsync, TAction = TActionAsync>
where
    TAction: Action,
    TActionAsync: ActionAsync,
{
    /// The name of this multiplier, used to identify and include/exclude it before mutating
    /// a test action-set.
    const NAME: &str;

    /// Returns the name of this multiplier, used to identify and include/exclude it before mutating
    /// a test action-set.
    fn name(&self) -> &str {
        Self::NAME
    }

    /// Applies the multiplier to given action set, returning a new set.
    ///
    /// This function may add, remove or replace [actions](crate::action::Action) as it sees fit.
    fn apply(
        &self,
        actions: &[ActionInstance<TActionAsync, TAction>],
    ) -> Vec<ActionInstance<TActionAsync, TAction>>;

    /// Defaults to [NoSkip](SkipReason::NoSkip), this function allows a multiplier to decide that
    /// the test should be skipped instead of executed whilst this multiplier is active.
    ///
    /// It should ideally never be used, and a [Multiplier] implementing it is likely a code smell.
    fn should_skip(&self, _actions: &[ActionInstance<TActionAsync, TAction>]) -> SkipReason {
        SkipReason::NoSkip
    }
}

/// Enum representing whether a test should be skipped or not, and why.
pub enum SkipReason {
    /// There was no reason found by the function returning this to skip the current test.
    NoSkip,
    /// The function returning this found a reason to skip the test.
    Skip(String),
}

impl From<Option<SkipReason>> for SkipReason {
    fn from(value: Option<SkipReason>) -> Self {
        match value {
            Some(reason) => reason,
            None => SkipReason::NoSkip,
        }
    }
}

/// Check the set of actions and multipliers against each other, looking to find a reason to skip the test.
pub fn skip<TState, TAction, TActionAsync, TMultiplier>(
    actions: &[ActionInstance<TActionAsync, TAction>],
    active_multipliers: &[&TMultiplier],
    includes: &[&str],
    excludes: &[&str],
) -> SkipReason
where
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
    TMultiplier: Multiplier<TActionAsync, TAction>,
{
    // todo - add tests to make sure this is not changed and an iterator is not reused (without reset) for multiple checks
    // (is what the wrapping closure is for, could be done with clones, but that feels easier to mess up)
    let active_names = || active_multipliers.iter().map(|m| m.name());

    let excluded = active_names().find(|multiplier| excludes.contains(multiplier));
    if let Some(excluded) = excluded {
        return SkipReason::Skip(format!("multiplier is excluded. Multiplier: {excluded}"));
    }

    let missing_include = includes
        .iter()
        .find(|&&include| !active_names().any(|multiplier| multiplier == include));
    if let Some(missing_include) = missing_include {
        return SkipReason::Skip(format!(
            "required multiplier is not included. Multiplier: {missing_include}"
        ));
    }

    let action_skipped_multiplier = active_multipliers
        .iter()
        .find_map(|multiplier| multiplier.should_skip(actions).into());
    if let Some(reason) = action_skipped_multiplier {
        return reason;
    }

    SkipReason::NoSkip
}

/// Returns the set of multipliers that should be used to mutate test [action](crate::action::Action) sets.
///
/// Selecting them from the full set of available [Multipliers](Multiplier) using the given name of an
/// environment variable that may contain a comma-seperated list of multiplier [names](Multiplier::NAME).
pub fn get_active_multipliers<'a, TState, TAction, TActionAsync, TMultiplier>(
    env_name: &str,
    possible_multipliers: &'a [&'a TMultiplier],
) -> Vec<&'a TMultiplier>
where
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
    TMultiplier: Multiplier<TActionAsync, TAction>,
{
    let names = std::env::var(env_name).unwrap_or_default();
    let names = names.split(',').collect::<Vec<&str>>();

    get_matching_multipliers::<TState, TAction, TActionAsync, TMultiplier>(
        &names,
        possible_multipliers,
    )
}

/// Returns the set of multipliers that should be used to mutate test [action](crate::action::Action) sets.
///
/// Selecting them from the full set of available [Multipliers](Multiplier) using the given set of multiplier
/// [names](Multiplier::NAME).
pub fn get_matching_multipliers<'a, TState, TAction, TActionAsync, TMultiplier>(
    names: &[&str],
    possible_multipliers: &'a [&'a TMultiplier],
) -> Vec<&'a TMultiplier>
where
    TAction: Action,
    TActionAsync: ActionAsync,
    TMultiplier: Multiplier<TActionAsync, TAction>,
{
    possible_multipliers
        .iter()
        .filter(|possible_match| names.iter().any(|name| possible_match.name() == *name))
        .map(|m| *m)
        .collect()
}

/// Applies the given set of [Multipliers](Multiplier) against the given set of actions.
///
/// [Multipliers](Multiplier) will be applied in the order that they appear in the given set.  This can matter.
pub fn apply<TState: Clone, TAction, TActionAsync, TMultiplier>(
    actions: &[ActionInstance<TActionAsync, TAction>],
    active_multipliers: &[&TMultiplier],
) -> Vec<ActionInstance<TActionAsync, TAction>>
where
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
    TMultiplier: Multiplier<TActionAsync, TAction>,
{
    let mut result = actions.to_vec();

    for active_multiplier in active_multipliers {
        result = active_multiplier.apply(&result);
    }

    result
}
