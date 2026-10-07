#[cfg(test)]
mod tests {
    use testo::action::{Action, ActionAsync, ActionsAsync, ActionMarker};
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[derive(Clone)]
    #[derive(Default)]
    struct State {
        sum: Arc<AtomicI32>,
    }

    #[derive(Clone)]
    struct SumAction{
        value: i32,
    }

    #[derive(Clone)]
    struct SubtractAction{
        value: i32,
    }

    impl ActionMarker for SubtractAction{}
    impl ActionMarker for SumAction{}

    impl Action<State> for SumAction {
        fn execute(&self, state: &mut State) {
            state.sum.fetch_add(self.value, Ordering::Relaxed);
        }
    }

    // todo - I really struggled to find a way to auto-implement this at the trait level,
    // have another go at somepoint, and maybe check in with Theo.
    impl Into<Arc<Box<dyn ActionAsync<State>>>> for SumAction {
        fn into(self) -> Arc<Box<dyn ActionAsync<State>>> {
            Arc::new(self.into())
        }
    }
 
    impl Action<State> for SubtractAction {
        fn execute(&self, state: &mut State) {
            state.sum.fetch_add(-self.value, Ordering::Relaxed);
        }
    }

    // todo - I really struggled to find a way to auto-implement this at the trait level,
    // have another go at somepoint, and maybe check in with Theo.
    impl Into<Arc<Box<dyn ActionAsync<State>>>> for SubtractAction {
        fn into(self) -> Arc<Box<dyn ActionAsync<State>>> {
            Arc::new(self.into())
        }
    }

    #[test]
    fn multithread() {
        let state: State = Default::default();

        let actions: &ActionsAsync<State> = &[
            SumAction{value: 20}.into(), 
            SubtractAction{value: 2}.into(),
            SubtractAction{value: 2}.into(),
            SubtractAction{value: 2}.into(),
        ];
        
        let state = Arc::new(state);
        let handles: Vec<_> = actions.iter().map(|action| {
            let mut state = Arc::clone(&state);
            let action = Arc::clone(&action);
            thread::spawn(move || 
                action.execute(&mut state)
            )
        })
        .collect();

        handles.into_iter().for_each(|h| {
            h.join().unwrap();
        });

        assert_eq!(state.sum.load(Ordering::Relaxed), 14);
    }
}
