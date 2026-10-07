use std::sync::Arc;
use std::marker::Sync;
use std::thread;
use std::any::Any;

pub trait Action<TState> : ActionClone<TState> + Any {
    fn execute(&self, state: &mut TState);
}

pub trait ActionAsync<TState>: Sync + Send + Any {
    fn execute(&self, state: &mut Arc<TState>);
}

impl<TState: Clone + Sized, TAction: Action<TState> + Sync + Send + 'static> ActionAsync<TState> for TAction  {
    fn execute(&self, state: &mut Arc<TState>) {
        let state = Arc::make_mut(state);
        self.execute(state);
    }
}

pub type Actions<TState> = [Box<dyn Action<TState>>];
pub type ActionsAsync<TState> = [Arc<Box<dyn ActionAsync<TState>>>];

#[derive(Clone)]
pub enum ActionInstance<TState> {
    Sync(Box<dyn Action<TState>>),
    Async(Arc<Box<dyn ActionAsync<TState>>>),
}

pub fn execute<TState: Clone + Sized + Sync + Send + 'static>(state: &mut Arc<TState>, actions: &[ActionInstance<TState>]) {
    let mut handles = Vec::new();
    for action in actions.iter() {
        match action {
            ActionInstance::<TState>::Async(a) => {
                let mut state = Arc::clone(&state);
                let a = Arc::clone(&a);
                let handle = thread::spawn(move || 
                    a.execute(&mut state)
                );

                handles.push(handle);
            },
            ActionInstance::Sync(a) =>{
                handles.into_iter().for_each(|h| {
                    h.join().unwrap();
                });
                handles = Vec::new();

                a.execute(Arc::make_mut(state));
            },
        }
    }

    handles.into_iter().for_each(|h| {
        h.join().unwrap();
    });
}

pub trait IntoAsync<TState>{
    fn into_async(self) -> ActionInstance::<TState>;
}

impl<TState: Clone + Sized, TAction: Action<TState> + Sync + Send + Into<Box<dyn ActionAsync<TState>>>> IntoAsync<TState> for TAction {
    fn into_async(self) -> ActionInstance<TState> {
        ActionInstance::Async(Arc::new(self.into()))
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

impl<TAction: ?Sized + 'static + Action<TState> + Clone, TState> ActionClone<TState> for TAction {
    fn clone_box(&self) -> Box<dyn Action<TState>> {
        Box::new(self.clone())
    }
}

impl<TState> Clone for Box<dyn Action<TState>> {
    fn clone(&self) -> Box<dyn Action<TState>> {
        self.clone_box()
    }
}

// ActionMarker is a marker trait that allows the auto-implementation of other traits
// by this package, without leaving the implmentations unbounded.
//
// Implementing it is not required, but it is convienient.
pub trait ActionMarker {}

impl<TState: Sized, TAction: Action<TState> + ActionMarker + 'static> From<TAction> for Box<dyn Action<TState>> {
    fn from(value: TAction) -> Self {
        Box::new(value)
    }
}

impl<TState: Sized, TAction: Action<TState> + ActionMarker + 'static> From<TAction> for ActionInstance::<TState> {
    fn from(value: TAction) -> ActionInstance<TState> {
        ActionInstance::Sync(value.into())
    }
}

impl<TState: Clone + Sized, TAction: Action<TState> + ActionMarker + Sync + Send + 'static> From<TAction> for Box<dyn ActionAsync<TState>> {
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
