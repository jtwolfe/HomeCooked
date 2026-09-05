//! wasm-bindgen JSON API wrapping [`homecooked_sim::Simulator`].
//!
//! JS calls the free functions below. Values cross the boundary as JSON
//! strings (except `create_device`, which returns the device id).
//!
//! Build for the web simulator:
//!
//! ```text
//! wasm-pack build crates/homecooked-wasm --target web --out-dir ../../apps/simulator-web/pkg
//! ```

mod api;
mod lab_checks;

use std::cell::RefCell;

use wasm_bindgen::prelude::*;

use crate::api::{ApiError, WasmApi};

thread_local! {
    static API: RefCell<WasmApi> = RefCell::new(WasmApi::new());
}

fn with_api<T>(f: impl FnOnce(&WasmApi) -> T) -> T {
    API.with(|api| f(&api.borrow()))
}

fn with_api_mut<T>(f: impl FnOnce(&mut WasmApi) -> T) -> T {
    API.with(|api| f(&mut api.borrow_mut()))
}

fn js_err(err: ApiError) -> JsError {
    JsError::new(&err.to_json())
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// JSON array of spawnable statically tabled classes (Tier-A ∪ Tier-B):
/// `[{"id":"kettle","label":"Kettle","group":"Beverage"}, ...]`.
#[wasm_bindgen]
pub fn list_appliance_classes() -> String {
    WasmApi::list_appliance_classes()
}

/// JSON array of static [`homecooked_schema::HeatPortSpec`] for a class
/// (`ClassTable.thermal_ports`). Empty array when unknown or none.
#[wasm_bindgen]
pub fn list_heat_port_specs(class_id: &str) -> String {
    WasmApi::list_heat_port_specs(class_id)
}

/// Spawn a simulated device. Returns the generated `device_id`.
#[wasm_bindgen]
pub fn create_device(class_id: &str) -> Result<String, JsError> {
    with_api_mut(|api| api.create_device(class_id)).map_err(js_err)
}

/// JSON array of `{device_id, class_id, display_name}`.
#[wasm_bindgen]
pub fn list_devices() -> String {
    with_api(|api| api.list_devices())
}

/// JSON `{identity, capability, points}` for one device.
#[wasm_bindgen]
pub fn describe(device_id: &str) -> Result<String, JsError> {
    with_api(|api| api.describe(device_id)).map_err(js_err)
}

/// Read points. `points_json` is a JSON string array, or omit/empty for full state.
#[wasm_bindgen]
pub fn read(device_id: &str, points_json: Option<String>) -> Result<String, JsError> {
    with_api(|api| api.read(device_id, points_json.as_deref())).map_err(js_err)
}

/// Write one point. `value` is tagged Value JSON or a JSON primitive coerced to the point type.
#[wasm_bindgen]
pub fn write(device_id: &str, point: &str, value: &str) -> Result<String, JsError> {
    with_api_mut(|api| api.write(device_id, point, value)).map_err(js_err)
}

/// Advance simulated time for one device by `dt_ms`. Returns `get_state` JSON.
#[wasm_bindgen]
pub fn tick(device_id: &str, dt_ms: u32) -> Result<String, JsError> {
    with_api_mut(|api| api.tick(device_id, dt_ms)).map_err(js_err)
}

/// JSON object of qualified point id → tagged `Value`.
#[wasm_bindgen]
pub fn get_state(device_id: &str) -> Result<String, JsError> {
    with_api(|api| api.get_state(device_id)).map_err(js_err)
}

/// JSON array of bundled examples: `{id, name, description?, class_hints?}`.
#[wasm_bindgen]
pub fn list_example_procedures() -> String {
    WasmApi::list_example_procedures()
}

/// Full procedure JSON for a bundled example id.
#[wasm_bindgen]
pub fn get_example_procedure(id: &str) -> Result<String, JsError> {
    WasmApi::get_example_procedure(id).map_err(js_err)
}

/// Parse + validate procedure JSON. Returns a summary or `ApiError` JSON.
#[wasm_bindgen]
pub fn parse_procedure(json: &str) -> Result<String, JsError> {
    WasmApi::parse_procedure(json).map_err(js_err)
}

/// Auto-bind / spawn sim devices by role, then run the procedure.
#[wasm_bindgen]
pub fn run_procedure(json: &str) -> Result<String, JsError> {
    with_api_mut(|api| api.run_procedure(json)).map_err(js_err)
}

/// Attach fridge→DHW demo plant, then run a bundled thermal procedure by id.
///
/// One-click helper for simulator-web thermal fixtures (`offer_fridge_dhw*`,
/// `wait_dhw_*`). Returns the same JSON as [`run_procedure`].
#[wasm_bindgen]
pub fn run_thermal_procedure(id: &str) -> Result<String, JsError> {
    with_api_mut(|api| api.run_thermal_procedure(id)).map_err(js_err)
}

/// Create/reset the fridge condenser → DHW demo thermal plant. Returns thermal_state JSON.
#[wasm_bindgen]
pub fn create_thermal_demo() -> Result<String, JsError> {
    with_api_mut(|api| api.create_thermal_demo()).map_err(js_err)
}

/// JSON snapshot of the thermal plant (reservoirs, ports, last transfer/reply).
#[wasm_bindgen]
pub fn thermal_state() -> Result<String, JsError> {
    with_api(|api| api.thermal_state()).map_err(js_err)
}

/// Negotiate the demo fridge→DHW offer. Returns thermal_state JSON.
#[wasm_bindgen]
pub fn thermal_negotiate_demo() -> Result<String, JsError> {
    with_api_mut(|api| api.thermal_negotiate_demo()).map_err(js_err)
}

/// Apply queued thermal accepts over `dt_s` seconds. Returns ThermalTickOut JSON.
#[wasm_bindgen]
pub fn thermal_tick(dt_s: f32) -> Result<String, JsError> {
    with_api_mut(|api| api.thermal_tick(dt_s)).map_err(js_err)
}

/// Negotiate demo offer then tick once. Returns ThermalTickOut JSON.
#[wasm_bindgen]
pub fn thermal_demo_transfer(dt_s: f32) -> Result<String, JsError> {
    with_api_mut(|api| api.thermal_demo_transfer(dt_s)).map_err(js_err)
}

/// Dual-path: fridge→DHW transfer then dishwasher_dhw_preheat procedure.
/// Returns ThermalThenDishwasherOut JSON.
#[wasm_bindgen]
pub fn run_thermal_then_dishwasher_preheat(dt_s: f32) -> Result<String, JsError> {
    with_api_mut(|api| api.run_thermal_then_dishwasher_preheat(dt_s)).map_err(js_err)
}

/// JSON array of conformance scenario catalog rows
/// (`docs/conformance/scenarios.json`): name, tags, native_only, summary.
#[wasm_bindgen]
pub fn list_conformance_scenarios() -> String {
    lab_checks::list_conformance_scenarios()
}

/// Run one thin in-process lab check by scenario name.
///
/// Runnable subset uses schema/sim/procedure/thermal only. Native-only rows
/// return `{ passed: false, native_only: true }` with a cargo-test hint.
#[wasm_bindgen]
pub fn run_conformance_lab_check(name: &str) -> Result<String, JsError> {
    lab_checks::run_conformance_lab_check(name).map_err(js_err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bindgen_list_classes_matches_api() {
        let from_fn = list_appliance_classes();
        let from_api = WasmApi::list_appliance_classes();
        assert_eq!(from_fn, from_api);
        assert!(from_fn.contains("\"kettle\""));
        assert!(from_fn.contains("\"group\""));
        assert!(from_fn.contains("\"wine_cooler\""));
        assert!(from_fn.contains("\"hvac\""));
        assert!(from_fn.contains("\"steam_oven\""));
    }

    #[test]
    fn bindgen_list_heat_port_specs_matches_api() {
        let from_fn = list_heat_port_specs("water_heater");
        let from_api = WasmApi::list_heat_port_specs("water_heater");
        assert_eq!(from_fn, from_api);
        assert!(from_fn.contains("\"preheat\""));
        assert!(from_fn.contains("\"sink\""));
        assert_eq!(list_heat_port_specs("kettle"), "[]");
    }

    #[test]
    fn bindgen_list_example_procedures_matches_api() {
        let from_fn = list_example_procedures();
        let from_api = WasmApi::list_example_procedures();
        assert_eq!(from_fn, from_api);
        assert!(from_fn.contains("\"kettle_heat_80\""));
        assert!(from_fn.contains("\"reheat_dominos_microwave\""));
        assert!(from_fn.contains("\"dishwasher_dhw_preheat\""));
    }

    #[test]
    fn bindgen_run_thermal_procedure_soft() {
        let raw = run_thermal_procedure("offer_fridge_dhw_soft").unwrap();
        assert!(raw.contains("completed"));
        assert!(raw.contains("thermal_offer"));
        assert!(raw.contains("120"));
    }

    #[test]
    fn bindgen_run_thermal_procedure_counter() {
        let raw = run_thermal_procedure("offer_fridge_dhw_counter").unwrap();
        assert!(raw.contains("completed"));
        assert!(raw.contains("thermal_offer"));
        assert!(raw.contains("counter"));
        assert!(raw.contains("120"));
    }

    #[test]
    fn bindgen_thermal_demo_raises_dhw() {
        let empty = thermal_state().unwrap();
        assert!(empty.contains("\"loaded\":false") || empty.contains("\"loaded\": false"));
        create_thermal_demo().unwrap();
        let tick = thermal_demo_transfer(3_600.0).unwrap();
        assert!(tick.contains("\"power_w\":120") || tick.contains("\"power_w\": 120"));
        assert!(tick.contains("36.2") || tick.contains("36.200"));
    }

    #[test]
    fn bindgen_thermal_then_dishwasher_preheat() {
        let raw = run_thermal_then_dishwasher_preheat(3_600.0).unwrap();
        assert!(raw.contains("thermal_then_dishwasher_preheat"));
        assert!(raw.contains("36.2") || raw.contains("36.200"));
        assert!(raw.contains("\"completed\"") || raw.contains("completed"));
        assert!(raw.contains("dishwasher"));
    }

    #[test]
    fn bindgen_list_conformance_scenarios_matches() {
        let from_fn = list_conformance_scenarios();
        let from_mod = lab_checks::list_conformance_scenarios();
        assert_eq!(from_fn, from_mod);
        assert!(from_fn.contains("\"catalog_hygiene\""));
        assert!(
            from_fn.contains("\"native_only\":true") || from_fn.contains("\"native_only\": true")
        );
    }

    #[test]
    fn bindgen_run_conformance_lab_check_hygiene_and_denial() {
        let h = run_conformance_lab_check("catalog_hygiene").unwrap();
        assert!(h.contains("\"passed\":true") || h.contains("\"passed\": true"));
        let w = run_conformance_lab_check("write_denial_matrix").unwrap();
        assert!(w.contains("\"passed\":true") || w.contains("\"passed\": true"));
        let n = run_conformance_lab_check("hub_lab_set_discover_describe").unwrap();
        assert!(n.contains("native_only"));
    }
}
