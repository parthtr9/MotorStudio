"""Loads and validates profiles/<family>/<model>_<fw>.yaml against ProtocolProfile."""
from __future__ import annotations

from pathlib import Path

import yaml

from harness.profile_schema import ProtocolProfile


def load_profile(path: str | Path) -> ProtocolProfile:
    data = yaml.safe_load(Path(path).read_text())
    return ProtocolProfile.model_validate(data)
