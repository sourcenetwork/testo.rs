use std::ops::Deref;
use std::sync::Arc;
use std::marker::Sync;
use std::thread;
use std::any::Any;

pub trait Action<TState> : ActionClone<TState> + Any {
    fn execute(&self, state: &mut TState);
}

pub trait ActionAsync<TState>: ActionAsyncClone<TState> + Action<TState> + Sync + Send {}
impl<TState: Clone + Sized, TAction: Action<TState> + Sync + Send + Clone> ActionAsync<TState> for TAction  {}

pub type Actions<TState> = [Box<dyn Action<TState>>];
pub type ActionsAsync<TState> = [Box<dyn ActionAsync<TState>>];

#[derive(Clone)]
pub enum ActionInstance<TState> {
    Sync(Box<dyn Action<TState>>),
    Async(Box<dyn ActionAsync<TState>>),
}

pub fn execute<TState: Clone + Sync + Send + 'static>(state: TState, actions: &[ActionInstance<TState>]) -> TState {
    let mut state = Arc::new(state);

    execute_a(&mut state, actions);

    // We cannot borrow the input `state` parameter as mutable, as we need to be able concurrently mutate elements within it,
    // which we do by cloning it - this deliberately discards any concurrent mutations made to inner elements not wrapped in
    // stuff like `Arc`s.
    //
    // Synchronous action executions avoid cloning of `state`, and do not need to protect any mutating inner elements with `Arc`.
    state.deref().clone()
}

pub fn execute_a<TState: Clone + Sync + Send + 'static>(state: &mut Arc<TState>, actions: &[ActionInstance<TState>]) {
    thread::scope(|scope| {
        let mut handles = Vec::new();

        for action in actions.iter() {
            match action {
                ActionInstance::<TState>::Async(a) => {
                    let mut state = Arc::clone(&state);
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

pub trait IntoAsync<TState>{
    fn into_async(self) -> ActionInstance::<TState>;
}

impl<TState: Clone + Sized, TAction: ActionAsync<TState> + Into<Box<dyn ActionAsync<TState>>> + Clone> IntoAsync<TState> for TAction {
    fn into_async(self) -> ActionInstance<TState> {
        ActionInstance::Async(self.into())
    }
}

// ActionClone is a workaround to allow the auto-implementation of `Clone` for
// `Box<dyn Action<TState>>`.
//
// `Action<TState>` cannot do this directly due to it needing to be vtable and dynamic-dispatch
// friendly.
pub trait ActionClone<TState> {
    fn clone_box(&self) -> Box<dyn Action<TState>>;
}

impl<TAction: Action<TState> + Clone, TState> ActionClone<TState> for TAction {
    fn clone_box(&self) -> Box<dyn Action<TState>> {
        Box::new(self.clone())
    }
}

impl<TState> Clone for Box<dyn Action<TState>> {
    fn clone(&self) -> Box<dyn Action<TState>> {
        self.clone_box()
    }
}

// ActionAsyncClone is a workaround to allow the auto-implementation of `Clone` for
// `Box<dyn ActionAsync<TState>>`.
//
// `ActionAsync<TState>` cannot do this directly due to it needing to be vtable and dynamic-dispatch
// friendly.
pub trait ActionAsyncClone<TState> {
    // Warning: This func must have a different name to the silimar one in ActionClone as otherwise the
    // compiler gets confused in impl<TState> Clone for Box<dyn ActionAsync<TState>>, direct referencing
    // via ActionAsyncClone::clone_async_box didn't work as the compiler then got stuck in an infinate loop
    // probably due to the horrible hack that this is.
    fn clone_async_box(&self) -> Box<dyn ActionAsync<TState>>;
}

impl<TAction: ActionAsync<TState> + Clone, TState> ActionAsyncClone<TState> for TAction {
    fn clone_async_box(&self) -> Box<dyn ActionAsync<TState>> {
        Box::new(self.clone())
    }
}

impl<TState> Clone for Box<dyn ActionAsync<TState>> {
    fn clone(&self) -> Box<dyn ActionAsync<TState>> {
        self.clone_async_box()
    }
}

// ActionMarker is a marker trait that allows the auto-implementation of other traits
// by this package, without leaving the implmentations unbounded.
//
// Implementing it is not required, but it is convienient.
pub trait ActionMarker {}

impl<TState: Sized, TAction: Action<TState> + ActionMarker> From<TAction> for Box<dyn Action<TState>> {
    fn from(value: TAction) -> Self {
        Box::new(value)
    }
}

impl<TState: Sized, TAction: Action<TState> + ActionMarker> From<TAction> for ActionInstance::<TState> {
    fn from(value: TAction) -> ActionInstance<TState> {
        ActionInstance::Sync(value.into())
    }
}

impl<TState: Clone + Sized, TAction: Action<TState> + ActionMarker + Sync + Send + Clone> From<TAction> for Box<dyn ActionAsync<TState>> {
    fn from(value: TAction) -> Self {
        Box::new(value)
    }
}

impl<TState: 'static> dyn Action<TState> {
    pub fn downcast_ref<TConcreteAction: Action<TState>>(&self) -> Option<&TConcreteAction> {
        (self as &dyn Any).downcast_ref()
    }
}

impl<TState: 'static> dyn ActionAsync<TState> {
    pub fn downcast_ref<TConcreteAction: ActionAsync<TState>>(&self) -> Option<&TConcreteAction> {
        (self as &dyn Any).downcast_ref()
    }
}
