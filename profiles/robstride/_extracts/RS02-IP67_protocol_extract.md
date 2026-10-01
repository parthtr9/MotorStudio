# RS02-IP67 protocol extract
Source: RS02-IP67User Manual260713.pdf (81 pages total per PDF metadata; the document's own printed footer numbering runs 1-80, with an unnumbered cover/title page before footer "1". All "p.N" citations below use the document's own printed footer page number, not the raw PDF page index.)

## Model/firmware identification

- Title page (unnumbered, before p.1): "ROBSTRIDE 02-IP67", "17N.M Quasi-Direct Drive Integrated Motor Module", "User Manual", "RobStride Dynamics", "https://www.robstride.com/"
- p.1: "RS02-IP67 instruction manual"
- p.1: Manual published by "Beijing Lingfoot Times Technology Co., LTD. (hereinafter referred to as Lingfoot)"
- p.1, p.13, p.64: download/support site "www.robstride.com"
- p.7: "Rated voltage: 48 VDC", "Operating voltage range: 24V-60 VDC"
- p.11 (Driver product Specifications table): "The rated working voltage | 48VDC"; "The maximum allowable voltage | 60VDC"; "Rated working phase current | 7Apk"; "Maximum allowable phase current | 23Apk"; "Standby power | ≤18mA"; "CAN bus bit rate | 1Mbps"; "Dimensions | Φ58mm"; "Working environment temperature | -20℃ to 50℃"; "The maximum allowable temperature of the control board | 145℃"; "encoder resolution | 14bit (absolute turn)"
- p.10: Mechanical characteristic: Weight 380g±3g, Number of poles 28, Phase number 3 phases, Drive mode FOC, Deceleration ratio 7.75:1
- p.80: Version History table — "Version Number: 1.0 | Description: Initial Release | Date: November 25, 2025" (this is the manual's own revision history, not explicitly stated to be the motor firmware version)
- p.20 (example/reference values shown in a host-computer parameter-table screenshot, function codes 0X1000-0X1007): BootCodeVersion "0.1.5", BootBuildDate "Mar 16 2022", BootBuildTime "20:22:09", AppCodeVersion "0.0.0.1" (remark column: "Motor program version number"), AppGitVersion "7b844b0fM", AppBuildDate "Apr 14 2022", AppBuildTime "20:30:22", AppCodeName "Lingzu_motor". These are captioned as example/reference values in a generic screenshot, not confirmed as RS02-IP67-specific shipped firmware (see Open Questions).
- p.15: host-tool screenshot "Connect Motor" panel shows field "motor type RS00" (not RS02-IP67) — appears to be a generic/reused screenshot (see Open Questions).
- p.77, p.78 (MIT protocol Commands 12-15): each explicitly annotated "(requires firmware update to 0.2.3.32)" — i.e. Save motor data, Motor actively reports, Read motor parameters, Write motor parameters all require firmware ≥ 0.2.3.32.
- No numeric "default CAN ID" is explicitly stated as a shipped factory default anywhere in the document (see Open Questions).
- No protocol-level statement distinguishing the "IP67" variant from a plain RS02 was found anywhere in the document (see Open Questions / section 6 below for note on this).

## CAN bus parameters

- p.40: "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted as follows:"
- p.11: "CAN bus bit rate | 1Mbps" (Driver product Specifications table)
- p.72 (MIT protocol intro): "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol. The standard frame format is as follows:" — MIT protocol uses the 11-bit (standard) CAN ID, not the 29-bit extended ID used by the default/private protocol.
- p.45-46 (Communication type 23, "Motor baud rate modification frame (re-power-on effect)"): F_CMD byte values — "01 is 1M", "02 is 500K", "03 is 250K", "04 is 125K". Change takes effect after re-power-on.
- p.47 (Communication type 25, "Motor protocol modification frame (re-power-on effect)"), p.71 (CANopen §5.10), p.75-76 (MIT Command 8): protocol-select F_CMD byte values — "0 is a private protocol (default)", "1 is the Canopen protocol", "2 is the MIT protocol". Change "Takes Effect After Power Cycle"; p.51 states a reboot is required after switching, and post-switch the host must address the motor using the new protocol's frame format (CANopen: send extended frame protocol-switch frame; MIT: send standard frame Command 8).
- p.11-12 (driver connector pinout): Pin1 = positive electrode of power supply (+); Pin2 = negative electrode of power supply (-); Pin3 = CAN_L; Pin4 = CAN_H (wire colors per diagram p.11: red=CAN_H, black=CAN_L, thick black=GND(-), coarse red=48V+).
- p.13: CAN-to-USB module notes — "When DIP switch 1 is in the ON position, the module enters Boot mode and cannot establish a connection with the host computer. When DIP switch 2 is in the ON position, a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer." Serial-port wrapper frame header is "41 54" ("AT"), frame tail "0D 0A".
- p.21 (parameter table, function code 0X200a "CAN_ID", type uint8, Setting attribute): Maximum value 127, Minimum value 0, "Current value (for reference)" = 1, remark "id of this object" — captioned as example/reference, not stated as a fixed factory default (see Open Questions).
- p.21 (0X200b "CAN_MASTER", uint8, Setting): range 0-127, example current value 0, remark "can host id".
- p.21 (0X200c "CAN_TIMEOUT", uint32, Read/Write): range 0-100000, example current value 0, remark "can timeout threshold. The default value is 0".
- p.30 (§3.3.6 "can communication failure protection"): "When the value of CAN_TIMEOUT is 0, this function is disabled. When the CAN_TIMEOUT value is non-0, when the motor does not receive the can command within a certain period of time, the motor enters the reset mode, and 20000 is 1s." (i.e. unit = counts, 20000 counts = 1 second)
- p.49 (0x7028 "canTimeout" in the read/write single-parameter-list, uint32): "can The timeout threshold, 20000 is 1s. The default value is 0."
- p.67 (§5.4 CANopen "Motor Watchdog"): "Set the CAN communication watchdog via index 1 of object 0x6099. After configuration, a value of 20000 represents 1 second. If no communication frame is received within the set time, the motor will be disabled. The default value is 0 (disabled), and a non-zero value enables the watchdog."

## Frame definitions (TX — host to motor)

### General private-protocol frame layout (p.40)
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | Communication type | data area 2 | Destination address | data area 1 |

Confirming C struct given on p.57 for the extended 29-bit arbitration field:
```
struct exCanIdInfo{
    uint32_t id:8;
    uint32_t data:16;
    uint32_t mode:5;
    uint32_t res:3;
};
```
(`id` = destination/target CAN ID in bits0-7, `data` = "data area 2" in bits8-23, `mode` = communication type in bits24-28, `res` = 3 unused/padding bits.)

### Communication type 0: Get device ID (p.40)
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x0 | bit15~8: identifies host CAN_ID | target motor CAN_ID | 0 |

Reply frame:
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x0 | target motor CAN_ID | 0XFE | 64-bit MCU unique identifier |

### Communication Type 1: operation control mode motor control instruction (p.41)
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x1 | Byte2: Torque (0~65535) corresponds to (-17Nm~17Nm) | target motor CAN_ID | Byte0~1: target Angle [0~65535] corresponds to (-4π~4π); Byte2~3: Target angular velocity [0~65535] corresponds to (-44rad/s~44rad/s); Byte4~5: Kp [0~65535] corresponds to (0.0~500.0); Byte6~7: Kd [0 to 65535] corresponds to the above data (0.0 to 5.0). After the conversion, the high byte is in front and the low byte is in [text cut off at page edge — see note below] |

Note: the data-field text is truncated at the bottom of p.41 mid-sentence ("the high byte is in front and the low byte is in"); the manual does not finish this sentence on the next page in a directly-quotable way, so the exact wording of where "high byte"/"low byte" placement applies (presumably to Byte2 Torque's 16-bit split, since that value also spans beyond the literal table cell) could not be confirmed — quoted verbatim, not completed by inference.

Response frame: Response motor feedback frame (see Communication Type 2, below, under RX).

### Communication Type 3: Motor enabled to run (p.42)
| data field | 29-bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x3 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | (none specified) |

Response frame: Response motor feedback frame (see communication type 2).

### Communication Type 4: Motor stops running (p.42)
| data field | 29-bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "When the motor is running normally, 0 must be cleared in the data field. Byte[0]=1: The fault is cleared." |

Response frame: Response motor feedback frame (see communication type 2).

### "Version number read frame" (p.42) — reuses the same 0x4 communication-type code as "Motor stops running" above, distinguished only by its data-byte payload
| Data field | 29 bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | Byte[0]=0x00, Byte[1]=0xC4 |

This overlap between "motor stop" and "version read" both using code 0x4, differentiated only by the data payload, is not explained further in the text (see Open Questions).

### Communication type 6: Set motor mechanical zero (p.43)
| data field | 29-bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x6 | bit15~8: Identifies the main CAN_ID | and target motor CAN_ID | Byte[0]=1 |

Response frame: Response motor feedback frame (see communication type 2).

### Communication type 7: Set motor CAN_ID (p.43)
"Change the current motor CAN_ID, effective immediately."
| data field | 29-bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x7 | bit15~8: used to identify main CAN_ID; Bit16~23: preset CAN_ID | Target motor CAN_ID | (none specified) |

Answer frame: Answer motor broadcast frame (see communication type 0).

### Communication type 17: Single parameter read (p.43-44)
| Data field | 29 bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x11 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below; Byte2~3:00; Byte4~7: In data above 00, the low byte is first and the high byte is second ( [sentence truncated at page break, resumes as the Reply-frame table below] |

Reply frame (p.44):
| Data field | 29 bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x11 | bit15~8: indicates that the master CAN_ID; Bit23~16:00 indicates that the master CAN_ID is successfully read. 01 indicates that the master can_ID [sentence also appears truncated] | (not separately given — see note) | Byte0~1: Byte2~3:00 Byte4~7: Parameter data. 1 byte of data above Byte4 is preceded by low bytes and followed by high bytes at [sentence truncated] |

Note: both the request and reply tables for type 17 have sentences that run off the page edge and are not completed verbatim elsewhere in the document; quoted exactly as printed rather than completed by inference.

### Communication type 18: Single parameter write (lost in power failure) (p.44)
"With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved"
| Data field | 29 bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x12 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below; Byte2~3: 00; Byte4~7: Parameter data. In the preceding data, the low byte is in the front and the high byte is in the rear |

Response frame: Response motor feedback frame (see communication type 2).

Worked numeric example confirming 0x12 = decimal 18 (p.29-30): raw extended-ID word "90 07 e8 0c" (as captured in the host-tool's serial-wrapper command box) converts to binary "1001 0000 0000 0111 1110 1000 0000 1100", shifted right 3 positions to "1 0010 0000 0000 1111 1101 0000 0001", converted to hex "12 00 FD 01" — "12 in hexadecimal | 0 | FD | 1" = "Communication type 18 (in decimal base) | No meaning | host id | motor canid" (p.30).

### Communication type 22: Motor data save frame (p.45)
| data field | 29-bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x16 | bit15~8: identifies the main CAN_ID | and target motor CAN_ID | 01 02 03 04 05 06 07 08 |

Response frame: Response motor feedback frame (see communication type 2).

### Communication type 23: Motor baud rate modification frame (re-power-on effect) (p.45-46)
| Data field | 29 bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x17 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD. "Among them, the F_CMD byte is the motor baud rate Among them, 01 is 1M; 02 is 500K; 03 is 250K; 04 is 125K" |

Response frame: Response motor feedback frame (see communication type 0). [Note: text literally says "communication type 0" here, not type 2 as other frames do — quoted verbatim.]

### Communication type 24: The motor actively reports frames (configuration command, p.46)
| data field | 29-bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x18 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD. "Among them, the F_CMD byte is the motor reporting switch 00 is to disable active reporting (default) 01 To enable active reporting, the default reporting interval is 10ms" |

(The response/periodic-report frame for Type 24 is listed under RX below.)

### Communication type 25: Motor protocol modification frame (re-power-on effect) (p.47)
| Data field | 29 bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x19 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD. "Among them, the F_CMD byte is the motor protocol type Among them, 0 is a private protocol (default) 1 is the Canopen protocol 2 is the MIT protocol" |

Response frame: Response motor feedback frame (see communication type 0).

### CANopen: Protocol Switching extended frame (§5.10, p.71)
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

## Frame definitions (RX — motor to host)

### Communication Type 2: motor feedback data (response to Type 1, Type 3, Type 4, Type 6, Type 18, Type 22) (p.41-42)
| Data field | 29 bit ID | | | 8Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has) — bit21: uncalibrated; bit20: Uncalibrated [sic, printed twice — see Open Questions]; bit20: Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status — 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run] | host CAN_ID | Byte0~1: The current Angle [0~65535] Corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-44rad/s~44rad/s); Byte4~5: Current torque [0~65535] corresponds to (-17Nm~17Nm); Byte6~7: Current temperature: Temp(Celsius)*10 If the value is higher than 10, the high byte is first and the low byte is last |

Note: bit20 is listed with two different meanings in the source text ("Uncalibrated" and "Gridlock overload fault") in immediate succession — quoted exactly as printed; could not resolve which is correct (see Open Questions).

### "Version number read frame" — Response frame (p.42-43)
| data field | 29 位ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has) — bit21: uncalibrated; bit20: Uncalibrated; bit20: Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status 0: Reset mode [reset] 1: Cali mode [calibration] 2: Motor mode [Run] | motor CAN_ID | Byte0=0x00; Byte1=0xC4; Byte2=0x56; Byte3~6: Motor version number, Order from high to low |

### Communication type 21: Fault feedback frame (p.44-45)
| data field | 29-bit ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x15 | bit15~8: identifies the main motor CAN_ID | the main CAN_ID | Byte0~3: fault value (non-0: faulty; 0: Normal); Byte4~7: warning Value |

Fault bits (Byte0~3, p.44-45): Bit 16: A-phase current sampling overcurrent; Bit 14: Motor stall overload algorithm protection; Bit 9: Position initialization fault; Bit 8: Hardware identification fault; Bit 7: Encoder uncalibrated (Motor encoder not calibrated); Bit 5: C-phase current sampling overcurrent; Bit 4: B-phase current sampling overcurrent; bit3: overvoltage fault; bit2: undervoltage fault; bit1: driver chip fault; bit0: motor overtemperature fault, default 135°C.

Warning bits (Byte4~7, p.45): bit0: motor overtemperature warning, the default is 135°C.

Note: the table's "bit15~8"/"the main CAN_ID" column assignment for host-vs-target CAN ID is worded ambiguously/possibly garbled by a page-break table-cell split (see Open Questions); quoted exactly as printed.

### Communication type 24: motor actively reports — Response (periodic report) frame (p.46)
| data field | 29 位ID | | 8Byte data field |
|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| description | 0x18 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none 1 has) — bit21: uncalibrated; bit20: Uncalibrated; bit20: Gridlock overload fault; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status 0: Reset mode [reset] 1: Cali mode [calibration] 2: Motor mode [Run] | target motor CAN_ID | Byte0~1: target Angle [0~65535] corresponds to (-4π~4π); Byte2~3: Target angular velocity [0~65535] corresponds to (-44rad/s~44rad/s); Byte4~5: Kp [0~65535] corresponds to (0.0~500.0); Byte6~7: Kd [0 to 65535] corresponds to the above data (0.0 to 5.0). After the conversion, the high byte is in front and the low byte is in [sentence cut off, same as Type 1 table] |

Note: this response-frame byte description (p.46) literally repeats the Type-1 command table's "target Angle / Target angular velocity / Kp / Kd" wording verbatim, rather than the "current Angle / Current angular velocity / Current torque / Current temperature" wording used in the Type-2 feedback table elsewhere (p.41-42, p.73). This looks like a copy-paste artifact in the source document; quoted exactly as printed rather than corrected (see Open Questions).

## Parameter table (flash config read/write)

Two distinct parameter/index spaces are documented:

### A. CAN protocol "Read and write a single parameter list" (used by Communication Types 17/18 over CAN) (p.48-50)

| index | name | Description | Type | Bytes | Range / Default | R/W |
|---|---|---|---|---|---|---|
| 0X7005 | run_mode | "0: operation mode; 1: position mode (PP); 2: Velocity mode; 3: Operation mode Current mode; 5: Position mode (CSP)" [text as printed — ambiguous wording for value 3, no value 4 listed; see Open Questions] | uint8 | 1 | — | W/R |
| 0X7006 | iq_ref | Current mode Iq command | float | 4 | -16 to 16A | W/R |
| 0X700A | spd_ref | Rotational Velocity mode Rotational speed command | float | 4 | -33 to 33rad/s | W/R |
| 0X700B | limit_torque | torque limit | float | 4 | 0 to 14Nm | W/R |
| 0X7010 | cur_kp | Kp | float | 4 | default 0.17 | W/R |
| 0X7011 | cur_ki | Ki | float | 4 | default 0.012 | W/R |
| 0X7014 | cur_filt_gain | filt_gain | float | 4 | 0 to 1.0, default 0.1 | W/R |
| 0X7016 | loc_ref | Position Mode Angle instruction | float | 4 | rad | W/R |
| 0X7017 | limit_spd | Location mode (CSP) speed limit | float | 4 | 0 to 33rad/s | W/R |
| 0X7018 | limit_cur | Velocity/position mode Current limitation | float | 4 | 0 to 16A | W/R |
| 0x7019 | mechPos | Mechanical Angle of the loading coil | float | 4 | rad | R |
| 0x701A | iqf | iq Filter | float | 4 | -16 to 16A | R |
| 0x701B | mechVel | Speed of the load | float | 4 | -33 to 33rad/s | R |
| 0x701C | VBUS | Bus voltage | float | 4 | V | R |
| 0x701E | loc_kp | "at" [printed description text, appears truncated/garbled — see Open Questions] | float | 4 | default 40 | W/R |
| 0x701F | spd_kp | Indicates the speed kp | float | 4 | default 6 | W/R |
| 0x7020 | spd_ki | ki | float | 4 | default 0.02 | W/R |
| 0x7021 | spd_filt_gain | Speed filter value | float | 4 | default 0.1 | W/R |
| 0x7022 | acc_rad | velocity mode acceleration | float | 4 | default 20rad/s^2 | W/R |
| 0x7024 | vel_max | Location mode (PP) speed | float | 4 | default 10rad/s | W/R |
| 0x7025 | acc_set | Location mode (PP) acceleration | float | 4 | default 10rad/s^2 | W/R |
| 0x7026 | EPScan_time | "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | default 1 | W/R |
| 0x7028 | canTimeout | "can The timeout threshold, 20000 is 1s" | uint32 | 4 | default 0 | W/R |
| 0x7029 | zero_sta | "Indicates the zero flag bit, 0 means 0-2π and 1 means -π-π" | uint8 | 1 | default 0 | W/R |
| 0x702A | damper | Damping switch | uint8 | 1 | default 0 | W/R |
| 0x702B | add_offset | Zero offset | float | 4 | default 0 | W/R |
| 0x702C | alveolous_open | Cogging Compensation Switch | uint8 | 1 | default 0 | W/R |
| 0x702D | iq_test | Motor Initialization Calibration Switch | uint8 | 1 | default 0 | W/R |
| 0x702E | dcc_set | Deceleration in Motor PP Mode (Normally Disabled) | float | 4 | default 10rad/s^2 | W/R |

Read example (p.50), reading loc_kp (index 0x701E) with target motor CAN_ID 0x7F, host id 0xFD:
- Read instruction: Type 17 (0x11), 29-bit ID bit23~8=0x00FD (host id), bit7~0=0x7F (target motor CAN_ID); data "1E 70 00 00 00 00 00 00"
- Feedback instruction: Type 17 (0x11), bit23~8=0x007F (target motor CAN_ID), bit7~0=0xFD (host id); data "1E 70 00 00 00 00 F0 41" — "Byte0~1: index, corresponding to loc_kp; Byte4~7: loc_kp value 30, high right byte, (32-bit single precision) hexadecimal IEEE-754 standard floating point number" (wording "high right byte" as printed — ambiguous, see Open Questions).

### B. "Upper computer module" host-tool parameter table (distinct index space, accessed via the host GUI, function codes 0X0000-0X3048) (p.19-28)

Selected entries most relevant to CAN/protocol behavior (full table spans p.20-28 and includes many motor-internal diagnostic/tuning fields not reproduced here in full since they are not CAN-protocol fields; CAN-relevant rows transcribed verbatim):

| function code | name | parameter type | attribute | max | min | current value (example) | remark |
|---|---|---|---|---|---|---|---|
| 0X2009 | motor_baud | uint8 | Settings | 20 | 0 | 1 | Baud rate flag bit |
| 0X200a | CAN_ID | uint8 | Settings | 127 | 0 | 1 | id of this object |
| 0X200b | CAN_MASTER | uint8 | Settings | 127 | 0 | 0 | can host id |
| 0X200c | CAN_TIMEOUT | uint32 | Read/Write | 100000 | 0 | 0 | can timeout threshold. The default value is 0 |
| 0x201e | zero_sta | float | Read/Write | 150 | 0 | (blank) | Zero marker |
| 0x201f | protocol_1 | float | Read/Write | 20 | 0 | (blank) | Protocol flag |
| 0X3022 | faultSta | float | Read only | — | — | 0 | Fault status value |
| 0X3023 | warnSta | float | Read only | — | — | 0 | Warning status value |
| 0X3024 | drv_fault | uint32 | Read only | — | — | 0 | "The driver chip fault value is 1" [see Fault Status Register 1 table below] |
| 0X3025 | drv_temp | uint32 | Read only | — | — | 0 | "The driver chip fault value is 2" [see Fault Status Register 2 table below] |
| 0X3048 | can_status | uin8 [sic] | Read only | — | — | 1 | CAN status |

Note: p.19 states "Note: Please do not change the torque limit, protection temperature and overtemperature time of the motor. Our company will not bear any legal responsibility for any damage to human body or irreversible damage to joints caused by illegal operation of this product." (This document explicitly only protects equipment/joints per its own text, consistent with project scope of protecting equipment, not claiming to be a human-safety system.)

### C. CANopen object dictionary (ch.5, p.67-71)

Status Feedback Parameters (§5.4, p.67-68):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 603F | Error_code | Read-only | UINTEGER16 | / |
| 6041 | Statusword | Read-only | UINTEGER16 | / |
| 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
| 6062 | Position_demand_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6064 | Position_actual_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 606B | Velocity_demand_value | Read-only | INTEGER32 | 0.1 rpm |
| 606C | Velocity_actual_value | Read-only | INTEGER32 | 0.1 rpm |
| 6077 | Torque_actual_value | Read-only | INTEGER16 | 0.1% load ratio (1000 = 6 N·m) |
| 6078 | Current_actual_value | Read-only | INTEGER16 | mA |
| 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | mV |

Homing Mode / Zero Position Setting (§5.5, p.68): 6040 Controlword (Read-write, UINTEGER16); 6060 Modes of operation (Read-write, INTEGER8). "Set Modes of operation to 6 while the motor is in the disabled state (SWITCH_ON_DISABLED). The motor will then define the current position as the zero point." "To hold the zero position, modify the Controlword to 15."

Position Mode PP (§5.6, p.68-69): 6040 Controlword; 6060 Modes of operation; 6067 Position_window (UINTEGER32, Pulses, 1 rev=16,384 pulses); 6068 Position_window_time (UINTEGER16, ms); 6071 Target_torque (INTEGER16, 0.1% load ratio, 1000=6 N·m); 607A Target_position (INTEGER32, Pulses); 6081 Profile_velocity (UINTEGER32, 0.1 rpm); 6083 Profile_acceleration (UINTEGER32, 0.1 rpm/s). "Modes of operation to 1" to enter PP.

Position Mode CSP (§5.7, p.69-70): same index set as PP minus 6083, plus same Profile_velocity; "Modes of operation to 5" to enter CSP.

Velocity Mode (§5.8, p.70): 6040 Controlword; 6060 Modes of operation; 6071 Target_torque; 60FF Target_velocity (Read-write, INTEGER32, 0.1 rpm). "Modes of operation to 3".

Torque Mode (§5.9, p.71): 6040 Controlword; 6060 Modes of operation; 6071 Target_torque. "Modes of operation to 4".

State machine (§5.3, p.66, Controlword=6040H values): 6, 7, or 15 step/direct transition into OPERATION_ENABLE; 1 = stop (return to SWITCH_ON_DISABLED); 11 = emergency Quick Stop ("Use with Caution—Risk of Voltage Surge"); fault clearance also done by modifying Controlword 6040H.

## Fault/error codes

### Function code 0x3022 fault bits (p.30, "Motor fault instructions")
Bit16: Motor current fault: A-phase current sampling overcurrent
Bit14: Motor stall overload algorithm protection
Bit9: Position initialization fault
Bit8: Hardware identification fault
Bit7: Encoder uncalibrated: Motor encoder not calibrated
Bit5: Motor current fault: C-phase current sampling overcurrent
bit3: Overvoltage fault: the motor voltage exceeds the protection voltage by 60V
bit2: Undervoltage fault: the motor voltage is lower than the protection voltage of 12V
bit1: Driver chip failure: Motor driver chip failure reported
bit0: Motor overtemperature fault: motor thermistor temperature exceeds 135 degrees

Note: Bit4 (B-phase overcurrent) is not listed in this particular enumeration, though it IS listed in the CAN Type-21 fault-frame bit list below (p.45) — discrepancy not explained in text (see Open Questions).

### Fault Status Register 1 — function code 0x3024 (p.31, "Table 11")
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

### Fault Status Register 2 — function code 0x3025 (p.31, "Table 12")
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

(Note: DRV8353xS is named explicitly in the source as the referenced gate-driver IC family for this fault register — this is the only place a specific driver-chip part number is mentioned.)

### CAN Type 21 "Fault feedback frame" fault bits (p.44-45) — see full frame definition above under RX
Byte0~3 (fault value, non-zero=faulty): Bit16 A-phase current sampling overcurrent; Bit14 Motor stall overload algorithm protection; Bit9 Position initialization fault; Bit8 Hardware identification fault; Bit7 Encoder uncalibrated; Bit5 C-phase current sampling overcurrent; Bit4 B-phase current sampling overcurrent; bit3 Overvoltage fault; bit2 Undervoltage fault; bit1 Driver chip fault; bit0 Motor overtemperature fault, default 135°C.
Byte4~7 (warning value): bit0 motor overtemperature warning, default 135°C.

### MIT protocol "Fault Status Response" (§6.8, p.74-75)
11-bit ID example "0FD" = Host ID. Byte0: Motor CAN ID. Byte1~4: Fault value (Non-zero: faulty; 0: normal) with bits: Bit16 Motor current fault: Phase A sampling overcurrent; Bit14 Motor stall and overload algorithm protection; Bit9 Position initialization fault; Bit8 Hardware identification fault; Bit7 Encoder uncalibrated: Motor encoder not calibrated; Bit5 Motor current fault: Phase C sampling overcurrent; Bit4 Motor current fault: Phase B sampling overcurrent; bit3 Overvoltage fault; bit2 Undervoltage fault; bit1 Driver chip fault; bit0 Motor over-temperature fault, default threshold: 135 ℃.

## MIT protocol (chapter 6, p.72-79) — supplementary summary (standard 11-bit CAN ID, same bus/bitrate as above)

General frame header given on p.72:
| Data Field | 11-bit ID | | 8-byte Data Area |
|---|---|---|---|
| Size | Bit 10~8 | Bit 7~0 | Byte 0~7 |
| Description | Mode type | ID | |

NOTE: this general header (splitting bits10~8 as "Mode type" and bits7~0 as "ID") does not match how every individual command/response table below is actually presented — those instead show the full "bit10~0" as a single field holding a CAN ID value (e.g. "07F"), with the command's meaning instead signaled by fixed sentinel byte patterns in the 8-byte data area (e.g. FF FF FF FF FF FF FF FC for Enable). This inconsistency is quoted as printed and not reconciled (see Open Questions).

Commands documented: Response Command 1 (Data Feedback/Motor Status, p.72-73), Response Command 2 (MCU Identification, p.73), Command 1 Enable (data=FF FF FF FF FF FF FF FC, p.73), Command 2 Stop (data=FF FF FF FF FF FF FF FD, p.73), Command 3 MIT Dynamic Parameters (p.73-74), Command 4 Set Zero Position/non-position mode (data=FF FF FF FF FF FF FF FE, p.74), Command 5 Clear Errors & Read Fault Status (data=FF FF FF FF FF FF F_CMD FB; F_CMD 0xFF clears, else error code returned in Byte1 of response, p.74), Command 6 Set Operation Mode (data=FF FF FF FF FF FF F_CMD FC; F_CMD 0=MIT mode default,1=Position mode,2=Velocity mode, p.75), Command 7 Modify Motor CAN ID (data=FF FF FF FF FF FF F_CMD FA, p.75), Command 8 Change Communication Protocol (data=FF FF FF FF FF FF F_CMD FD; F_CMD 0=Private default,1=CANopen,2=MIT; takes effect after power cycle, p.75-76), Command 9 Modify Host CAN ID (data=FF FF FF FF FF FF F_CMD 01, p.76), Command 10 Position Mode Control Command (Byte0~3 target position rad 32-bit float, Byte4~7 target speed rad/s 32-bit float, p.76), Command 11 Velocity Mode Control Command (Byte0~3 target speed rad/s float, Byte4~7 current limit A float, p.76-77), Command 12 Save motor data (requires fw 0.2.3.32, data=FF FF FF FF FF FF FF F8, p.77), Command 13 Motor actively reports (requires fw 0.2.3.32, data=FF FF FF FF FF FF F_CMD F9; F_CMD 0=no reporting default,1=reporting, p.77), Command 14 Read motor parameters (requires fw 0.2.3.32, p.77), Command 15 Write motor parameters (requires fw 0.2.3.32, p.77-78).

Response Command 1 byte layout (p.72-73) — quoted verbatim, bit-packing across byte boundaries is ambiguous (see Open Questions):
"Byte0: Motor CAN ID. Byte1~2: Current angle [0~65535], corresponding to (-12.57 rad ~ 12.57 rad). Byte3: High 8 bits; Byte4[7-4] (high 4 bits): Low 4 bits. Current speed [0~4096], corresponding to (-44 rad/s ~ 44 rad/s). Byte4[3-0] (low 4 bits): High 4 bits; Byte5: Low 8 bits. Current torque [0~4096], corresponding to (-17 N·m ~ 17 N·m). Byte6[7-6]: Operation mode 0: Reset Mode, 1: Cali Mode (Calibration), 2: Motor Mode (Running). Byte6[5]: 1 = Motor fault present, 0 = No motor fault. Byte6[4]: 1 = Motor warning without fault, 0 = No motor warning. Byte6[3-0] (low 4 bits): High 4 bits; Byte7: Low 8 bits. Winding temperature: Temperature(°C) × 10, Unit: °C."

Command 3 MIT Dynamic Parameters byte layout (p.73-74) — same nibble-split ambiguity, quoted verbatim:
"Byte0~1: Target angle [0~65535], corresponding to (-12.57 rad ~ 12.57 rad). Byte2: High 8 bits; Byte3[7-4] (high 4 bits): Low 4 bits of target speed. Target speed [0~4096], corresponding to (-44 rad/s ~ 44 rad/s). Byte3[3-0] (low 4 bits): High 4 bits; Byte4: Low 8 bits. Kp [0~4096], corresponding to (0 ~ 500). Byte5: High 8 bits; Byte6[7-4] (high 4 bits): Low 4 bits. Kd [0~4096], corresponding to (0 ~ 5). Byte6[3-0] (low 4 bits): High 4 bits; Byte7: Low 8 bits. Target torque [0~4096], corresponding to (-17 N·m ~ 17 N·m)."

Program-sample constants confirming the ±range scaling (p.56-57, C code):
```
#define P_MIN -12.57f
#define P_MAX 12.57f
#define V_MIN -44.0f
#define V_MAX 44.0f
#define KP_MIN 0.0f
#define KP_MAX 500.0f
#define KD_MIN 0.0f
#define KD_MAX 5.0f
#define T_MIN -17.0f
#define T_MAX 17.0f
```

Motion Control Mode (§6.19, p.78): motor defaults to Motion Control Mode on power-up. Sequence: Command 1 (Enable) → Command 3 (Motion Control/dynamic parameters) → Command 2 (Stop) when needed.

Velocity Mode (§6.20, p.79): Command 6 (Mode=2) → Command 1 (Enable) → Command 11 (set max current & target speed) → Command 2 (Stop).

Position Mode CSP (§6.21, p.79): Command 6 (Mode=1) → Command 1 (Enable) → Command 10 (set max speed & target position) → Command 2 (Stop).

## Functional notes relevant to CAN behavior (ch.4.2, p.50-53)

- Active Reporting (p.50-51): Disabled by default; enabled via Type 24. Report type is Type 2, default interval 10ms, adjustable via EPScan_time (parameter 0x7026) written with Type 18.
- Zero-Point Flag zero_sta (p.51): modify via host computer, or Type 18 (requires saving via Type 22). "Default flag: 0 → Power-on position range: 0–2π. If set to 1: Power-on position range: π–π." [as printed — likely missing a minus sign before the second π, compare parameter-table wording for 0x7029 on p.49 ("0 means 0-2π and 1 means -π-π"); not corrected, flagged in Open Questions]
- Type 2 Update (p.51): "Updated to periodic looping within -4π–4π (enables cycle counting). Note: Position interface parameters must be adjusted: P_MIN: 12.57f, P_MAX: 12.57f." [both printed as positive values; conflicts with the signed P_MIN -12.57f in the program-sample code on p.56 — see Open Questions]
- Protocol Switching (Requires CAN adapter) (p.51): methods — modify protocol_1 via host computer, or send Type 25 command. Reboot required after switching. Post-switch: CANopen → send extended frame (protocol switch frame); MIT Protocol → send standard frame (Command 8).
- Motor Post-Power-Off Anti-Backdrive Protection (p.52): can be enabled/disabled; disable by setting damper=1.
- Zero Calibration Rules (p.52): supported in CSP and Motion Control modes; blocked in PP mode.
- Position Offset / add_offset (p.52): adds an offset to the zero point; worked example given with a 1-radian zero point and add_offset=1.
- CANopen ID = CAN ID (p.52-53): "The CANopen ID is kept the same as the CAN ID under the private protocol."
- Motor Initialization Calibration (p.53): iq_test=1 extends initialization time for more accurate motor initial reference.
- Cogging Calibration Function (p.53): set iq_test=1, restart motor (no-load), perform cogging calibration via host computer; after success "max_alve will be non-zero" [no parameter named max_alve found in either parameter table — see Open Questions]; then set alveolous_open=1 to activate in Motion Control mode.
- PP Mode Deceleration (p.53): "By setting acc_stutus to 1, the acceleration and deceleration for PP mode can be configured separately. The deceleration value is stored at 0x702E [dcc_set]. When acc_stutus is set to 0, the acceleration parameter controls both acceleration and deceleration." [no parameter named acc_stutus found in either parameter table — see Open Questions]
- Operation control mode formula (p.53-54): "t_ref = Kd * (v_vset - v_actual) + Kp * (p_set - p_actual) + t_ff."

## OPEN QUESTIONS (ambiguous, inconsistent, or unconfirmable from the document text)

1. Type 21 "Fault feedback frame" (p.44-45): the host/target CAN-ID column assignment in the 29-bit ID fields ("bit15~8: identifies the main motor CAN_ID" / "the main CAN_ID") reads as garbled, likely from a table cell split across the p.44/p.45 page break. Could not confirm whether this frame is host-initiated (polling) or motor-initiated (unsolicited), since no triggering condition or "Response frame:" note is given (unlike other types).
2. "Version number read frame" (p.42) reuses Communication Type 0x4 — the same code as "Motor stops running" — distinguished only by the data payload bytes (0x00, 0xC4). The manual does not explain this overlap or how a receiver should disambiguate intent.
3. Communication Type 1 table's data-field description (p.41) is cut off mid-sentence ("the high byte is in front and the low byte is in"), and the same truncated phrase recurs verbatim in the Type 24 response frame (p.46). The exact intended meaning/scope of this clause was not recoverable from the text as printed.
4. The MIT protocol's general frame-header table (p.72, "Mode type" in bits10~8, "ID" in bits7~0) is inconsistent with every individual MIT command/response table (pp.72-79), which instead present the full 11-bit field as a single target/host CAN ID and distinguish command meaning via fixed data-byte sentinel patterns. Not reconciled.
5. MIT protocol Response Command 1 (p.72-73) and Command 3 (p.73-74) byte layouts describe speed/torque/Kp/Kd values split across byte boundaries using "high 4 bits"/"low 4 bits" nibble language that is internally inconsistent (e.g., "Byte4[7-4] (high 4 bits): Low 4 bits" — labeling the high nibble of a byte as holding "low 4 bits" of a value). Could not reconstruct an exact, unambiguous bit map; transcribed verbatim.
6. Communication Type 2 / Type 24-response fault-bit tables (p.41-42, p.46) list "bit20: Uncalibrated" immediately followed by "bit20: Gridlock overload fault" — the same bit number given two different meanings in immediate succession. Not resolved.
7. Type 24 "motor actively reports" response frame (p.46) textually repeats the Type-1 command wording ("target Angle", "Target angular velocity", "Kp", "Kd") rather than the "current"/feedback wording used by the Type-2 feedback frame and the MIT Response Command 1 frame. This looks like a copy/paste error in the source but is transcribed as printed; could not confirm the true field semantics of this specific response frame.
8. Parameter 0x7005 (run_mode) description text (p.48) is ambiguously worded for mode value 3 ("3: Operation mode Current mode") and lists no explicit value 4, despite the document elsewhere describing distinct Current/Torque modes. Exact enumerated mode-value mapping could not be confirmed beyond what is printed.
9. Zero-Point Flag section (p.51) states the "set to 1" position range as "π–π" (no minus sign before the first π), which conflicts with the parameter table's own wording for 0x7029 (p.49: "1 means -π-π"). Not corrected — flagged as a likely typo in the source.
10. "Type 2 Update" section (p.51) states P_MIN and P_MAX should both be adjusted to "12.57f" (both positive), conflicting with the program-sample C code (p.56) which defines `#define P_MIN -12.57f` (negative). Could not confirm which is authoritative.
11. Parameter 0x701E "loc_kp" (p.49) has a Description-column value of just "at", which appears to be truncated/garbled OCR-extracted text rather than a complete description.
12. The Read example on p.50 uses the phrase "high right byte" when describing the IEEE-754 float encoding of a parameter value — wording is unclear/possibly a translation artifact; transcribed verbatim rather than reinterpreted.
13. "Motor fault instructions" bit list under function code 0x3022 (p.30) does not include a Bit4 "B-phase current sampling overcurrent" entry, even though the CAN Type 21 fault-frame bit list (p.45) and the MIT Fault Status Response bit list (p.74-75) both do include it. Not clear if this is an intentional difference between reporting mechanisms or an omission.
14. Cogging Calibration Function section (p.53) references a parameter called "max_alve" becoming non-zero after calibration; no parameter by this name was found in either parameter table (0x7xxx CAN list, pp.48-50, or 0x1000-0x3048 host-tool list, pp.20-28).
15. PP Mode Deceleration section (p.53) references a flag "acc_stutus"; no parameter by this name was found in either parameter table.
16. The host-tool screenshot on p.15 (motor connection settings) displays "motor type RS00" rather than RS02-IP67, suggesting this specific screenshot is generic/reused across the RobStride manual family rather than captured from an actual RS02-IP67 unit. Similarly, the example firmware-version strings on p.20 (BootCodeVersion 0.1.5, AppCodeVersion 0.0.0.1, etc.) are captioned as example/reference values in a generic parameter-table screenshot, not confirmed as RS02-IP67-specific.
17. No explicit statement of a fixed, factory-shipped default CAN bus ID was found anywhere in the document. The CAN_ID parameter table entries (p.21 "0X200a", p.49 is a different index space entirely — 0x7xxx table has no explicit CAN_ID entry at all) only show example/reference current values (e.g. "1"), captioned "(for reference)", not stated as the shipped default.
18. No content anywhere in the 81 pages reviewed states any CAN-protocol-level difference between the "IP67" variant and a plain RS02 — all frame formats, parameter tables, fault-code tables, and MIT/CANopen protocol sections appear to be generic boilerplate shared across the RobStride motor manual family (the mechanical/electrical spec pages, e.g. p.7-10, are the only RS02-IP67-specific content identified, and those are not CAN-protocol related).
