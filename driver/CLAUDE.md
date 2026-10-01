Rust. Hot path = gateway loop. No allocation, no locks held across await, no panics in loop.
Every command: decode -> enforce -> encode -> send. enforce/ is human-owned.
Latency budget 0.5 ms p99 added. Bench with cargo bench before/after any loop change.
Backend trait is the only vendor seam.
