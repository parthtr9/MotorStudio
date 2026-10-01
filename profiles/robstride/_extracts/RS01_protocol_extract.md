# RS01 protocol extract

Source: RS01User Manual260713.pdf

**Page-citation note:** citations below ("p.N") refer to the PDF's physical page order as counted by the extraction tool (page 1 = cover). The manual's own printed footer number is one less than this on every page that has a footer (e.g. text on PDF physical page 13 carries the printed footer "12"). This offset was verified by tracing footer numbers against the physical page loop. If you re-open the PDF in a viewer that shows its own page counter, subtract 1 from "p.N" below to find the footer number, or add 1 to the footer number to find "p.N" here. PDF has 79 pages total (`mdls kMDItemNumberOfPages`).

---

## Model/firmware identification

- Product name, cover page: "ROBSTRIDE 01", "17N.M Quasi-Direct Drive Integrated Motor Module", "User Manual", publisher "RobStride Dynamics", https://www.robstride.com/ (p.1)
- "RS01 instruction manual" (p.2)
- Manual published by "Beijing Lingfoot Times Technology Co., LTD." (referred to as "Lingfoot") (p.2)
- Firmware/boot version strings, read from the "function code" parameter table (see Parameter table section below), Read-only, with "Current value (for reference)" as printed:
  - `0X1000` BootCodeVersion, type String: "0.1.5" (p.22)
  - `0X1001` BootBuildDate, type String: "Mar 16 2022" (p.22)
  - `0X1002` BootBuildTime, type String: "20:22:09" (p.22)
  - `0X1003` AppCodeVersion, type String: "0.0.0.1" — remark "Motor program version number" (p.22)
  - `0X1004` AppGitVersion, type String: "7b844b0fM" (p.22)
  - `0X1005` AppBuildDate, type String: "Apr 14 2022" (p.22)
  - `0X1006` AppBuildTime, type String: "20:30:22" (p.22)
  - `0X1007` AppCodeName, type String: "Lingzu_motor" (p.22)
- MIT-protocol commands 12-15 (save motor data, active report toggle, read/write parameters) each note: "requires firmware update to 0.1.3.14" (p.76, p.77)
- Version History table (final page of manual): Version Number "1.0", Description "Initial Release", Date "November 25, 2025" (p.79)
- Default CAN ID / ID range: no explicit default CAN ID value or valid-ID-range statement was found in prose. The parameter table lists `0X200a CAN_ID`, type uint8, attribute "Settings", Maximum value column "127", Minimum value column "0", Current value (for reference) column "1" (p.23) — see OPEN QUESTIONS about whether "Maximum/Minimum" here is a spec limit or just the field width, and whether "1" is a factory default or merely a snapshot.
- Host/master CAN ID: `0X200b CAN_MASTER`, type uint8, attribute "Settings", Maximum "127", Minimum "0", Current value "0", remark "can host id" (p.23). Elsewhere, worked examples use host id `0xFD` (p.30, p.50) — not stated as a universal default, just the example's value.

---

## CAN bus parameters

- Driver product specification table (p.13):
  - "The rated working voltage": "36VDC"
  - "The maximum allowable voltage": "50VDC"
  - "Rated working phase current": "7Apk"
  - "Maximum allowable phase current": "23Apk"
  - "Standby power": "≤18mA"
  - "CAN bus bit rate": "1Mbps"
  - "Dimensions": "Φ58mm"
  - "Working environment temperature": "-20℃ to 50℃"
  - "The maximum allowable temperature of the control board": "103℃"
  - "encoder resolution": "14bit（absolute turn）"
- Pin description (p.14): Pin1 power +, Pin2 power -, Pin3 "CAN CAN_L", Pin4 "CAN the high side of the communication CAN_H"
- "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted" (p.41, private protocol section)
- "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol." (p.71, MIT protocol section)
- CAN-to-USB debug tool (p.15): "The frame header of the corresponding serial port protocol is 41 54, and the frame tail is 0D 0A." DIP switch 1 ON = module enters Boot mode (cannot connect to host). DIP switch 2 ON = "a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer."
- Baud-rate modification frame (Comm type 23, p.46-47): F_CMD byte values — "01 is 1M", "02 is 500K", "03 is 250K", "04 is 125K". "(re-power-on effect)".
- Protocol-switch frame (p.70, p.74-75): switches among 0=Private protocol (default), 1=CANopen protocol, 2=MIT protocol; "Takes effect after power cycle"/"re-power-on effect".

---

## Frame definitions (TX - host to motor)

### A. Private protocol — extended (29-bit) frame, section 4.1 (p.41-50)

General layout (p.41), exactly as printed:

| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | Communication type | data area 2 | Destination address | data area 1 |

Supported control modes listed on p.41: "Operation control mode: set 5 parameters of motor operation control"; "Current mode: the specified Iq current of the given motor"; "Velocity mode: the specified running speed of the given motor"; "Position mode: Given the specified position of the motor, the motor will run to the specified position".

**Comm type 0x0 — Get device ID** (p.41):
- Bit28~24 = `0x0`; bit23~8 = "bit15~8: identifies host CAN_ID" (text only specifies bits 15-8 of this 16-bit field); bit7~0 = "target motor CAN_ID"; Byte0~7 = `0`.

**Comm type 0x1 — Operation control mode motor control instruction** (p.42):
- Bit28~24 = `0x1`; bit23~8 = "Byte2: Torque (0~65535) corresponds to (-17Nm~17Nm)" (as printed — this text sits in the ID-field column of the table but describes a data byte, see OPEN QUESTIONS); bit7~0 = target motor CAN_ID.
- Byte0~7: "Byte0~1: target Angle [0~65535] corresponds to (-4π~4π)"; "Byte2~3: Target angular velocity [0~65535] corresponds to (-44rad/s~44rad/s)"; "Byte4~5: Kp [0~65535] corresponds to (0.0~500.0)"; "Byte6~7: Kd [0 to 65535] corresponds to the above data (0.0 to 5.0)"; "After the conversion, the high byte is in front and the low byte is in [cut off on page]".
- Response: "Response motor feedback frame (see communication type 2)".
- Confirmed by C code sample (p.58-59): `float_to_uint` maps float range to 16-bit unsigned via `(x-x_min)*((1<<bits)-1)/(x_max-x_min)`, with constants (p.56): `P_MIN -12.57f`, `P_MAX 12.57f`, `V_MIN -44.0f`, `V_MAX 44.0f`, `KP_MIN 0.0f`, `KP_MAX 500.0f`, `KD_MIN 0.0f`, `KD_MAX 5.0f`, `T_MIN -17.0f`, `T_MAX 17.0f`. Byte order in code: data[0]=high byte of position, data[1]=low byte, etc. (p.58-59).

**Comm type 0x3 — Motor enabled to run** (p.43):
- Bit28~24=`0x3`; bit23~8 = "bit15~8: identifies the main CAN_ID"; bit7~0 = "and target motor CAN_ID" (printed exactly this way — ambiguous, see OPEN QUESTIONS).
- Response: "Response motor feedback frame (see communication type 2)".

**Comm type 0x4 — Motor stops running** (p.43):
- Bit28~24=`0x4`; bit23~8="bit15~8: used to identify the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte field: "When the motor is running normally, 0 must be cleared in the data field. Byte[0]=1: The fault is cleared."
- Response: "Response motor feedback frame (see communication type 2)".

**"Version number read frame" (also labeled comm type 0x4)** (p.43-44):
- Bit28~24=`0x4`; bit23~8="bit15~8: used to identify the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte field: "Byte[0]=0x00 Byte[1]=0xC4".
- (Same nominal comm type `0x4` as "motor stops running" above, distinguished only by the Byte0/Byte1 payload — flagged in OPEN QUESTIONS.)

**Comm type 0x6 — Set motor mechanical zero** (p.44):
- Bit28~24=`0x6`; bit23~8="bit15~8: Identifies the main CAN_ID"; bit7~0="and target motor CAN_ID".
- Byte field: "Byte[0]=1".
- Response: "Response motor feedback frame (see communication type 2)".

**Comm type 0x7 — Set motor CAN_ID** (p.44): "Change the current motor CAN_ID, effective immediately."
- Bit28~24=`0x7`; bit23~8="bit15~8: used to identify main CAN_ID Bit16~23: preset CAN_ID"; bit7~0="Target motor CAN_ID".
- Response: "Answer motor broadcast frame (see communication type 0)".

**Comm type 0x11 (17) — Single parameter read** (p.44-45):
- Bit28~24=`0x11`; bit23~8="bit15~8: Used to identify the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte0~7: "Byte0~1: index. For details, see the readability parameter table below. Byte2~3:00 Byte4~7: In data above 00, the low byte is first and the high byte is second".

**Comm type 0x12 (18) — Single parameter write (lost in power failure)** (p.45): "With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved"
- Bit28~24=`0x12`; bit23~8="bit15~8: Used to identify the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte0~7: "Byte0~1: index. For details, see the readability parameter table below. Byte2~3: 00. Byte4~7: Parameter data. In the preceding data, the low byte is in the front and the high byte is in the rear".
- Response: "Response motor feedback frame (see communication type 2)".

**Comm type 0x16 (22) — Motor data save frame** (p.46):
- Bit28~24=`0x16`; bit23~8="bit15~8: identifies the main CAN_ID"; bit7~0="and target motor CAN_ID".
- Byte0~7: "01 02 03 04 05 06 07 08" (printed literally as this fixed byte sequence).
- Response: "Response motor feedback frame (see communication type 2)".

**Comm type 0x17 (23) — Motor baud rate modification frame (re-power-on effect)** (p.46-47):
- Bit28~24=`0x17`; bit23~8="bit15~8: used to identify the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte0~7: "01 02 03 04 05 06 F_CMD"; F_CMD: 01=1M, 02=500K, 03=250K, 04=125K.
- Response: "Response motor feedback frame (see communication type 0)".

**Comm type 0x18 (24) — Motor actively reports frames (control)** (p.47):
- Bit28~24=`0x18`; bit23~8="bit15~8: identifies the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte0~7: "01 02 03 04 05 06 F_CMD"; F_CMD: "00 is to disable active reporting (default)", "01 To enable active reporting, the default reporting interval is 10ms".
- (Response reuses Type 2 layout — see RX section.)

**Comm type 0x19 (25) — Motor protocol modification frame (re-power-on effect)** (p.48):
- Bit28~24=`0x19`; bit23~8="bit15~8: used to identify the main CAN_ID"; bit7~0="target motor CAN_ID".
- Byte0~7: "01 02 03 04 05 06 F_CMD"; F_CMD: "0 is a private protocol (default)", "1 is the Canopen protocol", "2 is the MIT protocol".
- Response: "Response motor feedback frame (see communication type 0)".

**Protocol-switch extended frame, section 5.10** (p.70) — duplicate/alternate description of the comm-type-25 function, printed with a different bit layout:
- "Data Field 29-bit ID: Bit 28~0" (i.e. entire 29 bits, not split) = `0xFFF`; "8-Byte Data Area Byte 0~6" = "01 02 03 04 05 06 F_CMD".
- F_CMD: 0=Private protocol (default), 1=CANopen protocol, 2=MIT protocol.
- Response frame (same page): "Data Field 11-bit ID: Bit 10~0 = Motor ID; Byte 0~7 = 64-bit MCU unique identifier".

### B. MIT protocol — standard (11-bit) frame, section 6 (p.71-78)

General layout (p.71):

| Data Field | 11-bit ID | | 8-byte Data Area |
|---|---|---|---|
| Size | Bit 10~8 | Bit 7~0 | Byte 0~7 |
| Description | Mode type | ID | — |

Supported control modes (p.71): "MIT Mode: Provides five motion control parameters to the motor"; "Velocity Mode: Specifies the target speed for the motor"; "Position Mode: Specifies the target position and speed...".

**Command 1 — Enable Motor Operation** (p.72): ID field `07F` = target motor CAN ID; Byte0~7 = `FF FF FF FF FF FF FF FC` (printed as `FF FF FF FF FF FF FF FC` in the Size row and `FF FF FF FF FF FF FF FC` again in Description row — identical). Response: Response Command 1.

**Command 2 — Stop Motor Operation** (p.72): ID `07F`; Byte0~7 = `FF FF FF FF FF FF FF FD`. Response: Response Command 1.

**Command 3 — MIT Dynamic Parameters** (p.72-73): ID `07F` = target motor CAN ID.
- Byte0~1: "Target angle [0~65535], corresponding to (-12.57 rad ~ 12.57 rad)"
- Byte2 (high 8 bits) + Byte3[7-4] (low 4 bits): "Target speed [0~4096], corresponding to (-44 rad/s ~ 44 rad/s)"
- Byte3[3-0] (high 4 bits) + Byte4 (low 8 bits): "Kp [0~4096], corresponding to (0 ~ 500)"
- Byte5 (high 8 bits) + Byte6[7-4] (low 4 bits): "Kd [0~4096], corresponding to (0 ~ 5)"
- Byte6[3-0] (high 4 bits) + Byte7 (low 8 bits): "Target torque [0~4096], corresponding to (-17 N·m ~ 17 N·m)"
- Response: Response Command 1.
- NOTE: this packs 5 values into 8 bytes using mixed 12-bit/nibble boundaries that don't align cleanly to byte boundaries as transcribed from the text — see OPEN QUESTIONS.

**Command 4 — Set Zero Position (Non-Position Mode)** (p.73): ID `07F`; Byte0~7 = `FF FF FF FF FF FF FF FE`. Response: Response Command 1.

**Command 5 — Clear Errors & Read Fault Status** (p.73): ID `07F`; Byte0~7 printed as `FF FF FF FF FF FF FF FB` (Size row) / `FF FF FF FF FF FF F_CMD FB` (Description row). "F_CMD: 0xFF → clearing the current abnormality; Any other value → the error code will be returned in the BYTE1 field of the response message." Response (fault clear): Response Command 1.

**Command 6 — Set Operation Mode** (p.74): ID `07F`; Byte0~7 = `FF FF FF FF FF FF 00 FC` (Size row) / `FF FF FF FF FF FF F_CMD FC` (Description row). F_CMD: "0: MIT mode (default)", "1: Position mode", "2: Velocity mode". Response: Response Command 1.

**Command 7 — Modify Motor CAN ID** (p.74): ID `07F`; Byte0~7 = `FF FF FF FF FF FF 01 FA` / `FF FF FF FF FF FF F_CMD FA`. "F_CMD: Target motor CAN_ID". Response: Response Command 2.

**Command 8 — Change Communication Protocol (takes effect after power cycle)** (p.74-75): ID `07F`; Byte0~7 = `FF FF FF FF FF FF 00 FD` / `FF FF FF FF FF FF F_CMD FD`. F_CMD: "0: Private protocol", "1: CANopen", "2: MIT protocol". Response: Response Command 2.

**Command 9 — Modify Host CAN ID** (p.75): ID field shown as `07F` (Size row) but Description row labels it "Bit 10~0" with no fixed value given; Byte0~7 = `FF FF FF FF FF FF FD 01` / `FF FF FF FF FF FF F_CMD 01`. "F_CMD: Host CAN ID". Response: Response Command 2.

**Command 10 — Position Mode Control Command** (p.75): ID: "bit10~8 = 1" (mode type), "bit7~0 = target motor CAN ID (example `7F`)". Byte0~7: "Byte 0~3: Target position (rad, 32-bit float); Byte 4~7: Target speed (rad/s, 32-bit float)". Worked example bytes: `00 00 A0 40 00 00 A0 40` for "Position: 5 rad, Velocity: 5 rad/s". Response: Response Command 1.

**Command 11 — Velocity Mode Control Command** (p.75-76): ID: "bit10~8 = 2", "bit7~0 = target motor CAN ID (example `7F`)". Byte0~7: "Byte 0~3: Target speed (rad/s, 32-bit float); Byte 4~7: Current limit in speed/position mode (A, 32-bit float)". Worked example bytes: `00 00 A0 40 00 00 00 40` for "Velocity: 5rad/s, Current: 2A". Response: Response Command 1.

**Command 12 — Save motor data (requires firmware ≥0.1.3.14)** (p.76): ID `07F`; Byte0~7 = `FF FF FF FF FF FF FF F8`. Response frame: Response Command 2.

**Command 13 — Motor actively reports (requires firmware ≥0.1.3.14)** (p.76): ID `07F`; Byte0~7 = `FF FF FF FF FF FF 00 F9` / `FF FF FF FF FF FF F_CMD F9`. F_CMD: "0 indicates no reporting (default)", "1 is for reporting". Response frame: Response Command 1.

**Command 14 — Read motor parameters (requires firmware ≥0.1.3.14)** (p.76-77): ID: "bit10~8 = 3", "bit7~0 = target motor CAN ID (example `7F`)". Byte0~7 example given: `05 70 00 00 00 00 00 00`. "Byte0~1: index (See the read/write parameter table below for details.) Byte2~3: 00 Byte4~7: 00. The above data is in low byte first, high byte last." Response frame printed identically to the request (same ID and example bytes `05 70 00 00 00 00 00 00`) with the note the parameter data return is "least significant byte first and the most significant byte last."

**Command 15 — Write motor parameters (requires firmware ≥0.1.3.14)** (p.77): ID: "bit10~8 = 4", "bit7~0 = target motor CAN ID (example `7F`)". Byte0~7 example: `29 70 00 00 00 00 00 00`. Same byte-order note as Command 14. Response frame printed identically to the request.

---

## Frame definitions (RX - motor to host)

**Comm type 0x0 reply — Get device ID** (p.41):
- Bit28~24=`0x0`; bit23~8="target motor CAN_ID"; bit7~0=`0XFE`; Byte0~7 = "64-bit MCU unique identifier".

**Comm type 0x2 — Motor feedback data** (p.42-43): this is the core status frame; it is the documented response to comm types 1, 3, 4, 6.
- Bit28~24=`0x2`.
- bit23~8 field, printed as: "Bit8~Bit15: CAN ID of the current motor" + "bit21~16: fault information (0 none 1 has)" + "bit21: uncalibrated" + "bit20: Uncalibrated" + "bit20: Gridlock overload fault" + "bit19: magnetic coding fault" + "bit18: overtemperature" + "bit17: Three-phase overcurrent fault" + "bit16: undervoltage fault" + "bit22~23: Mode status 0: Reset mode [reset] 1: Cali mode [calibration] 2: Motor mode [Run]" (verbatim, including the apparently duplicated "bit20" label — see OPEN QUESTIONS).
- bit7~0 = "host CAN_ID".
- Byte0~1: "The current Angle [0~65535] Corresponding to (-4π~4π)"
- Byte2~3: "Current angular velocity [0~65535] corresponds to (-44rad/s~44rad/s)"
- Byte4~5: "Current torque [0~65535] corresponds to (-17Nm~17Nm)"
- Byte6~7: "Current temperature: Temp(Celsius) *10. If the value is higher than 10, the high byte is first and the low byte is last" (verbatim; conditional wording on byte order — see OPEN QUESTIONS).
- Per section 4.2.2 (p.51): "Type 2 is changed to a periodic cycle -4π-4π... P_MIN is -12.57f, P_MAX is 12.57f" (firmware revision note on the angle range/wrap behavior).

**Comm type 0x2 reply to "Version number read frame"** (p.43-44): reuses comm type `0x2` layout but with fixed payload semantics:
- Byte0=0x00, Byte1=0xC4, Byte2=0x56, Byte3~6="Motor version number. Order from high to low."

**Comm type 0x11 (17) reply — Single parameter read** (p.45):
- Bit28~24=`0x11`; bit23~8="bit15~8: indicates that the master CAN_ID. Bit23~16:00 indicates that the master CAN_ID is successfully read. 01 indicates that the master can_ID [sentence cut off on page]".
- Byte0~1: index; Byte2~3: 00; Byte4~7: "Parameter data. 1 byte of data above Byte4 is preceded by low bytes and followed by high bytes" (verbatim, awkward phrasing).
- Worked example on p.50 (reading `loc_kp`): request `Bit28~bit24=0x11, bit23~8=0x00FD (host id 0xFD), bit7~0=0x7F (target motor CAN_ID), data=1E 70 00 00 00 00 00 00`; reply `Bit28~bit24=0x11, bit23~8=0x007F (target motor CAN_ID), bit7~0=0xFD (host id), data=1E 70 00 00 00 00 F0 41` — Byte4~7 `00 00 F0 41` little-endian = IEEE-754 float 30.0 (manual states "loc_kp value 30").

**Comm type 0x15 (21) — Fault feedback frame** (p.45-46):
- Bit28~24=`0x15`; bit23~8="bit15~8: identifies motor CAN_ID"; bit7~0="the main CAN_ID" (printed order in source; see OPEN QUESTIONS about which side is host vs motor here).
- Byte0~3: "fault value (non-0: faulty; 0: Normal)" with 32-bit bitfield (see Fault/error codes section).
- Byte4~7: "warning Value", bit0="motor overtemperature warning, the default is 93°C".

**Comm type 0x18 (24) reply — Motor actively reports frames** (p.47): identical field layout and byte semantics to comm type 0x2 (current angle / angular velocity / torque / temperature), printed again in full under the type-24 heading.

### MIT protocol responses (p.71-74)

**Response Command 1 — Data Feedback (Motor Status)** (p.71-72): ID `0FD` = Host ID. Byte0 = Motor CAN ID.
- "Byte1~2: Current angle [0~65535], corresponding to (-12.57rad ~ 12.57rad)" — Byte3 "High 8 bits"
- "Byte4[7-4] (High 4 bits): Low 4 bits; Current speed [0~4096], corresponding to (-44rad/s ~ 44rad/s)" — "Byte4[3-0] (Low 4 bits): High 4 bits" (verbatim; nibble-order wording is confusing — see OPEN QUESTIONS)
- "Byte5: Low 8 bits; Current torque [0~4096], corresponding to (-17N·m ~ 17N·m)"
- "Byte6[7-6]: Operation Mode 0: Reset Mode, 1: Cali Mode (Calibration), 2: Motor Mode (Running)"
- "Byte6[5]: 1 = Motor fault exists, 0 = No motor fault"
- "Byte6[4]: 1 = Motor warning with no fault, 0 = No motor warning"
- "Byte6[3-0] (Low 4 bits): High 4 bits; Byte7: Low 8 bits; Winding temperature: Temp(°C) × 10, Unit: ℃"

**Response Command 2 — MCU Identification** (p.72): ID = Motor ID; Byte0~7 = 64-bit MCU unique identifier.

**Fault Status Response (to Command 5)** (p.73-74): ID `0FD` = Host ID; Byte0 = Motor CAN ID; Byte1~4 = "Fault value (Non-zero: fault occurred; 0: normal)" with a 32-bit bitfield (see Fault/error codes section — identical bit assignments to the private-protocol 0x3022/Type-21 fault register).

---

## Parameter table (flash config read/write)

### Table A — CAN read/write single-parameter list, section 4.1.13 (p.48-50)

Accessed via comm type 0x11 (read) / 0x12 (write). Header row as printed: "index | Description | Type | Number of bytes | R/W Read and write permission" — note the header text does not include an explicit "name" or "range/default" column label even though every data row clearly carries both; column meanings below are inferred from the data layout, not from a clean header (flagged in OPEN QUESTIONS).

| index | name | Description (as printed) | Type | bytes | Range/Default (as printed) | R/W |
|---|---|---|---|---|---|---|
| 0X7005 | run_mode | "0: operation mode / 1: position mode (PP) / 2: Velocity mode / 3: Operation mode Current mode / 5: Position mode (CSP)" (verbatim, enumeration text ambiguous — see OPEN QUESTIONS) | uint8 | 1 | — | W/R |
| 0X7006 | iq_ref | Current mode Iq command | float | 4 | "-23 to 23A" | W/R |
| 0X700A | spd_ref | Rotational Velocity mode Rotational speed command | float | 4 | "-44to 44rad/s" (printed exactly, no space before "to") | W/R |
| 0X700B | limit_torque | torque limit | float | 4 | "0 to 17Nm" | W/R |
| 0X7010 | cur_kp | Kp | float | 4 | "The default value is 0.17" | W/R |
| 0X7011 | cur_ki | Ki | float | 4 | "The default value is 0.012" | W/R |
| 0X7014 | cur_filt_gain | filt_gain | float | 4 | "0 to 1.0, The default value is 0.1" | W/R |
| 0X7016 | loc_ref | Position Mode Angle instruction | float | 4 | "rad" | W/R |
| 0X7017 | limit_spd | Location mode (CSP) speed limit | float | 4 | "0 to 44rad/s" | W/R |
| 0X7018 | limit_cur | Velocity position mode Current limitation | float | 4 | "0 to 23A" | W/R |
| 0x7019 | mechPos | Mechanical Angle of the loading coil | float | 4 | "rad" | R |
| 0x701A | iqf | iq Filter | float | 4 | "-16 to 16A" | R |
| 0x701B | mechVel | Speed of the load | float | 4 | "-44 to 44rad/s" | R |
| 0x701C | VBUS | Bus voltage | float | 4 | "V" | R |
| 0x701E | loc_kp | "kp at" (printed garbled — row label reads "loc_kp kp at") | float | 4 | "The default value is 40" | W/R |
| 0x701F | spd_kp | Indicates the speed kp | float | 4 | "The default value is 6" | W/R |
| 0x7020 | spd_ki | ki | float | 4 | "The default value is 0.02" | W/R |
| 0x7021 | spd_filt_gain | Speed filter value | float | 4 | "The default value is 0.1" | W/R |
| 0x7022 | acc_rad | velocity mode acceleration | float | 4 | "The default value is 20rad/s^2" | W/R |
| 0x7024 | vel_max | Location mode (PP) speed | float | 4 | "The default value is 10rad/s" | W/R |
| 0x7025 | acc_set | Location mode (PP) acceleration | float | 4 | "The default value is 10rad/s^2" | W/R |
| 0x7026 | EPScan_time | "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | "The default value is 1" | W/R |
| 0x7028 | canTimeout | "can The timeout threshold, 20000 is 1s" | uint32 | 4 | "The default value is 0" | W/R |
| 0x702C | alveolous_open | Cogging Compensation Switch | uint8 | 1 | "The default is 0" | W/R |
| 0x702D | iq_test | Motor Initialization Calibration Switch | uint8 | 1 | "The default is 0" | W/R |
| 0x702E | dcc_set | "Deceleration in Motor PP Mode (Normally Disabled)" | float | 4 | "The default is 10rad/s^2" | W/R |

Worked read example, index `loc_kp` (p.50): request ID fields `0x11 / 0x00FD / 0x7F`, data `1E 70 00 00 00 00 00 00`; reply ID fields `0x11 / 0x007F / 0xFD`, data `1E 70 00 00 00 00 F0 41` → value 30 (float, IEEE-754, byte order "high right byte" per the text).

### Table B — Device info / system-config / diagnostic parameter table (p.22-28)

This is a separate, much larger table from the host-computer "Refresh Parameter Table" feature (p.21); the manual text does not state explicitly whether these indices are reachable over the CAN wire protocol (comm types 17/18) or only via the vendor's USB-serial upper-computer tool — flagged in OPEN QUESTIONS. Header row as printed: "function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | 备注" (last header literally printed in Chinese, meaning "remarks/notes" — not translated in this section of the manual).

| function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | remark (备注, as printed) |
|---|---|---|---|---|---|---|---|
| 0X0000 | Name | String | Read/Write | — | — | "ÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿ" (garbled/placeholder bytes as extracted) | — |
| 0X0001 | BarCode | String | Read/Write | — | — | "ÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿ" | — |
| 0X1000 | BootCodeVersion | String | Read only | — | — | 0.1.5 | — |
| 0X1001 | BootBuildDate | String | Read only | — | — | Mar 16 2022 | — |
| 0X1002 | BootBuildTime | String | Read only | — | — | 20:22:09 | — |
| 0X1003 | AppCodeVersion | String | Read only | — | — | 0.0.0.1 | Motor program version number |
| 0X1004 | AppGitVersion | String | Read only | — | — | 7b844b0fM | — |
| 0X1005 | AppBuildDate | String | Read only | — | — | Apr 14 2022 | — |
| 0X1006 | AppBuildTime | String | Read only | — | — | 20:30:22 | — |
| 0X1007 | AppCodeName | String | Read only | — | — | Lingzu_motor | — |
| 0X2000 | echoPara1 | uint16 | disposition | 74 | 5 | 5 | — |
| 0X2001 | echoPara2 | uint16 | disposition | 74 | 5 | 5 | — |
| 0X2002 | echoPara3 | uint16 | disposition | 74 | 5 | 5 | — |
| 0X2003 | echoPara4 | uint16 | disposition | 74 | 5 | 5 | — |
| 0X2004 | echoFreHz | uint32 | Read/Write | 10000 | 1 | 500 | — |
| 0X2005 | MechOffset | float | Settings | 7 | -7 | 4.619583 | Motor magnetic encoder Angle offset |
| 0X2006 | MechPos_init | float | Read/Write | 50 | -50 | 4.52 | Reserved parameter |
| 0X2007 | limit_torque | float | Read/Write | 17 | 0 | 17 | Torque limitation |
| 0X2008 | I_FW_MAX | float | Read/Write | 33 | 0 | 0 | Weak magnetic current value, default 0 |
| 0X2009 | motor_baud | uint8 | Settings | 20 | 0 | 1 | Baud rate flag bit |
| 0X200a | CAN_ID | uint8 | Settings | 127 | 0 | 1 | id of this object |
| 0X200b | CAN_MASTER | uint8 | Settings | 127 | 0 | 0 | can host id |
| 0X200c | CAN_TIMEOUT | uint32 | Read/Write | 100000 | 0 | 0 | can timeout threshold. The default value is 0 |
| 0X200d | status2 | int16 | Read/Write | 1500 | 0 | 800 | Reserved parameter |
| 0X200e | status3 | uint32 | Read/Write | 1000000 | 1000 | 20000 | Reserved parameter |
| 0X200f | status1 | float | Read/Write | 64 | 1 | 7.75 | Reserved parameter |
| 0X2010 | Status6 | uint8 | Read/Write | 1 | 0 | 1 | Reserved parameter |
| 0X2011 | cur_filt_gain | float | Read/Write | 1 | 0 | 0.9 | Current filtering parameter |
| 0X2012 | cur_kp | float | Read/Write | 200 | 0 | 0.025 | Current kp |
| 0X2013 | cur_ki | float | Read/Write | 200 | 0 | 0.0258 | Current ki |
| 0X2014 | spd_kp | float | Read/Write | 200 | 0 | 2 | Velocity kp |
| 0X2015 | spd_ki | float | Read/Write | 200 | 0 | 0.021 | Speed ki |
| 0X2016 | loc_kp | float | Read/Write | 200 | 0 | 30 | Position kp |
| 0X2017 | spd_filt_gain | float | Read/Write | 1 | 0 | 0.1 | Velocity filter parameter |
| 0X2018 | limit_spd | float | Read/Write | 200 | 0 | 2 | Location mode speed limit |
| 0X2019 | limit_cur | float | Read/Write | 23 | 0 | 23 | Position, Velocity mode current limit |
| 0x2020 | protocol_1 | uint8 | Read/Write | 100 | 0 | 0 | Protocol flag |
| 0X3000 | timeUse0 | uint16 | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X3001 | timeUse1 | uint16 | Read/Write | 27 | 0 | 0 | — |
| 0X3002 | timeUse2 | uint16 | Read/Write | 27 | 0 | 0 | — |
| 0X3003 | timeUse3 | uint16 | Read/Write | 150 | 0 | — | — |
| 0X3004 | encoderRaw | int16 | Read/Write | 20 | 0 | — | Magnetic encoder sampling value |
| 0X3005 | mcuTemp | int16 | Read/Write | 1000 | 0 | — | mcu internal temperature, *10 |
| 0X3006 | motorTemp | int16 | Read/Write | 100 | 0 | 0 | Motor ntc temperature, *10 |
| 0X3007 | vBus(mv) | uint16 | — | — | — | — | Bus voltage |
| 0X3008 | adc1Offset | int32 | Read only | — | — | 5 | adc sampling channel 1 zero current bias |
| 0X3009 | adc2Offset | int32 | Read only | — | — | 0 | adc sampling channel 2 zero current bias |
| 0X300a | adc1Raw | uint16 | Read only | — | — | 10 | adc sampling value 1 |
| 0X300b | adc2Raw | uint16 | Read only | — | — | 0 | adc sampling value 2 |
| 0X300c | VBUS | float | Read only | — | — | 11396 | Bus voltage V |
| 0X300d | cmdId | float | Read only | — | — | 337 | id ring instruction, A |
| 0X300e | cmdIq | float | Read only | — | — | 333 | iq ring command, A |
| 0X300f | cmdlocref | float | Read only | — | — | 24195 | Position loop command, rad |
| 0X3010 | cmdspdref | float | Read only | — | — | 2084 | Speed loop command, rad/s |
| 0X3011 | cmdTorque | float | Read only | — | — | 2084 | Torque instruction, nm |
| 0X3012 | cmdPos | float | Read only | — | — | 1232 | mit Protocol Angle instruction |
| 0X3013 | cmdVel | float | Read only | — | — | 1212 | mit Protocol Speed instruction |
| 0X3014 | rotation | int16 | Read only | — | — | 36 | Number of turns |
| 0X3015 | modPos | float | Read only | — | — | 0 | Motor uncounted coil mechanical Angle, rad |
| 0X3016 | mechPos | float | Read only | — | — | 0 | Load end loop mechanical Angle, rad |
| 0X3017 | mechVel | float | Read only | — | — | 0 | Load speed: rad/s |
| 0X3018 | elecPos | float | Read only | — | — | 0 | Electrical Angle |
| 0X3019 | ia | float | Read only | — | — | 0 | U-wire current, A |
| 0X301a | ib | float | Read only | — | — | 0 | V-wire current, A |
| 0X301b | ic | float | Read only | — | — | 0 | W-wire current, A |
| 0X301c | timeout | uint32 | Read only | — | — | 1 | Timeout counter value |
| 0X301d | phaseOrder | uint8 | Read only | — | — | 4.363409 (note: non-integer current value for a uint8 field, as printed) | Directional marking |
| 0X301e | iqf | float | Read only | — | — | 0.777679 | iq filter value, A |
| 0X301f | boardTemp | int16 | Read only | — | — | 0.036618 (note: non-integer current value for an int16 field, as printed) | Plate temperature, *10 |
| 0X3020 | iq | float | Read only | — | — | 4.714761 | iq Original value, A |
| 0X3021 | id | float | Read only | — | — | 0 | id Original value, A |
| 0X3022 | faultSta | uint32 | Read only | — | — | 0 | Fault status value |
| 0X3023 | warnSta | uint32 | Read only | — | — | 0 | Warning status value |
| 0X3024 | drv_fault | uint16 | Read only | — | — | 31600 | "The driver chip fault value is 1" |
| 0X3025 | drv_temp | int16 | Read only | — | — | 0 | "The driver chip fault value is 2" (remark text as printed — name suggests temperature, remark suggests fault code; see OPEN QUESTIONS) |
| 0X3026 | Uq | float | Read only | — | — | 0 | Q-axis voltage |
| 0X3027 | Ud | float | Read only | — | — | 359 | D-axis voltage |
| 0X3028 | dtc_u | float | Read only | — | — | 0 | The duty cycle of the U-phase output |
| 0X3029 | dtc_v | float | Read only | — | — | 0 | The duty cycle of the V-phase output |
| 0X302a | dtc_w | float | Read only | — | — | 0 | The duty cycle of the W-phase output |
| 0X302b | v_bus | float | Read only | — | — | 0 | Vbus in the closed loop |
| 0X302c | ElecOffset | float | Read only | — | — | 0 | electrical Angle offset |
| 0X302d | torque_fdb | float | Read only | — | — | 48 | Torque feedback value, nm |
| 0X302e | rated_i | float | Read only | — | — | 0 | Rated current of motor |
| 0X302f | limit_i | float | Read only | — | — | 0 | The motor limits the maximum current |
| 0X3030 | mcOverTemp | int16 | Read only | — | — | 0 | Overtemperature threshold |
| 0X3031 | Kt_Nm/Amp | float | Read only | — | — | 0 | Moment coefficient |
| 0X3032 | Tqcali_Type | uint8 | Read only | — | — | 0 | Motor type |
| 0X3033 | fault1 | uint32 | Read only | — | — | 24.195 | Log failure |
| 0X3034 | fault2 | uint32 | Read only | — | — | 0 | Log failure |
| 0X3035 | fault3 | uint32 | Read only | — | — | 8 | Log failure |
| 0X3036 | fault4 | uint32 | Read only | — | — | 27 | Log failure |
| 0X3037 | fault5 | uint32 | Read only | — | — | 0 | Log failure |
| 0X3038 | fault6 | uint32 | Read only | — | — | 0 | Log failure |
| 0X3039 | fault7 | uint32 | Read only | — | — | 0 | Log failure |
| 0X303a | fault8 | uint32 | Read only | — | — | 0 | Log failure |
| 0X303b | theta_mech_1 | float | Read only | — | — | 0 | Type 2 Low speed Angle |

Note: many "Current value (for reference)" entries in this table (e.g. VBUS=11396, cmdId=337, Uq=0/Ud=359, iq=4.714761) look like a live snapshot captured from a running test unit at manual-writing time, not formal factory-default specifications — treat them as illustrative only, not as spec limits. This is an inference flagged for the reader's judgment, not stated explicitly in the text.

### CANopen object dictionary entries referenced in section 5 (p.65-70)

Not CAN-frame byte layouts — these are "index / name / attribute / type / unit" object-dictionary entries used when the motor is switched into CANopen protocol mode:

| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 603F | Error_code | Read-only | UINTEGER16 | / |
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6041 | Statusword | Read-only | UINTEGER16 | / |
| 6060 | Modes_of_operation / Modes of operation | Read-write | INTEGER8 | / |
| 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
| 6062 | Position_demand_value | Read-only | INTEGER32 | "Pulses (1 rev = 16,384 pulses)" |
| 6064 | Position_actual_value | Read-only | INTEGER32 | "Pulses (1 rev = 16,384 pulses)" |
| 6067 | Position_window | Read-write | UINTEGER32 | "Pulses (1 rev = 16,384 pulses)" |
| 6068 | Position_window_time | Read-write | UINTEGER16 | ms |
| 606B | Velocity_demand_value | Read-only | INTEGER32 | "0.1 rpm" |
| 606C | Velocity_actual_value | Read-only | INTEGER32 | "0.1 rpm" |
| 6071 | Target_torque | Read-write | INTEGER16 | "0.1% load ratio (1000 = 6 N·m)" |
| 6077 | Torque_actual_value | Read-only | INTEGER16 | "0.1% load ratio (1000 = 6 N·m)" |
| 6078 | Current_actual_value | Read-only | INTEGER16 | mA |
| 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | mV |
| 607A | Target_position | Read-write | INTEGER32 | "Pulses (1 rev = 16,384 pulses)" |
| 6081 | Profile_velocity | Read-write | UINTEGER32 | "0.1 rpm" |
| 6083 | Profile_acceleration | Read-write | UINTEGER32 | "0.1 rpm/s" |
| 60FF | Target_velocity | Read-write | INTEGER32 | "0.1 rpm" |

CANopen state machine notes (p.65-66, verbatim-paraphrase level): on power-up, motor defaults to `SWITCH_ON_DISABLED`. Controlword=6/7/15 progressively enables `OPERATION_ENABLE`, or 15 directly. Controlword=1 stops normally (returns to disabled). Controlword=11 triggers emergency stop ("Use with Caution — Risk of Voltage Surge"). Watchdog: object `0x6099` index 1 sets the CAN communication watchdog; "a value of 20000 represents 1 second"; default 0 = disabled. Mode changes must happen while `SWITCH_ON_DISABLED`.

---

## Fault/error codes

### 32-bit fault register (consistent bit assignment across three locations: function code `0x3022` "faultSta", private-protocol Comm type 0x15 Byte0~3, and MIT Fault Status Response Byte1~4)

As printed across p.29-31 (private protocol, "3.3.7 Motor fault instructions", function code 0x3022) and p.73-74 (MIT protocol, "6.8 Fault Status Response") — the two listings are consistent with each other:

- Bit 16: Motor current fault: A-phase current sampling overcurrent (p.29/p.73)
- Bit 14: Motor stall overload algorithm protection (p.29/p.74)
- Bit 9: Position initialization fault (p.29/p.74)
- Bit 8: Hardware identification fault (p.29/p.74)
- Bit 7: Encoder uncalibrated: Motor encoder not calibrated (p.29/p.74)
- Bit 5: Motor current fault: C-phase current sampling overcurrent (p.29/p.74)
- Bit 4: Motor current fault: B-phase current sampling overcurrent (p.29/p.74)
- bit 3: Overvoltage fault: "the motor voltage exceeds the protection voltage by 50V" (p.30-31; the MIT listing at p.74 just says "Overvoltage fault" with no threshold repeated)
- bit 2: Undervoltage fault: "the motor voltage is lower than the protection voltage of 12V" (p.31)
- bit 1: Driver chip failure: Motor driver chip failure reported / "Driver chip fault" (p.31/p.74)
- bit 0: Motor overtemperature fault: "motor thermistor temperature exceeds 145 degrees" (p.31) — note: elsewhere the manual gives a different overtemperature threshold of "103°C" for this same fault condition (p.13 "maximum allowable temperature of the control board", p.46 "motor overtemperature fault, Default 103°C") — see OPEN QUESTIONS for this discrepancy.

Bits 6, 10-13, 15, and above 16 (other than 16) are not described in the extracted text.

### Driver chip fault sub-code tables — NOT EXTRACTED

- "Function code 0x3024 is driver chip fault code 1. The specific faults are as follows" (p.31) — followed by what appears to be an embedded image/table that did not extract as text (blank in the text layer). Content unknown.
- "Function code 0x3025 is driver chip fault code 2. The specific faults are as follows" (p.31) — same issue, content unknown.

### Warning register (private-protocol Comm type 0x15, Byte4~7) (p.46)

- Byte4~7: "warning Value"; "bit0: motor overtemperature warning, the default is 93°C". Other warning bits not described in the extracted text.

### Compressed status/fault bits embedded in the CAN arbitration ID of Comm type 0x2 / 0x18(24) feedback frames (p.42-44, p.46-47)

This is a *different*, smaller bitfield from the 32-bit register above — it lives in bits 16-23 of the 29-bit CAN ID of the feedback frame itself, not in the data payload:

- bit21~16: "fault information (0 none 1 has)"
- bit21: uncalibrated
- bit20: "Uncalibrated" — ALSO printed as bit20: "Gridlock overload fault" (duplicate bit number with two different meanings, verbatim from source — see OPEN QUESTIONS)
- bit19: magnetic coding fault
- bit18: overtemperature
- bit17: Three-phase overcurrent fault
- bit16: undervoltage fault
- bit22~23: "Mode status" — 0: Reset mode [reset]; 1: Cali mode [calibration]; 2: Motor mode [Run]

Do not confuse this ID-embedded bitfield's bit numbers (e.g. "bit16" = undervoltage here) with the same-numbered bits in the 32-bit `0x3022`/Type-21 fault register above (e.g. "Bit 16" = A-phase overcurrent there) — they are different fields with independently-assigned bit meanings, both printed using the same bit-number labels.

---

## OPEN QUESTIONS

1. **Comm type 0x1 table layout (p.42):** the printed table places "Byte2: Torque (0~65535) corresponds to (-17Nm~17Nm)" inside the bit23~8 column of the 29-bit ID row, which cannot be literally correct (torque is clearly a data-byte field, confirmed by the Byte0~7 column and by the C code on p.58-59). The text extraction could not resolve whether this is a genuine table-layout quirk in the source PDF or an artifact of text-extraction column-merging. Treat the Byte0~7 description (and the C code) as authoritative; do not use the bit23~8 cell literally.

2. **Comm type 0x3 / 0x4 / 0x6 ID-field wording:** several TX frames describe bit23~8 and bit7~0 with wording like "bit15~8: identifies the main CAN_ID" / "and target motor CAN_ID" that doesn't cleanly say which 8-bit sub-field is the host ID vs. the target motor ID, or confirm the field width (8 vs 16 bits) consistently across types. Compare against the one fully-decoded worked example on p.30 (type 18 write, ID `0x1200FD01`) for the canonical layout: bit28~24=comm type, bit23~8=host id (16 bits, here only low byte 0xFD populated), bit7~0=target motor CAN_ID. I generalized this layout where the text was ambiguous, but did not do so for the precise per-field descriptions above — those are quoted exactly as the (ambiguous) source states them.

3. **"Version number read frame" reuses comm type `0x4`** (p.43-44), the same nominal type as "Motor stops running" (p.43), distinguished only by payload (`Byte[0]=0x00, Byte[1]=0xC4` vs. a stop/fault-clear payload). The manual does not explain how a receiver is meant to distinguish these two semantically different commands sharing one type field — possibly a documentation duplication/error, not verified.

4. **Comm type 0x15 (21) "Fault feedback frame"** (p.45-46): unclear whether this frame is sent host→motor (a query) or motor→host (an unsolicited/polled report). I classified it under RX based on context (it carries fault/warning data that only the motor would know), but the printed bit23~8 / bit7~0 descriptions ("identifies motor CAN_ID" / "the main CAN_ID") are reversed relative to other RX frames where bit23~8 typically carries the *target* (host) ID and bit7~0 the motor ID. Not resolved from text alone.

5. **MIT Command 3 "MIT Dynamic Parameters" bit packing** (p.72-73): the printed byte/nibble boundaries for speed, Kp, and Kd ("Byte3[7-4]... Low 4 bits of target speed", "Byte4: Low 8 bits", etc.) describe a 12-bit-per-value packing scheme across byte and nibble boundaries that is difficult to resolve unambiguously from prose text alone — I transcribed the wording verbatim but could not independently confirm the exact bit-to-field mapping without seeing the original table/figure rendering (likely a diagram in the source PDF that didn't extract as text).

6. **MIT Response Command 1, current-speed nibble wording** (p.71-72): "Byte4[7-4] (High 4 bits): Low 4 bits... Byte4[3-0] (Low 4 bits): High 4 bits" is self-contradictory as extracted (says the high nibble holds the low bits and vice versa). Possibly a translation/OCR artifact of a table that had the nibble assignment in an adjacent cell. Flagging rather than guessing the correct mapping.

7. **Comm type 0x2 feedback — duplicate "bit20" bit assignment** (p.42, p.44, p.47): printed as both "bit20: Uncalibrated" and "bit20: Gridlock overload fault" in the same field description. Could not determine which is correct, or whether these are two different sub-bits that got merged in extraction.

8. **run_mode (0X7005) enum text** (p.48): printed as "0: operation mode / 1: position mode (PP) / 2: Velocity mode / 3: Operation mode Current mode / 5: Position mode (CSP)" — entries for value 3 and the absence of an explicit value-4 entry are garbled/unclear (no "4:" line appears at all, and "3: Operation mode Current mode" reads like two merged labels). Could not confidently determine the exact mode number for "Current mode" vs. the restated "Operation mode" label.

9. **Overtemperature fault threshold inconsistency:** the driver spec table states "The maximum allowable temperature of the control board: 103℃" (p.13) and the Comm-type-0x15 warning-register note states "motor overtemperature fault, Default 103°C" (p.46), but the 0x3022/MIT fault-bit description for bit0 states "motor thermistor temperature exceeds 145 degrees" (p.31). These may refer to different sensors (control-board thermistor vs. motor winding thermistor) rather than being contradictory, but the manual does not explicitly reconcile the two numbers in the text extracted.

10. **Relationship between Parameter Table A (0x7005-0x702E, CAN-protocol accessible) and Parameter Table B (0x0000/0x1000s/0x2000s/0x3000s, host-computer tool)**: the manual does not state in the extracted text whether Table B's indices are reachable over the CAN wire protocol (comm types 0x11/0x12) at all, or are exclusively surfaced through the separate USB-serial "upper computer" debugger tool described in section 3. Several named parameters overlap conceptually (e.g. `cur_kp`/`cur_ki`/`spd_kp`/`loc_kp`/`limit_torque`/`limit_cur` appear in both tables under similar names but different index numbers/ranges — e.g. `0X2016 loc_kp` default "30" in Table B vs. example value "30" read via CAN for an index also called `loc_kp` in the p.50 worked example using Table-A-style comm type 0x11/0x7F — it is not clear from the text whether these are the same parameter exposed through two index spaces, or two distinct parameters that happen to share a name).

11. **`max_alve` parameter referenced in prose but not in either parameter table** (p.52, section 4.2.7 "Cogging Calibration Function"): "After successful calibration, max_alve will be non-zero." No `max_alve` entry appears in either Table A or Table B as extracted.

12. **`acc_stutus` parameter referenced in prose but not found in either parameter table** (p.52, section 4.2.8 "PP Mode Deceleration"): "By setting acc_stutus to 1, the acceleration and deceleration for PP mode can be configured separately... When acc_stutus is set to 0, the acceleration parameter controls both [sentence cut off at page break, p.52→53]." No `acc_stutus` index found in either table.

13. **Driver chip fault code tables for function codes 0x3024 and 0x3025** (p.31): referenced by name ("driver chip fault code 1" / "driver chip fault code 2") but their bit-by-bit content is an embedded image that produced no extractable text. Could not transcribe.

14. **Parameter Table B "Maximum value"/"Minimum value" semantics**: for many rows (e.g. `0X2000 echoPara1`: Max=74, Min=5, Current=5; or `0X200a CAN_ID`: Max=127, Min=0, Current=1) it is unclear whether "Maximum/Minimum value" denotes an enforced valid range for writes, or merely reflects the field's bit-width/display formatting. Not stated explicitly in the extracted text.

15. **Table B "Current value (for reference)" column**: several rows show non-integer values for integer-typed fields (e.g. `0X301d phaseOrder` uint8 = "4.363409"; `0X301f boardTemp` int16 = "0.036618"), and several float-typed "current value" numbers look like large raw example readings (e.g. `VBUS` = "11396") rather than physically-scaled units matching the stated unit column. These are reproduced exactly as printed; they may be artifacts of how the host-computer tool's live snapshot was captured into the manual, not authoritative defaults — flagged for caution rather than resolved.

16. **CANopen section does not give raw CAN frame byte layouts** — only the object-dictionary table (index/name/attribute/type/unit) and a state-machine narrative. No raw SDO/PDO frame byte-format was found in the extracted pages (p.63-70), only protocol classification prose (p.63-64) and per-mode object tables (p.65-70). If raw CANopen frame byte layouts exist elsewhere in the PDF (e.g. in an image/diagram), they were not captured by text extraction.
