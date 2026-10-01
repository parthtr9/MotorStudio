.PHONY: vcan-up test gates check quick-test

vcan-up:
	./scripts/vcan-up.sh

test:
	cd spec && python3 -m venv .venv --upgrade-deps >/dev/null 2>&1 || true
	cd spec && .venv/bin/pip install -q -e . pytest
	cd spec && .venv/bin/python -m pytest -q
ifneq (,$(wildcard driver/Cargo.toml))
	cd driver && cargo test
endif
ifneq (,$(wildcard studio/package.json))
	cd studio && pnpm install && pnpm test
endif
	cd mcp && python3 -m venv .venv --upgrade-deps >/dev/null 2>&1 || true
	cd mcp && .venv/bin/pip install -q -e . -e ../spec pytest
	cd mcp && .venv/bin/python -m pytest -q

# make gates M=2
gates:
	@if [ -z "$(M)" ]; then echo "usage: make gates M=<milestone number>"; exit 1; fi
	cd spec && .venv/bin/python -m pytest -q ../tests/gates/m$(M)

# make check SPEC=path/to/robot_hw.yaml
check:
	@if [ -z "$(SPEC)" ]; then echo "usage: make check SPEC=<path to robot_hw.yaml>"; exit 1; fi
	cd spec && .venv/bin/python -m harness.cli check $(SPEC)

quick-test:
	@cd spec && .venv/bin/python -m pytest -q -x 2>/dev/null || true
