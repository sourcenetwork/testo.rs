use crate::{
    action::{Action, ActionAsync, ActionInstance},
    multiplier::SkipReason::NoSkip,
};

pub trait Multiplier<TActionAsync, TAction = TActionAsync>
where
    TAction: Action,
    TActionAsync: ActionAsync,
{
    const NAME: &str;

    fn name(&self) -> &str {
        Self::NAME
    }

    fn apply(
        &self,
        actions: &[ActionInstance<TActionAsync, TAction>],
    ) -> Vec<ActionInstance<TActionAsync, TAction>>;

    fn should_skip(&self, _actions: &[ActionInstance<TActionAsync, TAction>]) -> SkipReason {
        NoSkip
    }
}

pub enum SkipReason {
    NoSkip,
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

// todo - this is likely to be converted to a derive macro in the near future.
//
// Either way, its usage is completely optional.
#[macro_export]
macro_rules! multipliers {
    ($actions:ident, $actions_async:ident, $( $multiplier:ident ),*) => {
        pub enum Multipliers {
            $($multiplier($multiplier),)*
        }

        // todo - long term, this should really be a derive macro, but thats a pita right now, might get away without it for initial release
        impl Multiplier<$actions, $actions_async> for Multipliers {
            const NAME: &str = "MultiplierEnum"; // todo - this is ugly, reconsider

            fn name(&self) -> &str {
                match self {
                    $(Multipliers::$multiplier(m) => m.name(),)*
                }
            }

            fn apply(
                &self,
                actions: &[ActionInstance<$actions_async, $actions>],
            ) -> Vec<ActionInstance<$actions_async, $actions>> {
                match self {
                    $(Multipliers::$multiplier(m) => m.apply(actions),)*
                }
            }
        }

        // todo - also implementing `from` might be more consistent?
    }
}
