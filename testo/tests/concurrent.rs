use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread;
use testo::Action;
use testo::action::Action;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Clone, Default)]
pub struct State {
    sum: Arc<AtomicI32>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct SumAction {
    value: i32,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
struct SubtractAction {
    value: i32,
}

#[derive(Clone, Debug, Action)]
#[TState = "State"]
enum Actions {
    SumAction(SumAction),
    SubtractAction(SubtractAction),
}

impl Action for SumAction {
    type TState = State;
    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(self.value, Ordering::Relaxed);
    }
}

impl Action for SubtractAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum.fetch_add(-self.value, Ordering::Relaxed);
    }
}

#[test]
fn multithread() {
    let state: State = Default::default();

    let actions: &[Actions] = &[
        SumAction { value: 20 }.into(),
        SubtractAction { value: 2 }.into(),
        SubtractAction { value: 2 }.into(),
        SubtractAction { value: 2 }.into(),
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
