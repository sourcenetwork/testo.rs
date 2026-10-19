use crate::{action::ActionInstance};

// todo - is generic a problem here?
pub trait Multiplier<TState> : ActionAwareSkipper<TState> + Sync + Send {
    // todo - try find a way to make this const whilst still permitting vtable (`Box<dyn Multiplier<TState>>`)
    fn name(&self) -> &str;
    fn apply(&self, actions: &[ActionInstance<TState>]) -> Vec<ActionInstance<TState>>;
}

pub enum SkipReason {
    NoSkip,
    Skip(String),
}

pub fn skip<TState>(
    actions: &[ActionInstance<TState>],
    // todo - this really only needs to be the strs, but this might be easier - consider changing type or adding overload
    active_multipliers: &[&dyn Multiplier<TState>],
    includes: &[&str],
    excludes: &[&str],
) -> SkipReason {
    // todo - add tests to make sure this is not changed and an iterator is not reused (without reset) for multiple checks
    // (is what the wrapping closure is for, could be done with clones, but that feels easier to mess up)
    let active_names = || active_multipliers.iter().map(|m| m.name());

    let excluded = active_names().find(|multiplier| excludes.contains(multiplier));
    if let Some(excluded) = excluded {
        return SkipReason::Skip(format!("multiplier is excluded. Multiplier: {excluded}"));
    }

    let missing_include = includes.iter()
        .find(|&&include| !active_names().any(|multiplier| multiplier == include));
    if let Some(missing_include) = missing_include {
        return SkipReason::Skip(format!("required multiplier is not included. Multiplier: {missing_include}"));
    }

    let action_skipped_multiplier = active_multipliers.iter()
        .find(|multiplier| multiplier.should_skip(actions));
    if let Some(action_skipped_multiplier) = action_skipped_multiplier {
        return SkipReason::Skip(format!("multiplier based on actions. Multiplier: {}", action_skipped_multiplier.name()));
    }

    SkipReason::NoSkip
}

pub fn get_active_multipliers<'a, TState>(env_name: &str, possible_multipliers: &'a [&dyn Multiplier<TState>]) -> Vec<&'a dyn Multiplier<TState>> {
    let names = std::env::var(env_name).unwrap_or_default();
    let names = names.split(',').collect::<Vec<&str>>();

    get_matching_multipliers(&names, possible_multipliers)
}

pub fn get_matching_multipliers<'a, TState>(names: &[&str], possible_multipliers: &'a [&dyn Multiplier<TState>]) -> Vec<&'a dyn Multiplier<TState>> {
    possible_multipliers.iter()
        .filter(|possible_match| {
            names.iter()
                .any(|name| possible_match.name() == *name)
        })
        .map(|m| *m)
        .collect()
}

pub fn apply<TState: Clone>(actions: &[ActionInstance<TState>], active_multipliers: &[&dyn Multiplier<TState>]) -> Vec<ActionInstance<TState>>{
    let mut result = actions.to_vec();

    for active_multiplier in active_multipliers {
        result = active_multiplier.apply(&result);
    }

    result
}

pub trait ActionAwareSkipper<TState> {
    fn should_skip(&self, _actions: &[ActionInstance<TState>]) -> bool;
}

impl<TState, TMultiplier: Multiplier<TState>> ActionAwareSkipper<TState> for TMultiplier {
    fn should_skip(&self, _actions: &[ActionInstance<TState>]) -> bool {
        false
    }
}
