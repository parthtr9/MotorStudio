"""I/O layer: loads a robot_hw.yaml + its referenced URDF from disk. Kept
separate from checks.py (pure, no I/O) and schema.py (pure parsing of an
already-read dict) on purpose.
"""
from __future__ import annotations

from pathlib import Path

import yaml
from pydantic import ValidationError

from harness.schema import RobotSpec
from harness.urdf import Urdf


class SpecLoadError(Exception):
    """Wraps a pydantic ValidationError so callers that want Finding-shaped
    output don't need to import pydantic themselves."""

    def __init__(self, validation_error: ValidationError):
        self.validation_error = validation_error
        super().__init__(str(validation_error))


def load_robot(spec_path: str | Path) -> tuple[RobotSpec, Urdf]:
    """Loads robot_hw.yaml and the URDF it references (resolved relative
    to the yaml file's own directory). Raises SpecLoadError if the yaml
    doesn't validate against RobotSpec; URDF errors propagate as-is
    (ValueError/NotImplementedError from harness.urdf).
    """
    spec_path = Path(spec_path)
    raw = yaml.safe_load(spec_path.read_text())
    try:
        spec = RobotSpec.model_validate(raw)
    except ValidationError as e:
        raise SpecLoadError(e) from e

    urdf_path = spec_path.parent / spec.urdf_path
    urdf = Urdf.from_path(urdf_path)
    return spec, urdf


def scan_available_profiles(profiles_root: str | Path) -> set[tuple[str, str, str]]:
    """Scans profiles/<family>/<model>_<firmware>.yaml on disk, returning
    the set of (family, model, firmware) tuples found. I/O lives here,
    not in checks.py.
    """
    root = Path(profiles_root)
    found = set()
    if not root.is_dir():
        return found
    for family_dir in root.iterdir():
        if not family_dir.is_dir():
            continue
        for profile_file in family_dir.glob("*.yaml"):
            stem = profile_file.stem  # "<model>_<firmware>"
            if "_" not in stem:
                continue
            model, _, firmware = stem.rpartition("_")
            found.add((family_dir.name, model, firmware))
    return found
