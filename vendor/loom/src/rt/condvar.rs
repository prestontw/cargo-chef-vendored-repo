use crate::rt::object;
use crate::rt::{self, thread, Access, Mutex, VersionVec};

use std::collections::VecDeque;

#[derive(Debug, Copy, Clone)]
pub(crate) struct Condvar {
    state: object::Ref<State>,
}

#[derive(Debug)]
pub(super) struct State {
    /// Tracks access to the mutex
    last_access: Option<Access>,

    /// Threads waiting on the condvar
    waiters: VecDeque<thread::Id>,
}

impl Condvar {
    /// Create a new condition variable object
    pub(crate) fn new() -> Condvar {
        super::execution(|execution| {
            let state = execution.objects.insert(State {
                last_access: None,
                waiters: VecDeque::new(),
            });

            Condvar { state }
        })
    }

    /// Blocks the current thread until this condition variable receives a notification.
    pub(crate) fn wait(&self, mutex: &Mutex) {
        self.state.branch_opaque();

        rt::execution(|execution| {
            let state = self.state.get_mut(&mut execution.objects);

            // Track the current thread as a waiter
            state.waiters.push_back(execution.threads.active_id());
        });

        // Release the lock
        mutex.release_lock();

        // Disable the current thread
        rt::park();

        // Acquire the lock again
        mutex.acquire_lock();
    }

    /// Wakes up one blocked thread on this condvar.
    pub(crate) fn notify_one(&self) {
        self.state.branch_opaque();

        rt::execution(|execution| {
            let state = self.state.get_mut(&mut execution.objects);

            // Notify the first waiter
            let thread = state.waiters.pop_front();

            if let Some(thread) = thread {
                execution.threads.unpark(thread);
            }
        })
    }

    /// Wakes up all blocked threads on this condvar.
    pub(crate) fn notify_all(&self) {
        self.state.branch_opaque();

        rt::execution(|execution| {
            let state = self.state.get_mut(&mut execution.objects);

            for thread in state.waiters.drain(..) {
                execution.threads.unpark(thread);
            }
        })
    }
}

impl State {
    pub(super) fn last_dependent_access(&self) -> Option<&Access> {
        self.last_access.as_ref()
    }

    pub(crate) fn set_last_access(&mut self, path_id: usize, version: &VersionVec) {
        Access::set_or_create(&mut self.last_access, path_id, version);
    }
}
