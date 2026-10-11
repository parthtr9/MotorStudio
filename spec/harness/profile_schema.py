"""Schema for profiles/<family>/<model>_<fw>.yaml.

This is the enforcement mechanism for the project's hardest rule: never
guess a protocol detail. Every CAN frame and every field in a frame must
cite the manual section it came from (`manual_ref`). A profile author who
is unsure what a frame or field means must not write a value for it --
they record it in `open_questions` instead, and `ProtocolProfile` refuses
to validate a profile that invents a `manual_ref` for something undocumented.

This schema does not contain protocol data for any real motor. It is the
shape that real, manual-cited data gets poured into once sourced.
"""
from __future__ import annotations

from enum import Enum

from pydantic import BaseModel, Field, model_validator


class Direction(str, Enum):
    tx = "tx"  # host -> motor
    rx = "rx"  # motor -> host


class ByteOrder(str, Enum):
    big = "big"
    little = "little"


class FrameField(BaseModel):
    """One field inside a CAN frame's data bytes.

    `manual_ref` must point at a specific section/page/table in the vendor
    manual (e.g. "RobStride CAN Protocol v1.2 sec 4.3 table 7"). It is not
    a free-text description -- it must be checkable by a human against the
    source document.

    `byte_order` has no default on purpose: this project has already
    wrapped two real families (RobStride, Damiao -- see
    driver/src/backends/) whose own vendor crates disagree on it (RobStride
    packs big-endian; Damiao's pos/vel/torque command frames are
    little-endian), so defaulting to either would just be a different
    guess. The raw integer this field decodes to is always read as
    unsigned over `byte_order`; `scale`/`offset` are what map that into a
    signed physical range (the same affine pattern both real families
    already use -- see motor_vendor_robstride's/motor_vendor_damiao's own
    uint_to_float-style conversions), so no separate signed/two's-
    complement flag is needed here.
    """

    model_config = {"extra": "forbid"}

    name: str
    byte_offset: int = Field(ge=0)
    byte_length: int = Field(ge=1, le=8)
    byte_order: ByteOrder  # required, no default -- see this class's doc
    scale: float = 1.0
    offset: float = 0.0
    unit: str  # SI unit per root CLAUDE.md: rad, rad/s, Nm, A, V, degC, s, or "raw"/"enum"
    manual_ref: str = Field(min_length=1)

    @model_validator(mode="after")
    def _fits_in_frame(self) -> FrameField:
        if self.byte_offset + self.byte_length > 8:
            raise ValueError(
                f"field {self.name!r}: byte_offset+byte_length exceeds an 8-byte CAN frame"
            )
        return self


class FrameSpec(BaseModel):
    """One CAN frame: either something we send (tx) or something the motor
    sends back (rx). `can_id` is either a literal 11/29-bit ID or, when the
    family encodes the motor's bus ID into the arbitration ID, an
    expression string the codec evaluates (documented in `can_id_note`).
    """

    model_config = {"extra": "forbid"}

    name: str
    direction: Direction
    can_id: int | None = Field(default=None, ge=0, le=0x1FFFFFFF)
    can_id_expr: str | None = None
    extended: bool
    dlc: int = Field(ge=0, le=8)
    fields: list[FrameField] = Field(default_factory=list)
    manual_ref: str = Field(min_length=1)

    @model_validator(mode="after")
    def _one_id_form(self) -> FrameSpec:
        if (self.can_id is None) == (self.can_id_expr is None):
            raise ValueError(
                f"frame {self.name!r}: set exactly one of can_id or can_id_expr"
            )
        total = sum(f.byte_length for f in self.fields)
        if total > self.dlc:
            raise ValueError(
                f"frame {self.name!r}: fields use {total} bytes but dlc is {self.dlc}"
            )
        names = [f.name for f in self.fields]
        if len(names) != len(set(names)):
            raise ValueError(f"frame {self.name!r}: duplicate field names")
        # The total-bytes check above only catches overlap by accident
        # (e.g. two fields using more bytes combined than dlc allows);
        # it says nothing about two fields claiming the SAME byte, which
        # fits under dlc just fine (e.g. offsets 0..2 and 1..3 total 4
        # bytes, well under an 8-byte dlc, yet byte 1 belongs to both).
        occupied: dict[int, str] = {}
        for f in self.fields:
            for byte in range(f.byte_offset, f.byte_offset + f.byte_length):
                if byte in occupied:
                    raise ValueError(
                        f"frame {self.name!r}: fields {occupied[byte]!r} and {f.name!r} "
                        f"both claim byte {byte}"
                    )
                occupied[byte] = f.name
        return self


class ParamSpec(BaseModel):
    """One entry in the family's read/write parameter table (flash config)."""

    model_config = {"extra": "forbid"}

    name: str
    index: int | str  # some families key params by int address, some by name/string code
    unit: str
    read_only: bool = False
    manual_ref: str = Field(min_length=1)


class ProtocolProfile(BaseModel):
    """Root of profiles/<family>/<model>_<fw>.yaml."""

    model_config = {"extra": "forbid"}

    family: str
    model: str
    firmware: str
    manual_title: str = Field(min_length=1)
    manual_version: str | None = None
    frames: list[FrameSpec] = Field(default_factory=list)
    params: list[ParamSpec] = Field(default_factory=list)
    open_questions: list[str] = Field(default_factory=list)

    @model_validator(mode="after")
    def _frame_names_unique(self) -> ProtocolProfile:
        names = [f.name for f in self.frames]
        if len(names) != len(set(names)):
            raise ValueError("duplicate frame names in profile")
        return self

    def coverage_summary(self) -> dict:
        """No judgement on correctness -- just counts, for `harness profile-lint`."""
        return {
            "family": self.family,
            "model": self.model,
            "firmware": self.firmware,
            "frames_documented": len(self.frames),
            "params_documented": len(self.params),
            "open_questions": len(self.open_questions),
        }
