//! Profiling macros

#[macro_export]
macro_rules! profile_scope {
    ($name:literal) => {
        let _profile_guard = $crate::scope::ProfileScope::new_static($name);
    };
    ($name:expr) => {
        let _profile_guard = $crate::scope::ProfileScope::new($name);
    };
}

#[macro_export]
macro_rules! profile_scope_with_context {
    ($name:literal, $context:expr) => {
        let _profile_guard =
            $crate::scope::ProfileScope::new_with_context($name, ($context).clone());
    };
    ($name:expr, $context:expr) => {
        let _profile_guard =
            $crate::scope::ProfileScope::new_with_context($name, ($context).clone());
    };
}

/// A scope that also records where it is (`file:line`). The location is a
/// compile-time string: with the profiler off nothing is formatted or
/// allocated, so this is as cheap as `profile_scope!`.
#[macro_export]
macro_rules! profile_scope_loc {
    ($name:literal) => {
        let _profile_guard =
            $crate::scope::ProfileScope::new_static_at($name, concat!(file!(), ":", line!()));
    };
    ($name:expr) => {
        let _profile_guard =
            $crate::scope::ProfileScope::new_at($name, concat!(file!(), ":", line!()));
    };
}

#[macro_export]
macro_rules! profile_function {
    () => {
        let _profile_guard = $crate::scope::ProfileScope::new_static(module_path!());
    };
}
