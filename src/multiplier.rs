use crate::action::{ActionInstance, Action, ActionAsync};

// todo - is generic a problem here?
pub trait Multiplier<TAction, TActionAsync> : ActionAwareSkipper<TAction, TActionAsync> + Sync + Send 
where TAction: Action,
TActionAsync: ActionAsync{
    type TState;// where TAction: Action<TState=Self::TState>;
    const NAME: &str;

    fn name(&self) -> &str { Self::NAME }
    fn apply(&self, actions: &[ActionInstance<TAction, TActionAsync>]) -> Vec<ActionInstance<TAction, TActionAsync>>;
}

pub enum SkipReason {
    NoSkip,
    Skip(String),
}

pub fn skip<TState, TAction, TActionAsync, TMultiplier> (
    actions: &[ActionInstance<TAction, TActionAsync>],
    // todo - this really only needs to be the strs, but this might be easier - consider changing type or adding overload
    active_multipliers: &[&TMultiplier],
    includes: &[&str],
    excludes: &[&str],
) -> SkipReason 
where TAction: Action<TState=TState>,
TActionAsync: ActionAsync<TState=TState>,
TMultiplier: Multiplier<TAction, TActionAsync>{
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

pub fn get_active_multipliers<'a, TState, TAction, TActionAsync, TMultiplier>(env_name: &str, possible_multipliers: &'a [&'a TMultiplier]) -> Vec<&'a TMultiplier> 
where TAction: Action<TState = TState>,
TActionAsync: ActionAsync<TState = TState>,
TMultiplier: Multiplier<TAction, TActionAsync>{
    let names = std::env::var(env_name).unwrap_or_default();
    let names = names.split(',').collect::<Vec<&str>>();

    get_matching_multipliers::<TState, TAction,TActionAsync,TMultiplier>(&names, possible_multipliers)
}

pub fn get_matching_multipliers<'a, TState, TAction, TActionAsync, TMultiplier>(names: &[&str], possible_multipliers: &'a [&'a TMultiplier]) -> Vec<&'a TMultiplier> 
where TAction: Action,
TActionAsync: ActionAsync,
TMultiplier: Multiplier<TAction, TActionAsync>{
    possible_multipliers.iter()
        .filter(|possible_match| {
            names.iter()
                .any(|name| possible_match.name() == *name)
        })
        .map(|m| *m)
        .collect()
}

pub fn apply<TState: Clone, TAction, TActionAsync, TMultiplier>(actions: &[ActionInstance<TAction, TActionAsync>], active_multipliers: &[&TMultiplier]) -> Vec<ActionInstance<TAction, TActionAsync>>
where TAction: Action<TState=TState>,
TActionAsync: ActionAsync<TState=TState>,
TMultiplier: Multiplier<TAction, TActionAsync>{
    let mut result = actions.to_vec();

    for active_multiplier in active_multipliers {
        result = active_multiplier.apply(&result);
    }

    result
}

pub trait ActionAwareSkipper<TAction, TActionAsync>
where TAction: Action,
TActionAsync: ActionAsync{
    fn should_skip(&self, _actions: &[ActionInstance<TAction, TActionAsync>]) -> bool;
}

impl<TAction, TActionAsync, TMultiplier: Multiplier<TAction, TActionAsync>> ActionAwareSkipper<TAction, TActionAsync> for TMultiplier 
where TAction: Action,
TActionAsync: ActionAsync{
    fn should_skip(&self, _actions: &[ActionInstance<TAction, TActionAsync>]) -> bool {
        false
    }
}
