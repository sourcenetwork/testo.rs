mod state {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicI32};

    #[derive(Clone)]
    #[derive(Default)]
    pub struct State {
        pub sum: Arc<AtomicI32>,
        pub counter: u32,
    }
}

mod actions {
    use testo::action::{Action};
    use testo::{actions, actions_async};
    use std::sync::atomic::{Ordering};
    use super::state;
    use super::state::State;

    #[derive(Clone, Debug)]
    pub struct SumAction{
        pub value: i32,
    }

    #[derive(Clone, Debug)]
    pub struct CounterAction{}

    #[derive(Clone, Debug)]
    pub struct QueryAction{
        pub expected_sum: i32,
        pub expected_count: u32,
    }

    actions!(State, SumAction, CounterAction, QueryAction);
    actions_async!(State, SumAction, CounterAction);

    impl Action for SumAction {
        type TState = state::State;

        fn execute(&self, state: &mut state::State) {
            state.sum.fetch_add(self.value, Ordering::Relaxed);
        }
    }

    impl Action for CounterAction {
        type TState = state::State;

        fn execute(&self, state: &mut state::State) {
            state.counter += 1;
        }
    }

    impl Action for QueryAction {
        type TState = state::State;

        fn execute(&self, state: &mut state::State) {
            assert_eq!(state.sum.load(Ordering::Relaxed), self.expected_sum);
            assert_eq!(state.counter, self.expected_count);
        }
    }
}

mod multipliers {
    use testo::multiplier::{Multiplier};
    use testo::action::{IntoAsync, ActionInstance};
    use super::state;
    use super::actions;

    const MAGNITUDE: i32 = 2;
    pub struct MultiplyMultiplier{}

    pub enum Multipliers {
        MultiplyMultiplier(MultiplyMultiplier)
    }

    impl Multiplier<actions::Actions, actions::ActionsAsync> for Multipliers {
        type TState = state::State;
        const NAME: &str = "Multiplier";// todo - this is ugly, reconsider

        fn name(&self) -> &str {
            match self {
                Multipliers::MultiplyMultiplier(m) => m.name()
            }
        }

        fn apply(&self, actions: &[ActionInstance<actions::Actions, actions::ActionsAsync>]) -> Vec<ActionInstance<actions::Actions, actions::ActionsAsync>> {
            match self {
                Multipliers::MultiplyMultiplier(m) => m.apply(actions)
            }
        }
    }

    impl Multiplier<actions::Actions, actions::ActionsAsync> for MultiplyMultiplier {
        type TState = state::State;
        const NAME: &str = "MultiplyMultiplier";

        fn apply(&self, actions: &[ActionInstance<actions::Actions, actions::ActionsAsync>]) -> Vec<ActionInstance<actions::Actions, actions::ActionsAsync>> {
            let mut resultant_actions =  Vec::new();
            for action in actions.iter() {
                let sum_action;
                let is_async;
                match action { 
                    ActionInstance::Async(async_action) => {
                        sum_action = match async_action {
                            actions::ActionsAsync::SumAction(a) => Some(a),
                            _ => None,
                        };
                        is_async = true;
                    },
                    ActionInstance::Sync(sync_action) => {
                        sum_action = match sync_action {
                            actions::Actions::SumAction(a) => Some(a),
                            _ => None,
                        };
                        is_async = false;
                    },
                };

                let new_action = match sum_action {
                    Some(s) => {
                        let action = actions::SumAction{
                            value: MAGNITUDE * s.value,
                        };
                        if is_async{
                            action.into_async()
                        } else {
                            action.into()
                        }
                    },
                    None => {
                        action.clone()
                    }
                };
                resultant_actions.push(new_action);
            }
            resultant_actions
        }
    }
}

mod tests {
    use testo::action::{self, ActionInstance};
    use testo::multiplier::{self, SkipReason};
    use super::actions;
    use super::multipliers::{Multipliers};
    use std::env;
    use super::state;

    static GLOBAL_MULTIPLIERS: [Multipliers; 0] =[

    ];

    #[derive(Default)]
    pub struct Test<'a> {
        pub includes: &'a[&'a str],
        pub excludes: &'a[&'a str],
        pub actions: &'a [ActionInstance<actions::Actions, actions::ActionsAsync>],
    }

    impl Test<'_> {
        pub fn execute(&self, crate_specific_multipliers: &[Multipliers]) {
            let action_env_key = "EXAMPLE_MULTIPLIER_ENV_NAME";
            unsafe {
                // Test code only, don't actually do this :)
                env::set_var(action_env_key, "MultiplyMultiplier,");
            }

            let mut possible_multipliers = Vec::new();
            possible_multipliers.extend(&GLOBAL_MULTIPLIERS);
            possible_multipliers.extend(crate_specific_multipliers);

            let active_multipliers = multiplier::get_active_multipliers(action_env_key, possible_multipliers.as_slice());
            match multiplier::skip(&self.actions, &active_multipliers, self.includes, self.excludes) {
                SkipReason::Skip(reason) => {
                    println!("Skipping test. Reason: {reason}");
                    return;
                },
                SkipReason::NoSkip => {},
            };

            let actions = multiplier::apply(&self.actions, &active_multipliers);

            let state: state::State = Default::default();
            action::execute(state, actions.as_slice());
        }
    }
}

use testo::action::{IntoAsync};
use testo::multiplier::{Multiplier};
    use multipliers::{Multipliers};

static CRATE_LOCAL_MULTIPLIERS: [Multipliers; 1] = [
    multipliers::Multipliers::MultiplyMultiplier(multipliers::MultiplyMultiplier{}),
];

#[test]
fn full() {
    let test = tests::Test{
        actions: &[
            actions::CounterAction{}.into(),
            actions::SumAction{value: 20}.into_async(),
            actions::SumAction{value: -2}.into_async(),
            actions::CounterAction{}.into(),
            actions::CounterAction{}.into(),
            actions::SumAction{value: -3}.into_async(),
            actions::CounterAction{}.into(),
            actions::QueryAction{
                expected_sum: 30,
                expected_count: 4,
            }.into(),
        ],
        ..Default::default()
    };

    test.execute(&CRATE_LOCAL_MULTIPLIERS);
}

#[test]
fn full_base() {
    let test = tests::Test{
        actions: &[
            actions::CounterAction{}.into(),
            actions::SumAction{value: 20}.into_async(),
            actions::SumAction{value: -2}.into_async(),
            actions::CounterAction{}.into(),
            actions::CounterAction{}.into(),
            actions::SumAction{value: -3}.into_async(),
            actions::CounterAction{}.into(),
            actions::QueryAction{
                expected_sum: 15,
                expected_count: 4,
            }.into(),
        ],
        ..Default::default()
    };

    test.execute(&[]);
}

#[test]
fn full_with_includes() {
    let test = tests::Test{
        includes: &[multipliers::MultiplyMultiplier{}.name()],
        actions: &[
            actions::CounterAction{}.into(),
            actions::SumAction{value: 20}.into_async(),
            actions::SumAction{value: -2}.into_async(),
            actions::CounterAction{}.into(),
            actions::CounterAction{}.into(),
            actions::SumAction{value: -3}.into_async(),
            actions::CounterAction{}.into(),
            actions::QueryAction{
                expected_sum: 30,
                expected_count: 4,
            }.into(),
        ],
        ..Default::default()
    };

    test.execute(&CRATE_LOCAL_MULTIPLIERS);
}

#[test]
fn full_with_missing_includes() {
    let test = tests::Test{
        includes: &[multipliers::MultiplyMultiplier{}.name()],
        actions: &[
            actions::CounterAction{}.into(),
            actions::SumAction{value: 20}.into_async(),
            actions::SumAction{value: -2}.into_async(),
            actions::CounterAction{}.into(),
            actions::CounterAction{}.into(),
            actions::SumAction{value: -3}.into_async(),
            actions::CounterAction{}.into(),
            actions::QueryAction{
                // This test will always fail, if it is executed,
                // however we are running with `includes` expecting 
                // a multiplier that is only provided by the local
                // crate, leading this test to be skipped. Commenting
                // out the `includes` will result in this test failing.
                expected_sum: 999999,
                expected_count: 4,
            }.into(),
        ],
        ..Default::default()
    };

    test.execute(&[]);
}
