use std::future::Future;
use std::panic::catch_unwind;
use std::thread;

use async_executor::{Task};
use async_io::block_on;
use async_lock::OnceCell;
use futures_lite::future;
use smol::process;

pub trait Executor<'a> {
    fn spawn<T: Send + 'a>(&self, future: impl Future<Output = T> + Send + 'a) -> Task<T>;
}

pub struct Default{}

impl Executor<'static> for Default  {
    fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T> {
        static GLOBAL: OnceCell<async_executor::Executor<'_>> = OnceCell::new();

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
}

// todo - this is default, but allow tests to select the executor they want via an execute, or state param (maybe an action??)
// create a traited enum that can be passed as a generic to stuff (action/state), use async_executor as base/default 

pub trait State<'a> { 
    type TExecutor : Executor<'a>;
    fn executor() -> Self::TExecutor;
    /*
    // default associated types would be really handy here:
    // https://github.com/rust-lang/rust/issues/29661 
    type TExecutor : Executor<'a>;
    fn executor() -> Self::TExecutor {
        default{}
    }
     */
}

trait DefaultMarker {}
impl<T: DefaultMarker> State<'static> for T {
    type TExecutor = Default;

    fn executor() -> Self::TExecutor {
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