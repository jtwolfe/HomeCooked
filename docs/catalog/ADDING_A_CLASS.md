# Adding a new appliance class

Step-by-step checklist for introducing a new HomeCooked appliance **class**.
The catalog docs are the source of truth; Rust tables must track them.

Today every id in `ApplianceClassId::ALL` has a static `ClassTable` and can be
spawned in the sim / WASM UI:

| Constant | Meaning (current layout) |
|----------|--------------------------|
| `TIER_A_CLASS_IDS` | **25** fully tabled classes (laundry, cold, cooking core, …) |
| `TIER_B_CLASS_IDS` | **31** thin static tables for the remaining catalog ids |
| `STATIC_CLASS_IDS` | **`ApplianceClassId::ALL`** (= Tier-A ∪ Tier-B, **56** ids) |

There is no “index-only” class left: if it is in the catalog index, it needs a
`ClassTable`. Prefer a **thin Tier-B** table unless the class is a Tier-A
priority (see [`../ROADMAP.md`](../ROADMAP.md) §4).

Related:

- [`appliances.md`](./appliances.md) — class index, traits, settings/state notes
- [`variables-and-settings.md`](./variables-and-settings.md) — shared traits + per-class points
- [`../ROADMAP.md`](../ROADMAP.md) — Tier-A / Tier-B sets and workstreams
- [`../../CONTRIBUTING.md`](../../CONTRIBUTING.md) — branch / PR / CI norms

---

## Checklist (order matters)

1. [Catalog docs](#1-catalog-docs)
2. [`ids.rs`](#2-idsrs)
3. [`ClassTable`](#3-classtable)
4. [Sim defaults (and optional behavior)](#4-sim-defaults-and-optional-behavior)
5. [Tests](#5-tests)
6. [WASM list](#6-wasm-list)
7. [Optional procedure](#7-optional-procedure)

Do not invent core class / trait / point ids in code that are missing from
`docs/catalog/`.

---

## 1. Catalog docs

### 1a. Index + class section — `docs/catalog/appliances.md`

1. Add a row to the **Index** table (`id` | Name | Group).
2. Add a `### \`your_class\`` section with:
   - short description
   - **Typical traits** (reuse existing trait ids)
   - **Typical controllable settings** / **readable state**
   - **Notes** (safety, variants, composition)
3. If the class belongs in an existing group (Laundry, Cold, Wash, Cooking,
   Ventilation, Beverage, Countertop, Utility, Climate), use that group. A
   **new** group also requires updating `CATALOG_GROUP_ORDER` and
   `catalog_group()` in `crates/homecooked-schema/src/catalog/mod.rs`.

Naming: stable `snake_case`, starts with a letter, length ≤ 64 (see
`is_snake_case_id` in `ids.rs`). Never rename a published id in a minor version.

### 1b. Per-class points — `docs/catalog/variables-and-settings.md`

Add `### Class \`your_class\`` under **Per-class variables** with a table of
class-local points (`id`, type, unit, range/enum, access, req, description).

Reuse **shared traits** wherever possible (`power`, `temperature`, `cycle`,
…). Class points are only for things that are not already covered by a trait
(e.g. `shade` on `toaster`, `spin_rpm` on `washer`).

Point ids on the wire become:

| Kind | Qualified id |
|------|----------------|
| Class variable / setting / command | `class.<class_id>.<id>` |
| Trait point | `trait.<trait_id>.<id>` |

Copy types, units, ranges, and access modes from this catalog into code — do
not invent core point ids.

---

## 2. `ids.rs`

File: `crates/homecooked-schema/src/ids.rs`

Add a variant to the `ApplianceClassId` `snake_ids!` enum:

```rust
YourClass => "your_class",
```

`ApplianceClassId::ALL`, `as_str`, and `FromStr` update automatically via the
macro. Keep enum order aligned with the appliances.md Index when practical
(new classes usually append near related peers).

If you add a **new trait**, also extend `TraitId` and
`crates/homecooked-schema/src/catalog/traits.rs` — that is a separate, rarer
change; prefer existing traits.

---

## 3. `ClassTable`

File: `crates/homecooked-schema/src/catalog/classes.rs`

### 3a. Traits, points, and tokens

Mirror a thin peer (e.g. `TOASTER_*` or `BLENDER_*`):

```rust
const YOUR_CLASS_TRAITS: &[TraitId] = &[
    TraitId::Identity,
    TraitId::Power,
    TraitId::Connectivity,
    // … from appliances.md
];

static YOUR_CLASS_POINTS: &[CatalogPoint] = &[
    // v / s / cmd helpers — ids and ranges from variables-and-settings.md
];
```

Use the file’s `v`, `s`, `cmd`, `num`, `int`, `en` helpers. Put **required**
class points in the table with `required: true`; optional catalog points may
be omitted from `typical_capability` until a device advertises them. To keep
optional points in the typical/sim model (demos / catalog depth), allow them in
`extra_typical_class_point` / `extra_typical_trait_point` in
`crates/homecooked-schema/src/catalog/mod.rs` (see `wine_cooler` / `ice_maker` / `sous_vide` / `multi_cooker` / `toaster_oven` / `dehumidifier` / `range_hood` / `steam_oven` / `cooktop`).

For cycle/program classes, add `program_tokens` / `cycle_phase_tokens` slices
when the catalog documents them. For closed-loop temperature classes, set
`typical_setpoint_c` and optionally `typical_zones`.

### 3b. Register the table

1. Append a `ClassTable { … }` entry to `STATIC_CLASS_TABLES`.
2. Add the id to **`TIER_A_CLASS_IDS`** or **`TIER_B_CLASS_IDS`** (disjoint;
   together they must equal `ApplianceClassId::ALL`).
3. Leave `STATIC_CLASS_IDS` as `ApplianceClassId::ALL` unless the project
   deliberately reintroduces a partial static set (it does not today).

### 3c. Catalog group

Update `catalog_group()` in `crates/homecooked-schema/src/catalog/mod.rs` so
the new id matches its Index group. The WASM picker uses this for
`<optgroup>` labels.

`typical_capability(class_id)` builds advertised capabilities from the
`ClassTable` (typical traits’ required points + required class points +
setpoint/zones). No extra registration is needed beyond the table.

---

## 4. Sim defaults (and optional behavior)

### 4a. Defaults — `crates/homecooked-sim/src/defaults.rs`

`sim_capability` / `seed_state` already work for any class with a
`ClassTable`. Specialize only when seed values would be wrong:

- `SeedCtx::from_identity` — ambient / setpoint / power_state
- `clamp_to_typical` — setpoint clamps matching `typical_setpoint_c`
- `zoned_temp_c` — multi-zone cabinets (e.g. `fridge_freezer`)

Otherwise the `_ => (20.0, 40.0, "on")` fallback is enough for a thin Tier-B
class.

### 4b. Optional behavior — `crates/homecooked-sim/src/behavior.rs`

Only if the class needs tick/command simulation beyond static state (kettle
heat, washer/dryer/microwave cycle progress). Most Tier-B classes skip this.

`Simulator::spawn` / `spawn_static_kitchen` already iterate
`STATIC_CLASS_IDS`.

---

## 5. Tests

After adding a class, **hard-coded class counts** must move with the catalog
(currently **56** = 25 + 31). Update assertions in at least:

| Location | What to bump |
|----------|----------------|
| `crates/homecooked-schema/src/catalog/mod.rs` | `list_all_class_ids` / `STATIC_CLASS_IDS` / Tier-A / Tier-B length tests |
| `crates/homecooked-schema/src/export.rs` | catalog JSON export `class_count` / `classes.len()` |
| `crates/homecooked-sim/src/lib.rs` | `spawn_all_static_classes` |
| `crates/homecooked-wasm/src/api.rs` | `list_appliance_classes` length |

Also run:

```bash
cargo test -p homecooked-schema
cargo test -p homecooked-sim
cargo test -p homecooked-wasm
cargo test --workspace
```

Add a focused unit test when the class has non-trivial points, zones, or
setpoint ranges (pattern: existing Tier-A / Tier-B table tests in
`catalog/mod.rs`).

---

## 6. WASM list

File: `crates/homecooked-wasm/src/api.rs`

`list_appliance_classes()` walks `ApplianceClassId::ALL` filtered by
`STATIC_CLASS_IDS` and labels groups via `catalog_group()`. With
`STATIC_CLASS_IDS = ALL`, a new tabled class appears in the simulator-web
picker automatically after rebuild:

```bash
wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
```

No separate WASM allow-list to edit unless you intentionally diverge from
`STATIC_CLASS_IDS`.

---

## 7. Optional procedure

When the class should show up in the procedure runner demos:

1. Author JSON under `crates/homecooked-procedure/examples/`.
2. `include_str!` + register in `EXAMPLE_PROCEDURES` in
   `crates/homecooked-procedure/src/document.rs`.
3. Add parse / validate / (optional) sim-run tests in
   `crates/homecooked-procedure/src/tests.rs`.
4. Optionally expose via wasm `list_example_procedures` /
   `get_example_procedure` (bundled examples are listed from the procedure
   crate) and document in [`../standard/procedures.md`](../standard/procedures.md).

See existing fixtures: `kettle_heat_80.json`, `wash_then_dry.json`,
`reheat_dominos_microwave.json`.

Procedures are **optional** for a new class; catalog + `ClassTable` + sim
spawn are the minimum.

---

## Quick reference — files touched

| Step | Path |
|------|------|
| Docs | `docs/catalog/appliances.md`, `docs/catalog/variables-and-settings.md` |
| Id enum | `crates/homecooked-schema/src/ids.rs` |
| Group | `crates/homecooked-schema/src/catalog/mod.rs` (`catalog_group`) |
| Table | `crates/homecooked-schema/src/catalog/classes.rs` |
| Sim seeds | `crates/homecooked-sim/src/defaults.rs` (optional) |
| Sim ticks | `crates/homecooked-sim/src/behavior.rs` (optional) |
| Counts | schema / sim / wasm / export tests (see §5) |
| Procedure | `crates/homecooked-procedure/examples/` (optional) |

---

## Definition of done

- Catalog Index + class section + per-class points committed.
- `ApplianceClassId` variant + `catalog_group` arm + `ClassTable` + Tier-A **or**
  Tier-B membership; `STATIC_CLASS_IDS` still equals `ALL`.
- `cargo test --workspace` green; class-count assertions updated.
- WASM picker lists the class after `wasm-pack build` (CI builds wasm).
- Optional: example procedure + tests if you claim a demo path.
