use testo::action::{Action, ActionMarker, execute, ActionInstance};
use testo::multiplier::{Multiplier};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

#[derive(Clone)]
#[derive(Default)]
struct State {
    sum: Arc<AtomicI32>,
    counter: u32,
}

#[derive(Clone)]
struct SumAction{
    value: i32,
}

#[derive(Clone)]
struct CounterAction{}

impl ActionMarker for CounterAction{}
impl ActionMarker for SumAction{}

struct CounterMultiplier{}

impl Multiplier<State> for CounterMultiplier {
    fn name(&self) -> &str {
        "CounterMultiplier"
    }

    fn apply(&self, actions: &[ActionInstance<State>]) -> Vec<ActionInstance<State>> {
        let mut resultant_actions =  Vec::new();
        for action in actions.iter() {
            resultant_actions.push(action.clone());
            resultant_actions.push(CounterAction{}.into());
        }

        resultant_actions
    }
}

impl Action<State> for SumAction {
    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(self.value, Ordering::Relaxed);
    }
}

impl Action<State> for CounterAction {
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
