use std::{future::Future, marker::PhantomData};
use std::panic::catch_unwind;
use std::thread::{self, ScopedJoinHandle};

use async_executor::{Task};
use async_io::block_on;
use async_lock::OnceCell;
use futures_lite::future;
use smol::process;

pub trait Executor {
    type TScope<'s, 'e> : Scope<'s, 'e> where 'e : 's;

    fn scope<'env, F>(&self, f: F)
    where
        F: for<'scope> FnOnce(Self::TScope<'scope, 'env>);
}

pub trait Scope<'scope, 'env> {
    fn spawn<T: Send + 'scope>(&mut self, future: impl Future<Output = T> + Send + 'scope);
}

// todo - this is default, but allow tests to select the executor they want via an execute, or state param (maybe an action??)
// create a traited enum that can be passed as a generic to stuff (action/state), use async_executor as base/default 

pub use crate::executor_tpa::DefaultMarker;
pub trait State<'a> {
    // todo - don't expose it this way.
    // it would be *very* powerful to allow actions (i.e. multipliers!) to control the threading model
    // although... would actions setting this be a pointless complication?  Would we ever *want* to use that?
    // is it not more of a case of a test declaring that it wants x/y/z happening at t?  In which case a generic on execute (w default)
    // would be simpler for users
    type TExecutor  : Executor;
    fn executor<'b>(&self) ->  Self::TExecutor  where 'a:'b;
    /*
    // default associated types would be really handy here:
    // https://github.com/rust-lang/rust/issues/29661 
    type TExecutor : Executor<'a>;
    fn executor() -> Self::TExecutor {
        default{}
    }
     */
}
