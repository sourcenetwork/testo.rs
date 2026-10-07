use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use testo::Action;
use testo::action::{Action, ActionInstance, ActionMarker, execute};
use testo::multiplier::Multiplier;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Clone, Default)]
pub struct State {
    sum: Arc<AtomicI32>,
    counter: u32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct SumAction {
    value: i32,
}
impl ActionMarker for SumAction {}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct CounterAction {}
impl ActionMarker for CounterAction {}

#[derive(Clone, Debug, Action)]
#[TState = "State"]
enum Actions {
    SumAction(SumAction),
    CounterAction(CounterAction),
}

struct CounterMultiplier {}

impl Multiplier<Actions> for CounterMultiplier {
    const NAME: &str = "CounterMultiplier";

    fn apply(
        &self,
        actions: &[ActionInstance<Actions>],
    ) -> Vec<ActionInstance<Actions>> {
        let mut resultant_actions = Vec::new();
        for action in actions.iter() {
            resultant_actions.push(action.clone());
            resultant_actions.push(CounterAction {}.into());
        }

        resultant_actions
    }
}

impl Action for SumAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(self.value, Ordering::Relaxed);
    }
}

impl Action for CounterAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.counter += 1;
    }
}

#[test]
fn multiplier() {
    let state: State = Default::default();

    let actions = &[
        SumAction { value: 20 }.into(),
        SumAction { value: -2 }.into(),
        SumAction { value: -3 }.into(),
    ];

    let actions = CounterMultiplier {}.apply(actions);

    //let mut state = Arc::new(state);
    let state = execute(state, actions.as_slice());

    assert_eq!(state.sum.load(Ordering::Relaxed), 15);
    assert_eq!(state.counter, 3);
}
