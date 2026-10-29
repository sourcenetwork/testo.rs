use std::{future::Future, marker::PhantomData};
use std::panic::catch_unwind;
use std::thread::{self, ScopedJoinHandle};

use async_executor::{Task};
use async_io::block_on;
use async_lock::OnceCell;
use futures_lite::future;
use smol::process;
use super::executor::*;// todo


pub trait DefaultMarker {}

//use crate::executor_tpa::Default;
impl<'a, T: DefaultMarker> State<'a> for T {
    type TExecutor =  Default;

    fn executor<'b>(&self) -> Self::TExecutor where 'a:'b{
        Default{}
    }
}

pub struct Default{}

impl Executor for Default  {
    type TScope<'s, 'e> = DefaultScope<'s, 'e> where 'e : 's;
    
    fn scope<'env, F>(&self, f: F)
    where
        F: for<'scope> FnOnce(DefaultScope<'scope, 'env>),
    {
        thread::scope(|scope|{
            let s = DefaultScope::new(scope);
            f(s);
        });
    }
}

pub struct DefaultScope<'scope,'env> {
    t: &'scope thread::Scope<'scope,'env>,
}

impl<'scope,'env> DefaultScope<'scope,'env> {
    fn new(scope: &'scope thread::Scope<'scope,'env>) -> DefaultScope<'scope,'env> {
        DefaultScope{
            t: scope,
        }
    }
}

impl<'scope, 'env> Scope<'scope, 'env> for DefaultScope<'scope, 'env>  {
    fn spawn<T: Send + 'scope>(&mut self, future: impl Future<Output = T> + Send +'scope)
     {
        self.t.spawn(
            || { 
                smol::block_on(future);
            }
        );
    }
}
