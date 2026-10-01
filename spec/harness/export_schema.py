"""Export RobotSpec as JSON Schema. Run via `python -m harness.export_schema`."""
import json
import sys
from pathlib import Path

from harness.schema import RobotSpec


def main() -> None:
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("spec/schema/robot_hw.schema.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(RobotSpec.model_json_schema(), indent=2) + "\n")
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
