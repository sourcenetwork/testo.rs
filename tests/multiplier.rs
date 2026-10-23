use testo::action::{Action, execute, ActionInstance};
use testo::{actions, actions_async};
use testo::multiplier::{Multiplier};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

#[derive(Clone)]
#[derive(Default)]
pub struct State {
    sum: Arc<AtomicI32>,
    counter: u32,
}
    impl testo::executor::DefaultMarker for State {}

#[derive(Clone, Debug)]
struct SumAction{
    value: i32,
}

#[derive(Clone, Debug)]
struct CounterAction{}

struct CounterMultiplier{}

actions!(State, SumAction, CounterAction);
actions_async!(State, SumAction);

impl Multiplier<Actions, ActionsAsync> for CounterMultiplier {
    type TState = State;
    const NAME: &str = "Multiplier";

    fn apply(&self, actions: &[ActionInstance<Actions, ActionsAsync>]) -> Vec<ActionInstance<Actions, ActionsAsync>> {
        let mut resultant_actions =  Vec::new();
        for action in actions.iter() {
            resultant_actions.push(action.clone());
            resultant_actions.push(CounterAction{}.into());
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
        SumAction{value: 20}.into(),
        SumAction{value: -2}.into(),
        SumAction{value: -3}.into(),
    ];

    let actions = CounterMultiplier{}.apply(actions);
    
    //let mut state = Arc::new(state);
    let state = execute(state, actions.as_slice());

    assert_eq!(state.sum.load(Ordering::Relaxed), 15);
    assert_eq!(state.counter, 3);
}
