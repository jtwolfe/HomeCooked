const statusEl = document.getElementById("status");
const bootError = document.getElementById("boot-error");
const bootDetail = document.getElementById("boot-error-detail");
const appEl = document.getElementById("app");
const classSelect = document.getElementById("class-select");
const createBtn = document.getElementById("create-btn");
const deviceList = document.getElementById("device-list");
const emptyDevices = document.getElementById("empty-devices");
const placeholder = document.getElementById("placeholder");
const deviceView = document.getElementById("device-view");
const deviceClass = document.getElementById("device-class");
const deviceTitle = document.getElementById("device-title");
const deviceIdEl = document.getElementById("device-id");
const deviceHighlights = document.getElementById("device-highlights");
const pointGroups = document.getElementById("point-groups");
const rawState = document.getElementById("raw-state");
const dtInput = document.getElementById("dt-ms");
const tickBtn = document.getElementById("tick-btn");
const autoTick = document.getElementById("auto-tick");
const writeError = document.getElementById("write-error");
const procedureSelect = document.getElementById("procedure-select");
const loadProcedureBtn = document.getElementById("load-procedure-btn");
const parseProcedureBtn = document.getElementById("parse-procedure-btn");
const runProcedureBtn = document.getElementById("run-procedure-btn");
const procedureJson = document.getElementById("procedure-json");
const procedureError = document.getElementById("procedure-error");
const procedureSummary = document.getElementById("procedure-summary");
const procedureResult = document.getElementById("procedure-result");
const procedureStatus = document.getElementById("procedure-status");
const procedureFail = document.getElementById("procedure-fail");
const procedureBindings = document.getElementById("procedure-bindings");
const procedureSteps = document.getElementById("procedure-steps");

let wasm = null;
let selectedId = null;
let describeDoc = null;
let autoHandle = null;

function setStatus(text, warn = false) {
  statusEl.textContent = text;
  statusEl.classList.toggle("warn", warn);
}

function parseErr(err) {
  const text = err && err.message ? err.message : String(err);
  const start = text.indexOf("{");
  const end = text.lastIndexOf("}");
  if (start >= 0 && end > start) {
    try {
      return JSON.parse(text.slice(start, end + 1));
    } catch {
      /* fall through */
    }
  }
  return { message: text };
}

function showWriteError(err) {
  const info = parseErr(err);
  const bits = [info.code, info.point_id, info.message, info.expected].filter(Boolean);
  writeError.hidden = bits.length === 0;
  writeError.textContent = bits.join(" · ");
}

function showProcedureError(err) {
  const info = parseErr(err);
  const bits = [info.code, info.point_id, info.message, info.expected].filter(Boolean);
  procedureError.hidden = bits.length === 0;
  procedureError.textContent = bits.join(" · ") || String(err);
}

function clearProcedureError() {
  procedureError.hidden = true;
  procedureError.textContent = "";
}

function displayValue(tagged) {
  if (tagged == null) return "—";
  if (tagged.type === "void") return "void";
  if (tagged.type === "list") {
    return (tagged.value || []).map((item) => displayValue(item)).join(", ");
  }
  if (typeof tagged.value === "number") {
    return Number.isInteger(tagged.value) ? String(tagged.value) : tagged.value.toFixed(2);
  }
  return String(tagged.value);
}

function groupKey(id) {
  const parts = id.split(".");
  return parts.length >= 2 ? `${parts[0]}.${parts[1]}` : id;
}

function expandPoints(points, state) {
  const rows = [];
  for (const point of points) {
    const zones = point.zones && point.zones.length ? point.zones : [null];
    for (const zone of zones) {
      const id = zone ? `${point.id.split("#")[0]}#${zone}` : point.id;
      rows.push({
        ...point,
        id,
        current: state[id] || null,
      });
    }
  }
  return rows;
}

function renderDevices(devices) {
  deviceList.innerHTML = "";
  emptyDevices.hidden = devices.length > 0;
  for (const dev of devices) {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = dev.device_id === selectedId ? "active" : "";
    btn.innerHTML = `<strong>${dev.display_name || dev.class_id}</strong><span class="id">${dev.class_id} · ${dev.device_id}</span>`;
    btn.addEventListener("click", () => selectDevice(dev.device_id));
    li.appendChild(btn);
    deviceList.appendChild(li);
  }
}

function controlFor(point) {
  const wrap = document.createElement("div");
  wrap.className = "controls";
  if (point.command) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "cmd";
    btn.textContent = point.id.split(".").pop().replaceAll("_", " ");
    btn.addEventListener("click", () => applyWrite(point.id, null));
    wrap.appendChild(btn);
    return wrap;
  }
  if (!point.writable) {
    return wrap;
  }
  const type = point.type;
  if (type === "bool") {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "cmd secondary";
    const current = point.current && point.current.value;
    btn.textContent = current ? "Set false" : "Set true";
    btn.addEventListener("click", () => applyWrite(point.id, !current));
    wrap.appendChild(btn);
    return wrap;
  }
  if (type === "enum" && point.range && point.range.kind === "enum") {
    const select = document.createElement("select");
    for (const token of point.range.tokens || []) {
      const opt = document.createElement("option");
      opt.value = token;
      opt.textContent = token;
      if (point.current && point.current.value === token) opt.selected = true;
      select.appendChild(opt);
    }
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "cmd";
    btn.textContent = "Set";
    btn.addEventListener("click", () => applyWrite(point.id, select.value));
    wrap.append(select, btn);
    return wrap;
  }
  const input = document.createElement("input");
  const numeric = ["u8", "u16", "u32", "i16", "i32", "f32", "percent", "duration_s", "timestamp_ms"].includes(
    type,
  );
  if (numeric) {
    input.type = "number";
    if (point.range && (point.range.kind === "numeric" || point.range.kind === "integer")) {
      input.min = point.range.min;
      input.max = point.range.max;
    }
    if (["f32", "percent"].includes(type)) input.step = "any";
    if (point.current && typeof point.current.value === "number") {
      input.value = point.current.value;
    }
  } else {
    input.type = "text";
    if (point.current && point.current.value != null) input.value = point.current.value;
  }
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "cmd";
  btn.textContent = "Set";
  btn.addEventListener("click", () => {
    const value = numeric ? Number(input.value) : input.value;
    applyWrite(point.id, value);
  });
  wrap.append(input, btn);
  return wrap;
}

function highlightLabel(id) {
  const [base, zone] = id.split("#");
  const leaf = base.split(".").pop().replaceAll("_", " ");
  return zone ? `${leaf} (${zone})` : leaf;
}

function highlightRows(state) {
  const preferred = [
    "trait.power.power_state",
    "trait.temperature.current_c",
    "trait.temperature.setpoint_c",
    "trait.cycle.cycle_state",
  ];
  const rows = [];
  const seen = new Set();
  for (const id of preferred) {
    if (state && state[id] != null) {
      rows.push({ id, value: state[id] });
      seen.add(id);
    }
  }
  for (const id of Object.keys(state || {})) {
    if (seen.has(id)) continue;
    if (
      id.startsWith("trait.temperature.current_c") ||
      id.startsWith("trait.temperature.setpoint_c")
    ) {
      rows.push({ id, value: state[id] });
      seen.add(id);
    }
    if (rows.length >= 6) break;
  }
  return rows;
}

function renderHighlights(state) {
  const rows = highlightRows(state);
  deviceHighlights.innerHTML = "";
  deviceHighlights.hidden = rows.length === 0;
  for (const row of rows) {
    const li = document.createElement("li");
    li.innerHTML = `<span class="hl-label">${highlightLabel(row.id)}</span><span class="hl-value">${displayValue(row.value)}</span>`;
    deviceHighlights.appendChild(li);
  }
}

function fillClassSelect(classes) {
  classSelect.innerHTML = "";
  let currentGroup = null;
  let optgroup = null;
  for (const cls of classes) {
    const group = cls.group || "Other";
    if (group !== currentGroup) {
      currentGroup = group;
      optgroup = document.createElement("optgroup");
      optgroup.label = group;
      classSelect.appendChild(optgroup);
    }
    const opt = document.createElement("option");
    opt.value = cls.id;
    opt.textContent = `${cls.label} (${cls.id})`;
    optgroup.appendChild(opt);
  }
  if (classes.some((c) => c.id === "kettle")) {
    classSelect.value = "kettle";
  }
}

function renderDevice(desc, state) {
  describeDoc = desc;
  const classId = desc.identity.class_id;
  const name = desc.identity.display_name || classId;
  deviceClass.textContent = classId;
  deviceTitle.textContent = `${name} · ${classId}`;
  deviceIdEl.textContent = desc.identity.device_id;
  renderHighlights(state);
  rawState.textContent = JSON.stringify(state, null, 2);

  const rows = expandPoints(desc.points || [], state);
  const groups = new Map();
  for (const row of rows) {
    const key = groupKey(row.id);
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(row);
  }

  pointGroups.innerHTML = "";
  for (const [key, items] of groups) {
    const section = document.createElement("section");
    section.className = "group";
    const h = document.createElement("h3");
    h.textContent = key;
    const list = document.createElement("div");
    list.className = "points";
    for (const point of items) {
      const row = document.createElement("div");
      row.className = "row";
      const left = document.createElement("div");
      const access = point.command ? "command" : point.access;
      left.innerHTML = `<span class="pid">${point.id}</span><span class="meta">${point.type}${
        point.unit ? " · " + point.unit : ""
      } · ${access}</span>`;
      const mid = document.createElement("div");
      mid.className = "value";
      mid.textContent = displayValue(point.current);
      if (point.type === "percent" && point.current && typeof point.current.value === "number") {
        const bar = document.createElement("div");
        bar.className = "progress";
        bar.innerHTML = `<span style="width:${Math.max(0, Math.min(100, point.current.value))}%"></span>`;
        mid.appendChild(bar);
      }
      row.append(left, mid, controlFor(point));
      list.appendChild(row);
    }
    section.append(h, list);
    pointGroups.appendChild(section);
  }
}

async function refreshDevices() {
  const devices = JSON.parse(wasm.list_devices());
  renderDevices(devices);
  if (selectedId && !devices.some((d) => d.device_id === selectedId)) {
    selectedId = null;
    deviceView.hidden = true;
    placeholder.hidden = false;
  }
}

async function selectDevice(id) {
  selectedId = id;
  writeError.hidden = true;
  const desc = JSON.parse(wasm.describe(id));
  const state = JSON.parse(wasm.get_state(id));
  placeholder.hidden = true;
  deviceView.hidden = false;
  renderDevice(desc, state);
  await refreshDevices();
}

async function applyWrite(point, value) {
  if (!selectedId) return;
  try {
    wasm.write(selectedId, point, JSON.stringify(value));
    writeError.hidden = true;
    const state = JSON.parse(wasm.get_state(selectedId));
    renderDevice(describeDoc || JSON.parse(wasm.describe(selectedId)), state);
    setStatus(`Wrote ${point}`);
  } catch (err) {
    showWriteError(err);
    setStatus("Write rejected", true);
  }
}

function dtMs() {
  const n = Number(dtInput.value);
  return Number.isFinite(n) && n > 0 ? Math.floor(n) : 250;
}

function doTick() {
  if (!selectedId) return;
  try {
    const state = JSON.parse(wasm.tick(selectedId, dtMs()));
    renderDevice(describeDoc || JSON.parse(wasm.describe(selectedId)), state);
  } catch (err) {
    showWriteError(err);
  }
}

function syncAutoTick() {
  if (autoHandle) {
    clearInterval(autoHandle);
    autoHandle = null;
  }
  if (autoTick.checked) {
    autoHandle = setInterval(doTick, Math.max(50, dtMs()));
    setStatus("Auto tick on");
  }
}

async function boot() {
  try {
    wasm = await import("./pkg/homecooked_wasm.js");
    await wasm.default();
  } catch (err) {
    bootError.hidden = false;
    bootDetail.textContent = String(err);
    setStatus("WASM not loaded", true);
    return;
  }

  const classes = JSON.parse(wasm.list_appliance_classes());
  fillClassSelect(classes);
  fillProcedureSelect(listExampleProcedures());
  await loadSelectedProcedure();

  appEl.hidden = false;
  setStatus("Simulator ready");
  await refreshDevices();
}

createBtn.addEventListener("click", async () => {
  try {
    const id = wasm.create_device(classSelect.value);
    setStatus(`Created ${id}`);
    await selectDevice(id);
  } catch (err) {
    showWriteError(err);
    setStatus("Create failed", true);
  }
});

tickBtn.addEventListener("click", doTick);
autoTick.addEventListener("change", syncAutoTick);
dtInput.addEventListener("change", () => {
  if (autoTick.checked) syncAutoTick();
});

loadProcedureBtn.addEventListener("click", () => loadSelectedProcedure());
procedureSelect.addEventListener("change", () => loadSelectedProcedure());
parseProcedureBtn.addEventListener("click", () => parseCurrentProcedure());
runProcedureBtn.addEventListener("click", () => runCurrentProcedure());

boot();

function listExampleProcedures() {
  try {
    return JSON.parse(wasm.list_example_procedures());
  } catch {
    return [
      { id: "kettle_heat_80", name: "Heat kettle to 80C", class_hints: ["kettle"] },
      {
        id: "reheat_dominos_microwave",
        name: "Reheat 2 Domino's supreme slices (microwave)",
        class_hints: ["microwave"],
      },
    ];
  }
}

function fillProcedureSelect(examples) {
  procedureSelect.innerHTML = "";
  for (const ex of examples) {
    const opt = document.createElement("option");
    opt.value = ex.id;
    const hints = (ex.class_hints || []).join(", ");
    opt.textContent = hints ? `${ex.name} (${hints})` : ex.name;
    procedureSelect.appendChild(opt);
  }
}

async function fetchProcedureAsset(id) {
  const res = await fetch(`./procedures/${id}.json`);
  if (!res.ok) {
    throw new Error(`could not fetch ./procedures/${id}.json (${res.status})`);
  }
  return res.text();
}

async function loadSelectedProcedure() {
  const id = procedureSelect.value;
  if (!id) return;
  clearProcedureError();
  procedureResult.hidden = true;
  try {
    let json;
    try {
      json = wasm.get_example_procedure(id);
    } catch {
      json = await fetchProcedureAsset(id);
    }
    procedureJson.value = JSON.stringify(JSON.parse(json), null, 2);
    setStatus(`Loaded ${id}`);
    parseCurrentProcedure();
  } catch (err) {
    showProcedureError(err);
    setStatus("Could not load procedure", true);
  }
}

function parseCurrentProcedure() {
  clearProcedureError();
  procedureResult.hidden = true;
  const raw = procedureJson.value.trim();
  if (!raw) {
    showProcedureError({ message: "Paste or load a procedure JSON document first." });
    return null;
  }
  try {
    const summary = JSON.parse(wasm.parse_procedure(raw));
    const hints = (summary.class_hints || []).join(", ") || "no class hints";
    const roles = (summary.devices || [])
      .map((d) => (d.optional ? `${d.role}?` : d.role))
      .join(", ");
    procedureSummary.hidden = false;
    procedureSummary.textContent = `${summary.name} · ${summary.step_count} steps · ${hints}${
      roles ? ` · roles ${roles}` : ""
    }`;
    return summary;
  } catch (err) {
    procedureSummary.hidden = true;
    showProcedureError(err);
    setStatus("Procedure parse failed", true);
    return null;
  }
}

function renderProcedureResult(result) {
  procedureResult.hidden = false;
  const ok = result.status === "completed";
  procedureStatus.textContent = ok ? "Completed" : "Failed";
  procedureStatus.classList.toggle("ok", ok);
  procedureStatus.classList.toggle("fail", !ok);

  if (!ok && result.fail_reason) {
    const bits = [
      result.failed_step_id ? `step ${result.failed_step_id}` : null,
      result.fail_reason.kind,
      result.fail_reason.code,
      result.fail_reason.message,
    ].filter(Boolean);
    procedureFail.textContent = bits.join(" · ");
  } else {
    procedureFail.textContent = "";
  }

  procedureBindings.innerHTML = "";
  const bindings = result.bindings || [];
  procedureBindings.hidden = bindings.length === 0;
  for (const bind of bindings) {
    const li = document.createElement("li");
    li.textContent = `${bind.role} → ${bind.device_id} (${bind.class_id}${
      bind.spawned ? ", spawned" : ""
    })`;
    procedureBindings.appendChild(li);
  }

  procedureSteps.innerHTML = "";
  for (const step of result.outcomes || []) {
    const li = document.createElement("li");
    li.className = step.ok ? "ok" : "fail";
    const value =
      step.read_value != null ? ` · ${displayValue(step.read_value)}` : "";
    const msg = step.message ? ` — ${step.message}` : "";
    li.innerHTML = `<span class="step-flag">${step.ok ? "ok" : "fail"}</span><span class="step-id">${
      step.step_id
    }</span><span class="step-meta">${step.action}${value}${msg}</span>`;
    procedureSteps.appendChild(li);
  }
}

async function runCurrentProcedure() {
  clearProcedureError();
  const raw = procedureJson.value.trim();
  if (!raw) {
    showProcedureError({ message: "Paste or load a procedure JSON document first." });
    return;
  }
  try {
    const result = JSON.parse(wasm.run_procedure(raw));
    renderProcedureResult(result);
    await refreshDevices();
    const firstBound = result.bindings && result.bindings[0] && result.bindings[0].device_id;
    if (firstBound) {
      await selectDevice(firstBound);
    }
    setStatus(result.status === "completed" ? "Procedure completed" : "Procedure failed", result.status !== "completed");
  } catch (err) {
    procedureResult.hidden = true;
    showProcedureError(err);
    setStatus("Procedure run failed", true);
  }
}
