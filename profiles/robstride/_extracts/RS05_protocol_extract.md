# RS05 protocol extract
Source: RS05User Manual260713.pdf (84 pages total, confirmed via Version History table on p.84 and TOC on p.5-7)

All values below are transcribed verbatim from the PDF text/tables as rendered. Where the source table's column/row alignment was ambiguous when read as an image/text (merged cells, wrapped text, garbled OCR-like artifacts), this is called out explicitly next to the quote rather than guessed.

---

## Model/firmware identification

- Cover page product name (p.1): **"ROBSTRIDE 05"**, subtitle **"6N.M Quasi-Direct Drive Integrated Motor Module"**, document title **"User Manual"**. Publisher link: "RobStride Dynamics https://www.robstride.com/"
- First content page heading (p.1, body page numbered "1"): **"RS05 instruction manual"**
- Manual publisher (p.2-3): "This manual has been published by Beijing Lingfoot Times Technology Co., LTD. (hereinafter referred to as Lingfoot)"
- Manual "Version History" (p.84), Table: Version Number **"1.0"** | Description **"Initial Release"** | Date **"November 25, 2025"**. (This is the *document's* version, not confirmed to be the motor firmware version.)
- Driver/control-board model is not given a distinct part number anywhere in the reviewed text; section 2 ("Driver Product Information", p.12) only gives specs, not a model number.
- Connector hardware (p.12, table "2.3 Recommended driver interface brand and model"): board end model **"XT30PB(2+2)-M.G.B"** / brand **"AMASS (Ams)"**; line end model **"XT30(2+2)-F.G.B"** / brand **"AMASS (Ams)"**.
- No standalone "firmware version" field for the RS05 motor itself was found. The closest candidates are from the flash parameter table screenshot (p.19-20, "Current value (for reference)" column — labelled as reference/example data, not asserted as the actual shipped RS05 firmware):
  - `0X1000 BootCodeVersion` = "0.1.5"
  - `0X1001 BootBuildDate` = "Mar 16 2022"
  - `0X1002 BootBuildTime` = "20:22:09"
  - `0X1003 AppCodeVersion` = "0.0.0.1" (Remark: "Motor program version number")
  - `0X1004 AppGitVersion` = "7b844b0fM"
  - `0X1005 AppBuildDate` = "Apr 14 2022"
  - `0X1006 AppBuildTime` = "20:30:22"
  - `0X1007 AppCodeName` = "Lingzu_motor"
  - See OPEN QUESTIONS — these are flagged as example/reference values, not confirmed RS05-specific.
- Certain protocol commands (MIT protocol Commands 12-15, p.81-82) are explicitly marked **"(requires firmware update to 0.5.0.13)"**.
- Rated torque is stated inconsistently across the document: cover page says "6N.M" (p.1), but §1.2 "Rated load (CW): 1.6 N.m(Heat sink size: 70mm × 70mm)" (p.8) and §1.3 "Rated load: 1.6 N.m(Tested at 100 rpm)" / "Peak load: 5.5 N.m" (p.9). See OPEN QUESTIONS.

---

## CAN bus parameters

- CAN bus bit rate (driver spec table, p.12): **"CAN bus bit rate | 1Mbps"**
- Private protocol frame format statement (p.43): "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted as follows:"
- MIT protocol frame format statement (p.77): "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol. The standard frame format is as follows:"
  - i.e. **Private protocol and CANopen protocol-switch frame use 29-bit extended CAN IDs; the MIT protocol uses 11-bit standard CAN IDs.**

### General 29-bit extended ID layout (private protocol), p.43
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | Communication type | data area 2 | Destination address | data area 1 |

Note: individual per-frame tables later in the document (4.1.x, p.43-50) do **not** consistently match this general column assignment — some put "target motor CAN_ID" under the bit23~8 column, others under bit7~0. This is called out verbatim per-frame below; it was **not** normalized or guessed.

### General 11-bit standard ID layout (MIT protocol), p.77
| Data Field | 11-bit ID | | 8-byte Data Area |
|---|---|---|---|
| Size | Bit 10~8 | Bit 7~0 | Byte 0~7 |
| Description | Mode type | ID | |

### CAN ID range / defaults
- From flash parameter table (p.21): `0X200a CAN_ID` — uint8, attribute "Settings", Max 127, Min 0, Current value 1, Remark "id of this object"
- From flash parameter table (p.21): `0X200b CAN_MASTER` — uint8, attribute "Settings", Max 127, Min 0, Current value 0, Remark "can host id"
- From single-parameter R/W list, no separate CAN_ID index was found (only CAN_ID appears in the flash config table above, index 0X200a).
- Screenshot on p.15 ("Motor connection settings") shows example "CAN: 127  id: 6b02" and "motor type RS00" — this screenshot example is for a *different* model (RS00), reused in this RS05 manual. See OPEN QUESTIONS.

### CAN-to-USB adapter notes (p.13)
- "The frame header of the corresponding serial port protocol is 41 54, and the frame tail is 0D 0A." (for the recommended official USB-CAN module)
- DIP switch 1 ON → "Boot mode" (cannot connect to host)
- DIP switch 2 ON → "a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer"

### Adapter frame decode example (p.31)
Raw bytes sent to adapter: `41 54 90 07 e8 0c 08 05 70 00 00 01 00 00 00 0d 0a`

| 41 54 | 90 07 e8 0c | 8 | 05 70 00 00 01 00 00 00 | 0d 0a |
|---|---|---|---|---|
| frame header | Number of data bits | extended frame | data frame | frame tail |

Decode procedure as given in the text (p.31):
"90 07 e8 0c converts to binary as 1001 0000 0000 0111 1110 1000 0000 1100, remove the 100 on the right and it becomes 1 0010 0000 0000 1111 1101 0000 0001, convert it to hexadecimal, It is 12 00 FD 01. According to the communication protocol, the meaning is as follows:"

| 12 in hexadecimal | 0 | FD | 1 |
|---|---|---|---|
| Communication type 18 (in decimal base) | No meaning | host id | motor canid |

(This shows how the adapter's raw 32-bit field is bit-shifted by 3 bits — "remove the 100 on the right" — to recover the 29-bit CAN arbitration ID; not an independent protocol fact about the motor itself.)

### Baud rate change options — Communication type 23 (0x17), p.49
F_CMD byte values: `01 is 1M`, `02 is 500K`, `03 is 250K`, `04 is 125K`.

---

## Frame definitions (TX - host to motor)

All tables below are quoted exactly as printed, per communication type, with page citations. "Comm type" numbers are given both in decimal (section heading) and hex (table row), as printed.

### Communication type 0 (0x0): Get device ID — p.43
"Gets the device's ID and 64-bit MCU unique identifier"
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x0 | bit15~8: identifies host CAN_ID | target motor CAN_ID | 0 |

Reply frame:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x0 | target motor CAN_ID | 0XFE | 64-bit MCU unique identifier |

### Communication Type 1 (0x1): operation control mode motor control instruction — p.44
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x1 | Byte2: Torque (0~65535) corresponds to (-5.5Nm~5.5Nm) | target motor CAN_ID | Byte0~1: target Angle [0~65535] corresponds to (-4π~4π); Byte2~3: Target angular velocity [0~65535] corresponds to (-50rad/s~50rad/s); Byte4~5: Kp [0~65535] corresponds to (0.0~500.0); Byte6~7: Kd [0 to 65535] corresponds to the above data (0.0 to 5.0). After the conversion, the high byte is in front and the low byte is in |

**Table formatting note:** The source text places "Byte2: Torque (0~65535) corresponds to (-5.5Nm~5.5Nm)" under the bit23~8 column, which conflicts with the 8-byte data field actually holding Byte2-3 as angular velocity per the last column. This looks like a layout/OCR artifact in the original PDF rather than a real second torque field — not resolved, quoted as printed. The trailing sentence "the high byte is in front and the low byte is in" is cut off in the source.

Response frame: "Response motor feedback frame (see communication type 2)"

### Communication Type 3 (0x3): Motor enabled to run — p.45
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x3 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | (blank) |

Response frame: "Response motor feedback frame (see communication type 2)"

### Communication Type 4 (0x4): Motor stops running — p.45
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "When the motor is running normally, 0 must be cleared in the data field.   Byte[0]=1: The fault is cleared." |

Response frame: "Response motor feedback frame (see communication type 2)"

### "Version number read frame" (unnumbered subsection, p.45-46)
Request:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | Byte[0]=0x00, Byte[1]=0xC4 |

Response frame (continues onto p.46):
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has); bit21: uncalibrated; bit20: Uncalibrated; bit20: Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status — 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run] | target motor CAN_ID | Byte0=0x00, Byte1=0xC4, Byte2=0x56, Byte3~6: Motor version number, Order from high to low |

This "version read" request reuses comm-type nibble 0x4 (same type code as "Motor stops running") but is distinguished only by the data payload marker Byte[0]=0x00/Byte[1]=0xC4. Response reuses comm-type 0x2 ("motor feedback data") format. See OPEN QUESTIONS.

### Communication type 6 (0x6): Set motor mechanical zero — p.46
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x6 | bit15~8: Identifies the main CAN_ID | and target motor CAN_ID | Byte[0]=1 |

Response frame: "Response motor feedback frame (see communication type 2)"

### "Communication type 7: Set motor CAN_ID" (unnumbered subsection, p.46)
"Change the current motor CAN_ID, effective immediately."
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x7 | bit15~8: used to identify main CAN_ID; Bit16~23: preset CAN_ID | Target motor CAN_ID | (blank) |

Answer frame: "Answer motor broadcast frame (see communication type 0)"

### Communication type 17 (0x11): Single parameter read — p.47
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x11 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | "Byte0~1: index. For details, see the readability parameter table below; Byte2~3:00; Byte4~7: In data above 00, the low byte is first and the high byte is second (" |

(The sentence ends mid-word with an open parenthesis in the source — truncated, not completed anywhere else in the text.)

Reply frame:
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x11 | bit15~8: indicates that the master CAN_ID; Bit23~16:00 indicates that the master CAN_ID is successfully read. 01 indicates that the master can_ID | Byte0~1: Byte2~3:00 | Byte4~7: Parameter data. 1 byte of data above Byte4 is preceded by low bytes and followed by high bytes at |

(Also truncated at "followed by high bytes at" in the source. Column assignment here likewise does not cleanly match the earlier general layout.)

### Communication type 18 (0x12): Single parameter write (lost on power failure) — p.47-48
"With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved"
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x12 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below; Byte2~3: 00; Byte4~7: Parameter data — "In the preceding data, the low byte is in the front and the high byte is in the rear" |

Response frame: "Response motor feedback frame (see communication type 2)"

### Communication type 21 (0x15): Fault feedback frame — p.48
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x15 | bit15~8: identifies motor CAN_ID | identifies the main CAN_ID | see fault bit list below |

Byte0~3 fault value / Byte4~7 warning value bit fields — see "Fault/error codes" section below (Comm type 21 fault bits).

### Communication type 22 (0x16): Motor data save frame — p.49
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x16 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | 01 02 03 04 05 06 07 08 |

Response frame: "Response motor feedback frame (see communication type 2)"

### Communication type 23 (0x17): Motor baud rate modification frame (re-power-on effect) — p.49
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x17 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "01 02 03 04 05 06 F_CMD; Among them, the F_CMD byte is the motor baud rate Among them, 01 is 1M, 02 is 500K, 03 is 250K, 04 is 125K" |

Response frame: **"Response motor feedback frame (see communication type 0)"** — note this literally says "type 0" here, unlike most other frames in this chapter which reference "communication type 2". Quoted verbatim; see OPEN QUESTIONS.

### Communication type 24 (0x18): The motor actively reports frames — p.49-50
Request:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x18 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | "01 02 03 04 05 06 F_CMD; Among them, the F_CMD byte is the motor reporting switch; 00 is to disable active reporting (default); 01 To enable active reporting, the default reporting interval is 10ms" |

Response frame (continues p.50):
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x18 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has); bit21: uncalibrated; bit20: Uncalibrated; bit20: Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status — 0: Reset [reset], 1: Cali [calibration], 2: Motor mode [Run] | target motor CAN_ID | Byte0~1: Current Angle [0~65535] → (-4π~4π); Byte2~3: Current angular velocity [0~65535] → (-50rad/s~50rad/s); Byte4~5: Current torque [0~65535] → (-5.5Nm~5.5Nm); Byte6~7: Current temperature: Temp(Celsius)*10, "If the value is higher than 10, the high byte is first and the low byte is last" |

### Communication type 25 (0x19): Motor protocol modification frame (re-power-on effect) — p.50
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x19 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "01 02 03 04 05 06 F_CMD; Among them, the F_CMD byte is the motor protocol type; Among them, 0 is a private protocol (default), 1 is the Canopen protocol, 2 is the MIT protocol" |

Response frame: "Response motor feedback frame (see communication type 0)" (verbatim as printed).

### CANopen/MIT "Protocol Switching" extended frame — §5.10, p.75-76
| Data Field | 29-bit ID | 8-Byte Data Area |
|---|---|---|
| Size | Bit 28~0 | Byte 0~6 |
| Description | 0xFFF | 01 02 03 04 05 06 F_CMD |

"F_CMD (Byte 6) defines the motor protocol: 0: Private protocol (default); 1: CANopen protocol; 2: MIT protocol"

Response Frame:
| Data Field | 11-bit ID | 8-Byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Motor ID | 64-bit MCU unique identifier |

### Read example (§4.1.15, p.53)
Read instruction (reading `loc_kp`):
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| | 0x11 | 0x00FD | 0x7F | 1E 70 00 00 00 00 00 00 |
| Description | Type 17 | Host id 0xFD | Target motor CAN_ID 7F | Byte0~1: index, corresponding to loc_kp |

Feedback instruction:
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| | 0x11 | 0x007F | 0xFD | 1E 70 00 00 00 00 F0 41 |
| Description | Type 17 | bit15~8: Target motor CAN_ID 7F | Host id 0xFD | "Byte0~1: index, corresponding to loc_kp; Byte4~7: loc_kp value 30, high right byte, (32-bit single precision) hexadecimal IEEE-754 standard floating point number" |

---

## Frame definitions (RX - motor to host)

### Communication Type 2 (0x2): motor feedback data — p.44-45 (the primary status/feedback frame; also reused as the response frame for most TX commands above)
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x2 | "Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has); bit21: uncalibrated; bit20: Uncalibrated; bit20: Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status — 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run]" | host CAN_ID | "Byte0~1: The current Angle [0~65535] Corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-50rad/s~50rad/s); Byte4~5: Current torque [0~65535] corresponds to (-5.5Nm~5.5Nm); Byte6~7: Current temperature: Temp(Celsius)*10 — If the value is higher than 10, the high byte is first and the low byte is last" |

Note: "bit20" is defined twice with two different meanings ("Uncalibrated" and "Gridlock overload fault") verbatim in the source — this exact same duplicate appears in the type-2, type-18-response, and type-24-response copies of this bit list (pp.44-45, 50). Not resolved; flagged in OPEN QUESTIONS.

### MIT protocol Response Command 1: Data Feedback (Motor Status) — p.77
| Data Field | 11-bit ID | 8-byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Host ID | "Byte 0: Motor CAN ID; Byte 1~2: The current angle [0~65535], corresponds to (-12.57 rad ~ 12.57 rad); Byte 3 (high 8 bits), Byte 4[7-4] (low 4 bits): The current speed [0~4096], corresponds to (-50 rad/s ~ 50 rad/s); Byte 4[3-0] (high 4 bits), Byte 5 (low 8 bits): The current torque [0~4096], corresponds to (-5.5N·m ~ 5.5 N·m); Byte 6~7: Winding temperature (in degrees): Temp(Celsius) *10" |

(The source literally includes stray "<br>" tags inline in this table's description text — reproduced faithfully above by converting them to semicolons/line-breaks; the nibble-packing phrasing "Byte 4[7-4] (low 4 bits)" mixes high/low terminology in a way that is not self-consistent — flagged in OPEN QUESTIONS.)

### MIT protocol Response Command 2: MCU Identification — p.78
| Data Field | 11-bit ID | 8-byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Motor ID | 64-bit MCU unique identifier |

---

## Parameter table (flash config read/write)

Two distinct parameter tables exist in the manual. Both are reproduced below.

### Table A — "Upper computer" flash config table (function codes 0X0000-0X304E), §3.3.4, p.19-29

Columns as printed: function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | Remark

| Func code | Name | Type | Attribute | Max | Min | Current (ref) | Remark |
|---|---|---|---|---|---|---|---|
| 0X0000 | Name | String | Read/Write | | | (placeholder glyphs, unreadable as text) | |
| 0X0001 | BarCode | String | Read/Write | | | (placeholder glyphs, unreadable as text) | |
| 0X1000 | BootCodeVersion | String | Read only | | | 0.1.5 | |
| 0X1001 | BootBuildDate | String | Read only | | | Mar 16 2022 | |
| 0X1002 | BootBuildTime | String | Read only | | | 20:22:09 | |
| 0X1003 | AppCodeVersion | String | Read only | | | 0.0.0.1 | Motor program version number |
| 0X1004 | AppGitVersion | String | Read only | | | 7b844b0fM | |
| 0X1005 | AppBuildDate | String | Read only | | | Apr 14 2022 | |
| 0X1006 | AppBuildTime | String | Read only | | | 20:30:22 | |
| 0X1007 | AppCodeName | String | Read only | | | Lingzu_motor | |
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
| 0X200e | status3 | uint32 | Read/Write | 100000 | 1000 | 20000 | Reserved parameter |
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
| 0X201d | chasu_angle_offset | float | Read/Write | 27 | 0 | 0 | The low end is offset |
| 0X201e | spd_step_value | float | Read/Write | 150 | 0 | (blank) | Velocity-mode acceleration |
| 0X201f | vel_max | float | Read/Write | 20 | 0 | (blank) | PP mode speed |
| 0X2020 | acc_set | float | Read/Write | 1000 | 0 | (blank) | PP mode acceleration |
| 0X2021 | zero_sta | float | Read/Write | 100 | 0 | 0 | Zero marker |
| 0x2022 | protocol_1 | uint8 | Read/Write | (blank) | (blank) | 0 | Protocol flag |
| 0X3000 | timeUse0 | uint16 | Read only | | | 5 | |
| 0X3001 | timeUse1 | uint16 | Read only | | | 0 | |
| 0X3002 | timeUse2 | uint16 | Read only | | | 10 | |
| 0X3003 | timeUse3 | uint16 | Read only | | | 0 | |
| 0X3004 | encoderRaw | int16 | Read only | | | 11396 | Magnetic encoder sampling value |
| 0X3005 | mcuTemp | int16 | Read only | | | 337 | mcu internal temperature, *10 |
| 0X3006 | motorTemp | int16 | Read only | | | 333 | Motor ntc temperature, *10 |
| 0X3007 | vBus(mv) | uint16 | Read only | | | 24195 | Bus voltage |
| 0X3008 | adc1Offset | int32 | Read only | | | 2084 | adc sampling channel 1 Zero current bias |
| 0X3009 | adc2Offset | int32 | Read only | | | 2084 | adc sampling channel 2 Zero current bias |
| 0X300a | adc1Raw | uint16 | Read only | | | 1232 | adc sampling value 1 |
| 0X300b | adc2Raw | uint16 | Read only | | | 1212 | adc sampling value 2 |
| 0X300c | VBUS | float | Read only | | | 36 | Bus voltage V |
| 0X300d | cmdId | float | Read only | | | 0 | id ring instruction, A |
| 0X300e | cmdIq | float | Read only | | | 0 | iq ring command, A |
| 0X300f | cmdlocref | float | Read only | | | 0 | Position loop command, rad |
| 0X3010 | cmdspdref | float | Read only | | | 0 | Speed loop command, rad/s |
| 0X3011 | cmdTorque | float | Read only | | | 0 | Torque instruction, nm |
| 0X3012 | cmdPos | float | Read only | | | 0 | mit Protocol Angle instruction |
| 0X3013 | cmdVel | float | Read only | | | 0 | mit Protocol Speed instruction |
| 0X3014 | rotation | int16 | Read only | | | 1 | Number of turns |
| 0X3015 | modPos | float | Read only | | | 4.363409 | Motor uncounted coil mechanical Angle, rad |
| 0X3016 | mechPos | float | Read only | | | 0.777679 | Load end loop mechanical Angle, rad |
| 0X3017 | mechVel | float | Read only | | | 0.036618 | Load speed: rad/s |
| 0X3018 | elecPos | float | Read only | | | 4.714761 | Electrical Angle |
| 0X3019 | ia | float | Read only | | | 0 | U-wire current, A |
| 0X301a | ib | float | Read only | | | 0 | V-wire current, A |
| 0X301b | ic | float | Read only | | | 0 | W-wire current, A |
| 0X301c | timeout | uint32 | Read only | | | 31600 | Timeout counter value |
| 0X301d | phaseOrder | uint8 | Read only | | | 0 | Directional marking |
| 0X301e | iqf | float | Read only | | | 0 | iq filter value, A |
| 0X301f | boardTemp | int16 | Read only | | | 359 | Plate temperature, *10 |
| 0X3020 | iq | float | Read only | | | 0 | iq Original value, A |
| 0X3021 | id | float | Read only | | | 0 | id Original value, A |
| 0X3022 | faultSta | uint32 | Read only | | | 0 | Fault status value |
| 0X3023 | warnSta | uint32 | Read only | | | 0 | Warning status value |
| 0X3024 | drv_fault | uint16 | Read only | | | 0 | The driver chip fault value is 1 |
| 0X3025 | drv_temp | int16 | Read only | | | 48 | The driver chip fault value is 2 |
| 0X3026 | Uq | float | Read only | | | 0 | Q-axis voltage |
| 0X3027 | Ud | float | Read only | | | 0 | D-axis voltage |
| 0X3028 | dtc_u | float | Read only | | | 0 | The duty cycle of the U-phase output |
| 0X3029 | dtc_v | float | Read only | | | 0 | The duty cycle of the V-phase output |
| 0X302a | dtc_w | float | Read only | | | 0 | The duty cycle of the W-phase output |
| 0X302b | v_bus | float | Read only | | | 24.195 | Vbus in the closed loop |
| 0X302c | torque_fdb | float | Read only | | | 0 | Torque feedback value, nm |
| 0X302d | rated_i | float | Read only | | | 8 | Rated current of motor |
| 0X302e | limit_i | float | Read only | | | 27 | The motor limits the maximum current |
| 0X302f | spd_ref | float | Read only | | | 0 | Motor speed expectation |
| 0X3030 | spd_reff | float | Read only | | | 0 | Motor speed expectation 2 |
| 0X3031 | zero_fault | float | Read only | | | 0 | Motor position determination parameters |
| 0X3032 | chasu_coder_raw | float | Read only | | | 0 | Motor position determination parameters |
| 0X3033 | chasu_angle | float | Read only | | | 0 | Motor position determination parameters |
| 0X3034 | as_angle | float | Read only | | | 0 | Motor position determination parameters |
| 0X3035 | cs_angle | float | Read only | | | (blank) | (blank) |
| 0X3036 | vel_max | float | Read only | | | 0 | Motor position determination parameters |
| 0X3037 | judge | float | Read only | | | 0 | Motor position determination parameters |
| 0X3038 | fault1 | uint32 | Read only | | | 0 | Log failure |
| 0X3039 | fault2 | uint32 | Read only | | | 0 | Log failure |
| 0X303a | fault3 | uint32 | Read only | | | 0 | Log failure |
| 0X303b | fault4 | uint32 | Read only | | | 0 | Log failure |
| 0X303c | fault5 | uint32 | Read only | | | 0 | Log failure |
| 0X303d | fault6 | uint32 | Read only | | | 0 | Log failure |
| 0X303e | fault7 | uint32 | Read only | | | 0 | Log failure |
| 0X303f | fault8 | uint32 | Read only | | | 0 | Log failure |
| 0X3040 | ElecOffset | float | Read only | | | 0 | electrical Angle offset |
| 0X3041 | mcOverTemp | int16 | Read only | | | 0 | Overtemperature threshold |
| 0X3042 | Kt_Nm/Amp | float | Read only | | | 0 | Moment coefficient |
| 0X3043 | Tqcali_Type | uint8 | Read only | | | 0 | Motor type |
| 0X3044 | low_position | float | Read only | | | 0 | Motor position determination parameters |
| 0X3045 | theta_mech_1 | float | Read only | | | 0 | Type 2 Low speed Angle |
| 0X3046 | instep | float | Read only | | | 0 | Motor protection decision parameters |
| 0X3047 | position | float | Read only | | | 18.243250 | Initial position |
| 0X3048 | chasu_angle_init | float | Read only | | | 0.598253 | Motor position determination parameters |
| 0X3049 | chasu_angle_out | float | Read only | | | 18.545835 | Motor position determination parameters |
| 0X304A | motormechinit | float | Read only | | | 18.243250 | Motor position determination parameters |
| 0X304B | mech_angle_init2 | float | Read only | | | 2.353968 | Motor position determination parameters |
| 0X304C | mech_angle_rotat | int16 | Read only | | | (blank) | Motor position determination parameters |
| 0X304D | pos_cnt1 | uint16 | Read only | | | 0 | System parameters |
| 0X304E | H' | uint8 | Read only | | | 72 | System parameters |

(Source: §3.3.4 "Parameter settings," p.19-29. "Current value (for reference)" column explicitly labeled as reference/example in the table header.)

### Table B — "Read and write a single parameter list" (index 0X7005-0X702E), §4.1.14, p.51-53

Columns as printed: index | (name) | Description | Type | Number of bytes | (value range/default) | R/W Read and write permission

| Index | Name | Description | Type | Bytes | Range/Default | R/W |
|---|---|---|---|---|---|---|
| 0X7005 | run_mode | "0: operation mode; 1: position mode (PP); 2: Velocity mode; 3: Operation mode Current mode; 5: Position mode (CSP)" | uint8 | 1 | | W/R |
| 0X7006 | iq_ref | Current mode Iq command | float | 4 | -11 to 11A | W/R |
| 0X700A | spd_ref | Rotational Velocity mode Rotational speed command | float | 4 | -50 to 50rad/s | W/R |
| 0X700B | limit_torque | torque limit | float | 4 | 0 to 5.5Nm | W/R |
| 0X7010 | cur_kp | Kp | float | 4 | The default value is 0.17 | W/R |
| 0X7011 | cur_ki | Ki | float | 4 | The default value is 0.012 | W/R |
| 0X7014 | cur_filt_gain | filt_gain | float | 4 | 0 to 1.0, The default value is 0.1 | W/R |
| 0X7016 | loc_ref | Position Mode Angle instruction | float | 4 | rad | W/R |
| 0X7017 | limit_spd | Location mode (CSP) speed limit | float | 4 | 0 to 50rad/s | W/R |
| 0X7018 | limit_cur | Velocity position mode Current limitation | float | 4 | 0 to 11A | W/R |
| 0x7019 | mechPos | Mechanical Angle of the loading coil | float | 4 | rad | R |
| 0x701A | iqf | iq Filter | float | 4 | -11 to 11A | R |
| 0x701B | mechVel | Speed of the load | float | 4 | -50 to 50rad/s | R |
| 0x701C | VBUS | Bus voltage | float | 4 | V | R |
| 0x701E | loc_kp | (printed as name="loc_kp kp", description="at" — garbled, see OPEN QUESTIONS) | float | 4 | The default value is 40 | W/R |
| 0x701F | spd_kp | Indicates the speed kp | float | 4 | The default value is 6 | W/R |
| 0x7020 | spd_ki | ki | float | 4 | The default value is 0.02 | W/R |
| 0x7021 | spd_filt_gain | Speed filter value | float | 4 | The default value is 0.1 | W |
| 0x7022 | acc_rad | velocity mode acceleration | float | 4 | The default value is 20rad/s^2 | W |
| 0x7024 | vel_max | Location mode (PP) speed | float | 4 | The default value is 10rad/s | W |
| 0x7025 | acc_set | Location mode (PP) acceleration | float | 4 | The default value is 10rad/s^2 | W |
| 0x7026 | EPScan_time | "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | The default value is 1 | W/R |
| 0x7028 | canTimeout | can The timeout threshold, 20000 is 1s | uint32 | 4 | The default value is 0 | W/R |
| 0x7029 | zero_sta | "Indicates the zero flag bit, 0 means 0-2π and 1 means -π-π" | uint8 | 1 | The default is 0 | W/R |
| 0x702B | add_offset | Zero offset | float | 4 | The default is 0 | W/R |
| 0x702C | alveolous_open | Cogging Compensation Switch | uint8 | 1 | The default is 0 | W/R |
| 0x702D | iq_test | Motor Initialization Calibration Switch | uint8 | 1 | The default is 0 | W/R |
| 0x702E | dcc_set | Deceleration in Motor PP Mode (Normally Disabled) | float | 4 | The default is 10rad/s^2 | W/R |

Cross-reference with §4.3 "Control mode instructions" (p.56-58): run_mode (0x7005) = 3 is used for "Current mode" (p.57, "Set the runmode parameter to 3"), run_mode = 2 for "Velocity mode" (p.57), run_mode = 5 for "Location Mode (CSP)" (p.58), run_mode = 1 for "Location Mode (PP)" (p.58). This is consistent with (and clarifies) the garbled "3: Operation mode Current mode" table entry above, but the table text itself was not altered/guessed — quoted verbatim.

### CANopen object dictionary entries (§5.4-5.9, p.71-75; distinct index space, different from above)

Status feedback (§5.4, p.71-72):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 603F | Error_code | Read-only | UINTEGER16 | / |
| 6041 | Statusword | Read-only | UINTEGER16 | / |
| 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
| 6062 | Position_demand_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6064 | Position_actual_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 606B | Velocity_demand_value | Read-only | INTEGER32 | 0.1 rpm |
| 606C | Velocity_actual_value | Read-only | INTEGER32 | 0.1 rpm |
| 6077 | Torque_actual_value | Read-only | INTEGER16 | 0.1% load ratio (1000 = 2 N·m) |
| 6078 | Current_actual_value | Read-only | INTEGER16 | mA |
| 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | mV |

Homing Mode (§5.5, p.72): 6040 Controlword (Read-write, UINTEGER16); 6060 Modes of operation (Read-write, INTEGER8). "Set Modes of operation to 6 while the motor is in the disabled state (SWITCH_ON_DISABLED). The motor will then define the current position as the zero point." "To hold the zero position, modify the Controlword to 15."

Position Mode PP (§5.6, p.72-73): 6040 Controlword; 6060 Modes of operation = **1**; 6067 Position_window (UINTEGER32, Pulses, 1 rev=16,384 pulses); 6068 Position_window_time (UINTEGER16, ms); 6071 Target_torque (INTEGER16, 0.1% load ratio, 1000=2N·m); 607A Target_position (INTEGER32, Pulses); 6081 Profile_velocity (UINTEGER32, 0.1 rpm); 6083 Profile_acceleration (UINTEGER32, 0.1 rpm/s).

Position Mode CSP (§5.7, p.73-74): same index set as PP minus 6083, plus Modes of operation = **5**.

Velocity Mode (§5.8, p.74-75): 6040 Controlword; 6060 Modes of operation = **3**; 6071 Target_torque; 60FF Target_velocity (INTEGER32, 0.1 rpm).

Torque Mode (§5.9, p.75): 6040 Controlword; 6060 Modes of operation = **4**; 6071 Target_torque.

Note: these CANopen "Modes of operation" values (PP=1, Velocity=3, Torque=4, CSP=5, Homing=6) use a **different numbering scheme** than the private-protocol `run_mode` (0X7005) table above (where 1=PP, 2=Velocity, 3=Current, 5=CSP). Not reconciled — flagged in OPEN QUESTIONS.

---

## Fault/error codes

Four different bit-field listings for faults appear in the document, under different communication types, and they **do not fully agree with each other** on bit numbering for equivalently-named faults. Each is quoted separately below; no attempt was made to merge/reconcile them.

### 1. Flash parameter `0X3022 faultSta` bit meanings — §3.3.7 "Motor fault instructions," p.32
"Function code 0x3022 indicates the fault code, where
- bit14: motor blocking overload algorithm protection
- bit7: Encoder uncalibrated: Motor uncalibrated encoder
- bit3: Overvoltage fault: the motor voltage exceeds the protection voltage by 60V
- bit2: Undervoltage fault: the motor voltage is lower than the protection voltage of 12V
- bit1: Driver chip failure: Motor driver chip failure reported
- bit0: Motor overtemperature fault: motor thermistor temperature exceeds 135 degrees"

### 2. Driver chip fault register 1 — `0X3024 drv_fault`, "Table 11. Fault Status Register 1 Field Descriptions," p.32
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

### 3. Driver chip fault register 2 — `0X3025 drv_temp`, "Table 12. Fault Status Register 2 Field Descriptions," p.33
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

### 4. Communication type 2 / 18-response / 24-response in-frame fault bits — p.44-45, 50 (identical text repeated in each location)
"bit21~16: fault information (0 none 1 has)
- bit21: uncalibrated
- bit20: Uncalibrated
- bit20: Gridlock overload fault  [bit20 defined twice — verbatim duplicate in source]
- bit19: magnetic coding fault
- bit18: overtemperature
- bit17: Three-phase overcurrent fault
- bit16: undervoltage fault
bit22~23: Mode status — 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run]"

### 5. Communication type 21 (0x15) "Fault feedback frame" — p.48
"Byte0~3: fault value (non-0: faulty; 0: Normal)
- Bit 16: A-phase current sampling overcurrent
- Bit 14: Motor stall overload algorithm protection
- Bit 9: Position initialization fault
- Bit 8: Hardware identification fault
- Bit 7: Encoder uncalibrated: Motor encoder not calibrated
- Bit 5: C-phase current sampling overcurrent
- Bit 4: B-phase current sampling overcurrent
- bit3: overvoltage fault
- bit2: undervoltage fault
- bit1: driver chip fault
- bit0: motor overtemperature fault, Default 135°C
Byte4~7: warning Value
- bit0: motor overtemperature warning, the default is 135°c"

### 6. MIT protocol Command 5 "Clear Errors & Read Fault Status" response — p.79
"Byte 0: Motor CAN ID
Byte 1~4: Fault value (Non-zero: Fault present; 0: Normal)
- Bit 14: Stall/I²t overload fault
- Bit 7: Encoder not calibrated
- Bit 3: Overvoltage fault
- Bit 2: Undervoltage fault
- Bit 1: Driver IC fault
- Bit 0: Motor overtemperature fault (Default threshold: 135°C)"

**These six bit-field listings assign different bit numbers to similarly-named faults** (e.g. "motor stall/blocking overload" is bit14 in #1 and #6, but bit14 in #5 too — consistent there — while "driver chip fault" is bit1 in #1/#5/#6 but a completely different register layout in #2/#3; "encoder uncalibrated" is bit7 in #1/#5/#6 but bit21 in #4). Not reconciled. See OPEN QUESTIONS.

---

## OPEN QUESTIONS (ambiguous / not confidently extractable)

1. **RS05-specific firmware version not confirmed.** The parameter-table screenshot (p.19-20) showing BootCodeVersion/AppCodeVersion/AppBuildDate/AppCodeName is explicitly labeled "(for reference)" in the table header and may be a generic example captured from a different unit, not necessarily RS05. No other firmware version string specific to RS05 was found in the document.

2. **Screenshot example shows "motor type RS00", not RS05** (p.15, §3.3.1 "Motor connection settings" screenshot). This RS05 manual appears to reuse a generic screenshot from the RS00 manual; cannot confirm any RS05-specific UI/ID example from it.

3. **Rated/peak torque inconsistency**: cover page states "6N.M" (p.1) as the product designation, but §1.2 gives "Rated load (CW): 1.6 N.m" (p.8) and §1.3 gives "Rated load: 1.6 N.m(Tested at 100 rpm)" and "Peak load: 5.5 N.m" (p.9). The "6N.m" figure does not match either rated or peak torque as tabulated elsewhere. Could not determine which number is authoritative, or whether "6N.m" is a rounded/marketing figure for a sibling model.

4. **Communication Type 1 table layout is garbled** (p.44): "Byte2: Torque (0~65535) corresponds to (-5.5Nm~5.5Nm)" is printed under the bit23~8 ID column rather than the 8-byte data field column where Byte2-3 is independently defined as angular velocity. Could not determine if this is a duplicate/stray reference to the Byte6-7 Kd/torque fields or a genuine formatting defect in the source PDF.

5. **"Version number read frame" reuses comm-type code 0x4** (same as "Motor stops running," §4.1.5) distinguished only by payload bytes Byte[0]=0x00/Byte[1]=0xC4 (p.45-46). This is a plausible but unusual protocol design (overloading the type field) — not independently verified against a capture.

6. **Response-frame type references are inconsistent.** Most TX frames reference "Response motor feedback frame (see communication type 2)" as their reply, but Communication type 23 (baud rate modification, p.49) and Communication type 25 (protocol modification, p.50) both say "Response motor feedback frame (see communication type 0)" instead — i.e., referencing the Get-Device-ID reply format, not the standard feedback frame. Quoted verbatim; not corrected.

7. **"bit20" is defined twice with conflicting meanings** ("Uncalibrated" and "Gridlock overload fault") in the Communication-type-2 / type-18-response / type-24-response fault bit lists (p.44-45, p.50). Could not determine the intended correct single meaning.

8. **Fault bit numbering is inconsistent across the six different fault-bit listings** found in the document (flash param 0x3022, driver fault registers 0x3024/0x3025, comm-type-2 in-frame bits, comm-type-21 fault frame, MIT Command-5 fault response) — see "Fault/error codes" section above for full detail. Each is quoted as printed; none were reconciled or assumed correct over another.

9. **`0X7005 run_mode` table text is garbled**: "3: Operation mode Current mode" (p.51) does not parse as clean English. Cross-referenced against §4.3 Control mode instructions (p.56-58), which implies run_mode=3 should mean "Current mode" — but the parameter table's exact wording was preserved as printed, not corrected.

10. **`0x701E` row in the single-parameter R/W list is garbled** (p.52): printed as Name="loc_kp kp", Description="at", suggesting a line-wrap/OCR artifact where the intended text (likely something like "Position loop Kp") did not render cleanly in the source table. Transcribed exactly as printed.

11. **CANopen "Modes of operation" values (PP=1, Velocity=3, Torque=4, CSP=5, Homing=6)** (§5.5-5.9, p.72-75) do not match the private-protocol `run_mode` index 0X7005 values (0=operation, 1=PP, 2=Velocity, 3=Current/operation, 5=CSP) (p.51). These are presumably two independent, non-interchangeable numbering schemes (CANopen object 6060 vs. private-protocol run_mode), but the document does not explicitly state this — inferred only from context, not confirmed.

12. **MIT protocol §6.19 "Position Mode (CSP)" instructs sending Command 6 with Mode=1** (p.83), but Mode=1 is used elsewhere (CANopen §5.6, p.72) to mean Position Mode **PP**, and CSP is elsewhere assigned Mode=5 (CANopen §5.7, p.73). Could not determine whether the MIT-protocol "Set Operation Mode" (Command 6) uses yet another independent numbering scheme matching the private-protocol run_mode table (where 1=PP also, not CSP) — i.e., possibly a copy-paste/terminology error in the heading "Position Mode (CSP)" when the body text's Mode=1 actually selects PP. Not resolved.

13. **MIT protocol Command 3 "MIT Dynamic Parameters" nibble-packing description is internally inconsistent** (p.78): phrases like "Byte 3[7-4] (low 4 bits)" mix "low 4 bits" language with a nibble range "[7-4]" that is normally the high nibble. Same pattern repeats for Byte 6. Could not determine the actual bit order without a capture/firmware reference. Quoted exactly as printed, including the identical issue in Response Command 1 (p.77).

14. **MIT protocol Command 5 "Clear Errors & Read Fault Status" request-byte table text is garbled**: printed as "FF FF FF FF FF FF F_CMD FBF_CMD:- 0xFF → Clear current fault..." (p.79), where "FB" (the literal command byte) and "F_CMD" appear concatenated as "FBF_CMD" with no separating space/structure. Transcribed as printed.

15. **"Command 6: Set Operation Mode" (MIT protocol, p.79) has no request byte-layout table**, only "Response: Response Command 1" is given. No Mode-value table or byte description was found for this command anywhere in the MIT protocol chapter.

16. **MIT protocol Command 13 "Motor actively reports" description text says "The F_CMD byte indicates the motor protocol type"** (p.81) but then describes F_CMD as a reporting on/off switch (0=no reporting, 1=reporting) — internally contradictory phrase, likely a copy-paste error from the "Change Communication Protocol" command description. Quoted verbatim.

17. **Type 2 Update position-range note lists "P_MIN: 12.57f, P_MAX: 12.57f"** (§4.2.3, p.54) — both given the same positive value, whereas the C macro definitions elsewhere (p.59) define `P_MIN -12.57f` / `P_MAX 12.57f`. Likely a missing minus sign in §4.2.3's P_MIN, but not corrected — quoted as printed.

18. **No explicit CAN bus termination/wiring pinout diagram for the RS05 itself** was found in the pages reviewed (only the generic USB-CAN adapter DIP-switch/termination note on p.13, which concerns the debugging adapter, not the motor's own CAN connector pinout).

19. **Read/single-parameter read-request table (Comm type 17, p.47) and its reply table both end in a truncated/cut-off sentence** ("...the low byte is first and the high byte is second (" / "...followed by high bytes at") — the source document itself appears to cut off mid-sentence; no continuation was found on adjacent pages.
