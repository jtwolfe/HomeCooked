//! Keep `docs/conformance/scenarios.json` aligned with `all_scenarios()`.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use homecooked_conformance::all_scenarios;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
    tags: Vec<String>,
    native_only: bool,
}

#[test]
fn scenarios_json_matches_all_scenarios_names() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/conformance/scenarios.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read {}: {e}", path.display());
    });
    let rows: Vec<Row> = serde_json::from_str(&raw).expect("scenarios.json parses");

    let suite: BTreeSet<&str> = all_scenarios().iter().map(|(n, _)| *n).collect();
    let catalog: BTreeSet<&str> = rows.iter().map(|r| r.name.as_str()).collect();

    assert_eq!(
        suite, catalog,
        "docs/conformance/scenarios.json names must match all_scenarios()\n  only in suite: {:?}\n  only in catalog: {:?}",
        suite.difference(&catalog).collect::<Vec<_>>(),
        catalog.difference(&suite).collect::<Vec<_>>()
    );

    // Order should match all_scenarios for honest browsing.
    let suite_order: Vec<&str> = all_scenarios().iter().map(|(n, _)| *n).collect();
    let catalog_order: Vec<&str> = rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(suite_order, catalog_order);

    let runnable = rows.iter().filter(|r| !r.native_only).count();
    assert_eq!(runnable, 7, "expected thin wasm-runnable subset of 7");

    for row in &rows {
        assert!(!row.tags.is_empty(), "{} needs tags", row.name);
        if row.name.contains("tcp") || row.name.starts_with("controller_tcp") {
            assert!(row.native_only, "{} must be native_only (TCP)", row.name);
        }
        if row.name.starts_with("hub_") || row.name.contains("modbus_tcp") {
            assert!(row.native_only, "{} must be native_only", row.name);
        }
    }
}
