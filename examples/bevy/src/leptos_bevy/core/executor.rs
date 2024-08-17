use any_spawner::{PinnedFuture, PinnedLocalFuture};
use bevy::{log::info, tasks::futures_lite::FutureExt};
use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::Arc,
    task::{Context, Poll, Wake},
};

thread_local! {
    static LOCAL_FUTURES: RefCell<VecDeque<PinnedLocalFuture<()>>> = RefCell::new(VecDeque::new());
    static FUTURES: RefCell<VecDeque<PinnedFuture<()>>> = RefCell::new(VecDeque::new());
}

pub struct BevyLeptosExecutor {}

// Can be made more performant by using RawWaker
// which doesn't require an allocation?
// https://os.phil-opp.com/async-await/#simple-executor
pub struct DummyWaker;
impl Wake for DummyWaker {
    fn wake(self: std::sync::Arc<Self>) {
        // No-op
    }
    fn wake_by_ref(self: &std::sync::Arc<Self>) {
        // No-op
    }
}

impl BevyLeptosExecutor {
    pub fn spawn(fut: PinnedFuture<()>) {
        FUTURES.with_borrow_mut(move |futures| futures.push_back(fut));
    }

    pub fn spawn_local(fut: PinnedLocalFuture<()>) {
        LOCAL_FUTURES.with_borrow_mut(move |futures| futures.push_back(fut));
    }

    pub fn flush() {
        let waker = Arc::new(DummyWaker).into();
        let mut context = Context::from_waker(&waker);

        let mut futures = FUTURES.with_borrow_mut(std::mem::take);
        while let Some(mut task) = futures.pop_front() {
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    println!("\tReady!")
                } // task done
                Poll::Pending => FUTURES.with_borrow_mut(|f| f.push_back(task)),
            }
        }

        let mut local_futures = LOCAL_FUTURES.with_borrow_mut(std::mem::take);
        while let Some(mut task) = local_futures.pop_front() {
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    println!("\tReady!")
                } // task done
                Poll::Pending => {
                    LOCAL_FUTURES.with_borrow_mut(|f| f.push_back(task))
                }
            }
        }
    }
}
