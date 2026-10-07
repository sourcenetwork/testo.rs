#[cfg(test)]
mod tests {
    use testo::action::{Action, ActionMarker, IntoAsync, execute};
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

    impl Action<State> for SumAction {
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

    impl Action<State> for CounterAction {
        fn execute(&self, state: &mut State) {
            state.counter += 1;
        }
    }

    #[test]
    fn mixed_concurrency() {
        let state: State = Default::default();

        let actions = &[
            CounterAction{}.into(),
            SumAction{value: 20}.into_async(),
            SumAction{value: -2}.into_async(),
            CounterAction{}.into(),
            CounterAction{}.into(),
            SumAction{value: -3}.into_async(),
            CounterAction{}.into(),
        ];
        
        let mut state = Arc::new(state);
        execute(&mut state, actions);

        assert_eq!(state.sum.load(Ordering::Relaxed), 15);
        assert_eq!(state.counter, 4);
    }
}
