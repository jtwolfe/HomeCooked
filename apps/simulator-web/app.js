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
const pointGroups = document.getElementById("point-groups");
const rawState = document.getElementById("raw-state");
const dtInput = document.getElementById("dt-ms");
const tickBtn = document.getElementById("tick-btn");
const autoTick = document.getElementById("auto-tick");
const writeError = document.getElementById("write-error");

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
    btn.innerHTML = `<strong>${dev.display_name || dev.class_id}</strong><span class="id">${dev.device_id}</span>`;
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

function renderDevice(desc, state) {
  describeDoc = desc;
  const name = desc.identity.display_name || desc.identity.class_id;
  deviceClass.textContent = desc.identity.class_id;
  deviceTitle.textContent = name;
  deviceIdEl.textContent = desc.identity.device_id;
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
  classSelect.innerHTML = "";
  for (const cls of classes) {
    const opt = document.createElement("option");
    opt.value = cls.id;
    opt.textContent = cls.label;
    classSelect.appendChild(opt);
  }
  classSelect.value = "kettle";

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

boot();
