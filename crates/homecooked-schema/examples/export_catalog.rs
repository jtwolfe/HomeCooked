//! Dump all catalog class ids + typical points as machine-readable JSON.
//!
//! Not a full OpenAPI server — a small, auditable export for tooling.
//!
//! ```bash
//! cargo run -p homecooked-schema --example export_catalog
//! cargo run -p homecooked-schema --example export_catalog -- /tmp/catalog.json
//! ```

use std::env;
use std::fs;
use std::io::{self, Write};
use std::process;

use homecooked_schema::export_catalog_json;

fn main() {
    let json = match export_catalog_json() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to serialize catalog export: {e}");
            process::exit(1);
        }
    };

    let mut args = env::args().skip(1);
    if let Some(path) = args.next() {
        if let Err(e) = fs::write(&path, format!("{json}\n")) {
            eprintln!("failed to write {path}: {e}");
            process::exit(1);
        }
        eprintln!("wrote catalog export to {path}");
    } else if let Err(e) = writeln!(io::stdout(), "{json}") {
        eprintln!("failed to write stdout: {e}");
        process::exit(1);
    }
}
