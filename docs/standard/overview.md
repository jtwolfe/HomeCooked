# HomeCooked Standard Overview

Version **0.1.0** — docs-only revision.

HomeCooked is a **capability-based, versioned, extensible** communication
interface for whitegoods and kitchen appliances: discover a device, describe
what it can do, read telemetry and state, write settings, issue commands, and
subscribe to events.

This document maps **catalog → schema → wire protocol**. It is the standard
overview. The catalog remains the source of truth for *what appliances and
points exist*; this document is the source of truth for *how those points move
between peers*.

Related documents:

- [`../catalog/appliances.md`](../catalog/appliances.md) — appliance classes
- [`../catalog/variables-and-settings.md`](../catalog/variables-and-settings.md)
  — traits, variables, settings, commands, types, units, ranges, access

---

## 1. Goals and non-goals

### Goals

- One mental model for washer, oven, fridge, hob, HVAC-adjacent plant, and the
  rest of the catalog: **class + traits + points**.
- **Discovery** without prior vendor knowledge: a client can find a device,
  learn its capabilities, and operate the intersection it understands.
- **Capability checks** on every write: out-of-range, unsupported, unsafe, and
  unknown are distinct errors.
- **Additive evolution**: new classes, traits, points, and enum tokens in minor
  versions do not break old clients.
- **Vendor extensions** that cannot corrupt the core catalog namespace.

### Non-goals (this revision and generally)

This PR (**PR1**) is documentation only. It does **not** add Rust crates,
`Cargo.toml` files, schema codegen, a simulator, or WASM.

The standard itself also does **not** specify:

- A single physical transport (IP, BLE, Thread, serial are all allowed)
- User identity, OAuth, or cloud account linking
- Whole-home scenes, automations, or energy tariffs
- Functional safety certification (IEC 60335, etc.). Devices still enforce
  local interlocks; the protocol never bypasses them
- Pixel UI, recipe documents, or camera streams
- A global device registry or certificate authority (may appear later)

---

## 2. Architecture

```
 docs/catalog/*          source of truth (classes, traits, points)
         │
         ▼
 schema types            capabilities, variables, settings, commands
                         (future crate: homecooked-schema)
         │
         ▼
 wire protocol           framing, request/response, events, errors
                         (future crate: homecooked-protocol)
         │
         ▼
 core                    registry + validation against advertised caps
                         (future crate: homecooked-core)
         │
         ▼
 sim / wasm / apps       simulated devices and UIs driven by the catalog
```

Three layers, three version numbers:

| Layer | What versions | Where |
|-------|---------------|-------|
| Catalog | Classes, traits, point ids, enums, units | `docs/catalog/` `catalog_version` |
| Schema | Serializable types derived from the catalog | future schema crate `schema_version` |
| Protocol | Message kinds, framing, error codes | this document + future protocol crate `protocol_version` |

A device reports all three in `trait.identity`. Clients reject a peer only on
**protocol major** mismatch. Unknown catalog points are ignored on read and
rejected on write (`unknown_variable` / `unsupported_capability`).

### Roles

| Role | Responsibility |
|------|----------------|
| Device | Advertises class + traits, owns state, enforces ranges and safety |
| Client | Discovers, describes, reads, writes, subscribes |
| Hub (optional) | Aggregates devices, may proxy describe/read/write; not required |

A peer may be both (a simulator is a device; a web UI is a client).

---

## 3. Catalog as source of truth

The catalog defines:

1. **Classes** — what the appliance *is* (`washer`, `induction_hob`, …)
2. **Traits** — reusable bundles of points (`temperature`, `cycle`, `door_lid`)
3. **Points** — variables, settings, commands with type, unit, range, access
4. **Composition** — combo products (`washer_dryer`, `range`, `fridge_freezer`)

Rules:

- Code must not invent core class or trait ids that are absent from the catalog.
- Code may omit optional points. Code must not require points the catalog marks
  optional unless the device advertised them.
- When catalog and code disagree, **the catalog wins**; change the docs in the
  same PR as the code, or in a docs-only PR that lands first.
- Vendor points live under `vendor.<vendor_id>.*` and are not core catalog.

Catalog versioning is described in [§8 Versioning](#8-versioning).

---

## 4. Schema types (derived from catalog)

The future schema crate maps catalog rows to typed values. This overview fixes
the conceptual types so protocol and schema stay aligned.

### 4.1 Primitive types

| Catalog type | Schema meaning |
|--------------|----------------|
| `bool` | boolean |
| `u8` `u16` `u32` `i16` `i32` | unsigned / signed integers |
| `f32` | binary32 float |
| `percent` | `f32` 0–100 unless a tighter range is advertised |
| `enum` | string token from a closed set, plus unknown-token handling |
| `string` | UTF-8, length-limited |
| `timestamp_ms` | `u64` milliseconds since Unix epoch, UTC |
| `duration_s` | `u32` seconds |
| `list<T>` | ordered list, max length advertised |
| `command` | write of an action with an argument payload (`void` or fields) |

Units are metadata on the point, not a separate wire type. Temperatures are
always **celsius** on the wire.

### 4.2 Qualified identifiers

```
point_id     := trait_point | class_point | vendor_point
trait_point  := "trait." trait_id "." id
class_point  := "class." class_id "." id
vendor_point := "vendor." vendor_id "." id
zoned_point  := point_id "#" zone_id
```

`trait_id`, `class_id`, `id`, `zone_id`, `vendor_id` are `snake_case`
`[a-z][a-z0-9_]{0,63}`.

Examples:

- `trait.cycle.cycle_state`
- `trait.temperature.setpoint_c#freezer`
- `class.washer.spin_rpm`
- `class.multi_cooker.vent`
- `vendor.acme.steam_pulse`

### 4.3 Capability object

A **capability** is what a device *advertises*. It is not the catalog row; it
is the catalog row **intersected with this firmware**.

```
Capability {
  class_id:            ClassId          // primary
  class_version:       SemVer
  secondary_class_ids: [ClassId]
  traits:              [TraitCap]
  safety:              SafetyFlags
  catalog_version:     SemVer
}

TraitCap {
  trait_id:            TraitId
  trait_version:       SemVer
  points:              [PointCap]
}

PointCap {
  id:                  QualifiedId      // may include #zone
  type:                TypeTag
  unit:                Unit | none
  access:              { read, write, event }
  required:            bool
  range:               NumericRange | EnumSubset | StringLimit | CommandArg
  resolution:          f32?             // for f32 points
  zones:               [ZoneId]?        // if the unzoned id is zoned
}

SafetyFlags {
  remote_start_supported: bool          // default false
  gas_remote_ignite:      bool          // default false
  rf_remote_start:        bool          // microwave RF; default false
  remote_vent:            bool          // pressurized vent; default false
}
```

Clients cache `Capability` after `describe` and **must** re-`describe` after
`hello` / reconnect, firmware change (`fw_version`), or a `caps_changed` event.

### 4.4 Values

A **value** on the wire is a tagged union matching `TypeTag`. Commands use a
value of their argument type (`void` is encoded as null / omitted payload).

Reads may return `null` only for optional points that are currently
unavailable (probe unplugged). Required points never return `null`; use a
fault instead.

---

## 5. Capability-based discovery

Discovery answers: *who is there, what class are they, which traits and
versions, can I talk this protocol.*

### 5.1 Advertisement

A device periodically announces (transport-specific: mDNS, BLE advert, USB
descriptor, …) a **hello record**:

| Field | Required | Meaning |
|-------|----------|---------|
| `device_id` | yes | Stable id |
| `protocol_version` | yes | Wire protocol semver |
| `catalog_version` | yes | Catalog the firmware tracks |
| `class_id` | yes | Primary class |
| `trait_ids` | yes | List of advertised trait ids (no point list) |
| `display_name` | no | |
| `endpoint` | transport | How to open a session |

Hello is **small**. Full point ranges come from `describe`.

### 5.2 Session

1. Client receives hello (or scans).
2. Client opens a session (transport-specific).
3. Client sends `discover` (optional if hello already uniquely identified the
   peer) and/or `describe`.
4. Device returns capabilities.
5. Client `read`s, `write`s, `subscribe`s as allowed.

Multiple clients may share a device. Last well-formed write wins. Devices may
limit concurrent sessions; extra clients get `busy` or a transport error.

### 5.3 Matching

A client **understands** a device if:

- `protocol_version` major equals the client’s protocol major, and
- the client knows the `class_id` **or** is willing to operate on traits
  alone.

A client that does not know `class.pizza_oven` can still drive
`trait.temperature` and `trait.cycle` if those traits are advertised. Trait
operation is first-class, not a fallback hack.

Unknown traits in hello/describe are **ignored** (logged). They are not
errors.

---

## 6. Wire protocol

### 6.1 Transport and encoding

The protocol is **transport-agnostic**. A binding must provide:

- reliable, ordered delivery of messages **or** an explicit note that
  `event`s may drop (in which case clients re-`read` on gap)
- a session with a device id
- binary-safe payloads

Encoding is **structured** (JSON or CBOR in the protocol crate). This overview
specifies fields, not bytes. Field names on the wire are `snake_case` and
match this document.

Maximum message size: bindings should allow at least **64 KiB**. `describe` of
a large range may be chunked later; v1 expects describe to fit.

### 6.2 Envelope

Every message:

| Field | Type | Meaning |
|-------|------|---------|
| `proto` | semver string | Protocol version of this message |
| `kind` | enum | See [§7](#7-message-kinds) |
| `id` | string | Correlation id (UUID or monotonic). Required on requests; responses echo it; events use a new id |
| `ts_ms` | timestamp_ms | Sender clock |
| `device_id` | string | Target (requests) or source (events, responses) |
| `body` | object | Kind-specific |

Requests may include `timeout_ms` (advisory). Devices do not have to honor it.

### 6.3 Request / response

- One response per request, same `id`.
- Success: `kind` is the matching `*_ok` **or** the same kind with
  `status: ok` (bindings pick one style and stick to it). This standard uses
  **distinct ok kinds**: `discover_ok`, `describe_ok`, `read_ok`, `write_ok`,
  `subscribe_ok`, `unsubscribe_ok`.
- Failure: `kind = error`, `id` copied, body is an [error object](#9-error-model).
- No pipelining requirements; clients may pipeline if the binding allows.
- Idempotency: commands are **not** idempotent unless documented (`lock_door`
  is; `dispense` is not). Clients must not retry `command` writes without
  checking state, except when the error is `timeout` and the command is
  explicitly marked idempotent.

### 6.4 Events

Events are unsolicited (after subscribe) messages with `kind = event`. They
carry one or more point updates and optional fault snapshots.

Subscriptions are per-session. They die with the session. After reconnect the
client must `subscribe` again and `read` to fill gaps.

---

## 7. Message kinds

| kind | Direction | Purpose |
|------|-----------|---------|
| `discover` | C→D | List devices / confirm presence |
| `discover_ok` | D→C | Hello-class records |
| `describe` | C→D | Fetch capabilities |
| `describe_ok` | D→C | `Capability` |
| `read` | C→D | Read points |
| `read_ok` | D→C | Values |
| `write` | C→D | Write settings / commands |
| `write_ok` | D→C | Accepted values / command ack |
| `subscribe` | C→D | Watch points |
| `subscribe_ok` | D→C | Echo of subscribed ids |
| `unsubscribe` | C→D | Stop |
| `unsubscribe_ok` | D→C | |
| `event` | D→C | Point change, cycle, fault |
| `error` | D→C or C→D | Failure; usually D→C |
| `ping` | either | Liveness |
| `pong` | either | |
| `caps_changed` | D→C | Re-describe; capabilities mutated (OTA, zone add) |

Direction: C = client, D = device.

### 7.1 `discover` / `discover_ok`

**Request body:** optional `class_id` filter, optional `trait_ids` filter
(device must advertise all listed traits).

**Response body:** list of hello records (`device_id`, `class_id`,
`trait_ids`, versions, `display_name`).

On a point-to-point session, discover returns a single record (self). On a
hub, it returns the hub’s children.

### 7.2 `describe` / `describe_ok`

**Request body:** optional `points` filter (qualified ids). Empty = full
capability object.

**Response body:** `Capability` as in §4.3.

Describe is cheap compared to guessing ranges. Clients should describe once
per session and on `caps_changed`.

### 7.3 `read` / `read_ok`

**Request body:** `points: [qualified_id]`. Max 128 ids per request
(recommended). Empty list is `invalid_request`.

**Response body:** `values: [{ id, value, ts_ms }]`. Missing optional points
may be omitted or `value: null`. Unknown ids make the **whole request** fail
with `unknown_variable` (strict), unless the client sets `allow_partial:
true`, in which case each unknown id appears in `errors[]` and known ids
still return.

### 7.4 `write` / `write_ok`

**Request body:** `writes: [{ id, value }]`. Optional `dry_run: bool`.

Semantics:

1. Validate each write against advertised capabilities (type, range, enum,
   access, trait presence).
2. Validate safety / remote flags.
3. If any write fails and `atomic` is true (default **false**), apply none.
   v1 devices may reject `atomic: true` with `unsupported_operation`.
4. Apply in list order.
5. Commands (`start`, `dispense`, …) run **after** setpoint writes in the
   same request if both appear — clients should still send setpoints first in
   the list. Devices that cannot reorder must document FIFO application.

**Response body:** `accepted: [{ id, value }]` (echo, possibly clamped if the
device advertised `clamp: true`; **default is no clamp**, out-of-range fails).
Commands echo `accepted` with the argument value.

`dry_run: true` performs steps 1–2 only and returns what would be accepted.

### 7.5 `subscribe` / `unsubscribe` / `event`

**Subscribe body:** `points: [id]` and/or `traits: [trait_id]` (all eventable
points in those traits) and/or `all: true`. Optional `min_period_ms` (device
may coalesce). Optional `events: [cycle, fault, value]`.

**Event body:**

```
{
  reason: "value" | "cycle" | "fault" | "caps_changed",
  values: [{ id, value, ts_ms }],
  cycle_state?: enum,
  fault?: { fault_present, fault_code, fault_severity, alert_list }
}
```

Devices should emit events on:

- any subscribed point whose value changes (after debounce / `min_period_ms`)
- `cycle_state` transitions
- rising/falling `fault_present` and new `alert_list` tokens

They should **not** emit a high-rate temperature stream unless subscribed and
not coalesced; 1 Hz is a reasonable default for analog telemetry.

### 7.6 `ping` / `pong`

Body empty or `{ "echo": string }`. Used for session liveness. Bindings may
use transport pings instead.

---

## 8. Versioning

HomeCooked uses **semver-style** `MAJOR.MINOR.PATCH` on catalog, schema, and
protocol independently.

### 8.1 What is compatible

| Change | Bump | Client impact |
|--------|------|----------------|
| New optional class, trait, or point | catalog MINOR | Old clients ignore it |
| New optional enum token | catalog MINOR | Old clients must pass through on **read**; writes of unknown tokens fail `invalid_enum` |
| Docs-only clarification, typo, typical-range note | catalog PATCH | None |
| Tighten a typical range in docs without changing advertised-range rules | catalog PATCH | None (devices advertise actual ranges) |
| Remove / rename id | catalog MAJOR | Old clients break |
| Change type, unit, or meaning of an id | catalog MAJOR | Old clients break |
| Make an optional point required | catalog MAJOR | Old devices become non-compliant |
| New protocol message kind (optional) | protocol MINOR | Old peers ignore unknown kinds? **No** — unknown **request** kinds return `unknown_kind`. Unknown **event** fields are ignored |
| New error code | protocol MINOR | Old clients map to `internal` / `unsupported_operation` |
| Change envelope fields, correlation rules | protocol MAJOR | |

### 8.2 Unknown handling (normative)

1. **Unknown message kind** on a request → `error` / `unknown_kind`.
2. **Unknown field** in a body → ignore (forward compatible).
3. **Unknown enum token** on read → deliver as raw string to the client app;
   schema layer uses an `Unknown(string)` variant. Do not drop the point.
4. **Unknown enum token** on write → `invalid_enum`.
5. **Unknown point id** on read/write → `unknown_variable` (or partial if
   `allow_partial`).
6. **Protocol major mismatch** → do not send application writes; `error` /
   `version_mismatch`.

### 8.3 Device catalog lag

A device may advertise `catalog_version` older than the client’s. The client
operates on the intersection. A device newer than the client is also fine
(§8.2).

### 8.4 Stability of ids

Core ids never recycle. Retired ids stay reserved (listed as retired in a
future catalog MAJOR) so old firmware cannot collide with a new meaning.

---

## 9. Error model

`kind: error` body:

| Field | Type | Meaning |
|-------|------|---------|
| `code` | enum | Stable token below |
| `message` | string | Human, not parsed |
| `point_id` | string? | When the error is about a point |
| `expected` | string? | Type / range hint |
| `retryable` | bool | Client may retry the same request |

### 9.1 Codes

| code | Typical cause | retryable |
|------|---------------|-----------|
| `unknown_device` | Bad `device_id` | no |
| `unknown_kind` | Unknown message kind | no |
| `unknown_variable` | Point id not in catalog ∩ device | no |
| `unknown_capability` | Trait / class not advertised | no |
| `unsupported_capability` | Optional point / trait missing | no |
| `unsupported_operation` | Command or flag not implemented (`pause`, `atomic`) | no |
| `not_writable` | Access lacks `w` | no |
| `not_readable` | Access lacks `r` | no |
| `invalid_type` | Value tag ≠ advertised type | no |
| `invalid_enum` | Token not in advertised subset | no |
| `invalid_request` | Malformed body, empty read list | no |
| `out_of_range` | Numeric outside advertised range | no |
| `busy` | Cycle running, door locking, exclusive session | yes, later |
| `safety_interlock` | Door, lid, pan, pressure, leak, dry, RF door | no until state changes |
| `remote_disabled` | `remote_control_enabled` or `remote_start_enabled` false | no until user enables |
| `unauthorized` | Binding-level auth (optional) | no |
| `timeout` | Device aborted | yes |
| `version_mismatch` | Protocol major | no |
| `internal` | Bug | maybe |

`unknown_capability` vs `unsupported_capability`: use `unknown_capability`
when the **id is not recognized** by the device at all; use
`unsupported_capability` when the id is recognized as optional catalog but
not implemented on this SKU.

### 9.2 Validation order

Implementations should fail in this order so clients can fix the first
problem:

1. envelope / `version_mismatch` / `unknown_device` / `unknown_kind`
2. `invalid_request`
3. `unknown_variable` / `unknown_capability`
4. `unsupported_capability` / `unsupported_operation`
5. `not_writable` / `not_readable`
6. `invalid_type` / `invalid_enum` / `out_of_range`
7. `remote_disabled`
8. `safety_interlock`
9. `busy`
10. apply write

### 9.3 Safety is not an error to bypass

There is no “force” flag. A client that needs to run a clean cycle or a
remote start must satisfy the same interlocks a local user would, plus
`remote_*` flags. Documented default-deny operations:

- Microwave RF start with door open; remote RF start unless `rf_remote_start`
- Gas hob ignition unless `gas_remote_ignite`
- Multi-cooker vent unless `remote_vent`
- Garbage disposal run unless `remote_control_enabled`
- Pyrolytic self-clean: device may require local confirmation (`busy` or
  `safety_interlock`)

---

## 10. Extension model

### 10.1 Vendor namespace

```
vendor_id  := [a-z][a-z0-9_]{1,31}
vendor_cls := "vendor." vendor_id "." class_id
vendor_tr  := "vendor." vendor_id "." trait_id
vendor_pt  := "vendor." vendor_id "." id
```

Rules:

1. Core catalog ids **must not** start with `vendor_`.
2. Vendors **must not** mint core-looking ids (`washer2`, `temp`) to mean
   something else; extend with `vendor.acme.*`.
3. Vendor points may appear on a core class (an `oven` with
   `vendor.acme.steam_pulse`).
4. Vendor traits may be advertised next to core traits.
5. Clients that do not know a vendor id ignore it (discovery still works).
6. Vendor documentation should use the same columns as the core variables
   catalog (type, unit, range, access).

### 10.2 Experimental core

Proposed core points live in PRs against `docs/catalog/`. They do **not** ship
as `x_*` ids in released catalog MINOR versions. Drafts may use
`vendor.homecooked_exp.*` if a prototype must run before the catalog PR
merges.

### 10.3 Profiles

A **profile** is a named bundle of required traits for an ecosystem (e.g.
“laundry v1”: `washer`/`dryer` + cycle + remote + door_lid). Profiles are not
defined in this PR; they will be additive documents under `docs/standard/`
later. Until then, the per-class “typical traits” tables are guidance, not
certification.

---

## 11. Safety, privacy, and operations

- **Local enforcement:** heaters, valves, magnetrons, motors, and pressure
  relief are device-local. Protocol writes are requests.
- **No silent clamp** unless the device advertised `clamp: true` for that
  point (induction power-share is the main case; even then, prefer event
  `limiter_active`).
- **PII:** `serial`, `ip_address`, `mac_address` are optional and should not
  appear in hello broadcasts on open LAN if the binding can avoid it.
- **Logging:** `fault_message` is not a stable API. Automations should key off
  `fault_code` / `alert_list`.
- **Clock:** `timestamp_ms` may jump; clients should tolerate non-monotonic
  `ts_ms` on events.
- **Units:** never send Fahrenheit on the wire.

---

## 12. Mapping examples (informative)

### Start a washer eco cycle at 40 °C, 800 rpm

1. `describe` → confirm `class.washer`, traits `cycle`, `program`, `remote`.
2. `read` `trait.remote.remote_start_enabled`, `trait.door_lid.door_state`,
   `trait.safety.interlock_ok`.
3. `write` in one request (order):
   - `trait.program.program` = `eco`
   - `class.washer.wash_temp_c` = `40`
   - `class.washer.spin_rpm` = `800`
   - `trait.cycle.start` = void
4. `subscribe` `trait.cycle.*`, `trait.fault.*`.
5. Events: `cycle_state=running`, phases, `complete`.

Failures: door open → `safety_interlock`; remote start off →
`remote_disabled`; 2000 rpm → `out_of_range`; `pause` on a SKU without pause
→ `unsupported_operation`.

### Fridge-freezer two setpoints

Writes to `trait.temperature.setpoint_c#fridge` and
`trait.temperature.setpoint_c#freezer`. A write to unzoned
`trait.temperature.setpoint_c` on a multi-zone device is `invalid_request`
or `unknown_variable`.

### Induction zone without a pan

`write` `class.induction_hob.level#hob_2` = 5 succeeds. Device later emits
`class.induction_hob.pan_present#hob_2` = false and level returns to 0. That
is policy, not `error`.

---

## 13. Out of scope for this PR

This document and the catalog files are **PR1**. Explicitly **not** in this
change:

- `crates/homecooked-schema`
- `crates/homecooked-protocol`
- `crates/homecooked-core`
- `crates/homecooked-sim`
- `crates/homecooked-wasm`
- `apps/simulator-web`
- Any `Cargo.toml`, `*.rs`, or generated JSON Schema
- CI workflow beyond what already exists
- Formal conformance test suite

Follow-up work should track these docs: schema types from the variables
catalog, protocol messages from §6–§9, validation tables from the variables
catalog “Capability advertisement” section.

---

## 14. Document history

| Version | Notes |
|---------|--------|
| 0.1.0 | Initial overview; catalog-backed; no wire encoding frozen |
