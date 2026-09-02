//! RAII scope guard for profiling

use once_cell::sync::OnceCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Instant;

use crate::events::ProfileEvent;
use crate::profiler::Profiler;

static PROFILER: OnceCell<Profiler> = OnceCell::new();

pub fn init_profiler() -> &'static Profiler {
    PROFILER.get_or_init(|| Profiler::new())
}

/// Thread-local profiling state
#[derive(Default)]
pub struct ThreadState {
    scope_stack: Vec<ScopeName>,
    thread_id: Option<u64>,
}

enum ScopeName {
    Static(&'static str),
    Owned(String),
}

impl ScopeName {
    #[inline]
    fn as_str(&self) -> &str {
        match self {
            Self::Static(name) => name,
            Self::Owned(name) => name,
        }
    }

    #[inline]
    fn into_string(self) -> String {
        match self {
            Self::Static(name) => name.to_owned(),
            Self::Owned(name) => name,
        }
    }
}

thread_local! {
    pub(super) static THREAD_STATE: std::cell::RefCell<ThreadState> = std::cell::RefCell::new(ThreadState::default());
    pub(super) static THREAD_NAME: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// RAII scope guard for profiling
pub struct ProfileScope {
    start: Instant,
    start_ns: u64,
    depth: u32,
    thread_id: u64,
    active: bool,
    pub(super) location: Option<String>,
}

impl ProfileScope {
    /// Begin a new profiling scope
    pub fn new(name: impl Into<String>) -> Self {
        Self::new_name(ScopeName::Owned(name.into()), None)
    }

    /// Begin a scope whose label is a compile-time string.
    ///
    /// The macro uses this path for string literals so the hot path does not
    /// allocate a label until the event is actually emitted.
    pub fn new_static(name: &'static str) -> Self {
        Self::new_name(ScopeName::Static(name), None)
    }

    /// Begin a new profiling scope with file location
    pub fn new_with_location(name: impl Into<String>, location: Option<String>) -> Self {
        Self::new_name(ScopeName::Owned(name.into()), location)
    }

    /// Begin a static-label scope with file location.
    pub fn new_static_with_location(
        name: &'static str,
        location: Option<String>,
    ) -> Self {
        Self::new_name(ScopeName::Static(name), location)
    }

    fn new_name(name: ScopeName, location: Option<String>) -> Self {
        if !init_profiler().is_enabled() {
            return Self {
                start: Instant::now(),
                start_ns: 0,
                depth: 0,
                thread_id: 0,
                active: false,
                location: None,
            };
        }

        let start = Instant::now();
        let start_ns = get_time_ns();
        let (depth, thread_id) = THREAD_STATE.with(|ts| {
            let mut state = ts.borrow_mut();
            let depth = state.scope_stack.len() as u32;
            let thread_id = *state.thread_id.get_or_insert_with(get_thread_id);
            state.scope_stack.push(name);
            (depth, thread_id)
        });

        Self {
            start,
            start_ns,
            depth,
            thread_id,
            active: true,
            location,
        }
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let duration_ns = self.start.elapsed().as_nanos() as u64;

        let (name, parent_name) = THREAD_STATE.with(|ts| {
            let mut state = ts.borrow_mut();
            let name = state
                .scope_stack
                .pop()
                .expect("profiling scope stack must match scope guards");
            let parent_name = state.scope_stack.last().map(|name| name.as_str().to_owned());
            (name.into_string(), parent_name)
        });

        if !init_profiler().is_enabled() {
            return;
        }

        let event = ProfileEvent {
            name,
            thread_id: self.thread_id,
            thread_name: THREAD_NAME.with(|tn| tn.borrow().clone()),
            process_id: init_profiler().get_process_id(),
            parent_name,
            start_ns: self.start_ns,
            duration_ns,
            depth: self.depth,
            location: self.location.take(),
            metadata: None,
        };

        init_profiler().submit_event(event);
    }
}

fn get_time_ns() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

fn get_thread_id() -> u64 {
    let thread_id = thread::current().id();
    let mut hasher = DefaultHasher::new();
    thread_id.hash(&mut hasher);
    hasher.finish()
}

pub(super) fn current_thread_id() -> u64 {
    THREAD_STATE.with(|ts| {
        let mut state = ts.borrow_mut();
        *state.thread_id.get_or_insert_with(get_thread_id)
    })
}
