// @vitest-environment jsdom
import { beforeAll, describe, expect, it } from "vitest";
import type { Feedback, TripLogEntry } from "./gatewayClient";

/**
 * main.ts builds its DOM (and kicks off a real WebSocket connection) as
 * module-level side effects, so it has to be imported dynamically after
 * `#app` exists in the document -- a static top-of-file import would run
 * before jsdom's document is ready for it. `main()`'s own WebSocket
 * connection will fail immediately (jsdom has no WebSocket), which is
 * fine: it's caught by main.ts's own `.catch`, and every function under
 * test here is independent of that connection.
 */
let mod: typeof import("./main");

beforeAll(async () => {
  document.body.innerHTML = '<div id="app"></div>';
  mod = await import("./main");
});

describe("jointPanelHtml", () => {
  it("includes the joint name and every feedback field", () => {
    const html = mod.jointPanelHtml("knee");
    expect(html).toContain('data-joint="knee"');
    expect(html).toContain(">knee<");
    for (const field of ["position_rad", "velocity_rad_s", "torque_nm", "winding_temp_degc", "fault"]) {
      expect(html).toContain(`data-field="${field}"`);
    }
  });
});

describe("renderFeedback", () => {
  it("fills in every field for an existing joint panel, rounded to 3 decimals", () => {
    document.querySelector("#joints")!.innerHTML = mod.jointPanelHtml("knee");
    const feedback: Feedback = {
      position_rad: 1.23456,
      velocity_rad_s: 0,
      torque_nm: -0.5,
      winding_temp_degc: 30,
      fault: 0,
    };
    mod.renderFeedback("knee", feedback);
    const panel = document.querySelector('[data-joint="knee"]')!;
    expect(panel.querySelector('[data-field="position_rad"]')!.textContent).toBe("1.235");
    expect(panel.querySelector('[data-field="velocity_rad_s"]')!.textContent).toBe("0.000");
    expect(panel.querySelector('[data-field="fault"]')!.textContent).toBe("0");
  });

  it("shows -- for fields the feedback omits", () => {
    document.querySelector("#joints")!.innerHTML = mod.jointPanelHtml("elbow");
    mod.renderFeedback("elbow", {});
    const panel = document.querySelector('[data-joint="elbow"]')!;
    expect(panel.querySelector('[data-field="position_rad"]')!.textContent).toBe("--");
    expect(panel.querySelector('[data-field="fault"]')!.textContent).toBe("--");
  });

  it("does nothing (not throw) for a joint with no rendered panel", () => {
    expect(() => mod.renderFeedback("ghost", { position_rad: 1 })).not.toThrow();
  });
});

describe("renderArmedStatus", () => {
  it("renders armed", () => {
    mod.renderArmedStatus(true);
    const el = document.querySelector("#armed-status")!;
    expect(el.textContent).toBe("armed: true");
    expect(el.className).toBe("armed");
  });

  it("renders disarmed", () => {
    mod.renderArmedStatus(false);
    const el = document.querySelector("#armed-status")!;
    expect(el.textContent).toBe("armed: false");
    expect(el.className).toBe("disarmed");
  });

  it("renders unknown when the status hasn't arrived yet", () => {
    mod.renderArmedStatus(undefined);
    const el = document.querySelector("#armed-status")!;
    expect(el.textContent).toBe("armed: unknown");
    expect(el.className).toBe("unknown");
  });
});

describe("tripLogActionText", () => {
  it("passes a plain string action through unchanged", () => {
    expect(mod.tripLogActionText("clamp")).toBe("clamp");
  });

  it("formats the single-key WatchdogFault(FaultPolicy) object shape", () => {
    expect(mod.tripLogActionText({ watchdog_fault: "damp" })).toBe("watchdog_fault(damp)");
  });
});

describe("tripLogEntryHtml", () => {
  it("includes joint, limit, commanded, allowed, and a string action", () => {
    const entry: TripLogEntry = {
      time_unix_ms: 0,
      joint: "knee",
      limit: "torque_max",
      commanded: 5,
      allowed: 3,
      action: "clamp",
    };
    const html = mod.tripLogEntryHtml(entry);
    expect(html).toContain("knee");
    expect(html).toContain("torque_max");
    expect(html).toContain("commanded 5");
    expect(html).toContain("allowed 3");
    expect(html).toContain("action clamp");
  });

  it("formats an object action the same way tripLogActionText does", () => {
    const entry: TripLogEntry = {
      time_unix_ms: 0,
      joint: "hip",
      limit: "watchdog",
      commanded: 1,
      allowed: 0,
      action: { watchdog_fault: "damp" },
    };
    expect(mod.tripLogEntryHtml(entry)).toContain("action watchdog_fault(damp)");
  });
});
