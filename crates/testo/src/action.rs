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

/// *The* core trait at the heart of Testo.
///
/// Every 'thing' that your test needs to *do* should be expressed via actions.
///
/// Ideally, each concrete implementation of this type should do exactly one, easy to understand, thing - to the test subject(s).  They may mutate
/// [TState](Self::TState) during the process, and they may assert the results of the action that they performed.  They should not mutate or directly
/// interact with other Actions.
///
/// ## Async
///
/// An [Action] that also implements [Sync] will auto-implement [ActionAsync], allowing tests to specify that they wish that action instance to execute
/// concurrently alongside similarly configured, adjacent actions.
///
/// ## Serde
///
/// If the crate feature `"serde"` is enabled, this trait will require the implementation of
/// [Serialize](https://docs.rs/serde/latest/serde/trait.Serialize.html).  This is useful if your testing library wishes to ensure that all tests may be
/// serializable.
#[cfg(not(feature = "serde"))]
pub trait Action: Clone + Any + Debug {
    /// The type of state that this Action executes against.
    ///
    /// All actions within a test must operate against the same [TState](Self::TState).
    type TState;

    /// Executes the action, doing whatever the action needs to do, including calling into the test subject and asserting the results, and mutating
    /// [TState](Self::TState).
    fn execute(&self, state: &mut Self::TState);
}

/// ActionAsync represents an action that may be executed concurrently alongside other
/// ActionAsync actions.
///
/// [IntoAsync] and [FromAsync] are auto-implemented for types implementing this trait
/// and can be used to configure which actions in a test declaration are to be executed
/// concurrently.
pub trait ActionAsync: Action + Sync {}
impl<TAction: Action + Sync> ActionAsync for TAction {}

/// ActionInstance wraps a single [Action] instance, allowing the instance to specify whether it
/// should execute in serial ([Sync](Self::Sync)), or concurrently ([Async](Self::Async)) alongside other
/// adjacent [Async](Self::Async) elements within the same set declared by a test.
///
/// Typically, `TAction` and `TActionAsync` will not be a concrete [Action], but instead will be a test-library
/// specific enum, declaring the possible Action variants.
///
/// `TActionAsync` will default to `TAction` in order to support the presumed norm - where all possible test actions
/// implement [ActionAsync].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum ActionInstance<TActionAsync, TAction = TActionAsync>
where
    TActionAsync: ActionAsync,
    TAction: Action,
{
    /// Actions wrapped in this variant will execute serially.
    ///
    /// Any [Async](Self::Async) actions declared prior to this action in the set will have their execution(s)
    /// waited on before this action, and subsequent actions begin execution.  This means that the presence of
    /// a [Sync](Self::Sync) action will act as a seperator between two blocks of [Async](Self::Async) actions.
    ///
    /// ## Example
    /// ```
    /// # use testo::prelude::*;
    /// # use testo::action::execute;
    /// # use testo::derive::Action;
    /// #
    /// # #[derive(Clone, Debug)]
    /// # struct ExampleAction{}
    /// #
    /// # impl Action for ExampleAction {
    /// #   type TState = i32;
    /// #   fn execute(&self, state: &mut i32){}
    /// # }
    /// #
    /// # #[derive(Clone, Debug, Action)]
    /// # #[TState = "i32"]
    /// # enum Actions {
    /// #   ExampleAction(ExampleAction),
    /// # }
    /// #
    /// # fn example_test() {
    /// #   let state = 0;
    /// #
    ///      let actions: &[ActionInstance<Actions>] = &[
    ///         // These first two actions will execute concurrently.
    ///         ExampleAction{}.into_async(),
    ///         ExampleAction{}.into_async(),
    ///         // The first two actions will be waited on before executing this action.
    ///         ExampleAction{}.into(),
    ///         // These last two actions will execute concurrently, after the previous
    ///         // three actions have completed.
    ///         ExampleAction{}.into_async(),
    ///         ExampleAction{}.into_async(),
    ///      ];
    ///
    ///     execute(state, actions);
    /// # }
    /// ```
    Sync(TAction),
    /// Actions wrapped in this variant will execute concurrently alongside other adjacent [Async](Self::Async) actions.
    ///
    /// For more information and an example, see [Sync](Self::Sync).
    Async(TActionAsync),
}

/// Executes the given set of actions against the given state.
///
/// The new state is returned.
///
/// ## Async
///
/// [Async](ActionInstance::Async) actions will execute concurrently alongside similar adjacent members.  See [Sync](ActionInstance::Sync)
/// for more information.
///
/// Mutations made by [Async](ActionInstance::Async) [actions](ActionAsync) to unprotected members of `TState` will be discarded.
///
/// Currently, each [Async](ActionInstance::Async) action will spawn its own thread, this is very likely to change soon with
/// <https://github.com/sourcenetwork/testo.rs/issues/3>.
pub fn execute<TState, TAction, TActionAsync>(
    state: TState,
    actions: &[ActionInstance<TActionAsync, TAction>],
) -> TState
where
    TState: Clone + Sync + Send,
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
{
    let mut state = Arc::new(state);

    execute_arc(&mut state, actions);

    // We cannot borrow the input `state` parameter as mutable, as we need to be able concurrently mutate elements within it,
    // which we do by cloning it - this deliberately discards any concurrent mutations made to inner elements not wrapped in
    // stuff like `Arc`s.
    //
    // Synchronous action executions avoid cloning of `state`, and do not need to protect any mutating inner elements with `Arc`.
    state.deref().clone()
}

fn execute_arc<TState, TAction, TActionAsync>(
    state: &mut Arc<TState>,
    actions: &[ActionInstance<TActionAsync, TAction>],
) where
    TState: Clone + Sync + Send,
    TAction: Action<TState = TState>,
    TActionAsync: ActionAsync<TState = TState>,
{
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

impl<TAction, TInner, TActionAsync> From<TInner> for ActionInstance<TActionAsync, TAction>
where
    TAction: Action,
    TInner: Into<TAction> + Action,
    TActionAsync: ActionAsync,
{
    fn from(value: TInner) -> ActionInstance<TActionAsync, TAction> {
        ActionInstance::Sync(value.into())
    }
}

/// Like [Into], but for returning the [ActionInstance::Async] variant.
///
/// Paired with [FromAsync].
pub trait IntoAsync<TAction, TActionAsync>
where
    TAction: Action,
    TActionAsync: ActionAsync,
{
    fn into_async(self) -> ActionInstance<TActionAsync, TAction>;
}

/// Like [From], but for returning the [ActionInstance::Async] variant.
///
/// Paired with [FromAsync].
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
