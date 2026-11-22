//! The profiler as one shared library.
//!
//! This crate contains `profiling` and every crate it links, SQLite among them
//! (`rusqlite` with the bundled SQLite). A binary or plugin library that links it
//! (`use profiling_dylib as _;`) shares one profiler: the flamegraph reads ONE global profiler,
//! and a per-library copy would record into a profiler nothing reads. SQLite's C API is exported
//! for the crates outside this library that call it (`rusqlite` users elsewhere): build.rs.

pub use profiling;

include!(concat!(env!("OUT_DIR"), "/native_exports.rs"));
