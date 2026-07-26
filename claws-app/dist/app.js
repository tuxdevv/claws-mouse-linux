const { invoke } = window.__TAURI__.core;

const POLL_VALUES = [125, 250, 500, 1000];
const HZ_TO_LEVEL = { 125: 1, 250: 2, 500: 3, 1000: 4 };
const POLL_HZ = { 1: 125, 2: 250, 3: 500, 4: 1000 };

const stagesEl = document.getElementById("stages");
const pollEl = document.getElementById("poll");
const statusEl = document.getElementById("status");
const modelEl = document.getElementById("model");

const stagePills = {};
const dpiSliders = {};
const dpiLabels = {};
const pollPills = {};

for (let stage = 1; stage <= 6; stage++) {
  const row = document.createElement("div");
  row.className = "stage-row";

  const pill = document.createElement("button");
  pill.className = "pill";
  pill.textContent = stage;
  pill.onclick = () => selectStage(stage);
  stagePills[stage] = pill;

  const slider = document.createElement("input");
  slider.type = "range";
  slider.min = 50;
  slider.max = 12000;
  slider.step = 50;
  slider.oninput = () => { dpiLabels[stage].textContent = slider.value; };
  slider.onchange = () => applyDpi(stage, parseInt(slider.value, 10));
  dpiSliders[stage] = slider;

  const label = document.createElement("div");
  label.className = "dpi-value";
  dpiLabels[stage] = label;

  row.append(pill, slider, label);
  stagesEl.appendChild(row);
}

for (const hz of POLL_VALUES) {
  const pill = document.createElement("button");
  pill.className = "pill";
  pill.textContent = `${hz} Hz`;
  pill.onclick = () => selectPoll(hz);
  pollPills[hz] = pill;
  pollEl.appendChild(pill);
}

function setStatus(text, kind) {
  statusEl.textContent = text;
  statusEl.className = kind || "";
}

async function refresh() {
  const p = await invoke("get_profile");
  for (const [stage, pill] of Object.entries(stagePills)) {
    pill.classList.toggle("active", Number(stage) === p.stage);
  }
  p.table.forEach((v, i) => {
    dpiSliders[i + 1].value = v;
    dpiLabels[i + 1].textContent = v;
  });
  const hz = POLL_HZ[p.polling] ?? p.polling;
  for (const [h, pill] of Object.entries(pollPills)) {
    pill.classList.toggle("active", Number(h) === hz);
  }
}

async function selectStage(stage) {
  try {
    await invoke("set_dpi_stage", { stage });
    setStatus(`active stage set to ${stage}`, "ok");
  } catch (e) {
    setStatus(String(e), "err");
  }
  refresh();
}

async function applyDpi(stage, value) {
  try {
    await invoke("set_dpi_value", { stage, value });
    setStatus(`stage ${stage} DPI set to ${value}`, "ok");
  } catch (e) {
    setStatus(String(e), "err");
  }
  refresh();
}

async function selectPoll(hz) {
  try {
    await invoke("set_polling", { level: HZ_TO_LEVEL[hz] });
    setStatus(`polling rate set to ${hz} Hz`, "ok");
  } catch (e) {
    setStatus(String(e), "err");
  }
  refresh();
}

(async () => {
  try {
    modelEl.textContent = await invoke("get_info");
  } catch (e) {
    modelEl.textContent = "";
  }
  await refresh();
})();
