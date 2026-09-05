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

/// JSON array of spawnable Tier-A classes:
/// `[{"id":"kettle","label":"Kettle","group":"Beverage"}, ...]`.
#[wasm_bindgen]
pub fn list_appliance_classes() -> String {
    WasmApi::list_appliance_classes()
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
}
