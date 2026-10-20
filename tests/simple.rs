use testo::action::{Action};
use testo::actions;

#[derive(Default)]
pub struct State {
    sum: i32,
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
        state.sum += self.value;
    }
}

impl Action for SubtractAction {
    type TState = State;

    fn execute(&self, state: &mut State) {
        state.sum -= self.value;
    }
}
actions!(State, SumAction, SubtractAction);
/* 
#[derive(Clone, Debug)]
enum Actions {
    SumAction(SumAction),
    SubtractAction(SubtractAction),
}

impl Action<State> for Actions {
    fn execute(&self, state: &mut State) {
        match self {
            Self::SumAction(a) => a.execute(state),
            Self::SubtractAction(a) => a.execute(state),
        }
    }
}

impl From<SumAction> for Actions{
    fn from(value: SumAction) -> Self {
        Actions::SumAction(value)
    }
}

impl From<SubtractAction> for Actions{
    fn from(value: SubtractAction) -> Self {
        Actions::SubtractAction(value)
    }
}
*/

#[test]
fn sync() {
    let mut state: State = Default::default();

    let actions: &[Actions] = &[
        SumAction{value: 20}.into(), 
        SubtractAction{value: 2}.into(),
    ];

    for action in actions.iter() {
        action.execute(&mut state);
    }

    assert_eq!(state.sum, 18);
}
