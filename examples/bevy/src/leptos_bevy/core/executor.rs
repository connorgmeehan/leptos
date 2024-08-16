use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::Arc,
    task::{Context, Poll, Wake},
};
use any_spawner::{PinnedLocalFuture, PinnedFuture};
use bevy::tasks::futures_lite::FutureExt;

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

        while let Some(mut task) = FUTURES.with_borrow_mut(|f| f.pop_front()) {
            println!("Polling future.");
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => FUTURES.with_borrow_mut(|f| f.push_back(task)),
            }
        }
        while let Some(mut task) = LOCAL_FUTURES.with_borrow_mut(|f| f.pop_front()) {
            println!("Polling future.");
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => LOCAL_FUTURES.with_borrow_mut(|f| f.push_back(task)),
            }
        }
    }
}
