# RS02 protocol extract
Source: RS02User Manual260713.pdf (80 pages total)

## Model/firmware identification
- Model name as printed on cover: "ROBSTRIDE 02", subtitle "17N.M Quasi-Direct Drive Integrated Motor Module" (p.1).
- Manufacturer: "RobStride Dynamics", https://www.robstride.com/ (p.1). Manual text also refers to "Beijing Lingfoot Times Technology Co., LTD. (hereinafter referred to as Lingfoot)" as publisher (p.2-3).
- No single "firmware version" is declared for the unit as shipped. Firmware/build fields appear only as example/reference values in the parameter table (section 3.3.4, p.20):
  - `0X1000 BootCodeVersion` (String, Read only) — example value "0.1.5" (p.20)
  - `0X1001 BootBuildDate` — example "Mar162022" (p.20)
  - `0X1002 BootBuildTime` — example "20:22:09" (p.20)
  - `0X1003 AppCodeVersion` — example "0.0.0.1", described as "Motor program version number" (p.20)
  - `0X1004 AppGitVersion` — example "7b844b0fM" (p.20)
  - `0X1005 AppBuildDate` — example "Apr142022" (p.20)
  - `0X1006 AppBuildTime` — example "20:30:22" (p.20)
  - `0X1007 AppCodeName` — example "Lingzu_motor" (p.20)
  - These are table example/current values, not a stated manual revision for this document. Treat as illustrative, not a confirmed shipped firmware version (flagged under Open Questions).
- Document version history table (p.78-79): "Version Number 1.0, Description: Initial Release, Date: November 25, 2025" (p.79). (Filename date string "260713" does not match this printed date; see Open Questions.)
- Commands 12-15 (save motor data, active report, read/write params in MIT protocol) are explicitly marked "(requires firmware update to 0.2.3.32)" (p.76-77).
- Firmware name convention: "the rs-0x in the firmware name is the selected motor type" (p.16), implying the firmware filename encodes the model (e.g. rs0x), but no exact RS02 firmware filename/number is printed anywhere in the manual text extracted.

## CAN bus parameters
- "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted" (p.41, section 4 intro).
- Driver Product Specifications table (p.12, section 2.1):
  - "CANbusbitrate 1Mbps" (printed without spaces as "CANbusbitrate" — table formatting collapsed spaces in extraction) (p.12)
  - "Theratedworkingvoltage 48VDC" (p.12)
  - "Themaximumallowablevoltage 60VDC" (p.12)
  - "Ratedworkingphasecurrent 7Apk" (p.12)
  - "Maximumallowablephasecurrent 23Apk" (p.12)
  - "Standbypower ≤18mA" (p.12)
  - "Dimensions Φ58mm" (p.12)
  - "Workingenvironmenttemperature -20℃to50℃" (p.12)
  - "Themaximumallowabletemperatureofthecontrolboard 145℃" (p.12)
  - "encoderresolution 14bit(absoluteturn)" (p.12)
- Pin description, Power supply and CAN communication connector (p.13):
  - Pin 1: "Thepositiveelectrodeofthepowersupply(+)"
  - Pin 2: "Negativeelectrodeofthepowersupply(-)"
  - Pin 3: "CANCAN_L" (CAN_L)
  - Pin 4: "CANthehighsideofthecommunicationCAN_H" (CAN_H)
- CAN-to-USB DIP switch notes (p.14): "When DIP switch 1 is in the ON position, the module enters Boot mode and cannot establish a connection with the host computer. When DIP switch 2 is in the ON position, a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer." (p.14)
- Baud rate is changeable via Communication type 23 (private protocol): F_CMD byte values "01 is 1M, 02 is 500K, 03 is 250K, 04 is 125K" (p.45-46).
- Default CAN ID: no single explicit "default CAN ID" statement found in the manual body. Example/reference parameter table shows `0X200a CAN_ID` current value "1" (p.21) and `0X200b CAN_MASTER` current value "0" (p.21) — these are example/current values in the parameter reference table, not a stated factory default. In worked examples elsewhere motor CAN ID "0x7F" (127 decimal) is used repeatedly as the example target ID (e.g. p.49-50, p.72-77), and host ID "0xFD" (253) is used as the example host ID (p.31, p.49-50). These are presented as examples, not stated defaults — flagged under Open Questions.
- CAN ID range: `0X200a CAN_ID` parameter table entry gives Maximum value 127, Minimum value 0 (type uint8) (p.21). `0X200b CAN_MASTER` (host ID) also Maximum 127, Minimum 0 (p.21).
- Frame types used: CAN 2.0 extended (29-bit) frames for the "private protocol" (p.41 and throughout section 4); CAN standard (11-bit) frames for CANopen (p.63, section 5) and for MIT protocol (p.71, section 6, "The standard frame format is as follows").
- Protocol selectable among 3 types via Communication type 25 (private protocol frame) or MIT Command 8: "0 is a private protocol (default) 1 is the Canopen protocol 2 is the MIT protocol" (p.47); worded identically for MIT Command 8 (p.75-76): "0: Private protocol (default) 1: CANopen 2: MIT protocol".

## Frame definitions (TX - host to motor) — Private ("operation control") protocol, extended 29-bit ID, CAN 2.0, section 4 (p.40-62)

General extended-ID layout stated at top of section 4 (p.41):
```
data field: 29-bit ID | 8-byte data field
Size:       Bit28~bit24 | bit23~8 | bit7~0 | Byte0~Byte7
Description: Communication type | data area 2 | Destination address | data area 1
```
NOTE (explicit, not inferred): the manual's own worked example (p.30-31) decodes an extended ID differently from this header row — it describes the structure as `uint32_t id:8; uint32_t data:16; uint32_t mode:5; uint32_t res:3;` (p.56-57, section 4.4 code sample `exCanIdInfo`), i.e. target motor CAN_ID in the low 8 bits, a 16-bit "data" field (used as host/master CAN_ID) above that, and a 5-bit "mode" (communication type) field above that, with 3 reserved/unused top bits. The worked decode example on p.31 is consistent with this: raw ID bytes `90 07 e8 0c` → binary `1001 0000 0000 0111 1110 1000 0000 1100`, right-shifted 3 bits → `1 0010 0000 0000 1111 1101 0000 0001` → hex `12 00 FD 01`, parsed as "Communication type 18 (0x12, decimal), no meaning, host id 0xFD, motor can id 01" (p.31). This confirms: low byte = target motor CAN_ID, next 16 bits = host/master CAN_ID ("no meaning" region + host id byte per their decode), top bits (communication/mode type) = 0x12 = 18. The per-field header table printed throughout section 4 ("Bit28~bit24 / bit23~8 / bit7~0") appears to be a simplified/approximate restatement and is somewhat inconsistent in column widths across frame types — flagged under Open Questions as the bit-field documentation is not self-consistent in the source.

### Communication type 0: Get device ID (p.40)
- Request: `0x0` in type field; `bit15~8: identifies host CAN_ID`; `target motor CAN_ID`; data byte0 = `0`.
- Reply: `0x0` type; `target motor CAN_ID`; data byte0 = `0XFE`; Byte0~Byte7 (reply) = "64-bit MCU unique identifier".

### Communication Type 1: operation control mode motor control instruction (p.41-42)
- Type = `0x1`.
- Byte layout as printed: "Byte2:Torque target (0~65535) corresponds to CAN_ID (-17Nm~17Nm) ... Byte0~1:target Angle[0~65535] corresponds to (-4π~4π) Byte2~3:Target angular velocity [0~65535] corresponds to (-44rad/s~44rad/s) Byte4~5:Kp [0~65535] corresponds to (0.0~500.0) Byte6~7:Kd[0 to 65535] corresponds to the above data (0.0 to 5.0). After the conversion, the high byte is in front and the low byte is in [rear, cut off]" (p.41-42, text runs together/garbled in source — transcribed verbatim as extracted; exact byte-to-field mapping is ambiguous as printed, see Open Questions).
- Cross-reference: the working C code sample for this frame type (p.58-59, section 4.4.2, `motor_controlmode`) is unambiguous and used as the authoritative byte layout:
  - `txCanIdEx.data = float_to_uint(torque, T_MIN, T_MAX, 16)` — placed in the extended-ID "data" field (i.e. torque target is packed into the CAN ID structure, not the 8-byte payload, per this code) (p.58)
  - `tx_data[0] = float_to_uint(MechPosition,P_MIN,P_MAX,16) >> 8` (high byte), `tx_data[1] = ... ` (low byte) — target angle
  - `tx_data[2]/[3]` = speed (V_MIN..V_MAX) high/low byte
  - `tx_data[4]/[5]` = kp (KP_MIN..KP_MAX) high/low byte
  - `tx_data[6]/[7]` = kd (KD_MIN..KD_MAX) high/low byte
  - Constants (p.57): `P_MIN -12.57f, P_MAX 12.57f, V_MIN -44.0f, V_MAX 44.0f, KP_MIN 0.0f, KP_MAX 500.0f, KD_MIN 0.0f, KD_MAX 5.0f, T_MIN -17.0f, T_MAX 17.0f`
  - `float_to_uint` formula (p.57): `span = x_max - x_min; offset = x_min; clamp x to [x_min,x_max]; return (int)((x-offset) * ((1<<bits)-1) / span)` — i.e. standard linear scale to unsigned N-bit range.
- Response frame: "Response motor feedback frame (see communication type 2)" (p.41).

### Communication Type 2: motor feedback data (response to types 1/3/4/6, p.41-42)
- Type = `0x2`.
- `Bit8~Bit15: CAN ID of the current motor`; `bit21~16: fault information (0 none, 1 has)`:
  - bit21: uncalibrated
  - bit20: "Uncalibrated" / "Gridlock overload fault" (both phrases printed against bit20 in source — ambiguous/duplicated label, see Open Questions) (p.41)
  - bit19: magnetic coding fault
  - bit18: overtemperature
  - bit17: three-phase overcurrent fault
  - bit16: undervoltage fault
  - bit22~23: Mode status: `0: Reset mode [reset]`, `1: Cali mode [calibration]`, `2: Motor mode [Run]` (p.41-42)
- Data bytes: "Byte0~1: The current Angle of the current motor [0~65535] Corresponding to (-4π~4π); Byte2~3: Current angular velocity [0~65535] corresponds to (-44rad/s~44rad/s); Byte4~5: Current torque [0~65535] corresponds to (-17Nm~17Nm); Byte6~7: Current temperature: Temp(Celsius)*10. If the value is higher than 10, the high byte is first and the low byte is last" (p.41, byte-order phrasing as printed is unclear, transcribed verbatim).

### Communication Type 3: Motor enabled to run (p.42)
- Type = `0x3`. `bit15~8: identifies the main CAN_ID`; target motor CAN_ID.
- Response: motor feedback frame (type 2).
- Code sample `motor_enable(uint8_t id, uint16_t master_id)` (p.58): sets `mode=3`, `id=id`, `res=0`, `data=master_id`, dlen=8, then overwrites `data=0` before transmit (as printed — the `data=master_id` assignment appears to be immediately overwritten by `data=0` in the sample code, p.58; transcribed verbatim, not corrected).

### Communication Type 4: Motor stops running (p.42)
- Type = `0x4`. "When the motor is running normally, 0 must be cleared in the data field. Byte[0]=1: The fault is cleared." (p.42)
- Response: motor feedback frame (type 2).
- Code sample `motor_reset` (p.59-60): mode=4, data=master_id, all 8 data bytes set to 0, then transmit.

### "Version number read frame" (also under type 0x4 heading, p.42-43)
- Request: Type `0x4`; `bit15~8: main CAN_ID`; target motor CAN_ID; `Byte[0]=0x00`, `Byte[1]=0xC4`.
- Response: Type `0x2`; `Bit8~Bit15: CAN ID of current motor`; fault info bits (same bit list as type 2 above, p.43-44); `Byte0=0x00`, `Byte1=0xC4`, `Byte2=0x56`, `Byte3~6: Motor version number, order from high to low` (p.43-44).

### Communication type 6: Set motor mechanical zero (p.44)
- Type = `0x6`. `bit15~8: identifies main CAN_ID`; target motor CAN_ID; `Byte[0]=1`.
- Response: motor feedback frame (type 2).

### Communication type 7: Set motor CAN_ID (p.44)
- "Change the current motor CAN_ID, effective immediately."
- Type = `0x7`. `bit15~8: used to identify main CAN_ID`; `Bit16~23: preset CAN_ID`; target motor CAN_ID.
- Answer frame: "motor broadcast frame (see communication type 0)".

### Communication type 17: Single parameter read (p.44-45)
- Type = `0x11` (17 decimal). `bit15~8: target motor CAN_ID`. Data: `Byte0~1: index` (see parameter table), `Byte2~3: 00`, `Byte4~7: parameter data above; low byte first, high byte second` (as printed, p.44-45).
- Reply: Type `0x11`; `bit15~8: master CAN_ID`; `Bit23~16: 00 indicates master CAN_ID successfully read; 01 indicates...` [sentence cut off in source after "01 indicates that the master can_ID" — incomplete, p.45]. Data: `Byte0~1`, `Byte2~3: 00`, `Byte4~7: Parameter data, 1 byte or more, above Byte4 is preceded by low bytes and followed by high bytes` (p.45).

### Communication type 18: Single parameter write (lost in power failure) (p.45)
- "With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved."
- Type = `0x12` (18 decimal). `bit15~8: target motor CAN_ID`. Data: `Byte0~1: index`, `Byte2~3: 00`, `Byte4~7: Parameter data`, low byte in front, high byte in rear (p.45).
- Response: motor feedback frame (type 2).
- Code samples (p.60-62): mode change writes `index` (2 bytes) to `tx_data[0:2]` and `runmode` (1 byte) to `tx_data[4]`; parameter write writes `index` (2 bytes) to `tx_data[0:2]` and `ref` (float, 4 bytes) to `tx_data[4:8]`.

### Communication type 21: Fault feedback frame (p.45-46)
- Type = `0x15` (21 decimal). `bit15~8: identifies main CAN_ID`. Data: `Byte0~3: fault value (non-0: faulty; 0: Normal)`:
  - Bit16: A-phase current sampling overcurrent
  - Bit14: Motor stall overload algorithm protection
  - Bit9: Position initialization fault
  - Bit8: Hardware identification fault
  - Bit7: Encoder uncalibrated
  - Bit5: C-phase current sampling overcurrent
  - Bit4: B-phase current sampling overcurrent
  - bit3: overvoltage fault
  - bit2: undervoltage fault
  - bit1: driver chip fault
  - bit0: motor overtemperature fault, default 135°C
  - `Byte4~7: warning Value`; `bit0: motor overtemperature warning, the default is 135°c` (p.46)

### Communication type 22: Motor data save frame (p.46)
- Type = `0x16` (22 decimal). `bit15~8: identifies main CAN_ID`; target motor CAN_ID. Data bytes (fixed): `01 02 03 04 05 06 07 08` (p.46).
- Response: motor feedback frame (type 2).

### Communication type 23: Motor baud rate modification frame (re-power-on effect) (p.46-47)
- Type = `0x17` (23 decimal). `bit15~8: target motor CAN_ID`. Data: `01 02 03 04 05 06 F_CMD` where F_CMD = baud rate: `01 is 1M, 02 is 500K, 03 is 250K, 04 is 125K` (p.46-47).
- Response: motor feedback frame (type 0).

### Communication type 24: The motor actively reports frames (p.47)
- Type = `0x18` (24 decimal). `bit15~8: identifies main CAN_ID`; target motor CAN_ID. Data: `01 02 03 04 05 06 F_CMD` where F_CMD: `00 = disable active reporting (default)`, `01 = enable active reporting, default reporting interval 10ms` (p.47).
- Response frame (p.47-48): Type `0x18`; `Bit8~Bit15: CAN ID of current motor`; `bit21~16: fault information` (same bit meanings as type 2, repeated p.47-48); `bit22~23: Mode status` (same 0/1/2 as type 2). Data: `Byte0~1: target Angle [0~65535] corresponds to (-4π~4π)`, `Byte2~3: Target angular velocity [0~65535] corresponds to (-44rad/s~44rad/s)`, `Byte4~5: Kp [0~65535] corresponds to (0.0~500.0)`, `Byte6~7: Kd [0 to 65535] corresponds to the above data (0.0 to 5.0)` (p.47-48).

### Communication type 25: Motor protocol modification frame (re-power-on effect) (p.48)
- Type = `0x19` (25 decimal). `bit15~8: target motor CAN_ID`. Data: `01 02 03 04 05 06 F_CMD` where F_CMD = protocol type: `0 = private protocol (default)`, `1 = CANopen protocol`, `2 = MIT protocol` (p.48).
- Response: motor feedback frame (type 0).

### Read example (p.50), reading `loc_kp`
- Read instruction: Type `0x11`; host id `0x00FD`; target motor CAN_ID `0x7F`; data `1E70000000000000` (index bytes = `1E 70` little-endian = index 0x701E = loc_kp, per parameter table).
- Feedback: Type `0x11`; `bit15~8: Host id 0xFD`; target motor CAN_ID `0x7F`; data `1E7000000000F041` — `Byte0~1: index`, `Byte4~7: loc_kp value, hex 0x4150... ` (printed as "30" decoded value with bytes "00 00 F0 41" — IEEE-754 32-bit float; the manual states "Byte4~7:loc_kp value 30, high right byte, hexadecimal IEEE-754 standard floating point number", p.50) — i.e. the 4 data bytes at offset 4 are a little-endian IEEE-754 single-precision float equal to 30.0 in this example.

## Frame definitions (RX - motor to host)
All reply/feedback frame layouts are documented inline above next to their corresponding request type (types 0, 2, 4-version-read, 17, 18, 21, 24), since the manual presents them as "Response frame:" blocks within each subsection of section 4.1 (p.40-48). See above for exact byte fields. Type 2 ("motor feedback data") is the canonical status frame returned after types 1, 3, 4, 6, 18 (p.41-42).

### MIT protocol RX — Response Command 1: Data Feedback (Motor Status) (p.72-73, section 6.1)
- Frame: 11-bit ID; example raw ID shown as `0FD` with payload bytes `7F 00 00 00 00 00 00 00` (p.72).
- "Byte0: Motor CAN ID"
- "Byte1~2: Current angle [0~65535], corresponding to (-12.57rad~12.57rad)"
- "Byte3: High 8 bits; Byte4[7-4](high 4 bits) as low 4 bits: Current speed [0~4096], corresponding to (-44rad/s~44rad/s)" (12-bit field split across byte3 + high nibble of byte4, as printed — bit packing is stated but the exact nibble order is ambiguous in the text, see Open Questions)
- "Byte4[3-0](low 4 bits) as high 4 bits, Byte5 as low 8 bits: Current torque [0~4096], corresponding to (-17N·m~17N·m)" (again 12-bit field split across low nibble of byte4 + byte5)
- "Byte6[7-6]: Operation Mode 0: Reset Mode, 1: Cali Mode, 2: Motor Mode"
- "Byte6[5]: 1=Motor fault exists, 0=No motor fault"
- "Byte6[4]: 1=Motor warning without fault, 0=No warning"
- "Byte6[3-0] (low 4 bits) as high 4 bits, Byte7 as low 8 bits: Winding temperature: Temperature(°C)×10, Unit: °C" (12-bit field, same split pattern)
- (p.72-73; the manual itself only uses 12-bit (0~4096) ranges here versus 16-bit (0~65535) ranges in the private-protocol type-1/2 frames — these are stated as different scalings for MIT vs private protocol, not an extraction error.)

### MIT protocol RX — Response Command 2: MCU Identification (p.73)
- 11-bit ID = Motor ID; Byte0~7 = "64-bit MCU unique identifier".

### MIT protocol RX — Fault Status Response (to Command 5) (p.73-74)
- Raw ID example `0FD`, payload `7F 00 00 00 00 00 00 00` header.
- "Byte0: Motor CAN ID"
- "Byte1~4: Fault value (Non-zero: fault; 0: normal)":
  - Bit16: Motor current fault: Phase A sampling overcurrent
  - Bit14: Motor stall and overload algorithm protection
  - Bit9: Position initialization fault
  - Bit8: Hardware identification fault
  - Bit7: Encoder uncalibrated
  - Bit5: Motor current fault: Phase C sampling overcurrent
  - Bit4: Motor current fault: Phase B sampling overcurrent
  - Bit3: Overvoltage fault
  - Bit2: Undervoltage fault
  - Bit1: Driver chip fault
  - Bit0: Motor overtemperature fault, default 135°C
  (p.74-75)

## TX frame definitions — MIT protocol (standard 11-bit ID, section 6, p.71-78)
General frame header (p.71): "DataField 11-bitID 8-byteDataArea; Size Bit10~8 Bit7~0 Byte0~7; Description Mode type ID". Default baud rate 1Mbps; "The baud rate can be modified by switching to the private protocol" (p.71).

- Command 1: Enable Motor Operation (p.73): ID=`07F`; payload `FF FF FF FF FF FF FF FC`. Response: Response Command 1.
- Command 2: Stop Motor Operation (p.73): ID=`07F`; payload `FF FF FF FF FF FF FF FD`. Response: Response Command 1.
- Command 3: MIT Dynamic Parameters (p.73-74): ID=`07F` (target motor CAN ID). Byte layout:
  - Byte0~1: Target angle [0~65535], corresponds to (-12.57rad~12.57rad)
  - Byte2: high 8 bits; Byte3[7-4] (high 4 bits) as low 4 bits: Target speed [0~4096], corresponds to (-44rad/s~44rad/s)
  - Byte3[3-0] (low 4 bits) as high 4 bits, Byte4 as low 8 bits: Kp [0~4096], corresponds to 0~500
  - Byte5: high 8 bits; Byte6[7-4] (high 4 bits) as low 4 bits: Kd [0~4096], corresponds to 0~5
  - Byte6[3-0] (low 4 bits) as high 4 bits, Byte7 as low 8 bits: Target torque [0~4096], corresponds to (-17N·m~17N·m)
  - Response: Response Command 1.
- Command 4: Set Zero Position (Non-Position Mode) (p.74-75): ID=`07F`; payload `FF FF FF FF FF FF FF FE`. Response: Response Command 1.
- Command 5: Clear Errors & Read Fault Status (p.75): ID=`07F`; payload `FF FF FF FF FF FF F_CMD FB`. "F_CMD: 0xFF → clearing the current abnormality; Any other value → the error code will be returned in the BYTE1 field of the response message." Response (Fault Clear): Response Command 1.
- Command 6: Set Operation Mode (p.76): ID=`07F`; payload `FF FF FF FF FF FF 00 FC` pattern, "F_CMD: Mode type — 0: MIT mode (default), 1: Position mode, 2: Velocity mode". Response: Response Command 1.
- Command 7: Modify Motor CAN ID (p.76): ID=`07F`; payload `FF FF FF FF FF FF 01 FA` pattern; "F_CMD: Target motor CAN ID". Response: Response Command 2.
- Command 8: Change Communication Protocol (takes effect after power cycle) (p.76-77): ID=`07F`; payload `FF FF FF FF FF FF 00 FD` pattern; "F_CMD: Protocol type — 0: Private protocol (default), 1: CANopen, 2: MIT protocol". Response: Response Command 2.
- Command 9: Modify Host CAN ID (p.77): ID=`07F`; payload `FF FF FF FF FF FF FD 01` pattern; "F_CMD: Host CAN ID". Response: Response Command 2.
- Command 10: Position Mode Control Command (p.77-78): ID bits10~8=`1`, bit7~0=`7F` (target motor CAN ID); example payload `00 00 A0 40 00 00 A0 40` (Position: 5rad, Speed: 5 rad/s). "Byte0~3: Target position (rad, 32-bit float); Byte4~7: Target speed (rad/s, 32-bit float)". Response: Response Command 1.
- Command 11: Velocity Mode Control Command (p.78-79): ID bits10~8=`2`, bit7~0=`7F`; example payload `00 00 A0 40 00 00 00 40` (Speed: 5 rad/s, Current: 2A). "Byte0~3: Target speed (rad/s, 32-bit float); Byte4~7: Current limit in speed/position mode (A, 32-bit float)". Response: Response Command 1.
- Command 12: Save motor data (requires firmware update to 0.2.3.32) (p.79-80): ID=`07F`; payload `FF FF FF FF FF FF FF F8`. Response frame: Response command 2.
- Command 13: Motor actively reports (requires firmware update to 0.2.3.32) (p.80): ID=`07F`; payload `FF FF FF FF FF FF F_CMD F9`. "F_CMD: 0 = no reporting (default), 1 = reporting". Response frame: Response command 1.
- Command 14: Read motor parameters (requires firmware update to 0.2.3.32) (p.80-81): ID bits10~8=`3`, bit7~0=`7F`; example payload `05 70 00 00 00 00 00 00` (index example = 0x7005). "Byte0~1: index (see parameter table); Byte2~3: 00; Byte4~7: 00 — data is low byte first, high byte last." Response frame: same field layout, "parameter data of 1 byte or more formatted least-significant byte first, most-significant byte last."
- Command 15: Write motor parameters (requires firmware update to 0.2.3.32) (p.81-82): ID bits10~8=`4`, bit7~0=`7F`; example payload `29 70 00 00 00 00 00 00` (index example = 0x7029). Same field layout as Command 14; "Parameter data: low byte first, high byte last." Response frame: identical layout echoed back.

### Protocol Switching frame (Extended Frame, p.70-71, section 5.10 — CANopen section, but frame itself is a raw extended-frame broadcast independent of CANopen SDO/PDO machinery)
- TX: `DataField 29-bitID 8-ByteDataArea`; `Size Bit28~0 Byte0~6`; payload `01 02 03 04 05 06 F_CMD`. "F_CMD (Byte 6) defines the motor protocol: 0 = Private protocol (default), 1 = CANopen protocol, 2 = MIT protocol" (p.70).
- RX (response): `DataField 11-bitID 8-ByteDataArea`; `Size Bit10~0 Byte0~7`; "Description: MotorID | 64-bit MCU unique identifier" (p.71).

## Parameter table (flash config read/write) — "Read and write a single parameter list" (p.48-51, section 4.1.13)
Columns as printed: index | Description | Type | Number of bytes | R/W permission.

| index | name/description | Type | Bytes | R/W | Range / notes (as printed) |
|---|---|---|---|---|---|
| 0X7005 | run_mode | uint8 | 1 | W/R | "0:operation mode 1:position mode (PP) 2:Velocity mode 3:Operation mode Current mode 5:Position mode (CSP)" (p.49) |
| 0X7006 | iq_ref — Current mode Iq command | float | 4 | W/R | -16 to 16A |
| 0X700A | spd_ref — Velocity mode Rotational speed command | float | 4 | W/R | -33 to 33rad/s |
| 0X700B | limit_torque — torque limit | float | 4 | W/R | 0 to 14Nm |
| 0X7010 | cur_kp — Kp | float | 4 | W/R | default value is 0.17 |
| 0X7011 | cur_ki — Ki | float | 4 | W/R | default value is 0.012 |
| 0X7014 | cur_filt_gain — filt_gain | float | 4 | W/R | 0 to 1.0, default value is 0.1 |
| 0X7016 | loc_ref — Position Mode Angle instruction | float | 4 | W/R | rad |
| 0X7017 | limit_spd — Location mode (CSP) speed limit | float | 4 | W/R | 0 to 33rad/s |
| 0X7018 | limit_cur — Velocity/position mode Current limitation | float | 4 | W/R | 0 to 16A |
| 0x7019 | mechPos — Mechanical Angle of the loading coil | float | 4 | R | rad |
| 0x701A | iqf — iqFilter | float | 4 | R | -16 to 16A |
| 0x701B | mechVel — Speed of the load | float | 4 | R | -33 to 33rad/s |
| 0x701C | VBUS — Bus voltage | float | 4 | R | V |
| 0x701E | loc_kp — kp | float | 4 | W/R | default value is 40 |
| 0x701F | spd_kp — speed kp | float | 4 | W/R | default value is 6 |
| 0x7020 | spd_ki — ki | float | 4 | W/R | default value is 0.02 |
| 0x7021 | spd_filt_gain — Speed filter value | float | 4 | W/R | default value is 0.1 |
| 0x7022 | acc_rad — velocity mode acceleration | float | 4 | W/R | default value is 20rad/s^2 |
| 0x7024 | vel_max — Location mode (PP) speed | float | 4 | W/R | default value is 10rad/s |
| 0x7025 | acc_set — Location mode (PP) acceleration | float | 4 | W/R | default value is 10rad/s^2 |
| 0x7026 | EPScan_time — report time | uint16 | 2 | W/R | "1 indicates 10ms. default value is 1. Plus 1 increments by 5ms" |
| 0x7028 | canTimeout — timeout threshold | uint32 | 4 | W/R | "20000 is 1s. default value is 0" |
| 0x7029 | zero_sta — zero flag bit | uint8 | 1 | W/R | "0 means 0-2π and 1 means -π-π. default is 0" |
| 0x702A | damper — Damping switch | uint8 | 1 | W/R | default value is 0 |
| 0x702B | add_offset — Zero offset | float | 4 | W/R | default is 0 |
| 0x702C | alveolous_open — Cogging Compensation Switch | uint8 | 1 | W/R | default is 0 |
| 0x702D | iq_test — Motor Initialization Calibration Switch | uint8 | 1 | W/R | default is 0 |
| 0x702E | dcc_set — Deceleration in Motor PP Mode (normally disabled) | float | 4 | W/R | default is 10rad/s^2 |

(All rows p.49-51.)

### Flash/diagnostic parameter table — "function code" table, section 3.3.4 (p.20-29)
Columns as printed: function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | 备注(remarks).

| code | name | type | attribute | max | min | current value (example) | remark |
|---|---|---|---|---|---|---|---|
| 0X0000 | Name | String | Read/Write | | | | |
| 0X0001 | BarCode | String | Read/Write | | | | |
| 0X1000 | BootCodeVersion | String | Read only | | | 0.1.5 | |
| 0X1001 | BootBuildDate | String | Read only | | | Mar162022 | |
| 0X1002 | BootBuildTime | String | Read only | | | 20:22:09 | |
| 0X1003 | AppCodeVersion | String | Read only | | | 0.0.0.1 | Motor program version number |
| 0X1004 | AppGitVersion | String | Read only | | | 7b844b0fM | |
| 0X1005 | AppBuildDate | String | Read only | | | Apr142022 | |
| 0X1006 | AppBuildTime | String | Read only | | | 20:30:22 | |
| 0X1007 | AppCodeName | String | Read only | | | Lingzu_motor | |
| 0X2000 | echoPara1 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2001 | echoPara2 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2002 | echoPara3 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2003 | echoPara4 | uint16 | disposition | 74 | 5 | 5 | |
| 0X2004 | echoFreHz | uint32 | Read/Write | 10000 | 1 | 500 | |
| 0X2005 | MechOffset | float | Settings | 7 | -7 | 4.619583 | Motor magnetic encoder Angle offset |
| 0X2006 | status2 | float | Read/Write | 50 | -50 | 4.52 | Reserved parameter |
| 0X2007 | limit_torque | float | Read/Write | 17 | 0 | 17 | Torque limitation |
| 0X2008 | I_FW_MAX | float | Read/Write | 33 | 0 | 0 | Weak magnetic current value, default 0 |
| 0X2009 | motor_baud | uint8 | Settings | 20 | 0 | 1 | Baudrate flag bit |
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
| 0X2019 | limit_cur | float | Read/Write | 23 | 0 | 23 | Position/Velocity mode current limit |
| 0X201a | loc_ref_filt_gain | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201b | limit_loc | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201c | position_offset | float | Read/Write | 27 | 0 | 0 | High speed segment offset |
| 0X201d | chasu_angle_offset | float | Read/Write | 27 | 0 | 0 | "The low end is offset" |
| 0X201e | zero_sta | float | Read/Write | 150 | 0 | | Zero marker |
| 0x201f | protocol_1 | float | Read/Write | 20 | 0 | | Protocol flag |
| 0X3000 | timeUse0 | float | Read/Write | 1000 | 0 | | |
| 0X3001 | timeUse1 | float | Read/Write | 100 | 0 | 0 | |
| 0X3002 | timeUse2 | uint16 | Read only | | | 5 | |
| 0X3003 | timeUse3 | uint16 | Read only | | | 0 | |
| 0X3004 | encoderRaw | uint16 | Read only | | | 10 | Magnetic encoder sampling value |
| 0X3005 | mcuTemp | uint16 | Read only | | | 0 | mcu internal temperature, *10 |
| 0X3006 | motorTemp | int16 | Read only | | | 11396 | Motor ntc temperature, *10 |
| 0X3007 | vBus(mv) | int16 | Read only | | | 337 | Bus voltage |
| 0X3008 | adc1Offset | int16 | Read only | | | 333 | adc sampling channel1 Zero current bias |
| 0X3009 | adc2Offset | uint16 | Read only | | | 24195 | adc sampling channel2 Zero current bias |
| 0X300a | adc1Raw | int32 | Read only | | | 2084 | adc sampling value1 |
| 0X300b | adc2Raw | int32 | Read only | | | 2084 | adc sampling value2 |
| 0X300c | VBUS | uint16 | Read only | | | 1232 | Bus voltage V |
| 0X300d | cmdId | uint16 | Read only | | | 1212 | id ring instruction, A |
| 0X300e | cmdIq | float | Read only | | | 36 | iq ring command, A |
| 0X300f | cmdlocref | float | Read only | | | 0 | Position loop command, rad |
| 0X3010 | cmdspdref | float | Read only | | | 0 | Speed loop command, rad/s |
| 0X3011 | cmdTorque | float | Read only | | | 0 | Torque instruction, nm |
| 0X3012 | cmdPos | float | Read only | | | 0 | mit Protocol Angle instruction |
| 0X3013 | cmdVel | float | Read only | | | 0 | mit Protocol Speed instruction |
| 0X3014 | rotation | float | Read only | | | 0 | Number of turns |
| 0X3015 | modPos | float | Read only | | | 0 | Motor uncounted coil mechanical Angle, rad |
| 0X3016 | mechPos | int16 | Read only | | | 1 | Load end loop mechanical Angle, rad |
| 0X3017 | mechVel | float | Read only | | | 4.363409 | Load speed: rad/s |
| 0X3018 | elecPos | float | Read only | | | 0.777679 | Electrical Angle |
| 0X3019 | ia | float | Read only | | | 0.036618 | U-wire current, A |
| 0X301a | ib | float | Read only | | | 4.714761 | V-wire current, A |
| 0X301b | ic | float | Read only | | | 0 | W-wire current, A |
| 0X301c | timeout | float | Read only | | | 0 | Timeout counter value |
| 0X301d | phaseOrder | float | Read only | | | 0 | Directional marking |
| 0X301e | iqf | uint32 | Read only | | | 31600 | iq filter value, A |
| 0X301f | boardTemp | uint8 | Read only | | | 0 | Plate temperature, *10 |
| 0X3020 | iq | float | Read only | | | 0 | iq Original value, A |
| 0X3021 | id | int16 | Read only | | | 359 | id Original value, A |
| 0X3022 | faultSta | float | Read only | | | 0 | Fault status value |
| 0X3023 | warnSta | float | Read only | | | 0 | Warning status value |
| 0X3024 | drv_fault | uint32 | Read only | | | 0 | "The driver chip fault value is 1" (see Fault/error codes section) |
| 0X3025 | drv_temp | uint32 | Read only | | | 0 | "The driver chip fault value is 2" (see Fault/error codes section) |
| 0X3026 | Uq | uint16 | Read only | | | 0 | Q-axis voltage |
| 0X3027 | Ud | int16 | Read only | | | 48 | D-axis voltage |
| 0X3028 | dtc_u | float | Read only | | | 0 | Duty cycle of U-phase output |
| 0X3029 | dtc_v | float | Read only | | | 0 | Duty cycle of V-phase output |
| 0X302a | dtc_w | float | Read only | | | 0 | Duty cycle of W-phase output |
| 0X302b | v_bus | float | Read only | | | 0 | Vbus in the closed loop |
| 0X302c | torque_fdb | float | Read only | | | 0 | Torque feedback value, nm |
| 0X302d | rated_i | float | Read only | | | 24.195 | Rated current of motor |
| 0X302e | limit_i | float | Read only | | | 0 | The motor limits the maximum current |
| 0X302f | spd_ref | float | Read only | | | 8 | Motor speed expectation |
| 0X3030 | motor_mech_angle2 | float | Read only | | | 27 | Motor speed expectation |
| 0X3031 | position | float | Read only | | | 0 | Motor position determination parameters |
| 0X3032 | chasu_angle_init | float | Read only | | | 0 | Motor position determination parameters |
| 0X3033 | chasu_angle_out | float | Read only | | | 0 | Motor position determination parameters |
| 0X3034 | motormechinit1 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3035 | mech_angle_init2 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3036 | mech_angle_rotations | float | Read only | | | 0 | Motor position determination parameters |
| 0X3037 | cmdlocref_1 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3038 | status_1 | float | Read only | | | 0 | Retention parameter |
| 0X3039 | ElecOffset | uint32 | Read only | | | 0 | electrical Angle offset |
| 0X303a | mcOverTemp | uint32 | Read only | | | 0 | Overtemperature threshold |
| 0X303b | Kt_Nm/Amp | uint32 | Read only | | | 0 | Moment coefficient |
| 0X303c | Tqcali_Type | uint32 | Read only | | | 0 | Motor type |
| 0X303d | fault1 | uint32 | Read only | | | 0 | Log failure |
| 0X303e | fault2 | uint32 | Read only | | | 0 | Log failure |
| 0X303f | fault3 | uint32 | Read only | | | 0 | Log failure |
| 0X3040 | fault4 | uint32 | Read only | | | 0 | Log failure |
| 0X3041 | fault5 | float | Read only | | | 0 | Log failure |
| 0X3042 | fault6 | int16 | Read only | | | 0 | Log failure |
| 0X3043 | fault7 | float | Read only | | | 0 | Log failure |
| 0X3044 | fault8 | uint8 | Read only | | | 0 | Log failure |
| 0X3045 | theta_mech_1 | float | Read only | | | 0 | Type 2 Low speed Angle |
| 0X3046 | adc0ffset_1 | int32 | Read only | | | 2051 | adc sampling channel1 Zero current bias |
| 0X3047 | adc0ffset_2 | int32 | Read only | | | 2080 | adc sampling channel2 Zero current bias |
| 0X3048 | can_status | uint8 (printed "uin8") | Read only | | | 1 | CAN status |

(All rows p.20-29.)

### CANopen object dictionary tables (section 5, p.63-70)
- Status Feedback Parameters (p.65-66):
  | Index | Name | Attribute | Type | Unit |
  |---|---|---|---|---|
  | 603F | Error_code | Read-only | UINTEGER16 | / |
  | 6041 | Statusword | Read-only | UINTEGER16 | / |
  | 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
  | 6062 | Position_demand_value | Read-only | INTEGER32 | "Pulses(1rev=16,384 pulses)" |
  | 6064 | Position_actual_value | Read-only | INTEGER32 | "Pulses(1rev=16,384 pulses)" |
  | 606B | Velocity_demand_value | Read-only | INTEGER32 | 0.1rpm |
  | 606C | Velocity_actual_value | Read-only | INTEGER32 | 0.1rpm |
  | 6077 | Torque_actual_value | Read-only | INTEGER16 | "0.1% load ratio (1000=6N·m)" |
  | 6078 | Current_actual_value | Read-only | INTEGER16 | mA |
  | 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | mV |
- Homing Mode (p.67): `6040 Controlword` Read-write UINTEGER16; `6060 Modesofoperation` Read-write INTEGER8. "Set Modes of operation to 6 while disabled; motor defines current position as zero point. Controlword=15 to hold zero position."
- Position Mode PP (p.67-68): `6040 Controlword` RW UINTEGER16; `6060 Modesofoperation` RW INTEGER8; `6067 Position_window` RW UINTEGER32 "Pulses(1rev=16,384 pulses)"; `6068 Position_window_time` RW UINTEGER16 ms; `6071 Target_torque` RW INTEGER16 "0.1% load ratio (1000=6N·m)"; `607A Target_position` RW INTEGER32 "Pulses(1rev=16,384 pulses)"; `6081 Profile_velocity` RW UINTEGER32 0.1rpm; `6083 Profile_acceleration` RW UINTEGER32 0.1rpm/s. Mode value for PP = 1 (p.68).
- Position Mode CSP (p.68-69): same index set minus 6083, plus same others; Mode value for CSP = 5 (p.69).
- Velocity Mode (p.69-70): `6040` RW UINTEGER16; `6060` RW INTEGER8; `6071 Target_torque` RW INTEGER16 "0.1% load ratio(1000=6N·m)"; `60FF Target_velocity` RW INTEGER32 0.1rpm. Mode value = 3 (p.69).
- Torque Mode (p.70): `6040` RW UINTEGER16; `6060` RW INTEGER8; `6071 Target_torque` RW INTEGER16 "0.1% load ratio (1000=6N·m)". Mode value = 4 (p.70).
- Motor Watchdog (p.67): "Set the CAN communication watchdog via index 1 of object 0x6099. ... a value of 20000 represents 1 second. ... default value is 0 (disabled), and a non-zero value enables the watchdog." (Note: object given as "0x6099" here, distinct from the private-protocol `CAN_TIMEOUT` parameter at 0X200c / 0x7028 — transcribed exactly as printed, not reconciled.)
- Controlword values referenced in text (p.66-67): 6 / 7 / 15 (step transition or direct to OPERATION_ENABLE), 1 (stop, returns to SWITCH_ON_DISABLED), 11 (emergency stop — "Use with Caution, Risk of Voltage Surge").

## Fault/error codes
### Communication type 21 fault frame bit list (p.45-46) and Function code 0x3022 "faultSta" bit list (p.30) — identical content, two locations:
- Bit16: Motor current fault: A-phase current sampling overcurrent
- Bit14: Motor stall overload algorithm protection
- Bit9: Position initialization fault
- Bit8: Hardware identification fault
- Bit7: Encoder uncalibrated: Motor encoder not calibrated
- Bit5: Motor current fault: C-phase current sampling overcurrent
- Bit4 (type-21 frame only, p.46): B-phase current sampling overcurrent
- bit3: Overvoltage fault: "the motor voltage exceeds the protection voltage by 60V" (p.30) / "overvoltage fault" (p.46)
- bit2: Undervoltage fault: "the motor voltage is lower than the protection voltage of 12V" (p.30)
- bit1: Driver chip failure: Motor driver chip failure reported
- bit0: Motor overtemperature fault: "motor thermistor temperature exceeds 135 degrees" (p.30); "default 135°C" (p.46, p.74-75 MIT version)
(p.30 section 3.3.7 list omits bit4 explicitly; p.46 type-21 list includes bit4 "B-phase current sampling overcurrent". Transcribed exactly as printed in each location — not reconciled, see Open Questions.)

### MIT protocol Fault Status Response bit list (p.74-75) — same bits, re-labeled phases:
- Bit16: Phase A sampling overcurrent
- Bit14: Motor stall and overload algorithm protection
- Bit9: Position initialization fault
- Bit8: Hardware identification fault
- Bit7: Encoder uncalibrated
- Bit5: Phase C sampling overcurrent
- Bit4: Phase B sampling overcurrent
- Bit3: Overvoltage fault
- Bit2: Undervoltage fault
- Bit1: Driver chip fault
- Bit0: Motor overtemperature fault, default 135°C

### Driver chip fault code 1 — function code 0x3024, "Table 11. Fault Status Register 1 Field Descriptions" (p.31, rendered as an image/table, transcribed from the rendered table):
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

### Driver chip fault code 2 — function code 0x3025, "Table 12. Fault Status Register 2 Field Descriptions" (p.31, rendered image table):
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

(Note: driver IC family referenced by field names and "(DRV8353xS)" annotation implies a TI DRV8353-series gate driver, but the manual does not explicitly name the driver IC anywhere in running text — this is an inference from the register field names themselves as printed in the table, not a separate stated fact; flagged for awareness, not used as a hard claim elsewhere in this extract.)

### CAN communication failure protection (p.30, section 3.3.6)
- "When the value of CAN_TIMEOUT is 0, this function is disabled. When the CAN_TIMEOUT value is non-0, when the motor does not receive the can command within a certain period of time, the motor enters the reset mode, and 20000 is 1s" (p.30).

## OPEN QUESTIONS (anything ambiguous or could not confidently extract)
1. **No explicit "default CAN ID" statement for RS02.** Parameter table shows `CAN_ID` (0X200a) and `CAN_MASTER`/host id (0X200b) with example "current values" of 1 and 0 respectively (p.21), but the text never states these are factory defaults vs. just the values observed when the table example was captured. Worked examples throughout the manual instead use CAN ID `0x7F` (127) for the target motor and `0xFD` (253) for the host, consistently, but always introduced as "example" ("Take reading loc_kp as an example", p.50) rather than as defaults. Do not assume 0x7F/127 or 0xFD/253 are factory defaults without confirming against a capture or the Studio GUI default state.

2. **Bit-field layout of the 29-bit extended CAN ID is stated inconsistently across the document.** The per-frame-type header rows throughout section 4.1 (p.40-48) show columns "Bit28~bit24 | bit23~8 | bit7~0" labeled "Communication type | data area2 | Destination address", which doesn't by itself specify where the host/master CAN_ID lives. Elsewhere in the same frames, prose says things like "bit15~8: identifies host CAN_ID" (p.40) or "bit15~8: identifies the main CAN_ID" (p.42) — implying host ID sits in bits 15-8, motor/target ID in bits 7-0, and communication type in bits 28-24 (5 bits, since type values go up to 25 = 0x19, fits in 5 bits not 8). But the C struct in the code sample (p.56) defines `id:8, data:16, mode:5, res:3` (28 bits total), and the worked decode on p.31 confirms: low byte = target/motor CAN ID, next 16 bits = (unused/"no meaning") + host ID byte, top 5 bits = communication type (mode). This "id:8 / data:16 / mode:5 / res:3" structure from the code sample is the most concrete, self-consistent description found and was used as authoritative above, but the prose tables throughout section 4.1 do not exactly match it field-by-field (e.g. it's unclear whether the "data area2"/bit23~8 region is strictly the 16-bit host-id-bearing field, or something else) — flagged rather than resolved by guessing.

3. **Communication Type 1 frame (p.41-42) byte-to-field text is garbled/ambiguous as printed** ("Byte2:Torque target (0~65535) corresponds to CAN_ID (-17Nm~17Nm) ... Byte0~1:target Angle...Byte2~3:Target angular velocity...") — the printed prose seems to have fields overlapping/out of order (Byte2 used for both "Torque target" and start of "Target angular velocity"). I used the unambiguous C code sample (p.58, section 4.4.2) as the authoritative byte layout instead (position in Byte0-1, speed in Byte2-3, Kp in Byte4-5, Kd in Byte6-7, with torque packed separately into the CAN ID's "data" field via `txCanIdEx.data`), and flagged the prose table as unreliable. This needs confirmation against a real capture before being trusted for driver implementation.

4. **MIT protocol Response Command 1 / Command 3 nibble-packing order is stated but not fully unambiguous.** The text describes "Byte3: High 8 bits; Byte4[7-4] as low 4 bits" for a 12-bit speed field, and similar for torque and temperature — this literally describes a 12-bit value spread across one full byte plus a 4-bit nibble of the next byte, but doesn't fully spell out whether byte3 is the value's high byte or low byte in final assembled form. Transcribed exactly as printed (p.72-75); do not assume a specific endianness/order without testing against a capture.

5. **Fault bit list for "Motor current fault" bit4 is present in the Communication-Type-21 frame table (p.46, "Bit4: B-phase current sampling overcurrent") but absent from the otherwise-identical function-code-0x3022 bit list in section 3.3.7 (p.30)**, which jumps from bit5 to bit3 without mentioning bit4. Not reconciled — both transcribed as printed.

6. **bit20 in the Communication Type 2 / Type 24 feedback fault-info bit list is given two different labels** — "Uncalibrated" and "Gridlock overload fault" — both printed against bit20 in the same sentence (p.41, p.47-48). Could not determine which is correct, or whether these are two different conditions sharing one bit by mistake in the source. Transcribed verbatim.

7. **Document date mismatch.** Source PDF filename is "RS02User Manual260713.pdf" (suggesting a "26-07-13" or similar date code), but the in-document Version History table states Version 1.0, "Initial Release", dated "November 25, 2025" (p.79). These two dates are inconsistent; neither is explained in the document text. Not resolved.

8. **No single stated "firmware version" for the RS02 as shipped.** Only example/reference firmware field values appear in the parameter table (BootCodeVersion "0.1.5", AppCodeVersion "0.0.0.1", etc., p.20) and a minimum-required-firmware note "0.2.3.32" for MIT protocol Commands 12-15 (p.76-77, p.79-82). Cannot state a definitive "current/shipped firmware version" from this document alone.

9. **CANopen watchdog object "0x6099 index 1" (p.67) vs. private-protocol CAN_TIMEOUT parameters (0X200c in the flash-config table, p.21; 0x7028 in the read/write parameter list, p.49)** — three different-looking references to what may be the same underlying timeout concept, stated in three different places with three different addressing schemes (CANopen object/subindex vs. two different private-protocol function codes). The document does not reconcile these. Transcribed each exactly as printed; do not assume they are the same register without confirmation.

10. **Driver gate-driver IC identity** — Table 12 (p.31) field descriptions reference "(DRV8353xS)" in three rows (SA_OC, SB_OC, SC_OC), strongly suggesting a TI DRV8353-series driver IC, but no running text in the manual explicitly names the gate driver part number. Treat as a strong hint from the register table only, not a confirmed bill-of-materials fact.

11. **Table formatting collapse**: Many tables in the extracted text (e.g. Driver Product Specifications p.12, several frame headers throughout section 4) had their column spacing stripped by PDF text extraction (words run together, e.g. "CANbusbitrate 1Mbps"). Values themselves were verified against the rendered page images where checked (p.9, p.31, p.30) and are accurate, but column boundaries in tables not individually re-rendered as images (e.g. p.12 driver specs table, p.65-70 CANopen tables) are reconstructed from context/label adjacency rather than visually confirmed pixel layout. If exact column alignment matters, re-render those specific pages before implementing.
