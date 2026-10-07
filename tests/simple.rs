#[cfg(test)]
mod tests {
    use testo::action::{Action, Actions, ActionMarker};

    #[derive(Default)]
    struct State {
        sum: i32,
    }

    #[derive(Clone)]
    struct SumAction{
        value: i32,
    }

    #[derive(Clone)]
    struct SubtractAction{
        value: i32,
    }

    impl Action<State> for SumAction {
        fn execute(&self, state: &mut State) {
            state.sum += self.value;
        }
    }
 
    impl Action<State> for SubtractAction {
        fn execute(&self, state: &mut State) {
            state.sum -= self.value;
        }
    }

    impl ActionMarker for SubtractAction{}
    impl ActionMarker for SumAction{}

    #[test]
    fn sync() {
        let mut state: State = Default::default();

        let actions: &Actions<State> = &[
            SumAction{value: 20}.into(), 
            SubtractAction{value: 2}.into(),
        ];

        for action in actions.iter() {
            action.execute(&mut state);
        }

        assert_eq!(state.sum, 18);
    }
}
