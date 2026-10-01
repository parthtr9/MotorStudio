# RS00 protocol extract
Source: RS00User Manual260713.pdf
Total pages in PDF: 79 (per pdfinfo; page 1 is cover page "ROBSTRIDE 00").

## Model/firmware identification

- Model name as printed on cover: "ROBSTRIDE 00", subtitle "14N.M Quasi-Direct Drive Integrated Motor Module" (p.1). Manual title: "RobStride Motor User Manual" / "RS00 instruction manual" (p.1-2).
- Manufacturer: "Beijing Lingfoot Times Technology Co., LTD." (referred to as "Lingfoot") (p.2). Also referred to internally as "Lingzu"/"Lingzu Times" in other sections (p.3, p.12). Website: https://www.robstride.com/ (p.1).
- Firmware/version strings found in the default parameter table (read-only, Section 3.3.4, p.19-20):
  - BootCodeVersion = "0.1.5" (p.19)
  - BootBuildDate = "Mar 16 2022" (p.19)
  - BootBuildTime = "20:22:09" (p.19)
  - AppCodeVersion = "0.0.0.1" — annotated "Motor program version number" (p.19)
  - AppGitVersion = "7b844b0fM" (p.19)
  - AppBuildDate = "Apr 14 2022" (p.19)
  - AppBuildTime = "20:30:22" (p.19)
  - AppCodeName = "Lingzu_motor" (p.19)
  - Note: these look like example/default values read from a sample device, not necessarily the manual's own target firmware version.
- Several commands in Section 6 (MIT protocol) are explicitly gated by firmware version: "Firmware update to 0.0.3.27 is required." for Command 12 (Save motor data), Command 13 (Motor actively reports), Command 14 (Read motor parameters), Command 15 (Write motor parameters) (p.74-76).
- Version History table (p.79, Section 7): Version "1.0", Description "Initial Release", Date "November 25, 2025".
- Rated torque identification: "14N.M" in title (p.1); Peak load 14 N.m (p.9, see Electrical characteristic).
- No explicit single "model number" distinct from "RS00"/"ROBSTRIDE 00" was found.

## CAN bus parameters

- "The articulated motor uses the CAN communication mode and has two communication cables." (p.13, Section 3.1 Hardware disposition)
- "CAN bus bit rate: 1Mbps" — listed in Driver product Specifications table (Section 2.1, p.11).
- Section 4 (private/default protocol) explicit statement: "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted" (p.38).
- Section 6 (MIT protocol) explicit statement: "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol." (p.70).
- Baud-rate modification frame (private protocol Communication Type 23, 0x17) sets F_CMD values: "01 is 1M", "02 is 500K", "03 is 250K", "04 is 125K" (p.42-43). Effective after re-power-on (p.42, section title: "re-power-on effect").
- Termination resistor note (for the USB-CAN debug adapter, not the motor itself): "When DIP switch 2 is in the ON position, a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer." (p.12-13). DIP switch 1 ON puts the adapter module into Boot mode (p.13).
- Serial-port framing for the vendor's CAN-to-USB adapter (not the CAN bus itself): "The frame header of the corresponding serial port protocol is 41 54, and the frame tail is 0D 0A." (p.13).
- Protocol types supported, switchable via Communication Type 25 (0x19) / host-computer `protocol_1` parameter, re-power-on effect (p.44-45, p.49):
  - 0 = "private protocol" (default)
  - 1 = CANopen protocol
  - 2 = MIT protocol
- Default/example CAN IDs seen in worked examples: host id example "0xFD" (p.47), target motor CAN_ID example "0x7F" (p.47, and throughout Section 6 examples using "07F" as target motor CAN ID placeholder, e.g. p.71-76). The manual does not state these are factory-default IDs explicitly outside of being used as worked examples; treat with caution (see OPEN QUESTIONS).
- Default motor CAN ID value shown in the parameter table: `CAN_ID` function code 0X200a, type uint8, "Settings", Maximum value 127, Minimum value 0, Current value (for reference) = 1 (p.21). Likewise `CAN_MASTER` (host CAN ID) function code 0X200b, Current value (for reference) = 0 (p.21).

## Frame definitions (TX - host to motor)

NOTE: The manual documents THREE separate protocols selectable via `protocol_1` / Communication Type 25: (A) a private/default 29-bit extended-frame protocol (Section 4, p.37-59), (B) CANopen (Section 5, p.60-68), (C) MIT protocol using an 11-bit standard frame (Section 6, p.69-78). These are mutually exclusive on-wire formats, not simultaneous options. Frame layouts below are transcribed per protocol.

### (A) Private protocol — general extended-frame layout (p.38)

29-bit extended CAN ID + 8-byte data field:
| field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | Communication type | data area 2 | Destination address | data area 1 (8-byte data field) |

"The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted" (p.38).

Control modes supported (p.38, bulleted, exact wording):
- "Operation control mode: set 5 parameters of motor operation control;"
- "Current mode: the specified Iq current of the given motor;"
- "Velocity mode: the specified running speed of the given motor;"
- "Position mode: Given the specified position of the motor, the motor will run to the specified position;"

#### Communication type 0 (0x0): Get device ID — request (p.37)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x0 | bit15~8: identifies host CAN_ID | target motor CAN_ID | 0 |
"Gets the device's ID and 64-bit MCU unique identifier" (p.37). Reply frame documented under RX section below.

#### Communication Type 1 (0x1): operation control mode motor control instruction (p.39)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x1 | Byte2: Torque (0~65535) corresponds to (-14Nm~14Nm) [as printed; see OPEN QUESTIONS re: field placement] | target motor CAN_ID | Byte0~1: target Angle [0~65535] corresponds to (-4π~4π); Byte2~3: Target angular velocity [0~65535] corresponds to (-33rad/s~33rad/s); Byte4~5: Kp [0~65535] corresponds to (0.0~500.0); Byte6~7: Kd [0 to 65535] corresponds to (0.0 to 5.0). "After the conversion, the high byte is in front and the low byte is in [cut off]" |
Response: "Response motor feedback frame (see communication type 2)" (p.39).
NOTE: the "bit23~8" column text literally says "Byte2: Torque (0~65535) corresponds to (-14Nm~14Nm)" which conflicts with Byte2~3 appearing again under Byte0~Byte7 as angular velocity — this looks like a table-rendering/column-overflow artifact in the PDF-to-text extraction, not a reliable byte map. See OPEN QUESTIONS.

#### Communication Type 3 (0x3): Motor enabled to run (p.39-40)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x3 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | (not specified in table text) |
Response: "Response motor feedback frame (see communication type 2)" (p.40).

#### Communication Type 4 (0x4): Motor stops running (p.40)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "When the motor is running normally, 0 must be cleared in the data field. Byte[0]=1: The fault is cleared." |
Response: "Response motor feedback frame (see communication type 2)" (p.40).

#### Version number read frame (also uses comm type 0x4) (p.40)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | Byte[0]=0x00, Byte[1]=0xC4 |
Reply frame documented under RX section below (p.40-41).

#### Communication type 6 (0x6): Set motor mechanical zero (p.41)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x6 | bit15~8: Identifies the main CAN_ID | and target motor CAN_ID | Byte[0]=1 |
Response: "Response motor feedback frame (see communication type 2)" (p.41).

#### Communication type 7 (0x7): Set motor CAN_ID (p.41)
"Change the current motor CAN_ID, effective immediately."
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x7 | bit15~8: used to identify main CAN_ID; Bit16~23: preset CAN_ID | Target motor CAN_ID | (not specified) |
Answer frame: "Answer motor broadcast frame (see communication type 0)" (p.41).

#### Communication type 17 (0x11): Single parameter read (p.41-42)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x11 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index (see parameter table); Byte2~3: 00; Byte4~7: "In data above 00, the low byte is first and the high byte is second" [exact wording, somewhat garbled in source] |
Reply frame documented under RX section below (p.42).

#### Communication type 18 (0x12): Single parameter write (lost in power failure) (p.42)
"With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved"
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x12 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index (see parameter table); Byte2~3: 00; Byte4~7: Parameter data — "low byte is in the front and the high byte is in the rear" |
Response: "Response motor feedback frame (see communication type 2)" (p.42).

#### Communication type 22 (0x16): Motor data save frame (p.43)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x16 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | "01 02 03 04 05 06 07 08" |
Response: "Response motor feedback frame (see communication type 2)" (p.43).

#### Communication type 23 (0x17): Motor baud rate modification frame (re-power-on effect) (p.43-44)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x17 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "01 02 03 04 05 06 F_CMD" where F_CMD = motor baud rate: "01 is 1M, 02 is 500K, 03 is 250K, 04 is 125K" |
Response: "Response motor feedback frame (see communication type 0)" (p.44).

#### Communication type 24 (0x18): The motor actively reports frames (p.44)
Command (host->motor, to enable/disable):
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x18 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | "01 02 03 04 05 06 F_CMD" where F_CMD = reporting switch: "00 is to disable active reporting (default); 01 To enable active reporting, the default reporting interval is 10ms" |
Response frame (motor->host active report) documented under RX section below (p.44-45).

#### Communication type 25 (0x19): Motor protocol modification frame (re-power-on effect) (p.45)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x19 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "01 02 03 04 05 06 F_CMD" where F_CMD = protocol type: "0 is a private protocol (default), 1 is the Canopen protocol, 2 is the MIT protocol" |
Response: "Response motor feedback frame (see communication type 0)" (p.45).

#### Worked read example — Communication Type 17 request (p.47)
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| | 0x11 | Host id 0xFD | Target motor CAN_ID 0x7F | "1E 70 00 00 00 00 00 00" — "Byte0~1: index, corresponding to loc_kp" |

### (B) CANopen protocol frames (Section 5, p.60-68)

CAN layer described generically: "CAN simply implements the transmission of frames with an 11-bit CAN ID, a Remote Transmission (RTR) bit, and 64 data bits" (p.61) — this is generic CANopen/CAN background text, not RS00-specific framing detail beyond the object dictionary tables below.

Protocol Switching frame to select CANopen (sent as the private-protocol extended frame, p.67-68):
| Data Field | 29-bit ID: Bit 28~0 | 8-Byte Data Area: Byte 0~6 |
|---|---|---|
| Description | 0xFFF | "01 02 03 04 05 06 F_CMD" |
F_CMD (Byte 6) defines the motor protocol: "0: Private protocol (default); 1: CANopen protocol; 2: MIT protocol" (p.67).
Response frame documented under RX section below (p.68-69).

CANopen object-dictionary writes (Controlword 6040H, Modes of operation 6060H, Target_* objects) — see "Parameter table" section below for the full object tables as printed (p.63-68); these are read/write objects, not raw byte-offset frames, per the manual's own description (SDO-style access implied, not explicitly spelled out as raw CAN frame byte layout in this manual).

### (C) MIT protocol (Section 6, p.69-78)

General frame format (standard/11-bit frame):
| Data Field | 11-bit ID: Bit 10~8 | Bit 7~0 | 8-byte Data Area: Byte 0~7 |
|---|---|---|---|
| Description | Mode type | ID | (payload per command) |
"The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps... The standard frame format is as follows" (p.70).

Supported control modes (exact wording, p.70): "MIT Mode: Provides five motion control parameters to the motor." / "Velocity Mode: Specifies the target speed for the motor." / "Position Mode: Specifies the target position and speed, allowing the motor to run to the designated position at the configured speed."

#### Command 1: Enable Motor Operation (p.71)
| Data Field | 11-bit ID: Bit 10~0 | 8-byte Data Area: Byte 0~7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF FF FC |
Description: Target motor CAN ID = 07F; data = FF FF FF FF FF FF FF FC.
Response: "Response Command 1" (p.71).

#### Command 2: Stop Motor Operation (p.71)
| Data Field | 11-bit ID: Bit 10~0 | 8-byte Data Area: Byte 0~7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF FF FD |
Response: "Response Command 1" (p.71).

#### Command 3: MIT Dynamic Parameters (p.71-72)
| Data Field | 11-bit ID: bit10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | 00 00 00 00 00 00 00 00 (example/placeholder) |
Description:
- "Byte0~1: Target angle [0~65535], corresponding to (-12.57 rad ~ 12.57 rad)"
- "Byte2: High 8 bits;"
- "Byte3[7-4] (high 4 bits) as low 4 bits: Target speed [0~4096], corresponding to (-33 rad/s ~ 33 rad/s)"
- "Byte3[3-0] (low 4 bits) as high 4 bits;"
- "Byte4: Low 8 bits: Kp [0~4096], corresponding to 0 ~ 500"
- "Byte5: High 8 bits;"
- "Byte6[7-4] (high 4 bits) as low 4 bits: Kd [0~4096], corresponding to 0 ~ 5"
- "Byte6[3-0] (low 4 bits) as high 4 bits;"
- "Byte7: Low 8 bits: Target torque [0~4096], corresponding to (-14 N·m ~ 14 N·m)"
Response: "Response Command 1" (p.72).
NOTE: This bit-packing (12-bit fields split across byte boundaries described as "Byte3[7-4] as low 4 bits" / "Byte3[3-0] as high 4 bits") is transcribed exactly as printed, but the described nibble-swap pattern does not cleanly resolve into a standard little/big-endian 12-bit pack from the text alone — flagged in OPEN QUESTIONS.

#### Command 4: Set Zero Position (Non-Position Mode) (p.72)
| Data Field | 11-bit ID: Bit 10~0 | 8-byte Data Area: Byte 0~7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF FF FE |
Response: "Response Command 1" (p.72).

#### Command 5: Clear Errors & Read Fault Status (p.72)
| Data Field | 11-bit ID: Bit 10~0 | 8-byte Data Area: Byte 0~7 |
|---|---|---|
| | Target Motor CAN ID | FF FF FF FF FF FF F_CMD FB |
"F_CMD: 0xFF → clearing the current abnormality; Any other value → the error code will be returned in the BYTE1 field of the response message." (p.72)
Response (Fault Clear): "Response Command 1" (p.72).

#### Command 6: Set Operation Mode (p.73)
| Data Field | 11-bit ID: bit10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF 00 FC (example) |
Description: "FF FF FF FF FF FF F_CMD FC" — "F_CMD: Mode type; 0: MIT mode (default); 1: Position mode; 2: Velocity mode" (p.73)
Response: "Response Command 1" (p.73).

#### Command 7: Modify Motor CAN ID (p.73)
| Data Field | 11-bit ID: bit10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF 01 FA (example) |
Description: "FF FF FF FF FF FF F_CMD FA" — "F_CMD: Target motor CAN ID" (p.73)
Response: "Response Command 2" (p.73).

#### Command 8: Change Communication Protocol (Takes Effect After Power Cycle) (p.74)
| Data Field | 11-bit ID: bit10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF 00 FD (example) |
Description: "FF FF FF FF FF FF F_CMD FD" — "F_CMD: Protocol type; 0: Private protocol (default); 1: CANopen; 2: MIT protocol" (p.74)
Response: "Response Command 2" (p.74).

#### Command 9: Modify Host CAN ID (p.74)
| Data Field | 11-bit ID: bit10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF FD 01 (example) |
Description: "FF FF FF FF FF FF F_CMD 01" — "F_CMD: Host CAN ID" (p.74)
Response: "Response Command 2" (p.74).

#### Command 10: Position Mode Control Command (p.74)
| Data Field | 11-bit ID: bit10~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|
| | 1 | 7F | "00 00 A0 40 00 00 A0 40 (Position:5rad Speed: 5rad/s)" |
Description: "1" = mode-type field value; Target motor CAN ID = 7F; "Byte 0~3: Target position (rad, 32-bit float)"; "Byte 4~7: Target speed (rad/s, 32-bit float)" (p.74)
Response: "Response Command 1" (p.74).

#### Command 11: Velocity Mode Control Command (p.75)
| Data Field | 11-bit ID: bit10~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|
| | 2 | 7F | "00 00 A0 40 00 00 00 40 (Velocity: 5rad/s, Current: 2A)" |
Description: "2" = mode-type field value; Target motor CAN ID = 7F; "Byte 0~3: Target speed (rad/s, 32-bit float)"; "Byte 4~7: Current limit in speed/position mode (A, 32-bit float)" (p.75)
Response: "Response Command 1" (p.75).

#### Command 12: Save motor data (Firmware update to 0.0.3.27 is required.) (p.75)
| Data Field | 11-bit ID: Bit 10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF FF F8 |
Response: "Response command 2" (p.75).

#### Command 13: Motor actively reports (Firmware update to 0.0.3.27 is required.) (p.75)
| Data Field | 11-bit ID: Bit 10~0 | Byte0~Byte7 |
|---|---|---|
| | 07F | FF FF FF FF FF FF 00 F9 (example) |
Description: "FF FF FF FF FF FF F_CMD F9" — "The F_CMD byte indicates the motor protocol type. 0 indicates no reporting (default). 1 is for reporting" (p.75)
Response: "Response command 1" (p.75).

#### Command 14: Read motor parameters (Firmware update to 0.0.3.27 is required.) (p.75-76)
| Data Field | 11-bit ID: bit10~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|
| | 3 | 7F | "05 70 00 00 00 00 00 00" (example reading index 0x7005) |
Description: "Byte0~1: index (see read/write parameter table below); Byte2~3: 00; Byte4~7: 00. The above data is in low byte first, high byte last." (p.75-76)
Response frame layout identical (same fields), "same as the private protocol parameter table" (p.76).

#### Command 15: Write motor parameters (Firmware update to 0.0.3.27 is required.) (p.76)
| Data Field | 11-bit ID: bit10~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|
| | 4 | 7F | "29 70 00 00 00 00 00 00" (example writing index 0x7029) |
Description: "Byte0~1: index (see read/write parameter table for details, same as private protocol parameter table); Byte2~3: 00; Byte4~7: 00. Parameter data: the above data is in low byte first, high byte last." (p.76)
Response frame layout identical (p.76).

## Frame definitions (RX - motor to host)

### (A) Private protocol responses

#### Reply to Communication type 0 (Get device ID) (p.37)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x0 | target motor CAN_ID | 0XFE | 64-bit MCU unique identifier |

#### Communication Type 2 (0x2): motor feedback data (p.39-40)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none, 1 has) [bit21: uncalibrated; bit20: Uncalibrated/Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault]; bit22~23: Mode status [0: Reset mode (reset); 1: Cali mode (calibration); 2: Motor mode (Run)] | host CAN_ID | Byte0~1: The current Angle [0~65535] corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-33rad/s~33rad/s); Byte4~5: Current torque [0~65535] corresponds to (-14Nm~14Nm); Byte6~7: Current temperature: Temp(Celsius) *10. "If the value is higher than 10, the high byte is first and the low byte is last" |
(p.39-40; note the "bit21: uncalibrated" and "bit20: Uncalibrated" both appear — exact text as printed, likely a duplication/typo in the source; see OPEN QUESTIONS)

#### Reply to Version number read frame (p.40-41)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x2 | Bit8~Bit15: CAN ID of current motor; bit21~16: fault information (same bit table as Type 2); bit22~23: Mode status (same as Type 2) | target motor CAN_ID | Byte0=0x00; Byte1=0xC4; Byte2=0x56; Byte3~6: Motor version number, "Order from high to low" |

#### Reply to Communication type 17 (Single parameter read) (p.42)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x11 | bit15~8: indicates that the master CAN_ID; Bit23~16: "00 indicates that the master CAN_ID is successfully read. 01 indicates that the master can_ID" [sentence cuts off in source] | Byte0~1 / Byte2~3: 00 [column text malformed/ambiguous in extraction] | Byte4~7: Parameter data. "1 byte of data above Byte4 is preceded by low bytes and followed by high bytes" |
NOTE: this table's text did not extract cleanly (column boundaries ambiguous) — see OPEN QUESTIONS.

#### Communication type 24 (0x18) active-report response (motor -> host) (p.44-45)
| | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x18 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (same bit table as Type 2); bit22~23: Mode status (same as Type 2) | target motor CAN_ID | Byte0~1: Current Angle [0~65535] corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-33rad/s~33rad/s); Byte4~5: Current torque [0~65535] corresponds to (-14Nm~14Nm); Byte6~7: Current temperature: Temp(Celsius)*10, "If the value is higher than 10, the high byte is first and the low byte is last" |

#### Worked read example — Communication Type 17 feedback (p.47)
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| | 0x11 | bit15~8: Target motor CAN_ID 0x7F | Host id 0xFD | "1E 70 00 00 00 00 F0 41" — "Byte0~1: index, corresponding to loc_kp. Byte4~7: loc_kp value 30, high right byte, (32-bit single precision) hexadecimal IEEE-754 standard floating point number" |

### (B) CANopen response

Protocol-switch response frame (p.68-69):
| Data Field | 11-bit ID: Bit 10~0 | 8-Byte Data Area: Byte 0~7 |
|---|---|---|
| Description | Motor ID | 64-bit MCU unique identifier |

### (C) MIT protocol responses

#### Response Command 1: Data Feedback (Motor Status) (p.70-71)
| Data Field | 11-bit ID: bit10~0 | Byte0~Byte7 |
|---|---|---|
| | 0FD (Host ID) | 7F 00 00 00 00 00 00 00 (example) |
Description:
- "Byte0: Motor CAN ID"
- "Byte1~2: Current angle [0~65535], corresponding to (-12.57 rad ~ 12.57 rad)"
- "Byte3: High 8 bits;"
- "Byte4[7-4] (high 4 bits) as low 4 bits: Current speed [0~4096], corresponding to (-33 rad/s ~ 33 rad/s)"
- "Byte4[3-0] (low 4 bits) as high 4 bits;"
- "Byte5: Low 8 bits: Current torque [0~4096], corresponding to (-14 N·m ~ 14 N·m)"
- "Byte6[7-6]: Operating Mode — 0: Reset Mode; 1: Cali Mode; 2: Motor Mode"
- "Byte6[5]: 1 = Motor fault, 0 = No motor fault"
- "Byte6[4]: 1 = Motor warning, 0 = No warning"
- "Byte6[3-0] (low 4 bits) as high 4 bits;"
- "Byte7: Low 8 bits: Winding temperature: Temperature (°C) × 10, unit: °C"
(p.70-71) NOTE: same nibble-swap ambiguity as Command 3 TX frame — flagged in OPEN QUESTIONS.

#### Response Command 2: MCU Identification (p.71)
| Data Field | 11-bit ID: Bit 10~0 | Byte0~Byte7 |
|---|---|---|
| | Motor ID | 64-bit MCU unique identifier |

#### Fault Status Response (p.72-73)
| Data Field | 11-bit ID: Bit 10~0 (Host CAN ID = 0FD) | Byte0~Byte7 |
|---|---|---|
| | 0FD | 7F 00 00 00 00 00 00 00 (example) |
- "Byte0: Target motor CAN ID"
- "Byte1~4: Fault value (Non-zero: faulty; 0: normal)" — bit table identical to the private-protocol fault bits (see Fault/error codes section below).
(p.72-73)

## Parameter table (flash config read/write)

### Read-only identification/info rows (Section 3.3.4, p.19-20) — function code, name, parameter type, attribute, max value, min value, current value (for reference), remark, all columns as printed:

| function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | 备注 (remark) |
|---|---|---|---|---|---|---|---|
| 0X0000 | Name | String | Read/Write | | | ÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿ | |
| 0X0001 | BarCode | String | Read/Write | | | ÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿÿ | |
| 0X1000 | BootCodeVersion | String | Read only | | | 0.1.5 | |
| 0X1001 | BootBuildDate | String | Read only | | | Mar 16 2022 | |
| 0X1002 | BootBuildTime | String | Read only | | | 20:22:09 | |
| 0X1003 | AppCodeVersion | String | Read only | | | 0.0.0.1 | Motor program version number |
| 0X1004 | AppGitVersion | String | Read only | | | 7b844b0fM | |
| 0X1005 | AppBuildDate | String | Read only | | | Apr 14 2022 | |
| 0X1006 | AppBuildTime | String | Read only | | | 20:30:22 | |
| 0X1007 | AppCodeName | String | Read only | | | Lingzu_motor | |

### Settings/config parameters (p.20-22):

| function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | 备注 (remark) |
|---|---|---|---|---|---|---|---|
| 0X2000 | echoPara1 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2001 | echoPara2 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2002 | echoPara3 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2003 | echoPara4 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2004 | echoFreHz | uint32 | Read/Write | 10000 | 1 | 500 | |
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
| 0X201a | loc_ref_filt_gain | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201b | limit_loc | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201c | position_offset | float | Read/Write | 27 | 0 | 0 | High speed segment offset |
| 0X201d | chasu_angle_offset | float | Read/Write | 27 | 0 | (not given — row split across p.21/22) | "The low end is offset" |
| 0X201e | spd_step_value | float | Read/Write | 150 | 0 | (blank) | Velocity-mode acceleration |
| 0X201f | vel_max | float | Read/Write | 20 | 0 | (blank) | PP mode speed |
| 0X2020 | acc_set | float | Read/Write | 1000 | 0 | (blank) | PP mode acceleration |
| 0X2021 | zero_sta | float | Read/Write | 100 | 0 | 0 | Zero marker |
| 0x2022 | protocol_1 | uint8 | Read/Write | (blank) | (blank) | 0 | Protocol flag |
| 0x2023 | damper | uint8 | Read/Write | 0 | 20 | 0 | Damping switch — NOTE: Max=0, Min=20 as printed, i.e. max < min; transcribed exactly, flagged in OPEN QUESTIONS |
| 0x2024 | add_offset | float | Read/Write | -7 | 7 | 0 | Position offset parameter — NOTE: Max=-7, Min=7 as printed (max < min), flagged in OPEN QUESTIONS |

### Read-only runtime/telemetry parameters (p.22-26):

| function code | name | type | attribute | Current value (example) | remark |
|---|---|---|---|---|---|
| 0X3000 | timeUse0 | uint16 | Read only | 5 | |
| 0X3001 | timeUse1 | uint16 | Read only | 0 | |
| 0X3002 | timeUse2 | uint16 | Read only | 10 | |
| 0X3003 | timeUse3 | uint16 | Read only | 0 | |
| 0X3004 | encoderRaw | int16 | Read only | 11396 | Magnetic encoder sampling value |
| 0X3005 | mcuTemp | int16 | Read only | 337 | mcu internal temperature, *10 |
| 0X3006 | motorTemp | int16 | Read only | 333 | Motor ntc temperature, *10 |
| 0X3007 | vBus(mv) | uint16 | Read only | 24195 | Bus voltage |
| 0X3008 | adc1Offset | int32 | Read only | 2084 | adc sampling channel 1 Zero current bias |
| 0X3009 | adc2Offset | int32 | Read only | 2084 | adc sampling channel 2 Zero current bias |
| 0X300a | adc1Raw | uint16 | Read only | 1232 | adc sampling value 1 |
| 0X300b | adc2Raw | uint16 | Read only | 1212 | adc sampling value 2 |
| 0X300c | VBUS | float | Read only | 36 | Bus voltage V |
| 0X300d | cmdId | float | Read only | 0 | id ring instruction, A |
| 0X300e | cmdIq | float | Read only | 0 | iq ring command, A |
| 0X300f | cmdlocref | float | Read only | 0 | Position loop command, rad |
| 0X3010 | cmdspdref | float | Read only | 0 | Speed loop command, rad/s |
| 0X3011 | cmdTorque | float | Read only | 0 | Torque instruction, nm |
| 0X3012 | cmdPos | float | Read only | 0 | mit Protocol Angle instruction |
| 0X3013 | cmdVel | float | Read only | 0 | mit Protocol Speed instruction |
| 0X3014 | rotation | int16 | Read only | 1 | Number of turns |
| 0X3015 | modPos | float | Read only | 4.363409 | Motor uncounted coil mechanical Angle, rad |
| 0X3016 | mechPos | float | Read only | 0.777679 | Load end loop mechanical Angle, rad |
| 0X3017 | mechVel | float | Read only | 0.036618 | Load speed: rad/s |
| 0X3018 | elecPos | float | Read only | 4.714761 | Electrical Angle |
| 0X3019 | ia | float | Read only | 0 | U-wire current, A |
| 0X301a | ib | float | Read only | 0 | V-wire current, A |
| 0X301b | ic | float | Read only | 0 | W-wire current, A |
| 0X301c | timeout | uint32 | Read only | 31600 | Timeout counter value |
| 0X301d | phaseOrder | uint8 | Read only | 0 | Directional marking |
| 0X301e | iqf | float | Read only | 0 | iq filter value，A |
| 0X301f | boardTemp | int16 | Read only | 359 | Plate temperature，*10 |
| 0X3020 | iq | float | Read only | 0 | iq Original value，A |
| 0X3021 | id | float | Read only | 0 | id Original value，A |
| 0X3022 | faultSta | uint32 | Read only | 0 | Fault status value |
| 0X3023 | warnSta | uint32 | Read only | 0 | Warning status value |
| 0X3024 | drv_fault | uint16 | Read only | 0 | "The driver chip fault value is 1" |
| 0X3025 | drv_temp | int16 | Read only | 48 | "The driver chip fault value is 2" |
| 0X3026 | Uq | float | Read only | 0 | Q-axis voltage |
| 0X3027 | Ud | float | Read only | 0 | D-axis voltage |
| 0X3028 | dtc_u | float | Read only | 0 | The duty cycle of the U-phase output |
| 0X3029 | dtc_v | float | Read only | 0 | The duty cycle of the V-phase output |
| 0X302a | dtc_w | float | Read only | 0 | The duty cycle of the W-phase output |
| 0X302b | v_bus | float | Read only | 24.195 | Vbus in the closed loop |
| 0X302c | torque_fdb | float | Read only | 0 | Torque feedback value，nm |
| 0X302d | rated_i | float | Read only | 8 | Rated current of motor |
| 0X302e | limit_i | float | Read only | 27 | The motor limits the maximum current |
| 0X302f | spd_ref | float | Read only | 0 | Motor speed expectation |
| 0X3030 | spd_reff | float | Read only | 0 | Motor speed expectation 2 |
| 0X3031 | zero_fault | float | Read only | 0 | Motor position determination parameters |
| 0X3032 | chasu_coder_raw | float | Read only | 0 | Motor position determination parameters |
| 0X3033 | chasu_angle | float | Read only | 0 | Motor position determination parameters |
| 0X3034 | as_angle | float | Read only | 0 | Motor position determination |
| 0X3035 | vel_max | float | Read only | 0 | Motor position determination parameters |
| 0X3036 | judge | float | Read only | 0 | Motor position determination parameters |
| 0X3037 | position | float | Read only | 0 | Position value |
| 0X3038 | chasu_angle_init | float | Read only | -0.002301 | Angle initialization |
| 0X3039 | chasu_angle_out | float | Read only | -0.23012 | Motor position determination parameters |
| 0X303a | motormechinit | float | Read only | -0.000383 | Motor position determination parameters |
| 0X303b | mech_angle_init2 | float | Read only | -0.000115 | Motor position determination parameters |
| 0X303c | mech_angle_rotat | float | Read only | 0 | Motor position determination parameters |
| 0X303d | fault1 | uint32 | Read only | 0 | Log failure |
| 0X303e | fault2 | uint32 | Read only | 0 | Log failure |
| 0X303f | fault3 | uint32 | Read only | 0 | Log failure |
| 0X3040 | fault4 | uint32 | Read only | 0 | Log failure |
| 0X3041 | fault5 | uint32 | Read only | 0 | Log failure |
| 0X3042 | fault6 | uint32 | Read only | 0 | Log failure |
| 0X3043 | fault7 | uint32 | Read only | 0 | Log failure |
| 0X3044 | fault8 | uint32 | Read only | 0 | Log failure |
| 0X3045 | ElecOffset | float | Read only | 0 | electrical Angle offset |
| 0X3046 | mcOverTemp | int16 | Read only | 0 | Overtemperature threshold |
| 0X3047 | Kt_Nm/Amp | float | Read only | 0 | Moment coefficient |
| 0X3048 | Tqcali_Type | uint8 | Read only | 0 | Motor type |
| 0X3049 | low_position | float | Read only | 0 | Motor position determination parameters |
| 0X304a | theta_mech_1 | float | Read only | 0 | Type 2 Low speed Angle |
| 0X304b | instep | float | Read only | 0 | Motor protection decision parameters |
| 0X304c | adc0ffset_1 | int32 | Read only | 2034 | adc sampling channel 1 Zero current bias |
| 0X304d | adc0ffset_2 | int32 | Read only | 2040 | adc sampling channel 2 Zero current bias |
| 0X304e | pos_cnt1 | uint16 | Read only | 0 | System parameters |
| 0X304f | H' | uint8 | Read only | 72 | System parameters |

(All above from p.20-26, Section 3.3.4.)

### Read/write parameter list via Communication type 17/18 (Section 4.1.13, p.44-47) — "Read and write a single parameter list" table, columns: index, name (as table sub-header; names appear left of description in source), Description, Type, Number of bytes, Range, R/W Read and write permission:

| index | name | Description | Type | Number of bytes | Range (as printed) | R/W |
|---|---|---|---|---|---|---|
| 0X7005 | run_mode | "0: operation mode / 1: position mode (PP) / 2: Velocity mode / 3: Operation mode Current mode / 5: Position mode (CSP)" [note: "3: Operation mode / Current mode" text ambiguous — see OPEN QUESTIONS; no explicit "4" value given in source] | uint8 | 1 | | W/R |
| 0X7006 | iq_ref | Current mode Iq command | float | 4 | -16 to 16A | W/R |
| 0X700A | spd_ref | Rotational Velocity mode Rotational speed command | float | 4 | -33 to 33rad/s | W/R |
| 0X700B | limit_torque | torque limit | float | 4 | 0 to 14Nm | W/R |
| 0X7010 | cur_kp | Kp | float | 4 | The default value is 0.17 | W/R |
| 0X7011 | cur_ki | Ki | float | 4 | The default value is 0.012 | W/R |
| 0X7014 | cur_filt_gain | filt_gain | float | 4 | 0 to 1.0, The default value is 0.1 | W/R |
| 0X7016 | loc_ref | Position Mode Angle instruction | float | 4 | rad | W/R |
| 0X7017 | limit_spd | Location mode (CSP) speed limit | float | 4 | 0 to 33rad/s | W/R |
| 0X7018 | limit_cur | Velocity position mode Current limitation | float | 4 | 0 to 16A | W/R |
| 0x7019 | mechPos | Mechanical Angle of the loading coil | float | 4 | rad | R |
| 0x701A | iqf | iq Filter | float | 4 | -16 to 16A | R |
| 0x701B | mechVel | Speed of the load | float | 4 | -33 to 33rad/s | R |
| 0x701C | VBUS | Bus voltage | float | 4 | V | R |
| 0x701E | loc_kp | "at kp" [printed as two-line "at" / "kp"; likely just "loc_kp"] | float | 4 | The default value is 40 | W/R |
| 0x701F | spd_kp | Indicates the speed kp | float | 4 | The default value is 6 | W/R |
| 0x7020 | spd_ki | ki | float | 4 | The default value is 0.02 | W/R |
| 0x7021 | spd_filt_gain | Speed filter value | float | 4 | The default value is 0.1 | W/R |
| 0x7022 | acc_rad | velocity mode acceleration | float | 4 | The default value is 20rad/s^2 | W/R |
| 0x7024 | vel_max | Location mode (PP) speed | float | 4 | The default value is 10rad/s | W/R |
| 0x7025 | acc_set | Location mode (PP) acceleration | float | 4 | The default value is 10rad/s^2 | W/R |
| 0x7026 | EPScan_time | "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | The default value is 1 | W/R |
| 0x7028 | canTimeout | "can The timeout threshold, 20000 is 1s" | uint32 | 4 | The default value is 0 | W/R |
| 0x7029 | zero_sta | "Indicates the zero flag bit, 0 means 0-2π and 1 means -π-π" | uint8 | 1 | The default is 0 | W/R |
| 0x702A | damper | Damping switch | uint8 | 1 | The default value is 0 | W/R |
| 0x702B | add_offset | Zero offset | float | 4 | The default is 0 | W/R |
| 0x702C | alveolous_open | Cogging Compensation Switch | uint8 | 1 | The default is 0 | W/R |
| 0x702D | iq_test | Motor Initialization Calibration Switch | uint8 | 1 | The default is 0 | W/R |
| 0x702E | dcc_set | "Deceleration in Motor PP Mode (Normally Disabled)" | float | 4 | The default is 10rad/s^2 | W/R |

(All above from p.44-47. Note: the index values/sub-indices run 0X7005 through 0x702E and do not strictly correspond 1:1 to the 0x20xx "function code" settings table on p.20-22 by numeric offset — they appear to be a parallel/distinct index space used specifically by Communication Type 17/18 and MIT Commands 14/15. Manual states explicitly for MIT Commands 14/15 that this index table is "same as the private protocol parameter table" (p.75-76), implying the two tables should be the same logical parameter set, but the numeric index values printed (0x70xx) differ from the 0x20xx/0x30xx codes in Section 3.3.4 — see OPEN QUESTIONS.)

Worked example confirms encoding: reading index corresponding to `loc_kp` returns "1E 70 00 00 00 00 F0 41" where bytes 0-1 "1E 70" = index 0x701E (little-endian), and bytes 4-7 "00 00 F0 41" decode (little-endian IEEE-754 float) to 30.0 — manual states "loc_kp value 30" (p.47).

### CANopen object dictionary tables (Section 5, p.63-68):

#### Status Feedback Parameters (p.63-64):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 603F | Error_code | Read-only | UINTEGER16 | / |
| 6041 | Statusword | Read-only | UINTEGER16 | / |
| 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
| 6062 | Position_demand_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6064 | Position_actual_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 606B | Velocity_demand_value | Read-only | INTEGER32 | 0.1 rpm |
| 606C | Velocity_actual_value | Read-only | INTEGER32 | 0.1 rpm |
| 6077 | Torque_actual_value | Read-only | INTEGER16 | 0.1% load ratio (1000 = 5 N·m) |
| 6078 | Current_actual_value | Read-only | INTEGER16 | mA |
| 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | mV |

#### Homing Mode (p.64):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
"Set Modes of operation to 6 while the motor is in the disabled state... The motor will then define the current position as the zero point." "To hold the zero position, modify the Controlword to 15..." (p.64)

#### Position Mode PP (p.64-65):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6067 | Position_window | Read-write | UINTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6068 | Position_window_time | Read-write | UINTEGER16 | ms |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 5 N·m) |
| 607A | Target_position | Read-write | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6081 | Profile_velocity | Read-write | UINTEGER32 | 0.1 rpm |
| 6083 | Profile_acceleration | Read-write | UINTEGER32 | 0.1 rpm/s |
Configure: set Modes of operation = 1 (disabled state); mandatory: Target_torque, Profile_velocity, Profile_acceleration; optional: Position_window, Position_window_time; set Controlword = 15 to enable; set Target_position to move (p.65-66).

#### Position Mode CSP (p.66-67):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6067 | Position_window | Read-write | UINTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6068 | Position_window_time | Read-write | UINTEGER16 | ms |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 5 N·m) |
| 607A | Target_position | Read-write | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6081 | Profile_velocity | Read-write | UINTEGER32 | 0.1 rpm |
Configure: set Modes of operation = 5 (disabled state); mandatory: Target_torque, Profile_velocity; optional: Position_window (0=disabled), Position_window_time (0=disabled); Controlword=15 to enable; set Target_position (p.66-67).

#### Velocity Mode (p.67):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 5 N·m) |
| 60FF | Target_velocity | Read-write | INTEGER32 | 0.1 rpm |
Configure: Modes of operation = 3 (disabled state); mandatory: Target_torque; Controlword=15; set Target_velocity (p.67-68).

#### Torque Mode (p.68):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 5 N·m) |
Configure: Modes of operation = 4 (disabled state); Controlword=15; set Target_torque (p.68).

Motor Watchdog: "Set the CAN communication watchdog via index 1 of object 0x6099. After configuration, a value of 20000 represents 1 second... The default value is 0 (disabled), and a non-zero value enables the watchdog." (p.64)

Controlword values used in manual text: 6 / 7 / 15 (step transition to OPERATION_ENABLE, or 15 directly) (p.63); 1 (normal stop -> SWITCH_ON_DISABLED) (p.64); 11 (emergency stop, "Use with Caution—Risk of Voltage Surge") (p.64); modifying Controlword to clear standard errors in FAULT state (exact clearing value not given) (p.64).

## Fault/error codes

### faultSta (function code 0x3022) bit definitions — Section 3.3.7 "Motor fault instructions" (p.27-28), exact wording:
- "Bit 16: Motor current fault: A-phase current sampling overcurrent"
- "Bit 14: Motor stall overload algorithm protection"
- "Bit 9: Position initialization fault"
- "Bit 8: Hardware identification fault"
- "Bit 7: Encoder uncalibrated: Motor encoder not calibrated"
- "Bit 5: Motor current fault: C-phase current sampling overcurrent"
- "Bit 4: Motor current fault: B-phase current sampling overcurrent"
- "bit3: Overvoltage fault: the motor voltage exceeds the protection voltage by 60V"
- "bit2: Undervoltage fault: the motor voltage is lower than the protection voltage of 12V"
- "bit1: Driver chip failure: Motor driver chip failure reported"
- "bit0: Motor overtemperature fault: motor thermistor temperature exceeds 145 degrees"
(Note: no bits 6, 10-13, 15 defined/mentioned in source text; gap is as printed, not an extraction loss as far as can be determined — p.27-29.)

"Function code 0x3024 is driver chip fault code 1. The specific faults are as follows" — table/content referenced but not rendered as extractable text (likely an image/graphic in the PDF) (p.29). Same for "Function code 0x3025 is driver chip fault code 2. The specific faults are as follows" (p.29) — content not extractable as text; see OPEN QUESTIONS.

### Communication type 21 (0x15) Fault feedback frame — fault/warning bits (p.42-43), exact wording:
Byte0~3: fault value (non-0: faulty; 0: Normal). Same bit list as above (Bit 16, 14, 9, 8, 7, 5, 4, bit3, bit2, bit1, bit0 — identical wording to Section 3.3.7, repeated at p.42-43) plus:
Byte4~7: warning Value — "bit0: motor overtemperature warning, the default is 135°C" (p.43). (Only bit0 of the warning word is documented; no other warning bits given in source.)

### MIT protocol Fault Status Response bits (Section 6.8, p.72-73) — exact wording, same numbering as private protocol:
- "Bit 16: Motor current fault – Overcurrent in phase A sampling"
- "Bit 14: Motor stall & overload algorithm protection"
- "Bit 9: Position initialization fault"
- "Bit 8: Hardware identification fault"
- "Bit 7: Encoder uncalibrated – Motor encoder not calibrated"
- "Bit 5: Motor current fault – Overcurrent in phase C sampling"
- "Bit 4: Motor current fault – Overcurrent in phase B sampling"
- "Bit 3: Overvoltage fault – Motor voltage exceeds 60 V protection threshold"
- "Bit 2: Undervoltage fault – Motor voltage below 12 V protection threshold"
- "Bit 1: Driver chip fault – Motor driver chip alarm"
- "Bit 0: Motor over-temperature fault – Thermistor temperature exceeds 145 ℃"

### Communication Type 2 / Type 24 feedback fault bits (bit21~16 of the 29-bit extended ID field), p.39-40 and p.44-45:
- "bit21: uncalibrated"
- "bit20: Uncalibrated" / "bit20: Gridlock overload fault" (both phrases appear attached to bit20 in source — duplication as printed, see OPEN QUESTIONS)
- "bit19: magnetic coding fault"
- "bit18: overtemperature"
- "bit17: Three-phase overcurrent fault" (p.39-40) / "Three-phase current fault" (p.44, slightly different wording, transcribed as printed)
- "bit16: undervoltage fault"
Mode status bits22~23: "0: Reset mode [reset]; 1: Cali mode [calibration]; 2: Motor mode [Run]" (no value 3 defined in source).

### MIT Command 5 fault clear (p.72):
"F_CMD: 0xFF → clearing the current abnormality; Any other value → the error code will be returned in the BYTE1 field of the response message."

## OPEN QUESTIONS

1. **Communication Type 1 (TX) byte layout, p.39**: The printed table places "Byte2: Torque (0~65535) corresponds to (-14Nm~14Nm)" in the "bit23~8" (29-bit ID) column, while the "Byte0~Byte7" data-field column separately lists Byte0~1 (angle), Byte2~3 (velocity), Byte4~5 (Kp), Byte6~7 (Kd) — with no explicit torque byte range given in the data-field column. This conflicts: torque appears to be missing from the data-field byte map as printed, or the "Byte2: Torque" text is a layout artifact that got shifted into the wrong column during PDF text extraction. I could not confidently resolve where Torque is actually encoded in Type 1's 8-byte payload from the text alone. Recommend visually inspecting the original table/image on p.39 rather than relying on this transcription for Type 1's data field.

2. **Communication Type 2/24 feedback "bit21~16" field, p.39-40 & p.44-45**: "bit20: Uncalibrated" appears immediately followed by a second "bit20: Gridlock overload fault" — two different fault descriptions assigned to the same bit number as printed. Cannot determine which is correct, or whether one is bit20 and the other is a different bit (e.g. bit21 duplicated with "uncalibrated" above it). Do not assume either mapping without visual confirmation of the source table.

3. **Communication Type 17 (0x11) request/response column layout, p.41-42**: The Byte2~3 / Byte4~7 description wording is garbled in extraction ("Byte2~3:00 Byte4~7: In data above 00, the low byte is first and the high byte is second (" — sentence appears cut off mid-parenthesis). The reply frame's "bit23~16" column text ("00 indicates... 01 indicates that the master can_ID") also cuts off mid-sentence. Byte-exact meaning of Byte2~3 in this frame is unclear beyond "00" placeholder.

4. **MIT protocol Command 3 TX frame and Response Command 1 RX frame bit-packing (p.71-72, p.70-71)**: Both frames describe 12-bit sub-fields using language like "Byte3[7-4] (high 4 bits) as low 4 bits: Target speed... Byte3[3-0] (low 4 bits) as high 4 bits" — this phrasing (nibble labeled "high 4 bits" being used "as low 4 bits") is internally confusing and does not cleanly specify, from text alone, which nibble of which byte holds which part of the 12-bit value, or the exact bit order/endianness. I transcribed the text verbatim but flag that the actual bit-packing must be confirmed against the source PDF's graphic/table rendering (this is likely a table with merged/rotated cells that didn't extract cleanly) or against a reference implementation/capture, not inferred from this text.

5. **0x2023 `damper` parameter row, p.22**: Printed as "Maximum value 0, Minimum value 20" (i.e., max < min) — this is almost certainly a column-order error in the source table (should probably be Max=20, Min=0, matching the pattern of nearby rows), but I am transcribing exactly as printed per instructions and NOT correcting it. Do not use this row's max/min without independent verification.

6. **0x2024 `add_offset` parameter row, p.22**: Printed as "Maximum value -7, Minimum value 7" (max < min) — same issue as above, transcribed exactly, not corrected.

7. **Relationship between the 0x20xx "Settings" parameter table (p.20-22, read/written via Communication Type 17/18 presumably) and the separate 0x70xx "Read and write a single parameter list" table (p.44-47, explicitly used by Type 17/18 and by MIT Commands 14/15)**: The manual's MIT section explicitly states the Commands 14/15 index table is "same as the private protocol parameter table" (p.75-76), implying one unified index space, but the printed index values differ (0x20xx/0x30xx vs 0x70xx) and I could not find an explicit statement reconciling these two numbering schemes, or confirming whether 0x70xx is the authoritative index space for all three protocols' parameter read/write commands. Needs clarification/cross-check with a capture before implementing.

8. **`run_mode` (index 0X7005) value table, p.44**: Text reads "0: operation mode / 1: position mode (PP) / 2: Velocity mode / 3: Operation mode Current mode / 5: Position mode (CSP)" — value 3's description "Operation mode Current mode" appears to be two labels run together (possibly "3: Current mode" was intended, with "Operation mode" belonging to value 0's second line); no value 4 is defined anywhere in the source. Transcribed exactly; do not assume what value 3 or the missing value 4 actually mean.

9. **Driver chip fault code tables for function codes 0x3024 and 0x3025, p.29**: The manual references detail tables ("The specific faults are as follows") but the actual fault-bit content did not extract as text — it is very likely an embedded image/graphic in the original PDF. I could not transcribe this content at all. Needs direct visual inspection of p.29 of the PDF.

10. **Default/example CAN IDs**: The manual's worked examples consistently use CAN_ID 0x7F (127) as "target motor CAN_ID" and 0xFD (253) as "host id" (e.g., p.42, p.47, p.70-76), and the factory parameter table shows CAN_ID default "Current value (for reference)" = 1 and CAN_MASTER default = 0 (p.21). These two pieces of information are NOT the same value and the manual never explicitly states which is the true factory-default CAN ID on a fresh unit — 0x7F/0xFD may just be convenient example values chosen for the worked examples, distinct from the factory default of 1/0 shown in the parameter table. Do not assume 0x7F is the factory default without checking an actual unit or capture.

11. **Manufacturer name inconsistency**: The manual uses both "Lingfoot" (p.2-4) and "Lingzu" / "Lingzu Times" / "Lingzu Company" (p.3, p.12-13, p.44) to refer to what appears to be the same company (likely two different translations/transliterations of the same Chinese name, 灵足时代 or similar, by different translators within the same document). Transcribed both exactly as printed; not reconciled.

12. **CANopen Controlword value for standard fault clearing, p.64**: "If the motor enters a FAULT state due to protection mechanisms, modifying the Controlword (6040H) can clear standard errors." — no specific numeric value is given for the fault-clearing Controlword write, unlike the other transitions (1, 11, 6/7/15) which do have explicit values. Left blank; do not guess a value.

13. **CANopen position-mode object tables omit Modes_of_operation_display's write path and 6060 vs 6061 relationship** is not explicitly cross-referenced beyond what's listed — not a contradiction, just noting the manual provides 6060 (write) and 6061 (read-only display) as separate objects without further detail on timing/update semantics (p.63-68).

14. Page count: pdfinfo via poppler reports **79 pages**; pdftotext's page-break splitting produced 80 segments but the last segment was empty (a trailing page-break artifact), consistent with 79 actual pages. Cover page is page 1.
