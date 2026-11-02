use testo::derive::{Action, Multiplier};
use testo::multiplier::SkipReason;
use testo::prelude::*;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Clone, Default)]
pub struct State {}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct SumAction {}

#[derive(Clone, Debug, Action)]
#[TState = "State"]
enum Actions {
    SumAction(SumAction),
}

impl Action for SumAction {
    type TState = State;

    fn execute(&self, _state: &mut State) {
        todo!()
    }
}

pub struct CounterMultiplier {
    should_skip: SkipReason,
}

impl Multiplier<Actions> for CounterMultiplier {
    const NAME: &str = "CounterMultiplier";

    fn apply(&self, _actions: &[ActionInstance<Actions>]) -> Vec<ActionInstance<Actions>> {
        todo!()
    }

    fn should_skip(&self, _actions: &[ActionInstance<Actions, Actions>]) -> SkipReason {
        self.should_skip.clone()
    }
}

#[derive(Multiplier)]
#[TActionAsync = "Actions"]
pub enum Multipliers {
    CounterMultiplier(CounterMultiplier),
}

#[test]
fn multiplier_enum_should_skip_returns_inner_result_no_skip() {
    let multiplier = Multipliers::CounterMultiplier(CounterMultiplier {
        should_skip: SkipReason::NoSkip,
    });

    let result = multiplier.should_skip(&[]);

    assert_eq!(result, SkipReason::NoSkip);
}

#[test]
fn multiplier_enum_should_skip_returns_inner_result_skip() {
    let multiplier = Multipliers::CounterMultiplier(CounterMultiplier {
        should_skip: SkipReason::Skip("test".to_string()),
    });

    let result = multiplier.should_skip(&[]);

    assert_eq!(result, SkipReason::Skip("test".to_string()));
}
