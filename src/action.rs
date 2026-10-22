use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use std::marker::{Sync};
use std::thread;
use std::any::Any;

// todo - are you sure we cannot make this an enum, owned by consuming crates?  Stuff could define a generic that implements this.
// Is there really a need for an open interface on ActionInstance?
pub trait Action : Clone + Any + Debug {
    type TState;
    fn execute(&self, state: &mut Self::TState);
}

pub trait ActionAsync: Clone + Action + Sync {}
impl<TAction: Action + Sync + Clone> ActionAsync for TAction  {}

// todo - Marker used to prevent auto-trait implemntation clashes (especially From when Debug implemented)
// preferably remove, if not, properly document
pub trait ActionMarker {}

// todo - besides the horrible generics, using an enum instead of obj might prevent us from mixing actions from multiple sources.
// without some kind of wrapping.  Think more on this, especially for multipliers.
//
// note - the double generic is horrible, but so far is really only exposed to consuming crates when defining multipliers, so it's not *that* bad.
//
// todo - generics can have defaults in Rust! (this seems limited to concrete types though, I think the double is still required on funcs, it might not save much)
#[derive(Clone)]
pub enum ActionInstance<TAction: Action, TActionAsync: ActionAsync> {
    Sync(TAction),
    Async(TActionAsync),
}

// todo - either manual implmentation, or TState must implement Debug (decide)
// todo - below implementation is intelisense and needs checking.
impl<TAction: Action, TActionAsync: ActionAsync> Debug for ActionInstance<TAction, TActionAsync> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sync(arg0) => f.debug_tuple("Sync").field(arg0).finish(),
            Self::Async(arg0) => f.debug_tuple("Async").field(arg0).finish(),
        }
    }
}

// todo - implement serde for ActionInstance, but strongly consider making this a feature

pub fn execute<TState: Clone + Sync + Send, TAction: Action<TState = TState>, TActionAsync: ActionAsync<TState = TState>>(state: TState, actions: &[ActionInstance<TAction, TActionAsync>]) -> TState {
    let mut state = Arc::new(state);

    execute_a(&mut state, actions);

    // We cannot borrow the input `state` parameter as mutable, as we need to be able concurrently mutate elements within it,
    // which we do by cloning it - this deliberately discards any concurrent mutations made to inner elements not wrapped in
    // stuff like `Arc`s.
    //
    // Synchronous action executions avoid cloning of `state`, and do not need to protect any mutating inner elements with `Arc`.
    state.deref().clone()
}

// todo - name
// todo - consider making private
pub fn execute_a<TState: Clone + Sync + Send, TAction: Action<TState = TState>, TActionAsync: ActionAsync<TState = TState>>(state: &mut Arc<TState>, actions: &[ActionInstance<TAction, TActionAsync>]) {
    thread::scope(|scope| {
        let mut handles = Vec::new();

        for action in actions.iter() {
            match action {
                ActionInstance::Async(a) => {
                    let mut state = Arc::clone(&state);
                    // thought - I think that not using futures might be the right way to go, each action *is* synchronous from the test
                    // perspective.  Internally they may use futures or whatever they like, but that is internal to themselves.  From
                    // a harness perspective, they are synchronous.

                    // thought - full Futures and an opaque threading model sounds crappy here.
                    // not having to wory about *if* they are executing concurrently would be really nice.
                    // Some fanciness should be required though perhaps, otherwise we will write crappy tests that try to spawn more threads than
                    // the machine can support, as well as being really slow.
                    //
                    // question - is the threading model here repo specific?

                    // todo - this is very simple code, have a think as to whether it is worth doing something fancier
                    // across tests (current code might still be preferable to shared test stuff)
                    let handle = scope.spawn(move ||{
                        let state = Arc::make_mut(&mut state);
                        a.execute(state)
                    });

                    handles.push(handle);
                },
                ActionInstance::Sync(a) =>{
                    handles.into_iter().for_each(|h| {
                        h.join().unwrap();
                    });
                    handles = Vec::new();

                    let state = Arc::get_mut(state)
                        .unwrap_or_else(||panic!("test thread leaked, programming error, likely in this function"));
                    a.execute(state);
                },
            }
        }
    });
}

impl<TAction: Action, TInner: Into<TAction> + Action+ActionMarker, TActionAsync: ActionAsync>
 From<TInner> for ActionInstance::<TAction, TActionAsync> {
    fn from(value: TInner) -> ActionInstance<TAction, TActionAsync> {
        ActionInstance::Sync(value.into())
    }
}

pub trait IntoAsync<TAction,TActionAsync>
        where TAction: Action,
        TActionAsync: ActionAsync {
    fn into_async(self) -> ActionInstance::<TAction, TActionAsync>;
}

pub trait FromAsync<TAction,TActionAsync, TValue>
        where TAction: Action,
        TActionAsync: ActionAsync {
    fn from_async(value: TValue) -> ActionInstance::<TAction, TActionAsync>;
}

impl<TAction,TActionAsync, TValue> FromAsync<TAction,TActionAsync, TValue> for TValue
        where TActionAsync: ActionAsync + From<TValue>,
        TAction: Action {
    fn from_async(value: TValue) -> ActionInstance::<TAction, TActionAsync> {
        ActionInstance::Async(value.into())
    }
}

impl<TAction,TActionAsync, TValue> IntoAsync<TAction,TActionAsync> for TValue
        where TValue : FromAsync<TAction,TActionAsync, TValue>,
        TAction: Action,
        TActionAsync: ActionAsync {
    fn into_async(self) -> ActionInstance::<TAction, TActionAsync> {
        TValue::from_async(self)
    }
}

// TO REVIEWERS: These macros are probably temporary, I suggest not bothering to review them right now (including signatures and usage)
//
// todo - decide what to do :)
#[macro_export]
macro_rules! actions {
    ($state:ident, $( $action:ident ),*) => {
        #[derive(Clone, Debug)]
        pub enum Actions {
            $($action($action),)*
        }

        // todo - long term, this should really be a derive macro, but thats a pita right now, might get away without it for initial release
        impl Action for Actions {
            type TState = $state;

            fn execute(&self, state: &mut $state) {
                match self {
                    $(Self::$action(a) => a.execute(state),)*
                }
            }
        }

        $(
            impl From<$action> for Actions {
                fn from(value: $action) -> Self {
                    Actions::$action(value)
                }
            }
        )*

        $(
            impl $crate::action::ActionMarker for $action {}
        )*
    }
}

#[macro_export]
macro_rules! actions_async {
    ($state:ident, $( $action:ident ),*) => {
        #[derive(Clone, Debug)]
        pub enum ActionsAsync {
            $($action($action),)*
        }

        // todo - long term, this should really be a derive macro, but thats a pita right now, might get away without it for initial release
        impl Action for ActionsAsync {
            type TState = $state;

            fn execute(&self, state: &mut $state) {
                match self {
                    $(Self::$action(a) => a.execute(state),)*
                }
            }
        }

        $(
            impl From<$action> for ActionsAsync {
                fn from(value: $action) -> Self {
                    ActionsAsync::$action(value)
                }
            }
        )*
    }
}
