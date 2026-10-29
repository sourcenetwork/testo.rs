use std::{future::Future, marker::PhantomData};
use std::panic::catch_unwind;
use std::thread::{self, ScopedJoinHandle};

use async_executor::{Task};
use async_io::block_on;
use async_lock::OnceCell;
use futures_lite::future;
use smol::process;
use super::executor::*;// todo

static GLOBAL: OnceCell<async_executor::Executor<'_>> = OnceCell::new();

pub struct AsyncExecutor{
}

impl Executor for AsyncExecutor  {
    type TScope<'s, 'e> = MultiScope<'s, 'e> where 'e : 's;
    
    
    fn scope<'env, F>(&self, f: F)
    where
        F: for<'scope> FnOnce(MultiScope<'scope, 'env>),
    {
        //let executor = async_executor::Executor::new();
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

        // todo - need to use threads from GLOBAL pool inside local executor
        let e = global();
        NScope::scope(f);
        //let s = MultiScope::new(executor);
        //s.scope(f);
        //f(s);
        //s.foo();
    }
}
// todo - this is all very very nasty, test and cleanup: 
// WARNING - I was very tired when writing this, it might have no value
pub struct MultiScope<'scope,'env> {
    s: &'scope PhantomData<()>,
    en: &'env PhantomData<()>,
    e: async_executor::Executor<'scope>,
    v :Vec<Task<()>>,
}

impl<'scope,'env> MultiScope<'scope,'env> {
    fn new(e: async_executor::Executor<'scope>) -> MultiScope<'scope,'env> {
        MultiScope{
            e:e,
            s: &PhantomData,
            en:&PhantomData,
            v:Vec::new(),
        }
    }
    fn foo(self){}
    fn scope<F>(self, f: F)
    where
        F: for<'s> FnOnce(MultiScope<'s, 'env>){
        f(self);
    }
}

impl<'scope, 'env> Scope<'scope, 'env> for MultiScope<'scope, 'env>  {
    fn spawn<T: Send + 'scope>(&mut self, future: impl Future<Output = T> + Send +'scope)
     {
        let t = self.e.spawn((async || { // todo - take thread from GLOBAL here
                smol::block_on(future);
            })());
        self.v.push(t.into_future());
    }
}

impl<'scope,'env> Drop for MultiScope<'scope,'env> {
    fn drop(&mut self) {
        for t in self.v.iter_mut() {
            smol::future::block_on(t)
            //smol::block_on((async|| t.await)());
        }
    }
}

struct NScope {}
impl NScope {
    fn scope<'env, 's, F>(f: F)
    where
        F: for<'scope> FnOnce(MultiScope<'scope, 'env>){
        let executor = async_executor::Executor::new();
        let s = MultiScope::new(executor);
        f(s);
    }
}
 