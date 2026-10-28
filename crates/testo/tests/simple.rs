use testo::action::Action;
use testo::derive::Action;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Default)]
pub struct State {
    sum: i32,
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

impl Action for SumAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum += self.value;
    }
}

impl Action for SubtractAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum -= self.value;
    }
}

#[derive(Clone, Debug, Action)]
#[TState = "State"]
enum Actions {
    SumAction(SumAction),
    SubtractAction(SubtractAction),
}

#[test]
fn sync() {
    let mut state: State = Default::default();

    let actions: &[Actions] = &[
        SumAction { value: 20 }.into(),
        SubtractAction { value: 2 }.into(),
    ];

    for action in actions.iter() {
        action.execute(&mut state);
    }

    assert_eq!(state.sum, 18);
}
