#!/usr/bin/env bash
# Brings up vcan0 and vcan1 for local/CI testing. Linux only (SocketCAN).
# M0 gate: `make vcan-up && make test` must pass with no manual steps on a
# clean Ubuntu 24.04 machine.
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "vcan-up: SocketCAN vcan is Linux-only (uname -s = $(uname -s))." >&2
  echo "vcan-up: on macOS/other, driver/spec tests that need a bus should use the replay transport instead." >&2
  exit 1
fi

if ! lsmod | grep -q '^vcan'; then
  sudo modprobe vcan
fi

for IF in vcan0 vcan1; do
  if ip link show "$IF" &>/dev/null; then
    echo "vcan-up: $IF already exists, skipping create"
  else
    sudo ip link add dev "$IF" type vcan
  fi
  sudo ip link set up "$IF"
  echo "vcan-up: $IF is up"
done
