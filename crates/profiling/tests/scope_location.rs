//! `profile_scope_loc!` records `file:line` for a recorded scope and costs a
//! disabled one nothing (the location is a compile-time string; it is turned
//! into a `String` only when the scope is recorded).

#[test]
fn a_located_scope_records_its_place_only_while_recording() {
    profiling::disable_profiling();
    {
        profiling::profile_scope_loc!("located-off");
    }
    profiling::clear_events();
    profiling::enable_profiling();
    {
        profiling::profile_scope_loc!("located-on");
    }
    {
        let name = String::from("located-dynamic");
        profiling::profile_scope_loc!(name.as_str());
    }
    profiling::disable_profiling();
    let events = profiling::collect_events();
    assert!(events.iter().all(|e| e.name != "located-off"));
    for name in ["located-on", "located-dynamic"] {
        let event = events
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("{name} was recorded"));
        let location = event.location.as_deref().expect("a location");
        assert!(location.contains("scope_location.rs:"), "{location}");
    }
}
