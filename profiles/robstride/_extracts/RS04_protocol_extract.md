# RS04 protocol extract
Source: RS04User Manual260713.pdf
PDF page count: 86 pages (confirmed via `pdfinfo`). Content ends with "7. Version History" on printed page 85 (PDF page 86 is blank/trailing).

## Model/firmware identification
- Model name as printed on cover: "ROBSTRIDE 04", subtitle "120N.M Quasi-Direct Drive Integrated Motor Module", "User Manual". Manufacturer: RobStride Dynamics (https://www.robstride.com/). (p.1)
- Body text repeatedly refers to the product as "RS04" (e.g. header "RS04 instruction manual", p.2).
- Legal/manufacturer entity: "Beijing Lingfoot Times Technology Co., LTD. (hereinafter referred to as Lingfoot)" (p.2–3).
- Manual version history: Version Number "1.0", Description "Initial Release", Date "November 25, 2025" (p.85). Note: this postdates the file name date-stamp "260713"; printed as-is, not reconciled further.
- Firmware/version fields visible as live example values in the parameter-table screenshots (these are sample "CurrentValue" readings from a specific unit, not stated as the shipped default firmware version):
  - BootCodeVersion = "0.1.5", BootBuildDate = "Mar 16 2022", BootBuildTime = "20:22:09" (p.22)
  - AppCodeVersion = "0.4.1.24" ("Motor program version number") (p.22)
  - AppGitVersion = "7b844b0fM", AppBuildDate = "Dec 8 2025", AppBuildTime = "20:30:22", AppCodeName = "Lingzu_motor" (p.22)
  - Elsewhere the manual gates several newer CAN commands behind "requires firmware update to 0.4.1.29" (Commands 12–15 of the MIT protocol section, p.82–83).
- No single explicit "default CAN ID" value is stated as a factory default. The parameter table's CAN_ID entry shows "CurrentValue (for reference)" = 127 (p.23), and CAN_MASTER (host id) shows CurrentValue = 253 (0xFD) (p.23). Multiple MIT-protocol example frames use host id 0xFD / 0x0FD (e.g. p.77, p.52), which is consistent with 253 being the conventional/default host CAN ID, but the manual never calls this out explicitly as "default."
- CAN ID range: CAN_ID parameter table row gives Maximum value 127, Minimum value 0, type uint8 (p.23) — i.e., motor CAN IDs are documented as 0–127.

## CAN bus parameters
- "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted" (p.42).
- Driver Product Specifications table (p.13):
  - The rated working voltage: 48VDC
  - The maximum allowable voltage: 60VDC
  - Rated working phase current: 27Apk
  - Maximum allowable phase current: 90Apk
  - Standby power: ≤40mA
  - CAN bus bit rate: 1Mbps
  - Dimensions: Φ84mm
  - Working environment temperature: -20°C to 50°C
  - The maximum allowable temperature of the control board: 145°C
  - encoder resolution: 14bit (absolute turn)
- Driver interface definition (connector silkscreen labels): CAN-H, CAN-L, GND, VBAT+ (p.13).
- Driver harness definition: blue = CAN_H, brown = CAN_L, black = GND, red = VBAT+ (p.13–14).
- Driver function pin description (power supply + CAN connector, 4-pin): Pin1 = positive electrode of power supply (+), Pin2 = negative electrode of power supply (-), Pin3 = CAN CAN_L, Pin4 = CAN the high side of the communication CAN_H (p.14).
- CAN-to-USB debug tool notes (p.15): recommended tool is "the official USB-CAN module of Lingzu Times." Serial frame header = "41 54", frame tail = "0D 0A". DIP switch 1 ON = Boot mode (no host connection). DIP switch 2 ON = connects a 120Ω terminal resistor to the module port, "allowing normal communication with the host computer."
- Baud-rate modification frame (Communication type 23, p.47–48) documents alternate selectable bit rates via an F_CMD byte: 01 = 1M, 02 = 500K, 03 = 250K, 04 = 125K. (These are configuration options, not necessarily all "supported" bitrates for shipped default — default per p.13/p.42 is 1Mbps.)
- MIT Communication Protocol section repeats: "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol." (p.77)

## Frame definitions (TX — host to motor)
All frames below use the CAN 2.0B **extended (29-bit) arbitration ID** format, except the MIT protocol and CANopen sections which use the **standard (11-bit) ID** format (noted explicitly in-document, see "MIT/CANopen" subsection below).

### General extended-frame layout (private/default protocol) — p.42
| data field | 29-bit ID (bit28~bit24) | 29-bit ID (bit23~8) | 29-bit ID (bit7~0) | 8-Byte data field |
|---|---|---|---|---|
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
| Description | Communication type | data area 2 | Destination address | data area 1 |

This is confirmed by the C struct given in the program-sample section (p.60):
```
struct exCanIdInfo{
  uint32_t id:8;
  uint32_t data:16;
  uint32_t mode:5;
  uint32_t res:3;
};
```
i.e. bits [7:0] = target/destination CAN_ID, bits [23:8] = "data" (16-bit field whose meaning is redefined per communication type), bits [28:24] = "mode" = communication type number, bits [31:29] = reserved/unused (3 bits, consistent with 29-bit extended ID: 8+16+5=29).

"The control modes supported by the motor include: Operation control mode (set 5 parameters of motor operation control); Current mode (specified Iq current); Velocity mode (specified running speed); Position mode (specified target position)." (p.42)

### Communication Type 0: Get device ID — p.42
Request:
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x0 | bit15~8: identifies host CAN_ID | target motor CAN_ID | 0 |

Reply:
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x0 | target motor CAN_ID | 0xFE | 64-bit MCU unique identifier |

### Communication Type 1: Operation control mode motor control instruction — p.43
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x1 | Byte2: Torque (0~65535) corresponds to (-120Nm~120Nm) | target motor CAN_ID | Byte0~1: target Angle [0~65535] corresponds to (-4π~4π); Byte2~3: Target angular velocity [0~65535] corresponds to (-15rad/s~15rad/s); Byte4~5: Kp [0~65535] corresponds to (0.0~5000.0); Byte6~7: Kd [0 to 65535] corresponds to (0.0 to 100.0). After the conversion, the high byte is in front and the low byte is in [front — text cuts off, see OPEN QUESTIONS] |

As printed, the table literally places "Byte2: Torque (0~65535) corresponds to (-120Nm~120Nm)" in the **bit23~8 column of the 29-bit arbitration ID** (not in the 8-byte data payload) — i.e., the torque command is carried in the CAN ID itself for this frame type, while angle/velocity/Kp/Kd are carried in the 8-byte data field. This is transcribed exactly as printed; see OPEN QUESTIONS for the ambiguity this creates.

Response frame: "Response motor feedback frame (see communication type 2)" (p.43).

Program-sample code (p.61–63) confirms the scaling/packing for this frame:
```
void motor_controlmode(uint8_t id, float torque, float MechPosition, float speed, float kp, float kd)
{
  txCanIdEx.mode = 1;
  txCanIdEx.id = id;
  txCanIdEx.res = 0;
  txCanIdEx.data = float_to_uint(torque,T_MIN,T_MAX,16);
  txMsg.tx_dlen = 8;
  txMsg.tx_data[0]=float_to_uint(MechPosition,P_MIN,P_MAX,16)>>8;
  txMsg.tx_data[1]=float_to_uint(MechPosition,P_MIN,P_MAX,16);
  txMsg.tx_data[2]=float_to_uint(speed,V_MIN,V_MAX,16)>>8;
  txMsg.tx_data[3]=float_to_uint(speed,V_MIN,V_MAX,16);
  txMsg.tx_data[4]=float_to_uint(kp,KP_MIN,KP_MAX,16)>>8;
  txMsg.tx_data[5]=float_to_uint(kp,KP_MIN,KP_MAX,16);
  txMsg.tx_data[6]=float_to_uint(kd,KD_MIN,KD_MAX,16)>>8;
  txMsg.tx_data[7]=float_to_uint(kd,KD_MIN,KD_MAX,16);
  can_txd();
}
```
with macro constants (p.59):
```
#define P_MIN -12.57f
#define P_MAX 12.57f
#define V_MIN -15.0f
#define V_MAX 15.0f
#define KP_MIN 0.0f
#define KP_MAX 5000.0f
#define KD_MIN 0.0f
#define KD_MAX 100.0f
#define T_MIN -120.0f
#define T_MAX 120.0f
```
Note: these code macros use ±12.57 rad for position range (≈ -4π~4π, since 4π≈12.566), consistent with the table's "(-4π~4π)" notation, and ±15.0 rad/s for velocity (consistent with "(-15rad/s~15rad/s)"), and 0–5000.0 for Kp and 0–100.0 for Kd, and ±120.0 N·m for torque — matching the table's stated ranges.

### Communication Type 3: Motor enabled to run — p.44
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x3 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | (blank in table) |

Response frame: "Response motor feedback frame (see communication type 2)" (p.44).
Program sample (p.61) sends id in bits[7:0], host/master id in the "data" field, dlen=8, all data bytes implicitly 0 (txCanIdEx.data = 0 set after dlen).

### Version number read frame — p.44
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | Byte[0]=0x00, Byte[1]=0xC4 |

Response frame (p.44), described using 0x2 (feedback) framing with version payload:
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none, 1 has); bit21: uncalibrated; bit20: Uncalibrated / bit20: Gridlock overload fault [text duplicates bit20, see OPEN QUESTIONS]; bit19: magnetic coding fault; bit18: overtemperature; bit17: Three-phase overcurrent fault; bit16: undervoltage fault; bit22~23: Mode status — 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run] | target motor CAN_ID | Byte0=0x00, Byte1=0xC4, Byte2=0x56, Byte3~6: Motor version number, order from high to low |

### Communication Type 4: Motor stops running — p.44–45
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x4 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | "When the motor is running normally, 0 must be cleared in the data field. Byte[0]=1: The fault is cleared." |

Response frame: "Response motor feedback frame (see communication type 2)" (p.45).

### Communication type 6: Set motor mechanical zero — p.45
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x6 | bit15~8: Identifies the main CAN_ID | target motor CAN_ID | Byte[0]=1 |

Response frame: "Response motor feedback frame (see communication type 2)" (p.45).

### Communication type 7: Set motor CAN_ID — p.45
"Change the current motor CAN_ID, effective immediately."
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x7 | bit15~8: used to identify main CAN_ID; Bit16~23: preset CAN_ID | Target motor CAN_ID | (blank) |

Answer frame: "Answer motor broadcast frame (see communication type 0)" (p.45). Note: bit field description "Bit16~23: preset CAN_ID" overlaps with "bit15~8: ... main CAN_ID" — transcribed exactly as printed (bit16 appears in both ranges); see OPEN QUESTIONS.

### Communication type 17: Single parameter read — p.45–46
Request:
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x11 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below; Byte2~3: 00; Byte4~7: In data above 00, the low byte is first and the high byte is second |

Reply frame (p.46):
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x11 | bit15~8: indicates the master CAN_ID; Bit23~16: 00 indicates that the master CAN_ID is successfully read, 01 indicates that the master can_ID ... [text cuts off] | (not separately given — column appears to continue from previous) | Byte0~1: index; Byte2~3: 00; Byte4~7: Parameter data. 1 byte of data above Byte4 is preceded by low bytes and followed by high bytes at [text cuts off] |
Note: this reply-frame row's column boundaries are unclear in the extracted text (the "bit7~0" column content is not distinctly separated from the bit23~8 description, and both the request/reply byte-order descriptions are truncated mid-sentence by a page/column break). Transcribed as printed; see OPEN QUESTIONS.

### Communication type 18: Single parameter write (lost in power failure) — p.46
"With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved"
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x12 | bit15~8: Used to identify the main CAN_ID | target motor CAN_ID | Byte0~1: index. For details, see the readability parameter table below; Byte2~3: 00; Byte4~7: Parameter data. In the preceding data, the low byte is in the front and the high byte is in the rear |

Response frame: "Response motor feedback frame (see communication type 2)" (p.46).

### Communication type 21: Fault feedback frame — p.46–47
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x15 | bit15~8: identifies the main CAN_ID | CAN_ID [of target motor, inferred from column position] | Byte0~3: fault value (non-0: faulty; 0: Normal). bit14: gridlock overload fault; Bit16: A-phase current sampling overcurrent; Bit14: Motor stall overload algorithm protection [bit14 listed twice — see OPEN QUESTIONS]; Bit9: Position initialization fault; Bit8: Hardware identification fault; Bit7: Encoder uncalibrated (Motor encoder not calibrated); Bit5: C-phase current sampling overcurrent; Bit4: B-phase current sampling overcurrent; bit3: overvoltage fault; bit2: undervoltage fault; bit1: driver chip fault; bit0: motor overtemperature fault, Default 145°C. Byte4~7: warning Value, bit0: motor overtemperature warning, the default is 135°C |

(p.46–47, bit list spans a page break; transcribed in full, duplication of bit14 as printed.)

### Communication type 22: Motor data save frame — p.47
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x16 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 07 08 |

Response frame: "Response motor feedback frame (see communication type 2)" (p.47).

### Communication type 23: Motor baud rate modification frame (re-power-on effect) — p.47–48
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x17 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD. "Among them, the F_CMD byte is the motor baud rate: 01 is 1M, 02 is 500K, 03 is 250K, 04 is 125K" |

Response frame: "Response motor feedback frame (see communication type 0)" (p.48). (Note: this contradicts the type-0-style response pattern used elsewhere which normally references type 2; transcribed as printed.)

### Communication type 24: The motor actively reports frames — p.48
Request (enable/disable active reporting):
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x18 | bit15~8: identifies the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD. "F_CMD byte is the motor reporting switch: 00 is to disable active reporting (default); 01 To enable active reporting, the default reporting interval is 10ms" |

Response frame (active report payload), same byte layout as Communication Type 2 (motor feedback data):
| data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x18 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault info (0 none, 1 has): bit21 uncalibrated, bit20 Uncalibrated / bit20 Gridlock overload fault [duplicate bit20 as printed], bit19 magnetic coding fault, bit18 overtemperature, bit17 three-phase overcurrent fault, bit16 undervoltage fault; bit22~23: Mode status 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run] | target motor CAN_ID | Byte0~1: Current Angle [0~65535] corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-15rad/s~15rad/s); Byte4~5: Current torque [0~65535] corresponds to (-120Nm~120Nm); Byte6~7: Current temperature: Temp(Celsius)*10. "If the value is higher than 10, the high byte is first and the low byte is last" |

(p.48)

### Communication type 25: Motor protocol modification frame (re-power-on effect) — p.49
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| description | 0x19 | bit15~8: used to identify the main CAN_ID | target motor CAN_ID | 01 02 03 04 05 06 F_CMD. "Among them, the F_CMD byte is the motor protocol type: 0 is a private protocol (default), 1 is the Canopen protocol, 2 is the MIT protocol" |

Response frame: "Response motor feedback frame (see communication type 0)" (p.49).

### Read example (type 17), worked example — p.52
"Take reading loc_kp as an example:"
Read instruction:
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| | 0x11 | 0x00FD | 0x7F | 1E 70 00 00 00 00 00 00 |
| Description | Type 17 | Host id 0xFD | Target motor CAN_ID 7F | Byte0~1: index, corresponding to loc_kp |

Feedback instruction:
| Size | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| | 0x11 | 0x007F | 0xFD | 1E 70 00 00 00 00 F0 41 |
| Description | Type 17 | bit15~8: Target motor CAN_ID 7F | Host id 0xFD | Byte0~1: index, corresponding to loc_kp; Byte4~7: loc_kp value 30, high right byte, (32-bit single precision) hexadecimal IEEE-754 standard floating point number |

### Oscilloscope communication-box worked example — p.30
"Communication box instruction example: 41 54 90 07 e8 0c 08 05 70 00 00 01 00 00 00 0d 0a"
| 41 54 | 90 07 e8 0c | 8 | 05 70 00 00 01 00 00 00 | 0d 0a |
|---|---|---|---|---|
| frame header | Number of data bits / extended frame / "00" | data frame | | frame tail |
(column structure of this row is unclear as printed — header row has "00" spanning only part of the row; see OPEN QUESTIONS)

"The translation of extended frame canid into real canid requires the following transformations: 90 07 e8 0c converts to binary as 1001 0000 0000 0111 1110 1000 0000 1100, remove the 100 on the right and it becomes 1 0010 0000 0000 1111 1101 0000 0001, convert it to hexadecimal, It is 12 00 FD 01." (p.31)
| 12 in hexadecimal | 0 | FD | 1 |
|---|---|---|---|
| Communication type 18 (in decimal base) | No meaning | host id | motor canid |
(p.31)

## Frame definitions (RX — motor to host)
### Communication Type 2: Motor feedback data — p.43 (also reused as the response for types 1, 3, 4, 6, 18, 22, and as the active-report payload for type 24)
| Data field | Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7 |
|---|---|---|---|---|
| Description | 0x2 | Bit8~Bit15: CAN ID of the current motor; bit21~16: fault information (0 none, 1 has): bit21 uncalibrated, bit20 Uncalibrated, bit20 Gridlock overload fault [bit20 duplicated as printed], bit19 magnetic coding fault, bit18 overtemperature, bit17 Three-phase overcurrent fault, bit16 undervoltage fault; bit22~23: Mode status — 0: Reset mode [reset], 1: Cali mode [calibration], 2: Motor mode [Run] | host CAN_ID | Byte0~1: The current Angle [0~65535] Corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-15rad/s~15rad/s); Byte4~5: Current torque [0~65535] corresponds to (-120Nm~120Nm); Byte6~7: Current temperature: Temp(Celsius)*10. "If the value is higher than 10, the high byte is first and the low byte is last" |

### Communication Type 0 reply (Get device ID) — p.42, see TX section above.
### Version read reply (type 4) — p.44, see TX section above.
### Fault feedback frame (type 21) — p.46–47, see TX section above (this is itself a RX/feedback frame originated unsolicited or on request; table does not explicitly distinguish direction beyond context — transcribed as shown).

## Parameter table (flash config read/write)
Two distinct parameter tables appear in the manual. The manual does not explicitly state how/whether these two index spaces are related; see OPEN QUESTIONS.

### Table A — Host-computer "Parameter Setting" table, function codes 0x0000–0x3048 (p.21–29)
Columns as printed: function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | 备注(remarks)

| code | name | type | attribute | max | min | current (sample) | remarks |
|---|---|---|---|---|---|---|---|
| 0X0000 | Name | String | Read/Write | | | (device name string) | |
| 0X0001 | BarCode | String | Read/Write | | | (barcode string) | |
| 0X1000 | BootCodeVersion | String | Read only | | | 0.1.5 | |
| 0X1001 | BootBuildDate | String | Read only | | | Mar 16 2022 | |
| 0X1002 | BootBuildTime | String | Read only | | | 20:22:09 | |
| 0X1003 | AppCodeVersion | String | Read only | | | 0.4.1.24 | Motor program version number |
| 0X1004 | AppGitVersion | String | Read only | | | 7b844b0fM | |
| 0X1005 | AppBuildDate | String | Read only | | | Dec 8 2025 | |
| 0X1006 | AppBuildTime | String | Read only | | | 20:30:22 | |
| 0X1007 | AppCodeName | String | Read only | | | Lingzu_motor | |
| 0X2000 | echoPara1 | uint16 | disposition | 115 | 5 | 5 | |
| 0X2001 | echoPara2 | uint16 | disposition | 115 | 5 | 5 | |
| 0X2002 | echoPara3 | uint16 | disposition | 115 | 5 | 5 | |
| 0X2003 | echoPara4 | uint16 | disposition | 115 | 5 | 5 | |
| 0X2004 | echoFreHz | uint32 | Read/Write | 10000 | 1 | 1000 (table p.23) / 500 (table p.21, same field, different sample value) | |
| 0X2005 | MechOffset | float | Read/Write | 50 | -50 | 5.844083 | Motor magnetic encoder Angle offset |
| 0X2006 | chasu_offset | float | Read/Write | 50 | -50 | 4.74652 | Reserved parameters |
| 0X2007 | Status1 | float | Settings | 10 | -10 | 0 | Torque limitation |
| 0X2008 | I_FW_MAX | float | Read/Write | 33 | 0 | 0 | Weak magnetic current value, default 0 |
| 0X2009 | CAN_ID | uint8 | Read/Write | 127 | 0 | 127 | id of this object |
| 0X200a | CAN_MASTER | uint8 | Read/Write | 300 | 0 | 253 | can host id (NOTE: max=300 exceeds uint8 range; see OPEN QUESTIONS) |
| 0X200b | CAN_TIMEOUT | uint32 | Read/Write | 100000 | -10000 | 0 | can timeout threshold. The default value is 0 (NOTE: min=-10000 on a uint32 field; see OPEN QUESTIONS) |
| 0X200c | status2 | int16 | Settings | 1500 | -200 | 0 | Reserved parameter |
| 0X200d | status3 | uint32 | Settings | 1000000 | 1000 | 0 | Reserved parameter |
| 0X200e | status4 | float | Settings | 64 | 1 | 0 | Reserved parameter |
| 0X200f | status5 | float | Settings | 20 | 0 | 0 | Reserved parameter |
| 0X2010 | status6 | uint8 | Settings | 1 | 0 | 0 | Reserved parameter |
| 0X2011 | cur_filt_gain | float | Read/Write | 1 | 0 | 0.9 | Current filtering parameter |
| 0X2012 | cur_kp | float | Read/Write | 200 | 0 | 0.025 | Current kp |
| 0X2013 | cur_ki | float | Read/Write | 200 | 0 | 0.0258 | Current ki |
| 0X2014 | spd_kp | float | Read/Write | 200 | 0 | 2 | Velocity kp |
| 0X2015 | spd_ki | float | Read/Write | 200 | 0 | 0.021 | Speed ki |
| 0X2016 | loc_kp | float | Read/Write | 200 | 0 | 30 | Position kp |
| 0X2017 | spd_filt_gain | float | Read/Write | 1 | 0 | 0.1 | Velocity filter parameter |
| 0X2018 | limit_spd | float | Read/Write | 200 | 0 | 2 | Location mode speed limit |
| 0X2019 | limit_cur | float | Read/Write | 23 | 0 | 23 | Position, Velocity mode current limit |
| 0X201a | spd_step_value | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201b | vel_max | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201c | acc_set | float | Read/Write | 27 | 0 | 0 | High speed segment offset |
| 0X201d | fault1 | uint32 | Settings | 27 | 0 | 0 | Reserved parameters |
| 0X201e | fault2 | uint32 | Settings | 150 | 0 | (blank) | Reserved parameters |
| 0X201f | fault3 | uint32 | Settings | 20 | 0 | (blank) | Reserved parameters |
| 0X2020 | fault4 | uint32 | Settings | 1000 | 0 | (blank) | Reserved parameters |
| 0X2021 | fault5 | uint32 | Settings | 100 | 0 | 0 | Reserved parameters |
| 0X2022 | fault6 | uint32 | Settings | (blank) | (blank) | (blank) | Reserved parameters |
| 0X2023 | fault7 | uint32 | Settings | (blank) | (blank) | (blank) | Reserved parameters |
| 0X2024 | baud | uint8 | Read/Write | (blank) | (blank) | (blank) | Baud rate flag |
| 0X2025 | zero_sta | uint8 | Read/Write | (blank) | (blank) | (blank) | Zero point flag |
| 0X2026 | position_offset | float | Read/Write | (blank) | (blank) | (blank) | Position offset |
| 0X2027 | protocol_1 | uint8 | Read/Write | (blank) | (blank) | (blank) | Protocol flag |
| 0X2028 | damper | uint8 | Read/Write | (blank) | (blank) | (blank) | Damping switch |
| 0X2029 | add_offset | float | Read/Write | (blank) | (blank) | (blank) | Position offset parameter |
| 0X3000 | timeUse0 | uint16 | Read only | | | 5 | |
| 0X3001 | timeUse1 | uint16 | Read only | | | 0 | |
| 0X3002 | timeUse2 | uint16 | Read only | | | 10 | |
| 0X3003 | timeUse3 | uint16 | Read only | | | 0 | |
| 0X3004 | encoderRaw | uint16 | Read only | | | 11396 | Magnetic encoder sampling value |
| 0X3005 | mcuTemp | int16 | Read only | | | 337 | mcu internal temperature, *10 |
| 0X3006 | motorTemp | int16 | Read only | | | 333 | Motor ntc temperature, *10 |
| 0X3007 | encoder2raw | uint16 | Read only | | | 24195 | "Bus voltage" (as printed; name suggests encoder but remark says bus voltage — see OPEN QUESTIONS) |
| 0X3008 | adc1Offset | int32 | Read only | | | 2084 | adc sampling channel 1 Zero current bias |
| 0X3009 | adc2Offset | int32 | Read only | | | 2084 | adc sampling channel 2 Zero current bias |
| 0X300a | adc1Raw | uint16 | Read only | | | 1232 | adc sampling value 1 |
| 0X300b | adc2Raw | uint16 | Read only | | | 1212 | adc sampling value 2 |
| 0X300c | VBUS | float | Read only | | | 36 | Bus voltage V |
| 0X300d | cmdId | float | Read only | | | 0 | id ring instruction, A |
| 0X300e | cmdIq | float | Read only | | | 0 | iq ring command, A |
| 0X300f | cmdlocref | float | Read only | | | 0 | Position loop command, rad |
| 0X3010 | cmdspdref | float | Read only | | | 0 | Speed loop command, rad/s |
| 0X3011 | cmdspdref | (blank — row appears to be a duplicate/continuation) | | | | | |
| 0X3012 | cmdTorque | float | Read only | | | 0 | Torque instruction, nm |
| 0X3013 | cmdPos | float | Read only | | | 0 | mit Protocol Angle instruction |
| 0X3014 | cmdVel | float | Read only | | | 0 | mit Protocol Speed instruction |
| 0X3015 | rotation | int16 | Read only | | | 1 | Number of turns |
| 0X3016 | modPos | float | Read only | | | 4.363409 | Motor uncounted coil mechanical Angle, rad |
| 0X3017 | mechPos | float | Read only | | | 0.777679 | Load end loop mechanical Angle, rad |
| 0X3018 | mechVel | float | Read only | | | 0.036618 | Load speed: rad/s |
| 0X3019 | elecPos | float | Read only | | | 4.714761 | Electrical Angle |
| 0X301a | ia | float | Read only | | | 0 | U-wire current, A |
| 0X301b | ib | float | Read only | | | 0 | V-wire current, A |
| 0X301c | ic | float | Read only | | | 0 | W-wire current, A |
| 0X301d | timeout | uint32 | Read only | | | 31600 | Timeout counter value |
| 0X301e | phaseOrder | uint8 | Read only | | | 0 | Directional marking |
| 0X301f | iqf | float | Read only | | | 0 | iq filter value, A |
| 0X3020 | boardTemp | int16 | Read only | | | 359 | Plate temperature, *10 |
| 0X3021 | iq | float | Read only | | | 0 | iq Original value, A |
| 0X3022 | id | float | Read only | | | 0 | id Original value, A (NOTE: this is the Function code referenced by "Motor fault instructions" §3.3.7 as the fault code register, which conflicts with the name "id"/"id Original value" here — see OPEN QUESTIONS) |
| 0X3023 | faultSta | uint32 | Read only | | | 0 | Fault status value |
| 0X3024 | warnSta | uint32 | Read only | | | 0 | Warning status value |
| 0X3025 | drv_fault | uint16 | Read only | | | 0 | "The driver chip fault value is 1" (NOTE: this is the register documented in detail as "driver chip fault code 1" / Table 11, see Fault/error codes section — index given there as 0x3024, conflicting with this row's 0x3025; see OPEN QUESTIONS) |
| 0X3026 | drv_temp | int16 | Read only | | | 48 | "The driver chip fault value is 2" (text as printed; likely mislabeled remark — see OPEN QUESTIONS) |
| 0X3027 | Uq | float | Read only | | | 0 | Q-axis voltage |
| 0X3028 | as_angle | float | Read only | | | 0.08667 | Magnetic encoder initial angle |
| 0X3029 | cs_angle | float | Read only | | | 1.547787 | Differential magnetic encoder initial angle |
| 0X302a | chasu_angle | float | Read only | | | 4.822069 | Differential angle |
| 0X302b | ibus | float | Read only | | | 0 | Motor protection parameters |
| 0X302c | torque_fdb | float | Read only | | | 0 | Torque feedback value, Nm |
| 0X302d | rated_i | float | Read only | | | 19 | Motor protection parameters |
| 0X302e | MechPos_init | float | Read only | | | 0 | Motor reserved parameters |
| 0X302f | vel_max | float | Read only | | | 10 | Motor speed setpoint |
| 0X3030 | loc_reff | float | Read only | | | 0.05876 | Motor speed expectation 2 |
| 0X3031 | instep | float | Read only | | | 0 | Motor position determination parameters |
| 0X3032 | position | float | Read only | | | 6.372923 | Motor position determination parameters |
| 0X3033 | chasu_angle_init | float | Read only | | | 0.075549 | Motor position determination parameters |
| 0X3034 | chasu_angle_out | float | Read only | | | 0.679938 | Motor position determination parameters |
| 0X3035 | motormechinit | float | Read only | | | 0.52884 | Motor position determination parameters |
| 0X3036 | mech_angle_init2 | float | Read only | | | 0.05876 | Motor position determination parameters |
| 0X3037 | mech_angle_rotat | int16 | Read only | | | 0 | Motor position determination parameters |
| 0X3038 | fault1 | uint32 | Read only | | | 0 | Log failure |
| 0X3039 | fault2 | uint32 | Read only | | | 0 | Log failure |
| 0X303a | fault3 | uint32 | Read only | | | 0 | Log failure |
| 0X303b | fault4 | uint32 | Read only | | | 0 | Log failure |
| 0X303c | fault5 | uint32 | Read only | | | 0 | Log failure |
| 0X303d | fault6 | uint32 | Read only | | | 0 | Log failure |
| 0X303e | fault7 | uint32 | Read only | | | 0 | Log failure |
| 0X303f | fault8 | uint32 | Read only | | | 1 | Log failure |
| 0X3040 | ElecOffset | float | Read only | | | 0 | electrical Angle offset |
| 0X3041 | mcOverTemp | int16 | Read only | | | 0 | Overtemperature threshold |
| 0X3042 | Kt_Nm/Amp | float | Read only | | | 0 | Moment coefficient |
| 0X3043 | Tqcali_Type | uint8 | Read only | | | 0 | Motor type |
| 0X3044 | theta_mech_1 | float | Read only | | | 0.05876 | Type 2 Low-Speed Angle |
| 0X3045 | adcOffset_1 | int32 | Read only | | | 2041 | ADC Sampling Channel 1 Zero Current Offset |
| 0X3046 | adcOffset_2 | int32 | Read only | | | 2040 | ADC Sampling Channel 2 Zero Current Offset |
| 0X3047 | coder_reg | uint16 | Read only | | | 1 | System Parameters |
| 0X3048 | pos_cnt1 | uint16 | Read only | | | 0 | System Parameters |

(p.21–29; blank cells are blank as printed in the sample screenshot/table, not necessarily unspecified limits.)

### Table B — "Read and write a single parameter list" (used with Communication type 17/18 over CAN), index 0x7005–0x702E (p.50–52)
Columns as printed: index | name | Description | Type | Number of bytes | (range, printed as a blended column with "Number of bytes") | R/W Read and write permission

| index | name | Description | Type | bytes | range | R/W |
|---|---|---|---|---|---|---|
| 0X7005 | run_mode | "0: operation mode, 1: position mode (PP), 2: Velocity mode, 3: Operation mode Current mode, 5: Position mode (CSP)" (NOTE: no value 4 listed; "3: Operation mode Current mode" wording as printed — see OPEN QUESTIONS) | uint8 | 1 | | W/R |
| 0X7006 | iq_ref | Current mode Iq command | float | 4 | -90 to 90A | W/R |
| 0X700A | spd_ref | Rotational Velocity mode Rotational speed command | float | 4 | -20 to 20rad/s | W/R |
| 0X700B | limit_torque | torque limit | float | 4 | 0 to 120Nm | W/R |
| 0X7010 | cur_kp | Kp | float | 4 | default value 0.17 | W/R |
| 0X7011 | cur_ki | Ki | float | 4 | default value 0.012 | W/R |
| 0X7014 | cur_filt_gain | filt_gain | float | 4 | 0 to 1.0, default value 0.1 | W/R |
| 0X7016 | loc_ref | Position Mode Angle instruction | float | 4 | rad | W/R |
| 0X7017 | limit_spd | Location mode (CSP) speed limit | float | 4 | 0 to 20rad/s | W/R |
| 0X7018 | limit_cur | Velocity position mode Current limitation | float | 4 | 0 to 90A | W/R |
| 0x7019 | mechPos | Mechanical Angle of the loading coil | float | 4 | rad | R |
| 0x701A | iqf | iq Filter | float | 4 | -90 to 90A | R |
| 0x701B | mechVel | Speed of the load | float | 4 | -15 to 15rad/s | R |
| 0x701C | VBUS | Bus voltage | float | 4 | V | R |
| 0x701E | loc_kp | "at" (description as printed, apparently truncated) | float | 4 | default value 60 | W/R |
| 0x701F | spd_kp | Indicates the speed kp | float | 4 | default value 6 | W/R |
| 0x7020 | spd_ki | ki | float | 4 | default value 0.02 | W/R |
| 0x7021 | spd_filt_gain | Speed filter value | float | 4 | default value 0.1 | W/R |
| 0x7022 | acc_rad | velocity mode acceleration | float | 4 | default value 15rad/s^2 | W/R |
| 0x7024 | vel_max | Location mode (PP) speed | float | 4 | default value 10rad/s | W/R |
| 0x7025 | acc_set | Location mode (PP) acceleration | float | 4 | default value 10rad/s^2 | W/R |
| 0x7026 | EPScan_time | "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | default value 1 | W/R |
| 0x7028 | canTimeout | can The timeout threshold, 20000 is 1s | uint32 | 4 | default value 0 | W/R |
| 0x7029 | zero_sta | "Indicates the zero flag bit, 0 means 0-2π and 1 means -π-π" | uint8 | 1 | default is 0 | W/R |
| 0x702A | damper | Damping switch | uint8 | 1 | default value 0 | W/R |
| 0x702B | add_offset | Zero offset | float | 4 | default is 0 | W/R |
| 0x702C | alveolous_open | Cogging Compensation Switch | uint8 | 1 | default is 0 | W/R |
| 0x702D | iq_test | Motor Initialization Calibration Switch | uint8 | 1 | default is 0 | W/R |
| 0x702E | dcc_set | Deceleration in Motor PP Mode (Normally Disabled) | float | 4 | default is 10rad/s^2 | W/R |

(p.50–52)

### CANopen object-dictionary style parameters (p.71–75)
Status Feedback Parameters (read-only unless noted):
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 603F | Error_code | Read-only | UINTEGER16 | / |
| 6041 | Statusword | Read-only | UINTEGER16 | / |
| 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
| 6062 | Position_demand_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6064 | Position_actual_value | Read-only | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 606B | Velocity_demand_value | Read-only | INTEGER32 | 0.1 rpm |
| 606C | Velocity_actual_value | Read-only | INTEGER32 | 0.1 rpm |
| 6077 | Torque_actual_value | Read-only | INTEGER16 | 0.1% load ratio (1000 = 40 N·m) |
| 6078 | Current_actual_value | Read-only | INTEGER16 | mA |
| 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | mV |

(p.71–72)

Homing Mode (Zero Position Setting), p.72:
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
"Set Modes of operation to 6 while the motor is in the disabled state (SWITCH_ON_DISABLED). The motor will then define the current position as the zero point." "To hold the zero position, modify the Controlword to 15."

Position Mode (PP – Profile Position), p.72–73:
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6067 | Position_window | Read-write | UINTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6068 | Position_window_time | Read-write | UINTEGER16 | ms |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 40N·m) |
| 607A | Target_position | Read-write | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6081 | Profile_velocity | Read-write | UINTEGER32 | 0.1 rpm |
| 6083 | Profile_acceleration | Read-write | UINTEGER32 | 0.1 rpm/s |
Steps: set Modes of operation=1 (disabled state); mandatory params Target_torque, Profile_velocity, Profile_acceleration; optional Position_window / Position_window_time; set Controlword=15 to enable; set Target_position.

Position Mode (CSP – Cyclic Synchronous Position), p.73–74:
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6067 | Position_window | Read-write | UINTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6068 | Position_window_time | Read-write | UINTEGER16 | ms |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 40 N·m) |
| 607A | Target_position | Read-write | INTEGER32 | Pulses (1 rev = 16,384 pulses) |
| 6081 | Profile_velocity | Read-write | UINTEGER32 | 0.1 rpm |
Steps: set Modes of operation=5 (disabled state); mandatory Target_torque, Profile_velocity; optional Position_window/time (0 = disabled); Controlword=15; set Target_position.

Velocity Mode, p.74–75:
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 40 N·m) |
| 60FF | Target_velocity | Read-write | INTEGER32 | 0.1 rpm |
Steps: Modes of operation=3 (disabled state); mandatory Target_torque; Controlword=15; set Target_velocity.

Torque Mode, p.75:
| Index | Name | Attribute | Type | Unit |
|---|---|---|---|---|
| 6040 | Controlword | Read-write | UINTEGER16 | / |
| 6060 | Modes of operation | Read-write | INTEGER8 | / |
| 6071 | Target_torque | Read-write | INTEGER16 | 0.1% load ratio (1000 = 40N·m) |
Steps: Modes of operation=4 (disabled state); Controlword=15; set Target_torque.

CANopen state machine controlword values (p.70–71): Controlword=6,7,15 step transitions SWITCH_ON_DISABLED→READY_TO_SWITCH_ON→SWITCHED_ON→OPERATION_ENABLE (or 15 directly for immediate enable); Controlword=1 → back to SWITCH_ON_DISABLED to stop; Controlword=11 → emergency stop (quick-stop active), "Use with Caution—Risk of Voltage Surge." Motor watchdog: object 0x6099 index 1, value 20000 = 1 second, default 0 = disabled, non-zero enables.

Protocol Switching (Extended Frame), p.75–76:
| Data Field | 29-bit ID | 8-Byte Data Area |
|---|---|---|
| Size | Bit 28~0 | Byte 0~6 |
| Description | 0xFFF | 01 02 03 04 05 06 F_CMD |
F_CMD (Byte 6) defines the motor protocol: 0 = Private protocol (default), 1 = CANopen protocol, 2 = MIT protocol.
Response Frame:
| Data Field | 11-bit ID | 8-Byte Data Area |
|---|---|---|
| Size | Bit 10~0 | Byte 0~7 |
| Description | Motor ID | 64-bit MCU unique identifier |

## MIT protocol frames (Section 6, p.77–84)
Standard (11-bit) CAN ID frame format: "Data Field: Size bit10~8 = Mode type, bit7~0 = ID; 8-byte Data Area Byte0~7" (p.77).

Supported control modes within MIT protocol: "MIT Mode: Provides five motion control parameters to the motor." "Velocity Mode: Specifies the target speed for the motor." "Position Mode: Specifies the target position and speed..." (p.77)

- Response Command 1: Data Feedback (Motor Status), p.77–78. 11-bit ID field = Host ID; example ID value "0FD". Data: Byte0: Motor CAN ID; Byte1~2: Current angle [0~65535] ↔ (-12.57 rad ~ 12.57 rad); Byte3 high 8 bits + Byte4[7:4] (high 4 bits) as low 4 bits: Current speed [0~4096] ↔ (-15 rad/s ~ 15 rad/s); Byte4[3:0] (low 4 bits) as high 4 bits + Byte5 low 8 bits: Current torque [0~4096] ↔ (-120 N·m ~ 120 N·m); Byte6[7:6]: Operation Mode (0: Reset Mode, 1: Cali Mode, 2: Motor Mode); Byte6[5]: 1 = Motor fault, 0 = No motor fault; Byte6[4]: 1 = Motor warning, 0 = No warning; Byte6[3:0] (low 4 bits) as high 4 bits + Byte7 low 8 bits: Winding temperature = Temperature(°C) × 10, unit °C.
  (NOTE: the nibble-splitting description for current-speed and current-torque fields — "Byte3[7-4]... as low 4 bits" / "Byte4[3-0]... as high 4 bits" — is inherently a 12-bit packed value split across adjacent bytes; transcribed exactly as printed; see OPEN QUESTIONS for exact bit order.)

- Response Command 2: MCU Identification, p.78. Data Field size bit10~0 / Byte0~7; Description: Motor ID | 64-bit MCU unique identifier.

- Command 1: Enable Motor Operation, p.78. 11-bit ID = 07F (target motor CAN ID). Data: FF FF FF FF FF FF FF FC. Response: Response Command 1.

- Command 2: Stop Motor Operation, p.78. ID = 07F. Data: FF FF FF FF FF FF FF FD. Response: Response Command 1.

- Command 3: MIT Dynamic Parameters, p.78–79. ID = 07F (target motor CAN ID), example data "00 00 00 00 00 00 00 00". Byte0~1: Target angle [0~65535] ↔ (-12.57 rad ~ 12.57 rad); Byte2 high 8 bits + Byte3[7:4] (high 4 bits) as low 4 bits: Target speed [0~4096] ↔ (-15 rad/s ~ 15 rad/s); Byte3[3:0] (low 4 bits) as high 4 bits + Byte4: Kp [0~4096] ↔ (0 ~ 5000); Byte5 high 8 bits + Byte6[7:4] (high 4 bits) as low 4 bits: Kd [0~4096] ↔ (0 ~ 100); Byte6[3:0] (low 4 bits) as high 4 bits + Byte7 low 8 bits: Target torque [0~4096] ↔ (-120 N·m ~ 120 N·m). Response: Response Command 1.

- Command 4: Set Zero Position (Non-Position Mode), p.79. ID = 07F. Data: FF FF FF FF FF FF FF FE. Response: Response Command 1.

- Command 5: Clear Errors & Read Fault Status, p.79. ID = 07F. Data: FF FF FF FF FF FF F_CMD FB. F_CMD: 0xFF → clearing the current abnormality; any other value → "the error code will be returned in the BYTE1 field of the response message." Response (Fault Clear): Response Command 1.

- Fault Status Response, p.79–80. 11-bit ID example "0FD" (Byte labeling: Description "Target motor CAN ID"). Data example: "7F 00 00 00 00 00 00 00". Byte0: Motor CAN ID; Byte1~4: Fault value (Non-zero: faulty; 0: normal). Bit16: Motor current fault: Phase A sampling overcurrent; Bit14: Motor stall and overload algorithm protection; Bit9: Position initialization fault; Bit8: Hardware identification fault; Bit7: Encoder uncalibrated; Bit5: Motor current fault: Phase C sampling overcurrent; Bit4: Motor current fault: Phase B sampling overcurrent; Bit3: Overvoltage fault; Bit2: Undervoltage fault; Bit1: Driver chip fault; Bit0: Motor overtemperature fault, default threshold 145°C.

- Command 6: Set Operation Mode, p.80. ID = 07F. Data: FF FF FF FF FF FF F_CMD FC. F_CMD: Mode type — 0: MIT mode (default), 1: Position mode, 2: Velocity mode. Response: Response Command 1.

- Command 7: Modify Motor CAN ID, p.80. ID = 07F. Data: FF FF FF FF FF FF F_CMD FA. F_CMD: Target motor CAN ID. Response: Response Command 2.

- Command 8: Change Communication Protocol (Takes Effect After Power Cycle), p.80–81. ID = 07F. Data: FF FF FF FF FF FF F_CMD FD. F_CMD: Protocol type — 0: Private protocol (default), 1: CANopen, 2: MIT protocol. Response: Response Command 2.

- Command 9: Modify Host CAN ID, p.81. ID = 07F. Data: FF FF FF FF FF FF FD 01 (F_CMD byte = Host CAN ID, per description "F_CMD: Host CAN ID"). Response: Response Command 2.

- Command 10: Position Mode Control Command, p.81. 11-bit ID size "bit10~8" value "1" (target motor CAN ID). Example data: "00 00 A0 40 00 00 A0 40" (Position: 5 rad, Speed: 5 rad/s). Byte0~3: Target position (rad, 32-bit float). Byte4~7: Target speed (rad/s, 32-bit float). Response: Response Command 1.

- Command 11: Velocity Mode Control Command, p.81–82. ID field value "2" (target motor CAN ID). Example data: "00 00 A0 40 00 00 00 40" (Speed: 5 rad/s, Current: 2 A). Byte0~3: Target speed (rad/s, 32-bit float). Byte4~7: Current limit in speed/position mode (A, 32-bit float). Response: Response Command 1.

- Command 12: Save motor data (requires firmware update to 0.4.1.29), p.82. ID = 07F. Data: FF FF FF FF FF FF FF F8. Response frame: Response command 2.

- Command 13: Motor actively reports (requires firmware update to 0.4.1.29), p.82. ID = 07F. Data: FF FF FF FF FF FF F_CMD F9. F_CMD indicates motor protocol/report type: 0 = no reporting (default), 1 = reporting. Response frame: Response command 1.

- Command 14: Read motor parameters (requires firmware update to 0.4.1.29), p.82. ID field value "3" (target motor CANID). Data: 05 70 00 00 00 00 00 00. Byte0~1: index (see read/write parameter table, i.e. Table B above); Byte2~3: 00; Byte4~7: 00, "the above data is in low byte first, high byte last." Response frame has identical layout (p.82), Byte4~7 "For parameter data, data of 1 byte or more is formatted with the least significant byte first and the most significant byte last" (p.83).

- Command 15: Write motor parameters (requires firmware update to 0.4.1.29), p.83. ID field value "4" (target motor CANID). Data: 29 70 00 00 00 00 00 00. Byte0~1: index (same table as private protocol); Byte2~3: 00; Byte4~7: Parameter data, low byte first, high byte last. Response frame: identical layout, same description (p.83).

- Motion Control Mode, p.83: "The motor defaults to Motion Control Mode upon power-up." Sequence: Send Motor Enable Command (Command 1) → Send Motion Control Command (Command 3) to activate dynamic parameter control → Send Motor Stop Command (Command 2) to halt when needed.

- Velocity Mode, p.84: Configure via Set Operation Mode Command (Command 6), Mode=2 (Velocity Mode) → Send Motor Enable Command (Command 1) → Send Velocity Mode Control Command (Command 11) to set maximum current (absolute value) and target speed → Send Motor Stop Command (Command 2) to stop.

- Position Mode (CSP), p.84: Configure via Set Operation Mode Command (Command 6), Mode=1 (Position Mode) → Send Motor Enable Command (Command 1) → Send Position Mode Control Command (Command 10) to set maximum speed (absolute value) and target position → Send Motor Stop Command (Command 2) to stop.

## Fault/error codes
### Motor fault instructions (private protocol), function code 0x3022 — p.31–32
"Function code 0x3022 indicates the fault code, where:"
- bit14: motor blocking overload algorithm protection
- bit7: Encoder uncalibrated: Motor uncalibrated encoder
- bit3: Overvoltage fault: the motor voltage exceeds the protection voltage by 60V
- bit2: Undervoltage fault: the motor voltage is lower than the protection voltage of 12V
- bit1: Driver chip failure: Motor driver chip failure reported
- bit0: Motor overtemperature fault: motor thermistor temperature exceeds 145 degrees
(p.31)

(NOTE: Table A above lists index 0x3022 as "id" / "id Original value, A" — read-only float — which conflicts with this description of 0x3022 as a fault-code bitfield; see OPEN QUESTIONS.)

### Driver chip fault code 1 — Function code 0x3024 — "Table 11. Fault Status Register 1 Field Descriptions" (p.32)
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

### Driver chip fault code 2 — Function code 0x3025 — "Table 12. Fault Status Register 2 Field Descriptions" (p.32)
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

(NOTE: Table A's function-code list gives 0x3025 the name "drv_fault" with remark "The driver chip fault value is 1" and 0x3024 as "warnSta"/"Warning status value" — this conflicts with the §3.3.7 prose, which assigns 0x3024 = driver chip fault code 1 / Table 11, and 0x3025 = driver chip fault code 2 / Table 12. Both are transcribed verbatim; see OPEN QUESTIONS.)

### Communication type 21 — Fault feedback frame bit list — see "Frame definitions (TX)" section above (p.46–47). Default overtemperature fault threshold 145°C, default overtemperature warning threshold 135°C.

### Communication type 2 / 24 feedback frame fault bits (packed into CAN ID bits 16–21) — see RX section above (p.43, p.48). Mode status bits 22–23: 0 = Reset mode, 1 = Cali mode, 2 = Motor mode [Run]; value 3 not documented.

### MIT protocol Fault Status Response bit list — see "MIT protocol frames" section above (p.79–80). Default overtemperature fault threshold 145°C (no separate warning threshold given in this table, unlike the private-protocol type-21 frame which also states a 135°C warning threshold).

### CANopen Error_code (index 603F, UINTEGER16, read-only) — listed in Status Feedback Parameters table (p.71) with no bit/value breakdown given anywhere in the document.

## OPEN QUESTIONS (ambiguous, unclear, or internally inconsistent — not guessed/resolved)
1. **Type 1 (operation-control) TX frame, p.43**: table places the Torque field in the arbitration-ID's bit23~8 range (middle ID column) rather than in the 8-byte data payload, and the description text for Byte6~7 (Kd) is cut off mid-sentence ("After the conversion, the high byte is in front and the low byte is in" — rest missing). Transcribed exactly; the torque-in-ID placement is unusual but is explicitly what's printed, and is at least structurally consistent with the `exCanIdInfo.data` 16-bit field being set to `float_to_uint(torque,...)` in the p.61–63 code sample.
2. **Communication type 2 / 24 feedback frames (p.43, p.48)**: the fault-bit list for bits 16–21 contains a duplicated/contradictory line: "bit21: uncalibrated bit20: Uncalibrated bit20: Gridlock overload fault bit19: magnetic coding fault..." — bit20 is given two different meanings ("Uncalibrated" and "Gridlock overload fault") in the same sentence. Could not determine the intended single meaning for bit20 from the text.
3. **Communication type 21 fault feedback frame (p.46–47)**: bit14 is listed twice with two different wordings in the same bit list ("bit14: gridlock overload fault" in the opening description and "Bit14: Motor stall overload algorithm protection" later in the same cell) — same ambiguity as #2, unresolved.
4. **Communication type 7 (Set motor CAN_ID, p.45)**: description text says "bit15~8: used to identify main CAN_ID Bit16~23: preset CAN_ID" — bit16 is included in both the "bit15~8" range description and the separately stated "Bit16~23" range, an overlap that doesn't parse as two disjoint fields. Could not determine the exact bit boundaries.
5. **Communication type 17 reply frame (p.46)**: the reply-frame table's description text is textually truncated/merged across columns ("...01 indicates that the master can_ID..." cuts off) and "Byte4~7...preceded by low bytes and followed by high bytes at" also cuts off before stating what comes "at" the end. Could not recover the missing words.
6. **MIT Response Command 1 / Command 3 (p.77–79)**: speed and torque/Kp/Kd values are packed as 12-bit fields split across nibble boundaries of adjacent bytes (e.g., "Byte3 high 8 bits; Byte4[7-4] (high 4 bits) as low 4 bits: Current speed [0~4096]"). The exact bit-concatenation order (which nibble is MSB vs LSB of the 12-bit value) is not fully unambiguous from the prose description alone; transcribed verbatim per field.
7. **CAN_MASTER parameter (Table A, 0x200a, p.23)**: Maximum value listed as 300, but the declared type is uint8 (max representable value 255). Not reconciled.
8. **CAN_TIMEOUT parameter (Table A, 0x200b, p.23)**: Minimum value listed as -10000, but the declared type is uint32 (unsigned). Not reconciled.
9. **Two parallel parameter/index tables exist** (Table A: function codes 0x0000–0x3048 used by the host-computer GUI, p.21–29; Table B: indices 0x7005–0x702E explicitly tied to CAN communication types 17/18, p.50–52). The manual never states the relationship between these two address spaces (e.g., whether Table A's 0x2000-range addresses are also reachable via type 17/18 CAN commands, or are host-computer-only). Not guessed.
10. **Table A, 0x3007 "encoder2raw"** (p.25): remark column says "Bus voltage," which conflicts with the name (encoder2raw) and with the separate 0x300c "VBUS" entry (also remarked "Bus voltage V"). Not reconciled.
11. **Table A, 0x3022 "id" / "id Original value, A"** (p.27) vs. the prose in §3.3.7 (p.31) stating "Function code 0x3022 indicates the fault code." These two descriptions of the same function code (0x3022) are contradictory (a read-only current-feedback float vs. a fault-code bitfield). Not reconciled.
12. **Table A entries 0x3024/0x3025** ("warnSta"/"Warning status value" and "drv_fault"/"The driver chip fault value is 1" respectively, p.27) vs. §3.3.7 prose (p.32) stating "Function code 0x3024 is driver chip fault code 1" (Table 11) and "Function code 0x3025 is driver chip fault code 2" (Table 12). The index-to-meaning assignment conflicts between the two parts of the document. Not reconciled.
13. **Oscilloscope communication-box example table (p.30)**: the 5-column header row ("frame header | Number of data bits | extended frame | data frame | frame tail") has a stray "00" appearing to span only the 3rd/4th columns, and it's unclear whether "00" is part of the "extended frame" field or a separate field. Transcribed as printed; exact column boundaries unclear.
14. **Run_mode parameter description (Table B, 0x7005, p.50)**: enumerates "0: operation mode, 1: position mode (PP), 2: Velocity mode, 3: Operation mode Current mode, 5: Position mode (CSP)" — value 4 is not listed/skipped, and "3: Operation mode Current mode" is grammatically unclear (possibly two merged lines: "3: ... mode" and "Current mode" as a separate value, but no value number is given for "Current mode"). Not guessed.
15. **No explicit stated factory-default CAN ID.** Current-value samples (127 for CAN_ID, 253/0xFD for CAN_MASTER/host id) appear in a live unit's parameter-table screenshot and are used consistently as example host-ID values (0xFD) throughout worked examples, but the manual never labels either value as "default" outright.
16. **CANopen Error_code (index 603F)**: type/unit given (UINTEGER16, unit "/") but no bit or value table defining its contents appears anywhere in the document.
17. **Mode-status bits 22–23** (private-protocol feedback frames, type 2/24, p.43/p.48): only values 0 (Reset), 1 (Cali), 2 (Motor/Run) are defined for a 2-bit field that can also hold value 3; value 3's meaning (if any) is not documented.
