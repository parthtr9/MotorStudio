# RS06 protocol extract
Source: RS06User Manual260713.pdf (84 pages total; content ends p.83, p.84 blank/not rendered)

## Model/firmware identification

- Cover page title: "ROBSTRIDE 06" / "36N.M Quasi-Direct Drive Integrated Motor Module" / "User Manual" (p.1). Publisher: "RobStride Dynamics", https://www.robstride.com/ (p.1).
- Running header on all content pages: "RobStride Motor User Manual" (p.2 onward). Section 0 heading: "RS06 instruction manual" (p.2).
- "This manual has been printed... This manual is published by Beijing Lingfoot Times Technology Co., LTD. (hereinafter referred to as Lingfoot)" (p.2-3).
- Version History table (p.83, Section 7): Version Number "1.0", Description "Initial Release", Date "November 25, 2025".
- Firmware/build identifiers appear as read-only parameters in the host-computer parameter table (p.20):
  - 0X1000 BootCodeVersion, String, Read only, current value "0.1.5"
  - 0X1001 BootBuildDate, String, Read only, "Mar 16 2022"
  - 0X1002 BootBuildTime, String, Read only, "20:22:09"
  - 0X1003 AppCodeVersion, String, Read only, "0.0.0.1", Remark: "Motor program version number"
  - 0X1004 AppGitVersion, String, Read only, "7b844b0fM"
  - 0X1005 AppBuildDate, String, Read only, "Apr 14 2022"
  - 0X1006 AppBuildTime, String, Read only, "20:30:22"
  - 0X1007 AppCodeName, String, Read only, "Lingzu_motor"
  - (These same rows are also shown with slightly different example current-values on p.19's screenshot: AppBuildDate "Sep 12 2024", AppBuildTime "14:02:13", AppGitVersion "0.0.3.6" labeled ppCodeVersion, AppBuildDate "Apr 17 2025", AppBuildTime "18:12:37", AppCodeName "motor" — see OPEN QUESTIONS, these two tables (p.19 screenshot vs p.20 printed table) disagree on example values for the same indices.)
- Section 6.15/6.16/6.17/6.18 (p.79-80) note certain MIT-protocol commands "(requires firmware update to 0.6.0.11)" for Command 12 (Save motor data), Command 13 (Motor actively reports), Command 14 (Read motor parameters), Command 15 (Write motor parameters).
- Host-computer software screenshots show "motor type RS00" (p.15) and firmware filenames "rs00_0.0.3.6.bin" (p.17-18) — these appear to be generic/reused UI screenshots not specific to RS06; see OPEN QUESTIONS.

## CAN bus parameters

- Driver product specifications table (p.12):
  | project | data |
  |---|---|
  | The rated working voltage | 48VDC |
  | The maximum allowable voltage | 60VDC |
  | Rated working phase current | 14.3Apk |
  | Maximum allowable phase current | 57.0Apk |
  | Standby power | ≤18mA |
  | CAN bus bit rate | 1Mbps |
  | Dimensions | Φ62mm |
  | Working environment temperature | -20℃ to 50℃ |
  | The maximum allowable temperature of the control board | 145℃ |
  | encoder resolution | 14bit (absolute turn) |
- Driver interface (p.12): connector pins labeled "VBAT+", "GND", "CAN-L", "CAN-H".
- Recommended driver interface brand/model (p.12): board end model "XT30PB(2+2)-M.G.B" (AMASS/Ams); line end model "XT30(2+2)-F.G.B" (AMASS/Ams).
- Section 4 intro (p.41): "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted as follows:" — i.e. the private/default protocol uses 29-bit extended CAN frame IDs.
- Section 6 intro, MIT protocol (p.75): "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol. The standard frame format is as follows:" — MIT protocol frames use 11-bit standard CAN ID (table columns "11-bit ID").
- Hardware disposition (p.13): "The articulated motor uses the CAN communication mode and has two communication cables. It is connected to the debugger through the can to USB tool... The frame header of the corresponding serial port protocol is 41 54, and the frame tail is 0D 0A." DIP switch settings on CAN-to-USB module: "When DIP switch 1 is in the ON position, the module enters Boot mode and cannot establish a connection with the host computer. When DIP switch 2 is in the ON position, a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer."
- CAN ID related parameters, from host-computer parameter table (p.21):
  - 0X2009 CAN_ID, uint8, Settings, Max 127, Min 0, Current value 1, Remark "本节点id" (this node's id)
  - 0X200a CAN_MASTER, uint8, Settings, Max 127, Min 0, Current value 0, Remark "id of this object"
  - 0X200b CAN_TIMEOUT, uint32, Read/Write, Max 100000, Min 0, Current value 0, Remark "can host id" (remark text as printed; appears to be a mislabeled copy — see "can communication failure protection" text below)
  - Note: a second field also named CAN_TIMEOUT (index 0x7028, p.50) is described: "can The timeout threshold, 20000 is 1s", default value 0, type uint32.
- CAN communication failure protection (Section 3.3.6, p.30, verbatim): "When the value of CAN_TIMEOUT is 0, this function is disabled" / "When the CAN_TIMEOUT value is non-0, when the motor does not receive the can command within a certain period of time, the motor enters the reset mode, and 20000 is 1s"
- Baud rate modification (Communication Type 23, p.47): F_CMD byte values — "01 is 1M", "02 is 500K", "03 is 250K", "04 is 125K".
- Protocol switching / F_CMD values for motor protocol type (Communication Type 25, p.48, and the Protocol Switching frame described in Section 5.10, p.74, and MIT Command 8, p.78) consistently state: "0 is a private protocol (default)", "1 is the Canopen protocol", "2 is the MIT protocol".
- No single explicit "default CAN ID" (e.g. a fixed factory ID like 127) is stated in prose anywhere I found; the parameter table's example "Current value" for CAN_ID is "1" (p.21) but the text does not explicitly call this the factory default — see OPEN QUESTIONS.

## Frame definitions (TX - host to motor)

All frames below are from Section 4 ("Driver protocol and instructions", p.41 onward) unless noted as CANopen (Section 5) or MIT (Section 6). Extended-frame private-protocol frames use the layout: 29-bit ID split into bit28~bit24 (communication type), bit23~8 (data area 2 / addressing), bit7~0 (destination/target CAN ID), plus an 8-byte data field (Byte0~Byte7).

**General extended-frame layout (p.41, verbatim table):**
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | Communication type | data area 2 | Destination address | data area 1 |

**Communication Type 0: Get device ID (p.41)** — request frame:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x0 | bit15~8: identifies host CAN_ID | target motor CAN_ID | 0 |
Text: "Gets the device's ID and 64-bit MCU unique identifier" (p.41).

**Communication Type 1: operation control mode motor control instruction (p.42)**:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x1 | Byte2: Torque (0~65535) corresponds to (-36Nm~36Nm) | target motor CAN_ID | Byte0~1: target Angle [0~65535] corresponds to (-4π~4π); Byte2~3: Target angular velocity [0~65535] corresponds to (-50rad/s~50rad/s); Byte4~5: Kp [0~65535] corresponds to (0.0~5000.0); Byte6~7: Kd [0 to 65535] corresponds to the above data (0.0 to 100.0). After the conversion, the high byte is in front and the low byte is in [text cut off at page bottom] |
Note: the "bit23~8" column text "Byte2: Torque..." appears to actually describe part of the data-field torque encoding rather than the ID bit range — the table's column alignment is ambiguous in the source formatting; transcribed as printed. See OPEN QUESTIONS.
Response frame: "Response motor feedback frame (see communication type 2)" (p.42).

**Communication Type 3: Motor enabled to run (p.43)**:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x3 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | (blank) |
Response: "Response motor feedback frame (see communication type 2)" (p.43).

**Communication Type 4: Motor stops running (p.43)**:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "When the motor is running normally, 0 must be cleared in the data field. Byte[0]=1: The fault is cleared." |
Response: "Response motor feedback frame (see communication type 2)" (p.43).

**Version number read frame (p.43)** (listed under 4.1.5 in body though ToC lists it separately) — request:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | Byte[0]=0x00, Byte[1]=0xC4 |
Response frame (p.43-44):
| data field | 29位ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has): bit21 uncalibrated, bit20 Uncalibrated, bit20 Gridlock overload fault [sic, bit20 listed twice], bit19 magnetic coding fault, bit18 overtemperature, bit17 Three-phase overcurrent fault, bit16 undervoltage fault; bit22~23: Mode status 0: Reset mode [reset] 1: Cali mode [calibration] 2: Motor mode [Run] | target motor CAN_ID | Byte0=0x00, Byte1=0xC4, Byte2=0x56, Byte3~6: Motor version number, Order from high to low |
(Note: this response table duplicates/extends onto p.44 for Byte3-6.)

**Communication type 6: Set motor mechanical zero (p.44)**:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x6 | bit15~8: Identifies the main CAN_ID | and target motor CAN_ID | Byte[0]=1 |
Response: "Response motor feedback frame (see communication type 2)" (p.44).

**Communication type 7: Set motor CAN_ID (p.44)** — "Change the current motor CAN_ID, effective immediately."
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x7 | bit15~8: used to identify main CAN_ID Bit16~23: preset CAN_ID | Target motor CAN_ID | (blank) |
Answer frame: "Answer motor broadcast frame (see communication type 0)" (p.44).

**Communication type 17: Single parameter read (p.45)** — request:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x11 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below. Byte2~3:00 Byte4~7: In data above 00, the low byte is first and the high byte is second ( [text cut off] |
Reply frame (p.45):
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x11 | bit15~8: indicates that the master CAN_ID Bit23~16:00 indicates that the master CAN_ID is successfully read. 01 indicates that the master can_ID [text cut off] | | Byte0~1: Byte2~3:00 Byte4~7:Parameter data. 1 byte of data above Byte4 is preceded by low bytes and followed by high bytes at [text cut off] |

**Communication type 18: Single parameter write (lost in power failure) (p.45-46)** — text: "With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved" (p.45).
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x12 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below. Byte2~3: 00. Byte4~7: Parameter data In the preceding data, the low byte is in the front and the high byte is in the rear |
Response: "Response motor feedback frame (see communication type 2)" (p.46).

**Communication type 21: Fault feedback frame (p.46)** — see FAULT/ERROR CODES section below for the bit table; frame header:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x15 | bit15~8: identifies the main CAN_ID | identifies motor CAN_ID | Byte0~3: fault value; Byte4~7: warning Value (see FAULT/ERROR CODES) |

**Communication type 22: Motor data save frame (p.47)**:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x16 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | 01 02 03 04 05 06 07 08 |
Response: "Response motor feedback frame (see communication type 2)" (p.47).

**Communication type 23: Motor baud rate modification frame (re-power-on effect) (p.47)**:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x17 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD — F_CMD byte is the motor baud rate: 01 is 1M, 02 is 500K, 03 is 250K, 04 is 125K |
Response: "Response motor feedback frame (see communication type 0)" (p.47) [note: text says "type 0" here though analogous frames elsewhere reference type 2 — transcribed as printed, see OPEN QUESTIONS].

**Communication type 24: The motor actively reports frames (p.47-48)**:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x18 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD — F_CMD byte is the motor reporting switch: 00 is to disable active reporting (default), 01 to enable active reporting, default reporting interval is 10ms |
Response frame (p.48) — same layout/content as Communication Type 2 (current angle, angular velocity, torque, temperature, fault bits) with data field header showing 0x18 and target motor CAN_ID in the ID fields; transcribed fields: "Byte0~1: The current Angle [0~65535] Corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-50rad/s~50rad/s); Byte4~5: Current torque [0~65535] corresponds to (-36Nm~36Nm); Byte6~7: Current temperature: Temp(Celius) *10 If the value is higher than 10, the high byte is first and the low byte is last" plus the same fault/mode-status bit list as Type 2 (see FAULT/ERROR CODES).

**Communication type 25: Motor protocol modification frame (re-power-on effect) (p.48)**:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x19 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD — F_CMD byte is the motor protocol type: 0 is a private protocol (default), 1 is the Canopen protocol, 2 is the MIT protocol |
Response: "Response motor feedback frame (see communication type 0)" (p.48).

**Read example (p.51, Section 4.1.16)** — reading loc_kp:
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| (values) | 0x11 | 0x00FD | 0x7F | 1E 70 00 00 00 00 00 00 |
| Description | Type 17 | Host id 0xFD | Target motor CAN_ID 7F | Byte0~1: index, corresponding to loc_kp |
Feedback instruction:
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| (values) | 0x11 | 0x007F | 0xFD | 1E 70 00 00 00 00 F0 41 |
| Description | Type 17 | bit15~8: Target motor CAN_ID 7F | Host id 0xFD | Byte0~1: index, corresponding to loc_kp. Byte4~7: loc_kp value 30, high right byte, (32-bit single precision) hexadecimal IEEE-754 standard floating point number |

**Protocol Switching extended frame (Section 5.10, p.74)**:
| Data Field | 29-bit ID | 8-Byte Data Area |
|---|---|---|
| Size | Bit 28~0 | Byte 0~6 |
| Description | 0xFFF | 01 02 03 04 05 06 F_CMD |
"F_CMD (Byte 6) defines the motor protocol: 0: Private protocol (default); 1: CANopen protocol; 2: MIT protocol" (p.74).
Response Frame:
| Data Field | 11-bit ID | 8-Byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Motor ID | 64-bit MCU unique identifier |

**MIT protocol (Section 6, p.75-80) — standard 11-bit ID frames, data area Byte0~7:**
- 6.1 Response Command 1: Data Feedback (Motor Status) — RX, see RX section.
- 6.2 Response Command 2: MCU Identification — RX, see RX section.
- 6.3 Command 1: Enable Motor Operation (p.76): ID field = "Target motor CAN ID"; Data = `FF FF FF FF FF FF FF FC`. Response: Response Command 1.
- 6.4 Command 2: Stop Motor Operation (p.76): ID field = "Target motor CAN ID"; Data = `FF FF FF FF FF FF FF FD`. Response: Response Command 1.
- 6.5 Command 3: MIT Dynamic Parameters (p.76): Data layout — "Byte 0~1: Target angle [0~65535], (-12.57 rad ~ 12.57 rad); Byte 2 (high 8 bits), Byte 3[7-4] (low 4 bits): Target speed [0~4096], (-50 rad/s ~ 50 rad/s); Byte 3[3-0] (high 4 bits), Byte 4 (low 8 bits): Kp [0~4096], (0~5000); Byte 5 (high 8 bits), Byte 6[7-4] (low 4 bits): Kd [0~4096], (0~100); Byte 6[3-0] (high 4 bits), Byte 7 (low 8 bits): Target torque [0~4096], (-36N·m ~ 36 N·m)". Response: Response Command 1.
- 6.6 Command 4: Set Zero Position (Non-Position Mode) (p.77): ID = "Target motor CAN ID"; Data = `FF FF FF FF FF FF FF FE`. Response: Response Command 1.
- 6.7 Command 5: Clear Errors & Read Fault Status (p.77): Data = `FF FF FF FF FF F_CMD` — "F_CMD:- 0xFF → Clear current fault- Any other value → Returns fault value in Byte 1 of the response". Response (Fault Clear): Response Command 1.
- 6.9 Command 6: Set Operation Mode (p.77): Data = `FF FF FF FF FF FF F_CMD` — "F_CMD: Mode type- 0: MIT mode (default)- 1: Position mode- 2: Velocity mode". Response: Response Command 1 (p.78).
- 6.10 Command 7: Modify Motor CAN ID (p.78): Data = `FF FF FF FF FF FF F_CMD FA`; "F_CMD: Target motor CAN ID". Response: Response Command 2.
- 6.11 Command 8: Change Communication Protocol (Takes Effect After Power Cycle) (p.78): Data = `FF FF FF FF FF FF F_CMD FD`; "F_CMD: Protocol type- 0: Private protocol (default)- 1: CANopen- 2: MIT protocol". Response: Response Command 2.
- 6.12 Command 9: Modify Host CAN ID (p.78): Data = `FF FF FF FF FF FF F_CMD 01`; "F_CMD: Host CAN ID". Response: Response Command 2.
- 6.13 Command 10: Position Mode Control Command (p.78): ID bit10~8 = 1 ("Target motor CAN ID"); "Byte 0~3: Target position (rad, 32-bit float); Byte 4~7: Target speed (rad/s, 32-bit float)". Response: Response Command 1.
- 6.14 Command 11: Velocity Mode Control Command (p.79): ID bit10~8 = 2 ("Target motor CAN ID"); "Byte 0~3: Target speed (rad/s, 32-bit float); Byte 4~7: Current limit in speed/position mode (A, 32-bit float)". Response: Response Command 1.
- 6.15 Command 12: Save motor data (requires firmware update to 0.6.0.11) (p.79): ID bit10~0 = "Target motor CANID"; Data = `FF FF FF FF FF FF FF F8`. Response: Response command 2.
- 6.16 Command 13: Motor actively reports (requires firmware update to 0.6.0.11) (p.79): ID bit10~0 = "Target motor CANID"; Data = `FF FF FF FF FF FF F_CMD F9` — "F_CMD byte indicates the motor protocol type. 0 indicates no reporting (default). 1 is for reporting". Response: Response command 1.
- 6.17 Command 14: Read motor parameters (requires firmware update to 0.6.0.11) (p.79-80): ID bit10~8=3, bit7~0=7F; Data = `05 70 00 00 00 00 00 00`; "Byte0~1: index (See the read/write parameter table below for details.) Byte2~3: 00 Byte4~7: 00 The above data is in low byte first, high byte last."
- 6.18 Command 15: Write motor parameters (requires firmware update to 0.6.0.11) (p.80): ID bit10~8=4, bit7~0=7F; Data = `29 70 00 00 00 00 00 00`; "Byte0~1: index See the read/write parameter table for details (same as the private protocol parameter table). Byte2~3: 00 Byte4~7: 00 Parameter data: The above data is in low byte first, high byte last." Response frame has identical layout/example values (p.80).
- 6.19-6.21 Motion Control Mode / Velocity Mode / Position Mode (CSP) sequencing instructions (p.80-82): e.g. Velocity Mode steps: "1. Configure the motor's operation mode by sending Set Operation Mode Command (Command 6) with Mode = 2 (Velocity Mode). 2. Send the Motor Enable Command (Command 1) to activate the motor. 3. Send the Velocity Mode Control Command (Command 11) to set the maximum current (absolute value) and target speed. 4. To stop, send the Motor Stop Command (Command 2)." (p.81). Position Mode (CSP) steps analogous using Command 6 Mode=1, Command 1, Command 10, Command 2 (p.81-82).

**CANopen protocol (Section 5, p.66-74):** CANopen is described as a higher-level protocol over CAN (ISO 11898); specific CANopen object dictionary indices are documented (see Parameter table section below) rather than raw TX byte-layout tables, since CANopen SDO/PDO framing is standard. Controlword (6040h) values used to drive the state machine (p.68-69): writing 6, 7, or 15 transitions SWITCH_ON_DISABLED → ... → OPERATION_ENABLE (step-by-step), or 15 directly for immediate enablement; writing 1 returns to SWITCH_ON_DISABLED (normal stop, p.69); writing 11 triggers Emergency Stop ("Use with Caution—Risk of Voltage Surge", p.69); Controlword can also be used for Fault Clearance (p.69). Motor Watchdog: "Set the CAN communication watchdog via index 1 of object 0x6099. After configuration, a value of 20000 represents 1 second. If no communication frame is received within the set time, the motor will be disabled. The default value is 0 (disabled), and a non-zero value enables the watchdog." (p.69).

## Frame definitions (RX - motor to host)

**Communication Type 0 reply (Get device ID) (p.41)**:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x0 | target motor CAN_ID | 0XFE | 64-bit MCU unique identifier |

**Communication Type 2: motor feedback data (p.42-43)** — this is the standard feedback/response frame referenced by many other TX frame types ("Response motor feedback frame (see communication type 2)"):
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has): bit21 uncalibrated, bit20 Uncalibrated, bit20 Gridlock overload fault [printed twice in source], bit19 magnetic coding fault, bit18 overtemperature, bit17 Three-phase overcurrent fault, bit16 undervoltage fault; (continued p.43) bit22~23: Mode status 0: Reset mode [reset] 1: Cali mode [calibration] 2: Motor mode [Run] | host CAN_ID | Byte0~1: The current Angle [0~65535] Corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-50rad/s~50rad/s); Byte4~5: Current torque [0~65535] corresponds to (-36Nm~36Nm); Byte6~7: Current temperature: Temp(Celius) *10 If the value is higher than 10, the high byte is first and the low byte is last |

**Version read response frame (p.43-44)** — see TX section above (duplicated here as RX): fault/mode bits identical to Type 2, plus "Byte0=0x00, Byte1=0xC4, Byte2=0x56, Byte3~6: Motor version number, Order from high to low".

**Communication type 17 reply (Single parameter read) (p.45)** — see TX section above (duplicated there since printed as one combined entry).

**Communication type 21 reply (Fault feedback frame) (p.46)** — see FAULT/ERROR CODES section for full bit table.

**Communication type 24 active-report response (p.48)** — same byte layout as Type 2 (angle/velocity/torque/temperature + fault/mode bits).

**MIT Response Command 1: Data Feedback (Motor Status) (p.75)**:
| Data Field | 11-bit ID | 8-byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Host ID | Byte 0: Motor CAN ID<br> Byte 1~2: The current angle [0~65535], corresponds to (-12.57 rad ~ 12.57 rad)<br> Byte 3 (high 8 bits), Byte 4[7-4] (low 4 bits): The current speed [0~4096], corresponds to (-50 rad/s ~ 50 rad/s)<br> Byte 4[3-0] (high 4 bits), Byte 5 (low 8 bits): The current torque [0~4096], corresponds to (-36N·m ~ 36 N·m)<br> Byte 6~7: Winding temperature (in degrees): Temp(Celsius) *10 |
(The "<br>" tags are printed literally in the source PDF text extraction — transcribed as-is.)

**MIT Response Command 2: MCU Identification (p.76)**:
| Data Field | 11-bit ID | 8-byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Motor ID | 64-bit MCU unique identifier |

**MIT Fault Status Response (Section 6.8, p.77)** — see FAULT/ERROR CODES section for bit table; frame header:
| Data Field | 11-bit ID | 8-byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0: Motor CAN ID; Byte 1~4: Fault value (Non-zero: Fault present; 0: Normal) |

## Parameter table (flash config read/write)

Two distinct tables appear in the manual:

### A. CAN protocol "Read and write a single parameter list" (Section 4.1.15, p.49-51)
Used specifically via Communication Type 17 (single param read) / Type 18 (single param write) over CAN, and by MIT Commands 14/15. Columns as printed: index, Description, Type, Number of bytes, [range column, unlabeled header], R/W Read and write permission.

| index | Name (inferred from Description text) | Description | Type | Bytes | Range/Default | R/W |
|---|---|---|---|---|---|---|
| 0X7005 | run_mode | "0: operation mode 1: position mode (PP) 2: Velocity mode 3: Operation mode Current mode 5: Position mode (CSP)" | uint8 | 1 | (none printed) | W/R |
| 0X7006 | iq_ref | "Current mode Iq command" | float | 4 | -57 to 57A | W/R |
| 0X700A | spd_ref | "Rotational Velocity mode Rotational speed command" | float | 4 | -50 to 50rad/s | W/R |
| 0X700B | limit_torque | "torque limit" | float | 4 | 0 to 36Nm | W/R |
| 0X7010 | cur_kp | "Kp" | float | 4 | "The default value is 0.17" | W/R |
| 0X7011 | cur_ki | "Ki" | float | 4 | "The default value is 0.012" | W/R |
| 0X7014 | cur_filt_gain | "filt_gain" | float | 4 | "0 to 1.0, The default value is 0.1" | W/R |
| 0X7016 | loc_ref | "Position Mode Angle instruction" | float | 4 | rad | W/R |
| 0X7017 | limit_spd | "Location mode (CSP) speed limit" | float | 4 | 0 to 50rad/s | W/R |
| 0X7018 | limit_cur | "Velocity position mode Current limitation" | float | 4 | 0 to 57A | W/R |
| 0x7019 | mechPos | "Mechanical Angle of the loading coil" | float | 4 | rad | R |
| 0x701A | iqf | "iq Filter" | float | 4 | -57 to 57A | R |
| 0x701B | mechVel | "Speed of the load" | float | 4 | -50 to 50rad/s | R |
| 0x701C | VBUS | "Bus voltage" | float | 4 | V | R |
| 0x701E | loc_kp | "at" [sic, printed as "at"; likely means "Position loop kp"] | float | 4 | "The default value is 40" | W/R |
| 0x701F | spd_kp | "Indicates the speed kp" | float | 4 | "The default value is 6" | W/R |
| 0x7020 | spd_ki | "ki" | float | 4 | "The default value is 0.02" | W/R |
| 0x7021 | spd_filt_gain | "Speed filter value" | float | 4 | "The default value is 0.1" | W |
| 0x7022 | acc_rad | "velocity mode acceleration" | float | 4 | "The default value is 20rad/s^2" | W |
| 0x7024 | vel_max | "Location mode (PP) speed" | float | 4 | "The default value is 10rad/s" | W |
| 0x7025 | acc_set | "Location mode (PP) acceleration" | float | 4 | "The default value is 10rad/s^2" | W |
| 0x7026 | EPScan_time | "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | "The default value is 1" | W/R |
| 0x7028 | canTimeout | "can The timeout threshold, 20000 is 1s" | uint32 | 4 | "The default value is 0" | W/R |
| 0x7029 | zero_sta | "Indicates the zero flag bit, 0 means 0-2π and 1 means -π-π" | uint8 | 1 | "The default is 0" | W/R |
| 0x702A | damper | "Damping switch" | uint8 | 1 | "The default value is 0" | W/R |
| 0x702B | add_offset | "Zero offset" | float | 4 | "The default is 0" | W/R |
| 0x702C | alveolous_open | "Cogging Compensation Switch" | uint8 | 1 | "The default is 0" | W/R |
| 0x702D | iq_test | "Motor Initialization Calibration Switch" | uint8 | 1 | "The default is 0" | W/R |
| 0x702E | dcc_set | "Deceleration in Motor PP Mode (Normally Disabled)" | float | 4 | "The default is 10rad/s^2" | W/R |

(p.49-51; all rows transcribed verbatim from the printed table.)

### B. Host-computer / debugger software "function code" (功能码) table (Section 3.3.4, p.19-28)
Used by the upper-computer debugger tool (Refresh Para Table / Read Para / Write Para); columns: 功能码 (function code), 名称 (Name), 参数类型 (param type), 属性 (attribute), 最大值 (max), 最小值 (min), 当前值(供参考) (current value, for reference), Remark.

Identification block (Read only), p.19-20:
- 0X0000 Name, String, Read/Write
- 0X0001 BarCode, String, Read/Write
- 0X1000 BootCodeVersion .. 0X1007 AppCodeName (see Model/firmware identification section above)

Setup/config block, p.21-23:
| 功能码 | Name | Type | Attribute | Max | Min | Current | Remark |
|---|---|---|---|---|---|---|---|
| 0X2000 | echoPara1 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2001 | echoPara2 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2002 | echoPara3 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2003 | echoPara4 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2004 | echoFreHz | uint32 | Read/Write | 10000 | 1 | 500 | |
| 0X2005 | MechOffset | float | Settings | 7 | -7 | 4.619583 | "Motor magnetic encoder Angle offset" |
| 0X2006 | chasu_offset | float | Read/Write | 50 | -50 | 4.52 | "Reserved parameter" |
| 0X2007 | status1 | float | Read/Write | 17 | 0 | 17 | "Torque limitation" |
| 0X2008 | I_FW_MAX | float | Read/Write | 33 | 0 | 0 | "Weak magnetic current value, default 0" |
| 0X2009 | CAN_ID | uint8 | Settings | 127 | 0 | 1 | "本节点id" (this node's id) |
| 0X200a | CAN_MASTER | uint8 | Settings | 127 | 0 | 0 | "id of this object" |
| 0X200b | CAN_TIMEOUT | uint32 | Read/Write | 100000 | 0 | 0 | "can host id" |
| 0X200c | status2 | int16 | Read/Write | 1500 | 0 | 0 | "can timeout threshold. The default value is 0" |
| 0X200d | status3 | uint32 | Read/Write | 1000000 | 1000 | 0 | "Reserved parameter" |
| 0X200e | status4 | float | Read/Write | 64 | 1 | 0 | "Reserved parameter" |
| 0X200f | status5 | float | Read/Write | 64 | 1 | 0 | "Reserved parameter" |
| 0X2010 | status6 | uint8 | Read/Write | 1 | 0 | 0 | "Reserved parameter" |
| 0X2011 | cur_filt_gain | float | Read/Write | 1 | 0 | 0.9 | "Current filtering parameter" |
| 0X2012 | cur_kp | float | Read/Write | 200 | 0 | 0.025 | "Current kp" |
| 0X2013 | cur_ki | float | Read/Write | 200 | 0 | 0.0258 | "Current ki" |
| 0X2014 | spd_kp | float | Read/Write | 200 | 0 | 2 | "Velocity kp" |
| 0X2015 | spd_ki | float | Read/Write | 200 | 0 | 0.021 | "Speed ki" |
| 0X2016 | loc_kp | float | Read/Write | 200 | 0 | 30 | "Position kp" |
| 0X2017 | spd_filt_gain | float | Read/Write | 1 | 0 | 0.1 | "Velocity filter parameter" |
| 0X2018 | limit_spd | float | Read/Write | 200 | 0 | 2 | "Location mode speed limit" |
| 0X2019 | limit_cur | float | Read/Write | 23 | 0 | 23 | "Position, Velocity mode current limit" |
| 0X201a | limit_a | float | Read/Write | 100 | 0 | 0 | "Acceleration coefficient of velocity mode" |
| 0X201b | fault1 .. 0X2021 fault7 | uint32 | Read/Write | 30000 | 0 | 0 | "Reserved parameter" (each) |
| 0X2022 | baud | uint8 | Read/Write | 10 | 0 | 0 | "Baud rate flag bit" |
| 0X2023 | zero_sta | uint8 | Read/Write | (blank) | (blank) | 0 | "Baud rate flag bit" [sic, printed remark appears duplicated from 0X2022] |
| 0X2024 | position_offset | float | Read/Write | (blank) | (blank) | 0 | "Position offset coefficient" |
| 0x2025 | protocol_1 | uint8 | Read/Write | (blank) | (blank) | 0 | "Protocol flag" |
| 0x2026 | damper | uint8 | Read/Write | 0 | 20 | 0 | "Damping switch" |
| 0X2027 | add_offset | float | 读/写 (Read/Write) | 7 | -7 | 0 | "Position offset" |

Diagnostics/telemetry block (Read only unless noted), p.23-28:
| 功能码 | Name | Type | Attribute | Current (example) | Remark |
|---|---|---|---|---|---|
| 0X3000 | timeUse0 | uint16 | Read only | 5 | |
| 0X3001 | timeUse1 | uint16 | Read only | 0 | |
| 0X3002 | timeUse2 | uint16 | Read only | 10 | |
| 0X3003 | timeUse3 | uint16 | Read only | 0 | |
| 0X3004 | encoderRaw | int16 | Read only | 11396 | "Magnetic encoder sampling value" |
| 0X3005 | mcuTemp | int16 | Read only | 337 | "mcu internal temperature, *10" |
| 0X3006 | motorTemp | int16 | Read only | 333 | "Motor ntc temperature, *10" |
| 0X3007 | encoder2raw | int16 | Read only | 24195 | "Sampling value of differential magnetic encoder" |
| 0X3008 | adc1Offset | int32 | Read only | 2084 | "adc sampling channel 1 Zero current bias" |
| 0X3009 | adc2Offset | int32 | Read only | 2084 | "adc sampling channel 2 Zero current bias" |
| 0X300a | adc1Raw | uint16 | Read only | 1232 | "adc sampling value 1" |
| 0X300b | adc2Raw | uint16 | Read only | 1212 | "adc sampling value 2" |
| 0X300c | VBUS | float | Read only | 36 | "Bus voltage V" |
| 0X300d | cmdId | float | Read only | 0 | "id ring instruction, A" |
| 0X300e | cmdIq | float | Read only | 0 | "iq ring command, A" |
| 0X300f | cmdlocref | float | Read only | 0 | "Position loop command, rad" |
| 0X3010 | cmdspdref | float | Read only | 0 | "Speed loop command, rad/s" |
| 0X3011 | cmdTorque | float | Read only | 0 | "Torque instruction, nm" |
| 0X3012 | cmdPos | float | Read only | 0 | "mit Protocol Angle instruction" |
| 0X3013 | cmdVel | float | Read only | 0 | "mit Protocol Speed instruction" |
| 0X3014 | rotation | int16 | Read only | 1 | "Number of turns" |
| 0X3015 | modPos | float | Read only | 4.363409 | "Motor uncounted coil mechanical Angle, rad" |
| 0X3016 | mechPos | float | Read only | 0.777679 | "Load end loop mechanical Angle, rad" |
| 0X3017 | mechVel | float | Read only | 0.036618 | "Load speed: rad/s" |
| 0X3018 | elecPos | float | Read only | 4.714761 | "Electrical Angle" |
| 0X3019 | ia | float | Read only | 0 | "U-wire current, A" |
| 0X301a | ib | float | Read only | 0 | "V-wire current, A" |
| 0X301b | ic | float | Read only | 0 | "W-wire current, A" |
| 0X301c | timeout | uint32 | Read only | 31600 | "Timeout counter value" |
| 0X301d | phaseOrder | uint8 | Read only | 0 | "Directional marking" |
| 0X301e | iqf | float | Read only | 0 | "iq filter value，A" |
| 0X301f | boardTemp | int16 | Read only | 359 | "Plate temperature，*10" |
| 0X3020 | iq | float | Read only | 0 | "iq Original value，A" |
| 0X3021 | id | float | Read only | 0 | "id Original value，A" |
| 0X3022 | faultSta | uint32 | Read only | 0 | "Fault status value" (see Fault/Error section) |
| 0X3023 | warnSta | uint32 | Read only | 0 | "Warning status value" |
| 0X3024 | drv_fault | uint16 | Read only | 0 | "The driver chip fault value is 1" (see Fault/Error section) |
| 0X3025 | drv_temp | int16 | Read only | 48 | "The driver chip fault value is 2" (see Fault/Error section) |
| 0X3026 | Uq | float | Read only | 0 | "Q-axis voltage" |
| 0X3027 | Ud | float | Read only | 0 | "D-axis voltage" |
| 0X3028 | as_angle | float | Read only | 0 | "Initial Angle of magnetic coding" |
| 0X3029 | cs_angle | float | Read only | 0 | "Initial Angle of differential magnetic programming" |
| 0X302a | chasu_angle | float | Read only | 0 | "Differential Angle" |
| 0X302b | v_bus | float | Read only | 0 | "Motor voltage" |
| 0X302c | ElecOffset | float | Read only | 0 | "Electrical Angle bias" |
| 0X302d | torque_fdb | float | Read only | 0 | "Torque feedback value, nm" |
| 0X302e | rated_i | float | Read only | 8 | "Motor protection parameters" |
| 0X302f | MechPos_init | float | Read only | 27 | "Motor retention parameters" |
| 0X3030 | instep | float | Read only | 0 | "Motor retention parameters" |
| 0X3031 | status | uint8 | Read only | 0 | "Retain parameters" |
| 0X3032 | cmdlocref | float | Read only | 0 | "Position expectations" |
| 0X3033 | vel_max | float | Read only | 0 | "Expected motor speed" |
| 0X3034 | fault1 .. 0X303b fault8 | float/uint32 | Read only | 0 | "Log failure" (each) |
| 0X303c | mcOverTemp | int16 | Read only | 0 | "Overtemperature threshold" |
| 0X303d | Kt_Nm/Amp | float | Read only | 0 | "Moment coefficient" |
| 0X303e | Tqcali_Type | uint8 | Read only | 0 | "Motor type" |
| 0X303f | theta_mech_1 | float | Read only | 0 | "Type 2 Low speed Angle" |
| 0X3040 | adcOffset_1 | int32 | Read only | 2049 | "ADC sampling channel 1 zero current bias" |
| 0X3041 | adcOffset_2 | int32 | Read only | 2040 | "ADC sampling channel 2 zero current bias" |
| 0X3042 | position | float | Read only | -4.131011 | "Motor position determination parameters" |
| 0X3043 | chasu_angle_init | float | Read only | -0.645806 | "Motor position determination parameters" |
| 0X3044 | chasu_angle_out | float | Read only | -5.812255 | "Motor position determination protection parameters" |
| 0X3045 | motormechinit | float | Read only | -5.773137 | "Motor position determination parameters" |
| 0X3046 | mech_angle_init2 | float | Read only | -0.64146 | "Motor position determination parameters" |
| 0X3047 | mech_angle_rotat | int16 | Read only | 0 | "Motor position determination protection parameters" |
| 0X3048 | pos_cnt1 | uint16 | Read only | 0 | "System parameters" |

(p.20-28, transcribed verbatim including apparent typos/remark duplications noted inline.)

### C. CANopen object dictionary indices (Section 5, p.69-74)
- Status Feedback Parameters (p.69-70): 603F Error_code (Read-only, UINTEGER16, unit "/"); 6041 Statusword (Read-only, UINTEGER16, "/"); 6061 Modes_of_operation_display (Read-only, INTEGER8, "/"); 6062 Position_demand_value (Read-only, INTEGER32, "Pulses (1 rev = 16,384 pulses)"); 6064 Position_actual_value (Read-only, INTEGER32, "Pulses (1 rev = 16,384 pulses)"); 606B Velocity_demand_value (Read-only, INTEGER32, "0.1 rpm"); 606C Velocity_actual_value (Read-only, INTEGER32, "0.1 rpm"); 6077 Torque_actual_value (Read-only, INTEGER16, "0.1% load ratio (1000 = 11 N·m)"); 6078 Current_actual_value (Read-only, INTEGER16, "mA"); 6079 DC_link_circuit_voltage (Read-only, INTEGER32, "mV").
- Homing Mode (p.70): 6040 Controlword (Read-write, UINTEGER16, "/"); 6060 Modes of operation (Read-write, INTEGER8, "/"). "Set Modes of operation to 6 while the motor is in the disabled state (SWITCH_ON_DISABLED). The motor will then define the current position as the zero point." / "To hold the zero position, modify the Controlword to 15..."
- Position Mode PP (p.71): 6040 Controlword; 6060 Modes of operation; 6067 Position_window (Read-write, UINTEGER32, "Pulses (1 rev = 16,384 pulses)"); 6068 Position_window_time (Read-write, UINTEGER16, "ms"); 6071 Target_torque (Read-write, INTEGER16, "0.1% load ratio (1000 = 11N·m)"); 607A Target_position (Read-write, INTEGER32, "Pulses (1 rev = 16,384 pulses)"); 6081 Profile_velocity (Read-write, UINTEGER32, "0.1 rpm"); 6083 Profile_acceleration (Read-write, UINTEGER32, "0.1 rpm/s"). "set Modes of operation to 1."
- Position Mode CSP (p.72): same indices as PP minus 6083 (Profile_acceleration not listed for CSP), "set Modes of operation to 5."
- Velocity Mode (p.73): 6040 Controlword; 6060 Modes of operation; 6071 Target_torque; 60FF Target_velocity (Read-write, INTEGER32, "0.1 rpm"). "set Modes of operation to 3."
- Torque Mode (p.73-74): 6040 Controlword; 6060 Modes of operation; 6071 Target_torque. "set Modes of operation to 4."

## Fault/error codes

### Function code 0x3022 (faultSta) bit definitions, Section 3.3.7 (p.30, verbatim):
"Function code 0x3022 indicates the fault code, where
bit14: motor blocking overload algorithm protection
bit7: Encoder uncalibrated: Motor uncalibrated encoder
bit3: Overvoltage fault: the motor voltage exceeds the protection voltage by 60V
bit2: Undervoltage fault: the motor voltage is lower than the protection voltage of 12V
bit1: Driver chip failure: Motor driver chip failure reported
bit0: Motor overtemperature fault: motor thermistor temperature exceeds 145 degrees"

### Function code 0x3024 (drv_fault) — "driver chip fault code 1" — Table 11, Fault Status Register 1 Field Descriptions (p.31):
| Bit | Field | Type | Default | Description |
|---|---|---|---|---|
| 10 | FAULT | R | 0b | Logic OR of FAULT status registers. Mirrors nFAULT pin. |
| 9 | VDS_OCP | R | 0b | Indicates VDS monitor overcurrent fault condition |
| 8 | GDF | R | 0b | Indicates gate drive fault condition |
| 7 | UVLO | R | 0b | Indicates undervoltage lockout fault condition |
| 6 | OTSD | R | 0b | Indicates overtemperature shutdown |
| 5 | VDS_HA | R | 0b | Indicates VDS overcurrent fault on the A high-side MOSFET |
| 4 | VDS_LA | R | 0b | Indicates VDS overcurrent fault on the A low-side MOSFET |
| 3 | VDS_HB | R | 0b | Indicates VDS overcurrent fault on the B high-side MOSFET |
| 2 | VDS_LB | R | 0b | Indicates VDS overcurrent fault on the B low-side MOSFET |
| 1 | VDS_HC | R | 0b | Indicates VDS overcurrent fault on the C high-side MOSFET |
| 0 | VDS_LC | R | 0b | Indicates VDS overcurrent fault on the C low-side MOSFET |

### Function code 0x3025 (drv_temp) — "driver chip fault code 2" — Table 12, Fault Status Register 2 Field Descriptions (p.31):
| Bit | Field | Type | Default | Description |
|---|---|---|---|---|
| 10 | SA_OC | R | 0b | Indicates overcurrent on phase A sense amplifier (DRV8353xS) |
| 9 | SB_OC | R | 0b | Indicates overcurrent on phase B sense amplifier (DRV8353xS) |
| 8 | SC_OC | R | 0b | Indicates overcurrent on phase C sense amplifier (DRV8353xS) |
| 7 | OTW | R | 0b | Indicates overtemperature warning |
| 6 | GDUV | R | 0b | Indicates VCP charge pump and/or VGLS undervoltage fault condition |
| 5 | VGS_HA | R | 0b | Indicates gate drive fault on the A high-side MOSFET |
| 4 | VGS_LA | R | 0b | Indicates gate drive fault on the A low-side MOSFET |
| 3 | VGS_HB | R | 0b | Indicates gate drive fault on the B high-side MOSFET |
| 2 | VGS_LB | R | 0b | Indicates gate drive fault on the B low-side MOSFET |
| 1 | VGS_HC | R | 0b | Indicates gate drive fault on the C high-side MOSFET |
| 0 | VGS_LC | R | 0b | Indicates gate drive fault on the C low-side MOSFET |
(This table matches a TI DRV8353-family gate driver register layout — it is presented as the motor's internal driver-chip fault register, not a CAN frame field.)

### Communication Type 2 / Type 24 feedback-frame fault bits (p.42-43, verbatim, bit15~8 / bit21~16 sub-field of the 29-bit ID):
"bit21~16: fault information (0 none 1 has)
bit21: uncalibrated
bit20: Uncalibrated
bit20: Gridlock overload fault [printed twice for bit20 — see OPEN QUESTIONS]
bit19: magnetic coding fault
bit18: overtemperature
bit17: Three-phase overcurrent fault
bit16: undervoltage fault
bit22~23: Mode status 0: Reset mode [reset] 1: Cali mode [calibration] 2: Motor mode [Run]"

### Communication Type 21: Fault feedback frame bit table (p.46, verbatim):
"Byte0~3: fault value (non-0: faulty; 0: Normal)
Bit 16: A-phase current sampling overcurrent
Bit 14: Motor stall overload algorithm protection
Bit 9: Position initialization fault
Bit 8: Hardware identification fault
Bit 7: Encoder uncalibrated: Motor encoder not calibrated
Bit 5: C-phase current sampling overcurrent
Bit 4: B-phase current sampling overcurrent
bit3: overvoltage fault
bit2: undervoltage fault
bit1: driver chip fault
bit0: motor overtemperature fault, Default 135°C
Byte4~7: warning Value
bit0: motor overtemperature warning, the default is 135°c"

### MIT protocol Fault Status Response bits (Section 6.8, p.77, verbatim):
"Byte 0: Motor CAN ID
Byte 1~4: Fault value (Non-zero: Fault present; 0: Normal)
Bit 14: Stall/I²t overload fault<Bit 7: Encoder not calibrated<br>Bit 3: Overvoltage fault
Bit 2: Undervoltage fault
Bit 1: Driver IC fault
Bit 0: Motor overtemperature fault (Default threshold: 145°C)"
(The "<Bit 7... <br>" artifacts are printed literally in the extracted text — transcribed as-is, apparent OCR/markup leakage in the source document.)

### CAN frame-send example decode (p.29-30, host-computer serial wrapper, not raw CAN):
Example bytes sent over the USB-serial wrapper: `41 54 90 07 e8 0c 08 05 70 00 00 01 00 00 00 0d 0a`
Decoded (p.29): "frame header" = `41 54`; "Number of data bits" extended-frame field = `90 07 e8 0c`; literal "00" shown above the "extended frame" column header in source; "8" = number of data bytes; data frame = `05 70 00 00 01 00 00`; frame tail = `0d 0a`.
Text: "The translation of extended frame canid into real canid requires the following transformations: 90 07 e8 0c converts to binary as 1001 0000 0000 0111 1110 1000 0000 1100, remove the 100 on the right and it becomes 1 0010 0000 0000 1111 1101 0000 0001, convert it to hexadecimal, It is 12 00 FD 01. According to the communication protocol, the meaning is as follows:
12 in hexadecimal = Communication type 18 (in decimal base); 0 = No meaning; FD = host id; 1 = motor canid" (p.29-30).

## OPEN QUESTIONS (ambiguous / not confidently extractable)

1. **Default factory CAN ID**: No sentence explicitly states a fixed default/factory CAN_ID value. The host-computer parameter table shows CAN_ID (0X2009) "Current value" as "1" (p.21), and a screenshot shows "CAN:127 id:6b02" (p.15, example UI state, not necessarily default). Do not assume 127 or 1 is the factory default without confirming against the actual device or a capture.

2. **Communication Type 1 table column alignment (p.42)**: The printed table's "bit23~8" column shows "Byte2: Torque (0~65535) corresponds to (-36Nm~36Nm)" which reads like a data-byte description, not an ID-bit-range description, and the text is truncated mid-sentence at the bottom of the page ("the high byte is in front and the low byte is in"). The exact bit-level packing of Kp/Kd/torque/velocity/angle within the 8-byte MIT-mode-equivalent data field for the *private protocol* Type 1 frame could not be confidently reconstructed byte-by-byte beyond what is printed. Cross-check against a captured frame before implementing.

3. **bit20 listed twice** in the Type 2 / Type 24 feedback frame fault bit list: "bit20: Uncalibrated" and "bit20: Gridlock overload fault" are both printed under bit20 (p.42-43, and again p.48). This is either a documentation error or bit20 genuinely carries two OR'd meanings — not clear from source. Also note bit21 is called "uncalibrated" while bit7 (in the Type 21 frame, p.46) is separately called "Encoder uncalibrated" — the relationship between these bit numbers across frame types (Type 2/24's bit16-23 fault field vs. Type 21's bit0-16 fault field) is not explained; they appear to be two different fault-bit encodings and should not be assumed equivalent.

4. **Overtemperature fault threshold discrepancy**: Section 3.3.7 (p.30) and Communication Type 21 (p.46) state the motor overtemperature fault default is "135°C" / "135°c", while the MIT protocol Fault Status Response (p.77) states "Default threshold: 145°C", and the Driver product specification table (p.12) lists "The maximum allowable temperature of the control board" as "145℃". These are inconsistent across sections of the same document — do not pick one without clarifying with RobStride or the actual device behavior.

5. **CAN_TIMEOUT remark "can host id" (p.21, index 0X200b)**: The Remark column text is "can host id", which does not match the parameter's evident purpose (timeout) as described in Section 3.3.6 (p.30) and in the separate 0x7028 canTimeout entry (p.50, "can The timeout threshold, 20000 is 1s"). Likely a copy-paste/typo in the source table; flagged rather than corrected.

6. **Communication Type 23 response frame reference inconsistency**: Most TX frame definitions reference "Response motor feedback frame (see communication type 2)" for their response, but Type 23 (Motor baud rate modification, p.47) states "Response frame: Response motor feedback frame (see communication type 0)" — referencing Type 0 instead of Type 2, unlike the structurally similar Type 22 and Type 25 frames which both say "see communication type 2" and "see communication type 0" respectively (Type 25 also says type 0). Not resolved from text alone whether this is intentional or a typo.

7. **"motor type RS00" and "rs00_x.x.x.x.bin" in UI screenshots (p.15, 17, 18)**: The host-computer software screenshots embedded in this RS06 manual show device-detection text "motor type RS00" and firmware filenames referencing "rs00". This strongly suggests the screenshots are generic/reused across the RS00–RS06 manual family and are not RS06-specific evidence. No RS06-specific screenshot of a successful detection was found in the document. Do not treat "RS00" as this motor's actual identifier.

8. **Table header/example value mismatch, identification block**: The printed function-code table on p.20 (Section 3.3.4 body text) shows example current-values for 0X1001/0X1002/0X1003/0X1005/0X1006/0X1007 (e.g., "Mar 16 2022", "0.0.0.1") that differ from the example values visible in the screenshot on p.19 for the same indices (e.g., "Sep 12 2024", "0.0.3.6", "Apr 17 2025", "motor"). Both are presented as illustrative "current value (供参考)" examples, not specified defaults — neither should be treated as the RS06's actual shipped firmware version.

9. **Byte-level packing of index/parameter-data fields for Types 17/18/Read example**: Several table cells are truncated at page boundaries (e.g., p.45 "the low byte is first and the high byte is second (" and "indicates that the master can_ID" cut off), so the exact wording of how multi-byte index/parameter fields are packed is incomplete as printed. The worked p.51 "Read example" (loc_kp, index bytes `70 1E`→ printed as `1E 70`, i.e., little-endian 0x701E) is the clearest unambiguous example and should be preferred as the authoritative byte-order reference over the truncated prose.

10. **CANopen 6067/6068 "if not set" / "0 = disabled" wording difference between PP (p.71) and CSP (p.72)**: PP mode describes Position_window/Position_window_time as "(if not set, window check is disabled)" while CSP describes the same indices as "(0 = disabled)". Both likely mean the same behavior (write 0 to disable), but the manual phrases them differently; not fully clarified whether "not set" vs "0" behave identically.

11. No page explicitly states a termination-resistor value for the motor's own CAN bus node (only the separate CAN-to-USB *debug adapter's* DIP-switch-controlled 120Ω resistor is mentioned, p.13) — whether the RS06 motor itself has an internal/switchable terminating resistor is not stated anywhere in this manual.

## RESOLUTION (human decision, 2026-10-01)

Item 4 (overtemperature fault threshold: 135°C vs 145°C conflicting across p.30/p.46 vs p.77/p.12) is resolved for this harness: **use 135°C**, the lower/more conservative of the two printed values, as the enforced thermal limit. This is a deliberate engineering choice to protect equipment given an unresolved manual conflict, not a claim that the manual actually says 135°C everywhere -- both printed values and their page citations above remain as written. When `profiles/robstride/rs06_<fw>.yaml` is drafted, its thermal-limit field's `manual_ref` must cite this resolution line (not just p.30/p.46) so the discrepancy and the reason 145°C wasn't used stay visible to anyone auditing the profile later.
