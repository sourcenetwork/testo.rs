use testo::action::{Action};
use testo::{actions, actions_async};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Clone)]
#[derive(Default)]
pub struct State {
    sum: Arc<AtomicI32>,
}

#[derive(Clone, Debug)]
struct SumAction{
    value: i32,
}

#[derive(Clone, Debug)]
struct SubtractAction{
    value: i32,
}

impl Action for SumAction {
    type TState = State;
    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(self.value, Ordering::Relaxed);
    }
}

actions!(State, SumAction, SubtractAction);
actions_async!(State, SumAction, SubtractAction);

/*
// todo - I really struggled to find a way to auto-implement this at the trait level,
// have another go at somepoint, and maybe check in with Theo.
impl Into<Arc<Box<dyn ActionAsync<State>>>> for SumAction {
    fn into(self) -> Arc<Box<dyn ActionAsync<State>>> {
        Arc::new(self.into())
    }
}
*/
impl Action for SubtractAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(-self.value, Ordering::Relaxed);
    }
}
/*
// todo - I really struggled to find a way to auto-implement this at the trait level,
// have another go at somepoint, and maybe check in with Theo.
impl Into<Arc<Box<dyn ActionAsync<State>>>> for SubtractAction {
    fn into(self) -> Arc<Box<dyn ActionAsync<State>>> {
        Arc::new(self.into())
    }
}
 */
#[test]
fn multithread() {
    let state: State = Default::default();

    let actions: &[ActionsAsync] = &[
        SumAction{value: 20}.into(), 
        SubtractAction{value: 2}.into(),
        SubtractAction{value: 2}.into(),
        SubtractAction{value: 2}.into(),
    ];
    
    let state = Arc::new(state);
    thread::scope(|scope| {
        actions.iter().for_each(|action| {
            let mut state = Arc::clone(&state);
            scope.spawn(move || {
                let state = Arc::make_mut(&mut state);
                action.execute(state)
            });
        });
    });

    assert_eq!(state.sum.load(Ordering::Relaxed), 14);
}
