use std::{future::Future, marker::PhantomData};
use std::panic::catch_unwind;
use std::thread;

use async_executor::{Task};
use async_io::block_on;
use async_lock::OnceCell;
use futures_lite::future;
use smol::process;

pub trait Executor<'env> {
    type TScope<'s, 'e> : Scope<'s, 'e> where 'e : 's;

    //fn spawn<T: Send + 'env>(&self, future: impl Future<Output = T> + Send + 'env) -> Task<T>;
    fn scope<F>(&self, f: F)
    where
        F: for<'scope> FnOnce(Self::TScope<'scope, 'env>),;
}

pub trait Scope<'scope, 'env> {
    fn spawn<'future, T: Send + 'future>(&'scope self, future: impl Future<Output = T> + Send + 'future) -> Task<T>
    where 'future: 'scope, 
    'env : 'scope;
}

pub struct Default{}

impl<'a> Executor<'a> for Default  {
    type TScope<'s, 'e> = DefaultScope<'s, 'e> where 'e : 's;
    
    fn scope<F>(&self, f: F)
    where
        F: for<'scope> FnOnce(DefaultScope<'scope, 'a>), //Self::TScope),
        //'a: 'scope,
    {
        thread::scope(|scope|{
            let s = DefaultScope{t: scope};
            f(s)
        });
    }
    /*
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T> {
        static GLOBAL: OnceCell<async_executor::Executor<'_>> = OnceCell::new(); //todo - not suitable, requires that futures have same lifetime as executor/state

        fn global() -> &'static async_executor::Executor<'static> {
            GLOBAL.get_or_init_blocking(|| {
                let num_threads = {
                    // Parse SMOL_THREADS or default to 1.
                    std::env::var("SMOL_THREADS")// todo 
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1)
                };

                for n in 1..=num_threads {
                    thread::Builder::new()
                        .name(format!("smol-{}", n))// todo 
                        .spawn(|| loop {
                            catch_unwind(|| block_on(global().run(future::pending::<()>()))).ok();
                        })
                        .expect("cannot spawn executor thread");
                }

                // Prevent spawning another thread by running the process driver on this thread.
                let ex = async_executor::Executor::new();
                #[cfg(not(target_os = "espidf"))]// todo - find out why
                ex.spawn(process::driver()).detach();
                ex
            })
        }

        global().spawn(future)
    }
     
    fn scope<'b>() -> impl Scope<'b> {
        todo!();
        return  DefaultScope { _f: PhantomData::<&'b i64>{} };
    }
    */
    
    /*
    fn scope<'b>() -> ScopeC {
        todo!()
    } */
}

pub struct DefaultScope<'scope,'env> {
    t: &'scope thread::Scope<'scope,'env>
}
impl<'scope, 'env> Scope<'scope, 'env> for DefaultScope<'scope, 'env>  {
    fn spawn<'future, T: Send + 'future>(&'scope self, future: impl Future<Output = T> + Send + 'future) -> Task<T>
    // The future must outlive the scope, as the scope may complete before the future (not the other way around)
    where 'future: 'scope {
        self.t.spawn(
            || smol::block_on(future)
        );
        todo!()
    }
}

// todo - this is default, but allow tests to select the executor they want via an execute, or state param (maybe an action??)
// create a traited enum that can be passed as a generic to stuff (action/state), use async_executor as base/default 

pub trait State<'a> {
    // todo - don't expose it this way.
    // it would be *very* powerful to allow actions (i.e. multipliers!) to control the threading model
    type TExecutor : Executor<'a>;
    fn executor(&self) -> Self::TExecutor;
    /*
    // default associated types would be really handy here:
    // https://github.com/rust-lang/rust/issues/29661 
    type TExecutor : Executor<'a>;
    fn executor() -> Self::TExecutor {
        default{}
    }
     */
}

pub trait DefaultMarker {}
impl<T: DefaultMarker> State<'static> for T {//todo - rm static
    type TExecutor = Default;

    fn executor(&self) -> Self::TExecutor {
        Default{}
    }
}
/*
fn fooo<'a>(s: impl State<'a>){}
struct barrrry{}
impl DefaultMarker for barrrry {}
fn barrr(){
    let b = barrrry{};
    fooo(b);
}
 */