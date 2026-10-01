import { describe, expect, it } from "vitest";
import { GatewayClient, type WebSocketLike } from "./gatewayClient";

/**
 * Minimal fake satisfying WebSocketLike -- no jsdom, no real socket.
 * `sent` captures every message the client tried to send, in order, so
 * tests can inspect exactly what went over the wire and reply to it
 * explicitly (mirroring real usage: the server only replies once it
 * has actually seen a request).
 */
type Listener = (event: { data?: unknown }) => void;

class FakeWebSocket implements WebSocketLike {
  sent: string[] = [];
  closed = false;
  private listeners: Record<string, Listener[]> = {};

  addEventListener(type: "open" | "message" | "close" | "error", listener: Listener): void {
    (this.listeners[type] ??= []).push(listener);
  }

  send(data: string): void {
    this.sent.push(data);
  }

  close(): void {
    this.closed = true;
    this.emit("close", {});
  }

  open(): void {
    this.emit("open", {});
  }

  reply(data: unknown): void {
    this.emit("message", { data: JSON.stringify(data) });
  }

  errorBeforeOpen(): void {
    this.emit("error", {});
  }

  private emit(type: string, event: { data?: unknown }): void {
    for (const listener of this.listeners[type] ?? []) listener(event);
  }
}

function connected(): { ws: FakeWebSocket; client: GatewayClient } {
  const ws = new FakeWebSocket();
  const client = new GatewayClient(ws);
  ws.open();
  return { ws, client };
}

describe("GatewayClient", () => {
  it("resolves ready() once the socket opens", async () => {
    const ws = new FakeWebSocket();
    const client = new GatewayClient(ws);
    const readyPromise = client.ready();
    ws.open();
    await expect(readyPromise).resolves.toBeUndefined();
  });

  it("rejects ready() if an error arrives before open", async () => {
    const ws = new FakeWebSocket();
    const client = new GatewayClient(ws);
    const readyPromise = client.ready();
    ws.errorBeforeOpen();
    await expect(readyPromise).rejects.toThrow();
  });

  it("sends a well-formed arm request and resolves with the response", async () => {
    const { ws, client } = connected();
    const armPromise = client.arm();
    expect(ws.sent).toEqual([JSON.stringify({ cmd: "arm" })]);
    ws.reply({ ok: true });
    await expect(armPromise).resolves.toEqual({ ok: true });
  });

  it("flattens command fields alongside joint, matching the Rust serde shape", async () => {
    const { ws, client } = connected();
    void client.command("knee", { torque_nm: 1.5 });
    expect(JSON.parse(ws.sent[0])).toEqual({ cmd: "command", joint: "knee", torque_nm: 1.5 });
  });

  it("matches responses to requests strictly by arrival order", async () => {
    const { ws, client } = connected();
    const first = client.status();
    const second = client.arm();

    // Server processes/answers in send order (per the module's own
    // documented contract) -- reply to them in that order here too.
    ws.reply({ ok: true, data: { armed: false, joints: ["knee"] } });
    ws.reply({ ok: true });

    await expect(first).resolves.toEqual({ ok: true, data: { armed: false, joints: ["knee"] } });
    await expect(second).resolves.toEqual({ ok: true });
  });

  it("rejects all pending requests when the connection closes", async () => {
    const { ws, client } = connected();
    const pending = client.arm();
    ws.close();
    await expect(pending).rejects.toThrow("connection closed");
  });

  it("trip_log omits limit when not provided", async () => {
    const { ws, client } = connected();
    void client.tripLog();
    expect(JSON.parse(ws.sent[0])).toEqual({ cmd: "trip_log" });
  });

  it("trip_log includes limit when provided", async () => {
    const { ws, client } = connected();
    void client.tripLog(5);
    expect(JSON.parse(ws.sent[0])).toEqual({ cmd: "trip_log", limit: 5 });
  });

  it("surfaces an error response without throwing (ok:false is a normal value, not an exception)", async () => {
    const { ws, client } = connected();
    const pending = client.command("knee", { torque_nm: 1.0 });
    ws.reply({ ok: false, error: "not armed" });
    await expect(pending).resolves.toEqual({ ok: false, error: "not armed" });
  });
});
