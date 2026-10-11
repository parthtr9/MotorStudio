"""Tests use a fake family ("testfam") with made-up IDs/offsets -- never
RobStride or any real family, since no real protocol data has been sourced
from a manual yet. See profiles/robstride/OPEN_QUESTIONS.md.
"""
import pytest
from harness.profile_schema import FrameField, FrameSpec, ProtocolProfile
from pydantic import ValidationError


def _minimal_profile_dict() -> dict:
    return {
        "family": "testfam",
        "model": "X1",
        "firmware": "0.0",
        "manual_title": "Testfam Fake Manual for unit tests",
        "frames": [
            {
                "name": "status_feedback",
                "direction": "rx",
                "can_id": 0x100,
                "extended": False,
                "dlc": 4,
                "fields": [
                    {
                        "name": "position",
                        "byte_offset": 0,
                        "byte_length": 2,
                        "byte_order": "big",
                        "scale": 0.01,
                        "unit": "rad",
                        "manual_ref": "Fake Manual sec 1.1",
                    },
                    {
                        "name": "velocity",
                        "byte_offset": 2,
                        "byte_length": 2,
                        "byte_order": "big",
                        "scale": 0.1,
                        "unit": "rad/s",
                        "manual_ref": "Fake Manual sec 1.1",
                    },
                ],
                "manual_ref": "Fake Manual sec 1.1",
            }
        ],
        "open_questions": ["Does firmware 0.0 differ from 0.1 for this frame? Unconfirmed."],
    }


def test_minimal_profile_parses():
    p = ProtocolProfile.model_validate(_minimal_profile_dict())
    assert p.coverage_summary()["frames_documented"] == 1
    assert p.coverage_summary()["open_questions"] == 1


def test_frame_field_without_manual_ref_rejected():
    with pytest.raises(ValidationError):
        FrameField(
            name="position",
            byte_offset=0,
            byte_length=2,
            byte_order="big",
            unit="rad",
            manual_ref="",
        )


def test_frame_field_without_byte_order_rejected():
    with pytest.raises(ValidationError):
        FrameField(
            name="position",
            byte_offset=0,
            byte_length=2,
            unit="rad",
            manual_ref="Fake Manual sec 1.1",
        )


def test_field_overflowing_frame_rejected():
    with pytest.raises(ValidationError):
        FrameField(
            name="position",
            byte_offset=6,
            byte_length=4,
            byte_order="big",
            unit="rad",
            manual_ref="Fake Manual sec 1.1",
        )


def test_fields_exceeding_dlc_rejected():
    d = _minimal_profile_dict()
    d["frames"][0]["dlc"] = 2  # fields use 4 bytes total
    with pytest.raises(ValidationError):
        ProtocolProfile.model_validate(d)


def test_both_can_id_forms_rejected():
    with pytest.raises(ValidationError):
        FrameSpec(
            name="cmd",
            direction="tx",
            can_id=0x100,
            can_id_expr="0x100 + motor_id",
            extended=False,
            dlc=0,
            manual_ref="Fake Manual sec 1.1",
        )


def test_neither_can_id_form_rejected():
    with pytest.raises(ValidationError):
        FrameSpec(
            name="cmd",
            direction="tx",
            extended=False,
            dlc=0,
            manual_ref="Fake Manual sec 1.1",
        )


def test_duplicate_frame_names_rejected():
    d = _minimal_profile_dict()
    d["frames"].append({**d["frames"][0]})
    with pytest.raises(ValidationError):
        ProtocolProfile.model_validate(d)


def test_overlapping_fields_rejected_even_under_dlc_budget():
    # offsets 0..2 and 1..3 total 4 bytes, well under dlc=8 -- the
    # byte-budget check alone would miss that byte 1 belongs to both.
    with pytest.raises(ValidationError):
        FrameSpec(
            name="cmd",
            direction="tx",
            can_id=0x100,
            extended=False,
            dlc=8,
            fields=[
                FrameField(
                    name="a",
                    byte_offset=0,
                    byte_length=2,
                    byte_order="big",
                    unit="raw",
                    manual_ref="Fake Manual sec 1.1",
                ),
                FrameField(
                    name="b",
                    byte_offset=1,
                    byte_length=2,
                    byte_order="big",
                    unit="raw",
                    manual_ref="Fake Manual sec 1.1",
                ),
            ],
            manual_ref="Fake Manual sec 1.1",
        )


def test_adjacent_non_overlapping_fields_accepted():
    fs = FrameSpec(
        name="cmd",
        direction="tx",
        can_id=0x100,
        extended=False,
        dlc=4,
        fields=[
            FrameField(
                name="a",
                byte_offset=0,
                byte_length=2,
                byte_order="big",
                unit="raw",
                manual_ref="Fake Manual sec 1.1",
            ),
            FrameField(
                name="b",
                byte_offset=2,
                byte_length=2,
                byte_order="big",
                unit="raw",
                manual_ref="Fake Manual sec 1.1",
            ),
        ],
        manual_ref="Fake Manual sec 1.1",
    )
    assert len(fs.fields) == 2


def test_duplicate_field_names_rejected():
    d = _minimal_profile_dict()
    d["frames"][0]["fields"].append(dict(d["frames"][0]["fields"][0]))
    with pytest.raises(ValidationError):
        ProtocolProfile.model_validate(d)
