use std::any::Any;
use std::fmt::Debug;
use std::marker::Sync;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "serde")]
pub trait Action: Clone + Any + Debug + Serialize {
    type TState;
    fn execute(&self, state: &mut Self::TState);
}

#[cfg(not(feature = "serde"))]
pub trait Action: Clone + Any + Debug {
    type TState;
    fn execute(&self, state: &mut Self::TState);
}

pub trait ActionAsync: Action + Sync {}
impl<TAction: Action + Sync> ActionAsync for TAction {}

// The ActionMarker trait can be used to flag an Action as an Action
// without the compiler freaking out when declaring auto-implemented functions.
//
// Its implmentation is completely optional, however implmenting it will
// provide the implementation of `into` (via `From`) for `ActionInstance`,
// allowing `FooAction{...}.into()` when declaring tests.
//
// In the near future it might be that this will be provided via
// the `#[derive(Action)]` macro.
pub trait ActionMarker {}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum ActionInstance<TActionAsync: ActionAsync, TAction: Action=TActionAsync> {
    Sync(TAction),
    Async(TActionAsync),
}

// todo - either manual implmentation, or TState must implement Debug (decide)
// todo - below implementation is intelisense and needs checking.
impl<TAction: Action, TActionAsync: ActionAsync> Debug for ActionInstance<TActionAsync, TAction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(arg0) => f.debug_tuple("Sync").field(arg0).finish(),
            Self::Async(arg0) => f.debug_tuple("Async").field(arg0).finish(),
        }
    }
}

pub fn execute<
    TState: Clone + Sync + Send,
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
>(
    state: TState,
    actions: &[ActionInstance<TActionAsync, TAction>],
) -> TState {
    let mut state = Arc::new(state);

    execute_arc(&mut state, actions);

    // We cannot borrow the input `state` parameter as mutable, as we need to be able concurrently mutate elements within it,
    // which we do by cloning it - this deliberately discards any concurrent mutations made to inner elements not wrapped in
    // stuff like `Arc`s.
    //
    // Synchronous action executions avoid cloning of `state`, and do not need to protect any mutating inner elements with `Arc`.
    state.deref().clone()
}

fn execute_arc<
    TState: Clone + Sync + Send,
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
>(
    state: &mut Arc<TState>,
    actions: &[ActionInstance<TActionAsync, TAction>],
) {
    thread::scope(|scope| {
        let mut handles = Vec::new();

        for action in actions.iter() {
            match action {
                ActionInstance::Async(a) => {
                    let mut state = Arc::clone(&state);
                    // todo - this is being reworked in the immediate future here:
                    // https://github.com/sourcenetwork/testo.rs/issues/3
                    let handle = scope.spawn(move || {
                        let state = Arc::make_mut(&mut state);
                        a.execute(state)
                    });

                    handles.push(handle);
                }
                ActionInstance::Sync(a) => {
                    handles.into_iter().for_each(|h| {
                        h.join().unwrap();
                    });
                    handles = Vec::new();

                    let state = Arc::get_mut(state).unwrap_or_else(|| {
                        panic!("test thread leaked, programming error, likely in this function")
                    });
                    a.execute(state);
                }
            }
        }
    });
}

impl<TAction: Action, TInner: Into<TAction> + Action + ActionMarker, TActionAsync: ActionAsync>
    From<TInner> for ActionInstance<TActionAsync, TAction>
{
    fn from(value: TInner) -> ActionInstance<TActionAsync, TAction> {
        ActionInstance::Sync(value.into())
    }
}

pub trait IntoAsync<TAction, TActionAsync>
where
    TAction: Action,
    TActionAsync: ActionAsync,
{
    fn into_async(self) -> ActionInstance<TActionAsync, TAction>;
}

pub trait FromAsync<TAction, TActionAsync, TValue>
where
    TAction: Action,
    TActionAsync: ActionAsync,
{
    fn from_async(value: TValue) -> ActionInstance<TActionAsync, TAction>;
}

impl<TAction, TActionAsync, TValue> FromAsync<TAction, TActionAsync, TValue> for TValue
where
    TActionAsync: ActionAsync + From<TValue>,
    TAction: Action,
{
    fn from_async(value: TValue) -> ActionInstance<TActionAsync, TAction> {
        ActionInstance::Async(value.into())
    }
}

impl<TAction, TActionAsync, TValue> IntoAsync<TAction, TActionAsync> for TValue
where
    TValue: FromAsync<TAction, TActionAsync, TValue>,
    TAction: Action,
    TActionAsync: ActionAsync,
{
    fn into_async(self) -> ActionInstance<TActionAsync, TAction> {
        TValue::from_async(self)
    }
}
