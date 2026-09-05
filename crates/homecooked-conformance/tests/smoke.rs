//! Single cargo-test entry for the Stream 7 conformance smoke suite.
//!
//! ```bash
//! cargo test -p homecooked-conformance
//! ```

use homecooked_conformance::run_all;

#[test]
fn conformance_smoke_suite() {
    let failures = run_all();
    assert!(
        failures.is_empty(),
        "conformance smoke failures ({}):\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}
