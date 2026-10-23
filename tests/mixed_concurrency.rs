use testo::action::{Action, ActionInstance, IntoAsync, execute};
use testo::{actions, actions_async};
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

actions!(State, SumAction, CounterAction);
actions_async!(State, SumAction);

impl Action for SumAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(self.value, Ordering::Relaxed);
        // Note - this is not protected by an inner Arc, and so is set only on the thread-specific cloned-instance of `state`,
        // it should not be reflected in the final result.
        //
        // It also represents a potential future source of confusion when consuming this library, but I havent yet found a better
        // alternative.
        state.counter += 1;
    }
}

impl Action for CounterAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.counter += 1;
    }
}

#[test]
fn mixed_concurrency() {
    let state: State = Default::default();

    let actions: &[ActionInstance<Actions, ActionsAsync>] = &[
        CounterAction{}.into(),// todo - figure out how to remove the double into (macro execution order makes it fiddly?)
        SumAction{value: 20}.into_async(),
        SumAction{value: -2}.into_async(),
        CounterAction{}.into(),
        CounterAction{}.into(),
        SumAction{value: -3}.into_async(),
        CounterAction{}.into(),
    ];
    
    let state = execute(state, actions);

    assert_eq!(state.sum.load(Ordering::Relaxed), 15);
    assert_eq!(state.counter, 4);
}
