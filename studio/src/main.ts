import { GatewayClient, type Feedback, type TripLogEntry } from "./gatewayClient";
import "./style.css";

/**
 * Entry point. Per the sprint plan's M4 goal, "no family-specific UI
 * code" -- every joint panel is generated from whatever `status()`
 * reports, not hand-written per motor. There is no real per-family
 * profile to drive richer panels (parameter tables, decoded faults)
 * yet -- see profiles/robstride/OPEN_QUESTIONS.md -- so this UI's
 * scope is deliberately limited to what the gateway's current API
 * actually provides: arm/disarm, per-joint feedback polling, jog
 * commands, and the trip log. Parameter tables and raw-vs-decoded
 * frame views wait on real profile data.
 */

function gatewayUrl(): string {
  const params = new URLSearchParams(window.location.search);
  return params.get("gateway") ?? `ws://${window.location.hostname || "localhost"}:8765`;
}

const app = document.querySelector<HTMLDivElement>("#app")!;
app.innerHTML = `
  <header>
    <h1>Actuator Harness Studio</h1>
    <div id="connection-status">connecting...</div>
    <div id="armed-status" class="unknown">armed: unknown</div>
    <div id="controls">
      <button id="arm-btn">Arm</button>
      <button id="disarm-btn">Disarm</button>
    </div>
  </header>
  <main id="joints"></main>
  <section id="trip-log">
    <h2>Trip log</h2>
    <ul id="trip-log-list"></ul>
  </section>
`;

const connectionStatusEl = document.querySelector<HTMLDivElement>("#connection-status")!;
const armedStatusEl = document.querySelector<HTMLDivElement>("#armed-status")!;
const jointsEl = document.querySelector<HTMLDivElement>("#joints")!;
const tripLogListEl = document.querySelector<HTMLUListElement>("#trip-log-list")!;
const armBtn = document.querySelector<HTMLButtonElement>("#arm-btn")!;
const disarmBtn = document.querySelector<HTMLButtonElement>("#disarm-btn")!;

/** Exported for main.test.ts -- pure HTML/formatting, no DOM dependency. */
export function renderArmedStatus(armed: boolean | undefined): void {
  armedStatusEl.textContent = armed === undefined ? "armed: unknown" : `armed: ${armed}`;
  armedStatusEl.className = armed === undefined ? "unknown" : armed ? "armed" : "disarmed";
}

/** `action` is either a plain string (unit EnforceAction variants) or a
 * single-key object (WatchdogFault(FaultPolicy)) -- see
 * gatewayClient.ts's TripLogEntry doc. */
export function tripLogActionText(action: string | Record<string, string>): string {
  if (typeof action === "string") return action;
  const [key, value] = Object.entries(action)[0];
  return `${key}(${value})`;
}

export function tripLogEntryHtml(entry: TripLogEntry): string {
  const time = new Date(entry.time_unix_ms).toLocaleTimeString();
  return `<li>${time} -- ${entry.joint}: ${entry.limit} (commanded ${entry.commanded}, allowed ${entry.allowed}, action ${tripLogActionText(entry.action)})</li>`;
}

export function jointPanelHtml(joint: string): string {
  return `
    <article class="joint-panel" data-joint="${joint}">
      <h3>${joint}</h3>
      <dl class="feedback">
        <dt>position (rad)</dt><dd data-field="position_rad">--</dd>
        <dt>velocity (rad/s)</dt><dd data-field="velocity_rad_s">--</dd>
        <dt>torque (Nm)</dt><dd data-field="torque_nm">--</dd>
        <dt>winding temp (degC)</dt><dd data-field="winding_temp_degc">--</dd>
        <dt>fault bits</dt><dd data-field="fault">--</dd>
      </dl>
      <label>
        torque_nm
        <input type="number" step="0.1" class="jog-torque" value="0" />
      </label>
      <button class="send-command">Send command</button>
    </article>
  `;
}

export function renderFeedback(joint: string, feedback: Feedback): void {
  const panel = jointsEl.querySelector(`[data-joint="${joint}"]`);
  if (!panel) return;
  const set = (field: string, value: number | undefined) => {
    const el = panel.querySelector(`[data-field="${field}"]`);
    if (el) el.textContent = value === undefined ? "--" : value.toFixed(3);
  };
  set("position_rad", feedback.position_rad);
  set("velocity_rad_s", feedback.velocity_rad_s);
  set("torque_nm", feedback.torque_nm);
  set("winding_temp_degc", feedback.winding_temp_degc);
  const faultEl = panel.querySelector('[data-field="fault"]');
  if (faultEl) faultEl.textContent = feedback.fault === undefined ? "--" : String(feedback.fault);
}

async function main(): Promise<void> {
  const ws = new WebSocket(gatewayUrl());
  const client = new GatewayClient(ws);

  try {
    await client.ready();
    connectionStatusEl.textContent = `connected to ${gatewayUrl()}`;
  } catch {
    connectionStatusEl.textContent = `failed to connect to ${gatewayUrl()}`;
    return;
  }

  const statusResp = await client.status();
  const joints = statusResp.data?.joints ?? [];
  jointsEl.innerHTML = joints.map(jointPanelHtml).join("");
  renderArmedStatus(statusResp.data?.armed);

  for (const joint of joints) {
    const panel = jointsEl.querySelector(`[data-joint="${joint}"]`)!;
    panel.querySelector(".send-command")!.addEventListener("click", async () => {
      const input = panel.querySelector<HTMLInputElement>(".jog-torque")!;
      const torque_nm = Number.parseFloat(input.value);
      const resp = await client.command(joint, { torque_nm });
      if (!resp.ok) {
        console.error(`command to ${joint} rejected: ${resp.error}`);
      }
    });
  }

  armBtn.addEventListener("click", async () => {
    const resp = await client.arm();
    if (!resp.ok) console.error(`arm rejected: ${resp.error}`);
    else renderArmedStatus(true);
  });
  disarmBtn.addEventListener("click", async () => {
    const resp = await client.disarm();
    if (!resp.ok) console.error(`disarm rejected: ${resp.error}`);
    else renderArmedStatus(false);
  });

  async function pollFeedback(): Promise<void> {
    for (const joint of joints) {
      const resp = await client.feedback(joint);
      if (resp.ok && resp.data) renderFeedback(joint, resp.data);
    }
  }

  async function pollTripLog(): Promise<void> {
    const resp = await client.tripLog(20);
    if (!resp.ok || !resp.data) return;
    tripLogListEl.innerHTML = resp.data.map(tripLogEntryHtml).join("");
  }

  setInterval(() => {
    void pollFeedback();
    void pollTripLog();
  }, 500);
}

main().catch((e) => {
  connectionStatusEl.textContent = `error: ${e instanceof Error ? e.message : String(e)}`;
});
