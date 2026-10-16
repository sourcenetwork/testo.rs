use crate::action::{ActionInstance};

// todo - is generic a problem here?
pub trait Multiplier<TState> : ActionAwareSkipper<TState> + Sync + Send {
    // todo - try find a way to make this const whilst still permitting vtable (`Box<dyn Multiplier<TState>>`)
    fn name(&self) -> &str;
    // todo 'actions'/return type
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
    for active_multiplier in active_multipliers {
        for exclude in excludes {
            if active_multiplier.name() == *exclude {
                return SkipReason::Skip(format!("multiplier is excluded. Multiplier: {exclude}"));
            }
        }
    }

    for include in includes {
		let mut included = false;
		for multiplier in active_multipliers {
			if multiplier.name() == *include {
				included = true;
				break
			}
		}

		if !included {
			return SkipReason::Skip(format!("required multiplier is not included. Multiplier: {include}"));
		}
	}

    for multiplier in active_multipliers {
		if multiplier.should_skip(actions) {
            let name = multiplier.name();
            return SkipReason::Skip(format!("multiplier based on actions. Multiplier: {name}"));
        }
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
        .filter(|possible_match| -> bool {
            for name in names {
                if possible_match.name() == *name {
                    return  true;
                }
            };
            false
        })
        .map(|n| *n)
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
