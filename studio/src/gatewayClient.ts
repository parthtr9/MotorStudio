/**
 * Typed client for the gateway's network API (driver/src/gateway/
 * protocol.rs). One request per WebSocket text message, one response
 * per message, matched to its request by ARRIVAL ORDER, not an
 * explicit ID -- the protocol has none. This is safe only because the
 * server (driver/src/gateway/server.rs's handle_ws_connection) reads
 * and fully dispatches one message before reading the next on a given
 * connection, so responses arrive in the same order requests were
 * sent. Do not add a second concurrent sender against the same
 * WebSocket without adding request IDs on both sides first.
 *
 * `WebSocketLike` is the minimal interface actually used, not the full
 * DOM `WebSocket` type -- this lets tests supply a plain mock object
 * with no jsdom/browser environment required, and lets the real
 * browser `WebSocket` (or Node's built-in one) satisfy it structurally
 * with no adapter.
 */

export interface WebSocketLike {
  send(data: string): void;
  close(): void;
  /** Single signature, not DOM-style overloads: an overloaded interface
   * method requires every implementer to repeat the exact same set of
   * overload signatures to satisfy it structurally, which is needless
   * friction for a test mock. A real `WebSocket` still satisfies this
   * fine -- TS allows a listener to ignore an event parameter it
   * doesn't need. */
  addEventListener(
    type: "open" | "message" | "close" | "error",
    listener: (event: { data?: unknown }) => void,
  ): void;
}

/** Matches driver/src/backend.rs's Command exactly -- all fields optional. */
export interface Command {
  position_rad?: number;
  velocity_rad_s?: number;
  torque_nm?: number;
  kp?: number;
  kd?: number;
}

/** Matches driver/src/backend.rs's Feedback. */
export interface Feedback {
  position_rad?: number;
  velocity_rad_s?: number;
  torque_nm?: number;
  winding_temp_degc?: number;
  /** FaultBits(u32) -- serde's newtype representation serializes this
   * as a bare number, not {"0": n} or [n]. */
  fault?: number;
}

/** Matches driver/src/gateway/types.rs's TripLogEntry. `action` is
 * either a plain snake_case string (unit variants of EnforceAction) or
 * a single-key object for the one variant that carries data
 * (WatchdogFault(FaultPolicy) -> {"watchdog_fault": "damp"}). */
export interface TripLogEntry {
  time_unix_ms: number;
  joint: string;
  limit: string;
  commanded: number;
  allowed: number;
  action: string | Record<string, string>;
}

export interface StatusData {
  armed: boolean;
  joints: string[];
}

export interface GatewayResponse<T = unknown> {
  ok: boolean;
  error?: string;
  data?: T;
}

type Request =
  | { cmd: "arm" }
  | { cmd: "disarm" }
  | { cmd: "status" }
  | { cmd: "feedback"; joint: string }
  | ({ cmd: "command"; joint: string } & Command)
  | { cmd: "trip_log"; limit?: number };

interface PendingRequest {
  resolve: (response: GatewayResponse) => void;
  reject: (error: Error) => void;
}

export class GatewayClient {
  private readonly ws: WebSocketLike;
  private readonly queue: PendingRequest[] = [];
  private readonly openPromise: Promise<void>;

  constructor(ws: WebSocketLike) {
    this.ws = ws;
    this.openPromise = new Promise((resolve, reject) => {
      this.ws.addEventListener("open", () => resolve());
      this.ws.addEventListener("error", () => reject(new Error("WebSocket error before open")));
    });
    this.ws.addEventListener("message", (event) => this.handleMessage(event.data));
    this.ws.addEventListener("close", () => this.rejectAllPending(new Error("connection closed")));
  }

  /** Resolves once the underlying WebSocket has opened. */
  ready(): Promise<void> {
    return this.openPromise;
  }

  close(): void {
    this.ws.close();
  }

  arm(): Promise<GatewayResponse> {
    return this.send({ cmd: "arm" });
  }

  disarm(): Promise<GatewayResponse> {
    return this.send({ cmd: "disarm" });
  }

  status(): Promise<GatewayResponse<StatusData>> {
    return this.send({ cmd: "status" }) as Promise<GatewayResponse<StatusData>>;
  }

  feedback(joint: string): Promise<GatewayResponse<Feedback>> {
    return this.send({ cmd: "feedback", joint }) as Promise<GatewayResponse<Feedback>>;
  }

  command(joint: string, command: Command): Promise<GatewayResponse> {
    return this.send({ cmd: "command", joint, ...command });
  }

  tripLog(limit?: number): Promise<GatewayResponse<TripLogEntry[]>> {
    return this.send({ cmd: "trip_log", limit }) as Promise<GatewayResponse<TripLogEntry[]>>;
  }

  private send(request: Request): Promise<GatewayResponse> {
    return new Promise((resolve, reject) => {
      this.queue.push({ resolve, reject });
      this.ws.send(JSON.stringify(request));
    });
  }

  private handleMessage(data: unknown): void {
    const pending = this.queue.shift();
    if (!pending) {
      // A message with nothing waiting for it. Per this module's own
      // one-in-flight-implied-by-ordering contract, this should not
      // happen; surfacing it as a dropped message (console, not a
      // thrown error with no one to catch it) beats pretending it
      // didn't happen.
      console.error("GatewayClient: received a message with no pending request", data);
      return;
    }
    try {
      const text = typeof data === "string" ? data : String(data);
      pending.resolve(JSON.parse(text) as GatewayResponse);
    } catch (e) {
      pending.reject(e instanceof Error ? e : new Error(String(e)));
    }
  }

  private rejectAllPending(error: Error): void {
    while (this.queue.length > 0) {
      const pending = this.queue.shift();
      pending?.reject(error);
    }
  }
}
