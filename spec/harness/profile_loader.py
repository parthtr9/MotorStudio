"""Loads and validates profiles/<family>/<model>_<fw>.yaml against ProtocolProfile."""
from __future__ import annotations

from pathlib import Path

import yaml
from pydantic import ValidationError

from harness.profile_schema import ProtocolProfile


class ProfileLoadError(Exception):
    """Wraps a pydantic ValidationError, mirroring loader.py's
    SpecLoadError -- so `harness profile-lint` can print a clean, named
    reason (per root CLAUDE.md's "never guess... log + reject") instead
    of a raw traceback, same as `harness check`/`compile` already do for
    a broken robot_hw.yaml.
    """

    def __init__(self, validation_error: ValidationError):
        self.validation_error = validation_error
        super().__init__(str(validation_error))


def load_profile(path: str | Path) -> ProtocolProfile:
    data = yaml.safe_load(Path(path).read_text())
    try:
        return ProtocolProfile.model_validate(data)
    except ValidationError as e:
        raise ProfileLoadError(e) from e
