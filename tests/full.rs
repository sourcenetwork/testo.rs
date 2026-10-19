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
    use testo::action::{Action, ActionMarker};
    use std::sync::atomic::{Ordering};
    use super::state;

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

    impl ActionMarker for CounterAction{}
    impl ActionMarker for SumAction{}
    impl ActionMarker for QueryAction{}

    impl Action<state::State> for SumAction {
        fn execute(&self, state: &mut state::State) {
            state.sum.fetch_add(self.value, Ordering::Relaxed);
        }
    }

    impl Action<state::State> for CounterAction {
        fn execute(&self, state: &mut state::State) {
            state.counter += 1;
        }
    }

    impl Action<state::State> for QueryAction {
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

    impl Multiplier<state::State> for MultiplyMultiplier {
        fn name(&self) -> &str {
            "MultiplyMultiplier"
        }

        fn apply(&self, actions: &[ActionInstance<state::State>]) -> Vec<ActionInstance<state::State>> {
            let mut resultant_actions =  Vec::new();
            for action in actions.iter() {
                let sum_action;
                let is_async;
                match action { 
                    ActionInstance::Async(async_action) => {
                        sum_action = async_action.downcast_ref::<actions::SumAction>();
                        is_async = true;
                    },
                    ActionInstance::Sync(sync_action) => {
                        sum_action = sync_action.downcast_ref::<actions::SumAction>();
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
    use testo::multiplier::{self, Multiplier, SkipReason};
    use std::env;
    use super::state;

    static GLOBAL_MULTIPLIERS: [&dyn Multiplier<state::State>; 0] =[

    ];

    #[derive(Default)]
    pub struct Test<'a> {
        pub includes: &'a[&'a str],
        pub excludes: &'a[&'a str],
        pub actions: &'a [ActionInstance<state::State>],
    }

    impl Test<'_> {
        pub fn execute(&self, crate_specific_multipliers: &[&dyn Multiplier<state::State>]) {
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

static CRATE_LOCAL_MULTIPLIERS: [&dyn Multiplier<state::State>; 1] = [
    &multipliers::MultiplyMultiplier{},
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
