# RS03 protocol extract
Source: RS03User Manual260713.pdf
PDF has 84 pages (confirmed via `mdls -name kMDItemNumberOfPages`). Page citations below ("p.N") are the absolute PDF page number (page 1 = cover). The manual also prints its own page numbers at the bottom of each page starting at "1" on PDF p.2; where useful the printed page number is given in brackets, e.g. "p.41 [40]".

Verification method: full text was extracted with `pdftotext -layout`, split per page, and grepped for CAN/protocol/parameter content. Every table transcribed below was additionally cross-checked by rendering the source page as an image via the Read tool to confirm the text extraction did not reorder or misalign table cells (confirmed accurate on pp.23, 24, 31, 41, 42, 68 — the pages with the most complex tables/diagrams).

## Model/firmware identification
- Model/product name as printed on cover: "ROBSTRIDE 03" with subtitle "60N.M Quasi-Direct Drive Integrated Motor Module", "User Manual", publisher "RobStride Dynamics", https://www.robstride.com/ (p.1).
- Manual title inside body text: "RS03 instruction manual" (p.2).
- Manufacturer/legal entity: "Beijing Lingfoot Times Technology Co., LTD. (hereinafter referred to as Lingfoot)" (p.3). Note: body text elsewhere also uses "Lingzu" / "Lingzu Times" (e.g. "official USB-CAN module of Lingzu Times", p.15; "Lingzu_motor" value, p.22) — manual is internally inconsistent between "Lingfoot" and "Lingzu" as the company name. Flagged, not resolved.
- Firmware/version fields visible in the default flash parameter table (p.22, function codes 0x1000–0x1007), "Current value (for reference)" column, i.e. example/factory values, not guaranteed defaults for every unit:
  - 0X1000 BootCodeVersion (String, Read only): "0.1.5"
  - 0X1001 BootBuildDate (String, Read only): "Mar 16 2022"
  - 0X1002 BootBuildTime (String, Read only): "20:22:09"
  - 0X1003 AppCodeVersion (String, Read only): "0.0.0.1" — remark "Motor program version number"
  - 0X1004 AppGitVersion (String, Read only): "7b844b0fM"
  - 0X1005 AppBuildDate (String, Read only): "Apr 14 2022"
  - 0X1006 AppBuildTime (String, Read only): "20:30:22"
  - 0X1007 AppCodeName (String, Read only): "Lingzu_motor"
- Several MIT-protocol commands (12, 13, 14, 15) are explicitly marked "(requires firmware update to 0.3.1.41)" (p.80–81 [79–80]).
- Version History table (p.84 [83]): "Version Number: 1.0 | Description: Initial Release | Date: November 25, 2025". (Note: this date is the only explicit document/firmware release date given; it does not match the file name date code "260713".)
- Default CAN ID: not explicitly stated as a single factory-default value anywhere found. The flash parameter table shows function code 0X2009 `CAN_ID` (uint8, attribute "Settings") with Maximum value "20", Minimum value "0", Current value (for reference) "1", remark "Motor index, marking the joint position of the motor" (p.23 [22]). Whether "1" is the true factory default or just an example reading is not stated — see Open Questions.
- Host ID value "0xFD" appears repeatedly as the example/host CAN ID in worked examples (pp.44–45 [43–44], p.51 [50], p.75 [74]) but is never explicitly called a "default" host ID in the text — see Open Questions.

## CAN bus parameters
- "CAN bus bit rate: 1Mbps" — Driver product Specifications table (p.14).
- "The motor communication is the CAN 2.0 communication interface, the baud rate is 1Mbps, and the extended frame format is adopted" (p.41 [40], section 4 intro).
- "The motor communication adopts the CAN 2.0 interface with a default baud rate of 1 Mbps. The baud rate can be modified by switching to the private protocol." (p.75 [74], MIT protocol section intro).
- Driver product specification table, full row set (p.14):
  - "The rated working voltage: 48VDC"
  - "The maximum allowable voltage: 60VDC"
  - "Rated working phase current: 12Apk"
  - "Maximum allowable phase current: 43Apk"
  - "Standby power: ≤40mA"
  - "CAN bus bit rate: 1Mbps"
  - "Dimensions: Φ70mm"
  - "Working environment temperature: -20℃ to 50℃"
  - "The maximum allowable temperature of the control board: 145℃"
  - "encoder resolution: 14bit (absolute turn)"
- Pinout for power/CAN connector (p.14–15, "1. Power supply and CAN communication"):
  - Pin 1: "The positive electrode of the power supply (+)"
  - Pin 2: "Negative electrode of the power supply (-)"
  - Pin 3: "CAN CAN_L"
  - Pin 4: "CAN the high side of the communication CAN_H"
- Termination resistor: "When DIP switch 2 is in the ON position, a 120Ω terminal resistor is connected to the module port, allowing normal communication with the host computer." (p.16 [15]). DIP switch 1 ON puts the recommended USB-CAN adapter module into Boot mode (not the motor itself) (p.15–16 [14–15]).
- Baud-rate modification options via Communication type 23 frame, F_CMD byte values (p.46 [45]): "01 is 1M", "02 is 500K", "03 is 250K", "04 is 125K". (Frame takes effect after re-power-on.)
- CAN communication watchdog/timeout (private protocol): parameter `CAN_TIMEOUT` at function code 0X200b (also exposed as read/write index 0x7028 `canTimeout`, uint32). Text: "When the value of CAN_TIMEOUT is 0, this function is disabled. When the CAN_TIMEOUT value is non-0, when the motor does not receive the can command within a certain period of time, the motor enters the reset mode, and 20000 is 1s" (p.30 [29], section 3.3.6).
- CANopen watchdog: "Set the CAN communication watchdog via index 1 of object 0x6099. After configuration, a value of 20000 represents 1 second. If no communication frame is received within the set time, the motor will be disabled. The default value is 0 (disabled), and a non-zero value enables the watchdog." (p.69 [68]).

## Frame definitions (TX - host to motor)
All frame tables below are quoted field-for-field as printed; where the manual's own column layout embeds bit-level sub-structure inside a single cell (common for the private/"driver" protocol), that sub-structure is reproduced as printed.

### General extended-frame (private/"driver") layout — p.41 [40]
```
data field        29-bit ID                                      8Byte data field
Size      Bit28~bit24      bit23~8          bit7~0                Byte0~Byte7
Desc.     Communication    data area 2      Destination address   data area 1
          type
```
Control modes listed (p.41 [40]): "Operation control mode", "Current mode", "Velocity mode", "Position mode".

### Type 0 (0x0): Get device ID — p.41 [40]
Request:
```
Size   Bit28~bit24  bit23~8                        bit7~0         Byte0~Byte7
Desc.  0x0          bit15~8: identifies host        target motor  0
                     CAN_ID                         CAN_ID
```
Reply:
```
Size   Bit28~bit24  bit23~8                 bit7~0   Byte0~Byte7
Desc.  0x0          target motor CAN_ID     0XFE     64-bit MCU unique identifier
```
(Note: reply's bit7~0 field is the fixed constant 0XFE, not an echoed host ID.)

### Type 1 (0x1): Operation control mode motor control instruction — p.42 [41]
```
Size   Bit28~bit24  bit23~8                      bit7~0        Byte0~Byte7
Desc.  0x1          Byte2: Torque (0~65535)      target motor  Byte0~1: target Angle
                     corresponds to (-60Nm~60Nm) CAN_ID        [0~65535] corresponds to
                                                                (-4π~4π)
                                                                Byte2~3: Target angular
                                                                velocity [0~65535]
                                                                corresponds to
                                                                (-20rad/s~20rad/s)
                                                                Byte4~5: Kp [0~65535]
                                                                corresponds to (0.0~5000.0)
                                                                Byte6~7: Kd [0~65535]
                                                                corresponds to (0.0~100.0)
                                                                After conversion, high byte
                                                                is first, low byte follows.
```
Response: "Response motor feedback frame (see communication type 2)".
Confirmed by C code sample (pp.61–62 [60–61]): `float_to_uint(torque,T_MIN,T_MAX,16)` packed into the ID's `data` field (bit23~8), and `P_MIN=-12.57f, P_MAX=12.57f, V_MIN=-20.0f, V_MAX=20.0f, KP_MIN=0.0f, KP_MAX=5000.0f, KD_MIN=0.0f, KD_MAX=100.0f, T_MIN=-60.0f, T_MAX=60.0f` (p.58 [57]). Note: these program-sample constants use P_MIN/P_MAX = ±12.57 rad, not ±4π as stated in the Type 1/Type 2 table text (4π ≈ 12.566) — consistent, not a conflict, just two ways of writing the same bound.

### Type 3 (0x3): Motor enabled to run — p.43 [42]
```
Size   Bit28~bit24  bit23~8                        bit7~0               Byte0~Byte7
Desc.  0x3          bit15~8: identifies the main    and target motor    (blank/unspecified
                     CAN_ID                         CAN_ID               in table)
```
Response: "Response motor feedback frame (see communication type 2)".
C sample `motor_enable()` (p.60 [59]): sets `txCanIdEx.mode=3`, `txCanIdEx.id=id`, `txCanIdEx.res=0`, `txCanIdEx.data=master_id` then overwrites `txCanIdEx.data=0`, dlen=8.

### Type 4 (0x4): Motor stops running — p.43 [42]
```
Size   Bit28~bit24  bit23~8                    bit7~0        Byte0~Byte7
Desc.  0x4          bit15~8: used to identify   target motor  When motor is running normally,
                     the main CAN_ID            CAN_ID        0 must be cleared in the data
                                                               field. Byte[0]=1: The fault is
                                                               cleared.
```
Response: "Response motor feedback frame (see communication type 2)".

### "Version number read frame" — p.43 [42] (also uses comm type 0x4)
Request:
```
Size   Bit28~bit24  bit23~8                      bit7~0        Byte0~Byte7
Desc.  0x4          bit15~8: used to identify     target motor  Byte[0]=0x00
                     the main CAN_ID              CAN_ID        Byte[1]=0xC4
```
Response (uses comm type 0x2, same frame type as motor feedback):
```
Size   Bit28~bit24  bit23~8 (fault/mode bits, see Type 2)  bit7~0        Byte0~Byte7
Desc.  0x2          [same fault-bit layout as Type 2]      target motor  Byte0=0x00
                                                            CAN_ID        Byte1=0xC4
                                                                          Byte2=0x56
                                                                          Byte3~6: Motor version
                                                                          number, order high to low
```
Note: this "version read" request reuses comm type 0x4 (same code as "stop running") distinguished only by its data-byte payload (Byte0=0x00, Byte1=0xC4); this is called out explicitly rather than inferred, since it is an unusual protocol design choice worth flagging.

### Type 6 (0x6): Set motor mechanical zero — p.44 [43]
```
Size   Bit28~bit24  bit23~8                      bit7~0              Byte0~Byte7
Desc.  0x6          bit15~8: Identifies the main  and target motor   Byte[0]=1
                     CAN_ID                       CAN_ID
```
Response: "Response motor feedback frame (see communication type 2)".

### Type 7 (0x7): Set motor CAN_ID — p.44 [43]
"Change the current motor CAN_ID, effective immediately."
```
Size   Bit28~bit24  bit23~8                                       bit7~0        Byte0~Byte7
Desc.  0x7          bit15~8: used to identify main CAN_ID         Target motor  (blank in table)
                     Bit16~23: preset CAN_ID                      CAN_ID
```
Response: "Answer motor broadcast frame (see communication type 0)".

### Type 17 (0x11): Single parameter read — p.44–45 [43–44]
Request:
```
Size   Bit28~bit24  bit23~8                     bit7~0        Byte0~Byte7
Desc.  0x11         bit15~8: Used to identify    target motor  Byte0~1: index (see read/write
                     the main CAN_ID             CAN_ID        parameter table) Byte2~3: 00
                                                                Byte4~7: 00 (low byte first,
                                                                high byte second)
```
Reply:
```
Size   Bit28~bit24  bit23~8                               bit7~0  Byte0~Byte7
Desc.  0x11         bit15~8: indicates the master CAN_ID  (text   Byte0~1: index  Byte2~3: 00
                     Bit23~16: 00 indicates master CAN_ID  runs   Byte4~7: Parameter data (low
                     successfully read; 01 indicates ...  into    byte first, high byte last)
                     ["the master can_ID" — sentence cut  bit7~0
                     off in source]                        col)
```
Note: the reply-frame table's text for bit23~16 is cut off mid-sentence in the source ("...01 indicates that the master can_ID") — transcribed verbatim, meaning of the "01" value is not completed in the manual text. See Open Questions.

### Type 18 (0x12): Single parameter write (lost in power failure) — p.45 [44]
"With type 22, the parameter starting with function code 0x20 of the parameter table in the upper computer module can be saved"
```
Size   Bit28~bit24  bit23~8                     bit7~0        Byte0~Byte7
Desc.  0x12         bit15~8: Used to identify    target motor  Byte0~1: index (see read/write
                     the main CAN_ID             CAN_ID        parameter table) Byte2~3: 00
                                                                Byte4~7: Parameter data (low
                                                                byte first, high byte second)
```
Response: "Response motor feedback frame (see communication type 2)".
C samples confirm byte layout (pp.63–65 [62–64]): `memcpy(&txMsg.tx_data[0],&index,2)` then for mode-switch: `memcpy(&txMsg.tx_data[4],&runmode,1)`; for generic param write: `memcpy(&txMsg.tx_data[4],&ref,4)`.

### Type 22 (0x16): Motor data save frame — p.46 [45]
```
Size   Bit28~bit24  bit23~8                      bit7~0              Byte0~Byte7
Desc.  0x16         bit15~8: identifies the main  and target motor   01 02 03 04 05 06 07 08
                     CAN_ID                       CAN_ID
```
Response: "Response motor feedback frame (see communication type 2)".

### Type 23 (0x17): Motor baud rate modification frame (re-power-on effect) — p.46 [45]
```
Size   Bit28~bit24  bit23~8                     bit7~0        Byte0~Byte7
Desc.  0x17         bit15~8: used to identify    target motor  01 02 03 04 05 06 F_CMD
                     the main CAN_ID             CAN_ID        F_CMD = motor baud rate:
                                                                01=1M, 02=500K, 03=250K,
                                                                04=125K
```
Response: "Response motor feedback frame (see communication type 0)".

### Type 24 (0x18): Motor actively reports frames — p.47 [46]
Enable/disable (host→motor):
```
Size   Bit28~bit24  bit23~8                     bit7~0        Byte0~Byte7
Desc.  0x18         bit15~8: identifies the main target motor  01 02 03 04 05 06 F_CMD
                     CAN_ID                      CAN_ID        F_CMD: 00=disable active
                                                                reporting (default), 01=enable;
                                                                default reporting interval 10ms
```
Response (motor→host, periodic report — see RX section): same byte layout as Type 2 feedback.

### Type 25 (0x19): Motor protocol modification frame (re-power-on effect) — p.48 [47]
```
Size   Bit28~bit24  bit23~8                     bit7~0        Byte0~Byte7
Desc.  0x19         bit15~8: used to identify    target motor  01 02 03 04 05 06 F_CMD
                     the main CAN_ID             CAN_ID        F_CMD: motor protocol type:
                                                                0=private protocol (default),
                                                                1=CANopen protocol,
                                                                2=MIT protocol
```
Response: "Response motor feedback frame (see communication type 0)".

### Private-protocol read/write parameter list (0x7xxx index), used by Type 17/18 frames — p.49–51 [48–50]
Columns as printed: index | Description | Type | Number of bytes | Range/default | R/W permission.
| index | name (from "Description" wording) | Type | Bytes | Range / default (verbatim) | R/W |
|---|---|---|---|---|---|
| 0X7005 | run_mode: "0: operation mode / 1: position mode (PP) / 2: Velocity mode / 3: Operation mode Current mode / 5: Position mode (CSP)" | uint8 | 1 | — | W/R |
| 0X7006 | iq_ref — "Current mode Iq command" | float | 4 | "-43 to 43A" | W/R |
| 0X700A | spd_ref — "Rotational Velocity mode Rotational speed command" | float | 4 | "-20to 20rad/s" | W/R |
| 0X700B | limit_torque — "torque limit" | float | 4 | "0 to 60Nm" | W/R |
| 0X7010 | cur_kp — "Kp" | float | 4 | "The default value is 0.17" | W/R |
| 0X7011 | cur_ki — "Ki" | float | 4 | "The default value is 0.012" | W/R |
| 0X7014 | cur_filt_gain — "filt_gain" | float | 4 | "0 to 1.0, The default value is 0.1" | W/R |
| 0X7016 | loc_ref — "Position Mode Angle instruction" | float | 4 | "rad" | W/R |
| 0X7017 | limit_spd — "Location mode (CSP) speed limit" | float | 4 | "0 to 20rad/s" | W/R |
| 0X7018 | limit_cur — "Velocity position mode Current limitation" | float | 4 | "0 to 43A" | W/R |
| 0x7019 | mechPos — "Mechanical Angle of the loading coil" | float | 4 | "rad" | R |
| 0x701A | iqf — "iq Filter" | float | 4 | "-43 to 43A" | R |
| 0x701B | mechVel — "Speed of the load" | float | 4 | "-20 to 20rad/s" | R |
| 0x701C | VBUS — "Bus voltage" | float | 4 | "V" | R |
| 0x701E | loc_kp — "kp at" | float | 4 | "The default value is 60" | W/R |
| 0x701F | spd_kp — "Indicates the speed kp" | float | 4 | "The default value is 6" | W/R |
| 0x7020 | spd_ki — "ki" | float | 4 | "The default value is 0.02" | W/R |
| 0x7021 | spd_filt_gain — "Speed filter value" | float | 4 | "The default value is 0.1" | W/R |
| 0x7022 | acc_rad — "velocity mode acceleration" | float | 4 | "The default value is 20rad/s^2" | W/R |
| 0x7024 | vel_max — "Location mode (PP) speed" | float | 4 | "The default value is 10rad/s" | W/R |
| 0x7025 | acc_set — "Location mode (PP) acceleration" | float | 4 | "The default value is 10rad/s^2" | W/R |
| 0x7026 | EPScan_time — "Indicates the report time. 1 indicates 10ms. Plus 1 increments by 5ms" | uint16 | 2 | "The default value is 1" | W/R |
| 0x7028 | canTimeout — "can The timeout threshold, 20000 is 1s" | uint32 | 4 | "The default value is 0" | W/R |
| 0x7029 | zero_sta — "Indicates the zero flag bit, 0 means 0-2π and 1 means -π-π" | uint8 | 1 | "The default is 0" | W/R |
| 0x702A | damper — "Damping switch" | uint8 | 1 | "The default value is 0" | W/R |
| 0x702B | add_offset — "Zero offset" | float | 4 | "The default is 0" | W/R |
| 0x702C | alveolous_open — "Cogging Compensation Switch" | uint8 | 1 | "The default is 0" | W/R |
| 0x702D | iq_test — "Motor Initialization Calibration Switch" | uint8 | 1 | "The default is 0" | W/R |
| 0x702E | dcc_set — "Deceleration in Motor PP Mode (Normally Disabled)" | float | 4 | "The default is 10rad/s^2" | W/R |

Read example (p.51 [50]), reading `loc_kp`:
Read instruction:
```
Bit28~bit24=0x11 (Type 17) | bit23~8=0x00FD (Host id 0xFD) | bit7~0=0x7F (Target motor CAN_ID 7F)
Byte0~7: 1E 70 00 00 00 00 00 00   (Byte0~1 = index, corresponds to loc_kp)
```
Feedback:
```
Bit28~bit24=0x11 (Type 17) | bit23~8=0x007F (Target motor CAN_ID 7F) | bit7~0=0xFD (Host id 0xFD)
Byte0~7: 1E 70 00 00 00 00 F0 41   (Byte0~1 = index; Byte4~7 = loc_kp value 30, IEEE-754 32-bit float, high byte right)
```

### CANopen: generic protocol-switch frame (extended) — p.73–74 [72–73]
```
Data Field    29-bit ID    8-Byte Data Area
Size          Bit 28~0     Byte 0~6
Desc.         0xFFF        01 02 03 04 05 06 F_CMD
```
F_CMD (Byte 6): "0: Private protocol (default) / 1: CANopen protocol / 2: MIT protocol". "Takes Effect After Power Cycle."
Response frame:
```
Data Field    11-bit ID    8-Byte Data Area
Size          Bit 10~0     Byte 0~7
Desc.         Motor ID     64-bit MCU unique identifier
```

### CANopen object dictionary writes (host→motor), standard DS402-style — pp.68–73 [67–72]
(These are SDO/PDO-style object accesses, not raw CAN frame byte tables — the manual gives only index/name/attribute/type/unit, not the underlying CAN frame bytes.)
- `6040 Controlword` (Read-write, UINTEGER16, unit "/"). State transitions cited in text (p.69 [68]/state diagram p.68 [67]): "modify the Controlword (6040H) to 6, 7, or 15" to progress toward OPERATION_ENABLE; "directly set it to 15 for immediate enablement"; "modify Controlword to 1" to stop normally (→ SWITCH_ON_DISABLED); "setting the Controlword to 11" for emergency stop; fault-state Controlword values clear "standard errors" (unspecified value). State diagram (p.68) shows transition labels: START→NOT_READY_TO_SWITCH_ON (unlabeled/dashed)→SWITCH_ON_DISABLED (unlabeled/dashed)→READY_TO_SWITCH_ON (label "6")→SWITCHED_ON (label "7")→OPERATION_ENABLE (label "15"); OPERATION_ENABLE↔QUICK_STOP_ACTIVE labeled "11" (out) / "15" (back); OPERATION_ENABLE→SWITCH_ON_DISABLED labeled "15" and "1" (two arrows, different paths, both pointing back to SWITCH_ON_DISABLED per diagram); FAULT_REACTION_ACTIVE→SWITCH_ON_DISABLED labeled "130".
- `6060 Modes of operation` (Read-write, INTEGER8, unit "/"). Mode values used in steps: 1=Position Mode(PP) (p.71 [70]), 5=Position Mode(CSP) (p.72 [71]), 3=Velocity Mode (p.73 [72]), 4=Torque Mode (p.73 [72]), 6=Homing mode (p.69 [68]).
- Status Feedback table (p.69–70 [68–69]):
  | Index | Name | Attribute | Type | Unit |
  |---|---|---|---|---|
  | 603F | Error_code | Read-only | UINTEGER16 | / |
  | 6041 | Statusword | Read-only | UINTEGER16 | / |
  | 6061 | Modes_of_operation_display | Read-only | INTEGER8 | / |
  | 6062 | Position_demand_value | Read-only | INTEGER32 | "Pulses (1 rev = 16,384 pulses)" |
  | 6064 | Position_actual_value | Read-only | INTEGER32 | "Pulses (1 rev = 16,384 pulses)" |
  | 606B | Velocity_demand_value | Read-only | INTEGER32 | "0.1 rpm" |
  | 606C | Velocity_actual_value | Read-only | INTEGER32 | "0.1 rpm" |
  | 6077 | Torque_actual_value | Read-only | INTEGER16 | "0.1% load ratio (1000 = 20 N·m)" |
  | 6078 | Current_actual_value | Read-only | INTEGER16 | "mA" |
  | 6079 | DC_link_circuit_voltage | Read-only | INTEGER32 | "mV" |
- Homing Mode (p.69 [68]): `6040 Controlword` (R/W, UINTEGER16), `6060 Modes of operation` (R/W, INTEGER8). "Set Modes of operation to 6 while disabled... motor will then define the current position as the zero point." "To hold the zero position, modify the Controlword to 15."
- Position Mode PP (p.69–70 [69–70]): `6040`, `6060`, `6067 Position_window` (R/W, UINTEGER32, "Pulses (1 rev = 16,384 pulses)"), `6068 Position_window_time` (R/W, UINTEGER16, "ms"), `6071 Target_torque` (R/W, INTEGER16, "0.1% load ratio (1000 = 20 N·m)"), `607A Target_position` (R/W, INTEGER32, "Pulses (1 rev = 16,384 pulses)"), `6081 Profile_velocity` (R/W, UINTEGER32, "0.1 rpm"), `6083 Profile_acceleration` (R/W, UINTEGER32, "0.1 rpm/s").
- Position Mode CSP (p.71–72 [70–71]): `6040`, `6060`, `6067`, `6068`, `6071`, `607A`, `6081` (same types/units as PP table; no `6083` listed for CSP).
- Velocity Mode (p.72–73 [71–72]): `6040`, `6060`, `6071 Target_torque`, `60FF Target_velocity` (R/W, INTEGER32, "0.1 rpm"). Mode value 3.
- Torque Mode (p.73 [72]): `6040`, `6060`, `6071 Target_torque` (unit "0.1% load ratio (1000 = 20N·m)"). Mode value 4.

### MIT protocol — standard 11-bit frame format — p.75 [74]
```
Data Field    11-bit ID            8-byte Data Area
Size          Bit 10~8  Bit 7~0    Byte 0~7
Desc.         Mode type ID
```
Control modes listed: "MIT Mode", "Velocity Mode", "Position Mode".

### MIT Command 1: Enable Motor Operation — p.76 [75]
```
11-bit ID: Bit 10~0 = 07F (Target motor CAN ID)
Data: FF FF FF FF FF FF FF FC
```
Response: Response Command 1.

### MIT Command 2: Stop Motor Operation — p.76 [75]
```
11-bit ID: Bit 10~0 = 07F (Target motor CAN ID)
Data: FF FF FF FF FF FF FF FD
```
Response: Response Command 1.

### MIT Command 3: MIT Dynamic Parameters — p.76–77 [75–76]
```
11-bit ID: Bit10~0 = 07F (Target motor CAN ID)
Byte0~1: Target angle [0~65535] corresponds to (-12.57 rad ~ 12.57 rad)
Byte2: high 8 bits / Byte3[7:4] (high 4 bits as low 4 bits): Target speed [0~4096] corresponds to (-20 rad/s ~ 20 rad/s)
Byte3[3:0] (low 4 bits as high 4 bits) / Byte4: low 8 bits: Kp [0~4096] corresponds to 0~5000
Byte5: high 8 bits / Byte6[7:4] (high 4 bits as low 4 bits): Kd [0~4096] corresponds to 0~100
Byte6[3:0] (low 4 bits as high 4 bits) / Byte7: low 8 bits: Target torque [0~4096] corresponds to (-60 N·m ~ 60 N·m)
```
This is a nibble-packed 12-bit-per-field layout split across byte boundaries — transcribed exactly as printed; the "high 4 bits as low 4 bits" / "low 4 bits as high 4 bits" wording is the manual's own (unusual) phrasing and is reproduced verbatim rather than reinterpreted. Response: Response Command 1.

### MIT Command 4: Set Zero Position (Non-Position Mode) — p.77 [76]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF FF FE
```
Response: Response Command 1.

### MIT Command 5: Clear Errors & Read Fault Status — p.77 [76]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF F_CMD FB
F_CMD: 0xFF = clear current abnormality; any other value = error code returned in BYTE1 of response.
```
Response (Fault Clear): Response Command 1.

### MIT Command 6: Set Operation Mode — p.78 [77]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF F_CMD FC
F_CMD: Mode type — 0: MIT mode (default), 1: Position mode, 2: Velocity mode
```
Response: Response Command 1.

### MIT Command 7: Modify Motor CAN ID — p.78 [77]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF F_CMD FA
F_CMD: Target motor CAN ID
```
Response: Response Command 2.

### MIT Command 8: Change Communication Protocol (Takes Effect After Power Cycle) — p.79 [78]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF F_CMD FD
F_CMD: Protocol type — 0: Private protocol (default), 1: CANopen, 2: MIT protocol
```
Response: Response Command 2.

### MIT Command 9: Modify Host CAN ID — p.79 [78]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF F_CMD 01
F_CMD: Host CAN ID
```
Response: Response Command 2.

### MIT Command 10: Position Mode Control Command — p.79 [78]
```
11-bit ID: Bit10~8 = 1, bit7~0 = 7F (Target motor CAN ID)
Data example: 00 00 A0 40 00 00 A0 40  (Position: 5 rad, Speed: 5 rad/s)
Byte0~3: Target position (rad, 32-bit float)
Byte4~7: Target speed (rad/s, 32-bit float)
```
Response: Response Command 1.

### MIT Command 11: Velocity Mode Control Command — p.80 [79]
```
11-bit ID: Bit10~8 = 2, bit7~0 = 7F (Target motor CAN ID)
Data example: 00 00 A0 40 00 00 00 40  (Speed: 5 rad/s, Current: 2 A)
Byte0~3: Target speed (rad/s, 32-bit float)
Byte4~7: Current limit in speed/position mode (A, 32-bit float)
```
Response: Response Command 1.

### MIT Command 12: Save motor data (requires firmware ≥ 0.3.1.41) — p.80 [79]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF FF F8
```
Response: Response Command 2.

### MIT Command 13: Motor actively reports (requires firmware ≥ 0.3.1.41) — p.80 [79]
```
11-bit ID: Bit10~0 = 07F
Data: FF FF FF FF FF FF F_CMD F9
F_CMD: 0 = no reporting (default), 1 = reporting
```
Response: Response Command 1.

### MIT Command 14: Read motor parameters (requires firmware ≥ 0.3.1.41) — p.80–81 [79–80]
```
11-bit ID: Bit10~8 = 3, bit7~0 = 7F
Data example: 05 70 00 00 00 00 00 00
Byte0~1: index (see read/write parameter table — same as private protocol parameter table)
Byte2~3: 00
Byte4~7: 00 (low byte first, high byte last)
```
Response frame: same layout, index echoed, "same as the private protocol parameter table".

### MIT Command 15: Write motor parameters (requires firmware ≥ 0.3.1.41) — p.81 [80]
```
11-bit ID: Bit10~8 = 4, bit7~0 = 7F
Data example: 29 70 00 00 00 00 00 00
Byte0~1: index (same parameter table as private protocol)
Byte2~3: 00
Byte4~7: Parameter data (low byte first, high byte last)
```
Response frame: identical layout.

## Frame definitions (RX - motor to host)

### Type 2 (0x2): motor feedback data — p.42 [41]
(This is the response frame for Type 1/3/4/6, and also the frame used for Type 24 active reporting and the Type-4 "version read" reply shares this ID type.)
```
Size   Bit28~bit24  bit23~8                                       bit7~0    Byte0~Byte7
Desc.  0x2          Bit8~Bit15: CAN ID of the current motor        host     Byte0~1: Current Angle
                     bit21~16: fault information (0 none, 1 has):  CAN_ID   [0~65535] corresponds to
                       bit21: uncalibrated                                  (-4π~4π)
                       bit20: Uncalibrated / Gridlock overload fault        Byte2~3: Current angular
                         (table lists bit20 twice, see note below)          velocity [0~65535]
                       bit19: magnetic coding fault                         corresponds to
                       bit18: overtemperature                               (-20rad/s~20rad/s)
                       bit17: Three-phase overcurrent fault                 Byte4~5: Current torque
                       bit16: undervoltage fault                            [0~65535] corresponds to
                     bit22~23: Mode status:                                 (-60Nm~60Nm)
                       0: Reset mode [reset]                                Byte6~7: Current
                       1: Cali mode [calibration]                           temperature: Temp(Celsius)*10.
                       2: Motor mode [Run]                                  If value >10 [as printed —
                                                                             see Open Questions], high
                                                                             byte is first, low byte is last.
```
Note on bit20: the source text literally reads "bit21: uncalibrated bit20: Uncalibrated bit20: Gridlock overload fault bit19: magnetic coding fault..." — bit20 is labeled twice with two different meanings ("Uncalibrated" and "Gridlock overload fault"). Transcribed verbatim; not resolved. This same wording/ambiguity repeats identically at: Type-4 version-read reply (p.43–44 [42–43]), Type 24 active-report reply (p.47 [46]).

### Type 0 reply, Type 7 reply, Type 23/25 replies
See TX section above — these reuse the Type 0 / Type 2 reply formats and are cited there to avoid duplication.

### "Version number read frame" reply — see TX section (shares comm type 0x2 format, p.43–44 [42–43]).

### Type 21 (0x15): Fault feedback frame — p.45–46 [44–45]
```
data field    29-bit ID                                     8Byte data field
Size          Bit28~bit24   bit23~8          bit7~0          Byte0~Byte7
Desc.         0x15          bit15~8:         identifies      Byte0~3: fault value (non-0: faulty;
                             motor            the main        0: Normal)
                                              CAN_ID            Bit 16: A-phase current sampling overcurrent
                                                                 Bit 14: Motor stall overload algorithm protection
                                                                 Bit 9: Position initialization fault
                                                                 Bit 8: Hardware identification fault
                                                                 Bit 7: Encoder uncalibrated
                                                                 Bit 5: C-phase current sampling overcurrent
                                                                 Bit 4: B-phase current sampling overcurrent
                                                                 Bit 3: overvoltage fault
                                                                 Bit 2: undervoltage fault
                                                                 Bit 1: driver chip fault
                                                                 Bit 0: motor overtemperature fault, default 145°C
                                                               Byte4~7: warning Value
                                                                 Bit 0: motor overtemperature warning, default 135°C
```
Note: the "Size/Desc" row for this frame's ID columns reads oddly in the source ("bit23~8: motor | bit7~0: identifies the main CAN_ID") — word order looks transposed versus every other frame table in the document (which put "identifies the main CAN_ID" in bit23~8 and the target/motor ID in bit7~0). Transcribed exactly as printed; flagged as likely a source typo rather than resolved by inference. No example CAN ID is given to disambiguate.

### Type 24 (0x18) active-report reply — p.47 [46]
Same byte layout as Type 2 feedback (Current Angle / angular velocity / torque / temperature), with the same bit20-dual-label fault/mode-status note as above. Target field is explicitly "target motor CAN_ID" rather than "host CAN_ID" in this instance of the table — transcribed as printed.

### MIT Response Command 1: Data Feedback (Motor Status) — p.75–76 [74–75]
```
11-bit ID: bit10~0 = 0FD (printed value; "Host ID")
Data example: 7F 00 00 00 00 00 00 00
Byte0: Motor CAN ID
Byte1~2: Current angle [0~65535], corresponds to (-12.57 rad ~ 12.57 rad)
Byte3: high 8 bits / Byte4[7:4] (high 4 bits as low 4 bits): Current speed [0~4096], corresponds to (-20 rad/s ~ 20 rad/s)
Byte4[3:0] (low 4 bits as high 4 bits) / Byte5: low 8 bits: Current torque [0~4096], corresponds to (-60 N·m ~ 60 N·m)
Byte6[7:6]: Operation mode — 0: Reset Mode, 1: Cali Mode, 2: Motor Mode
Byte6[5]: 1 = Motor fault, 0 = No motor fault
Byte6[4]: 1 = Motor warning, 0 = No warning
Byte6[3:0] (low 4 bits as high 4 bits) / Byte7: low 8 bits: Winding temperature = Temperature(°C) × 10, Unit: °C
```
Same nibble-packing style/ambiguity as MIT Command 3 — transcribed verbatim.

### MIT Response Command 2: MCU Identification — p.76 [75]
```
11-bit ID: Bit10~0 = Motor ID
Data: 64-bit MCU unique identifier
```

### MIT Fault Status Response (follow-up to Command 5) — p.77–78 [76–77]
```
11-bit ID: bit10~0 = 0FD
Data example: 7F 00 00 00 00 00 00 00
Byte0: Motor CAN ID
Byte1~4: Fault value (Non-zero: fault; 0: normal)
  Bit16: Motor current fault: Phase A sampling overcurrent
  Bit14: Motor stall and overload algorithm protection
  Bit9: Position initialization fault
  Bit8: Hardware identification fault
  Bit7: Encoder uncalibrated
  Bit5: Motor current fault: Phase C sampling overcurrent
  Bit4: Motor current fault: Phase B sampling overcurrent
  Bit3: Overvoltage fault
  Bit2: Undervoltage fault
  Bit1: Driver chip fault
  Bit0: Motor overtemperature fault, default: 145°C
```
This fault bit list matches the Type 21 fault-feedback bit list exactly (both: bits 16,14,9,8,7,5,4,3,2,1,0).

### CANopen protocol-switch response — p.74 [73]
```
Data Field    11-bit ID    8-Byte Data Area
Size          Bit 10~0     Byte 0~7
Desc.         Motor ID     64-bit MCU unique identifier
```

## Parameter table (flash config read/write)
Full default/flash parameter table, function codes 0X0000–0X3049, from the host-computer "Parameter settings" screen (pp.22–28 [21–27]). Header row as printed: "function code | name | parameter type | attribute | Maximum value | Minimum value | Current value (for reference) | 备注 (remarks)". "Current value (for reference)" is an example reading from a connected unit, not necessarily factory default, per manual text: "Click Refresh Parameter Table... the interface will display the motor's parameters" (p.21 [20]). Attribute column values seen: "Read/Write", "Read only", "Settings", and "disposition" (the last appears only for 0X2000–0X2003 `echoPara1..4`, printed exactly as "disposition" — likely an imperfect translation, not changed).

Caution printed verbatim: "Note: Please do not change the torque limit, protection temperature and overtemperature time of the motor. Our company will not bear any legal responsibility for any damage to human body or irreversible damage to joints caused by illegal operation of this product." (p.22 [21]).

| Code | Name | Type | Attribute | Max | Min | Current (ref.) | Remarks |
|---|---|---|---|---|---|---|---|
| 0X0000 | Name | String | Read/Write | | | (redacted/unreadable glyphs in source) | |
| 0X0001 | BarCode | String | Read/Write | | | (redacted/unreadable glyphs in source) | |
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
| 0X2006 | Chasu_offset | float | Read/Write | 50 | -50 | 4.52 | Reserved parameter |
| 0X2007 | Status1 | float | Read/Write | 17 | 0 | 17 | Torque limitation |
| 0X2008 | I_FW_MAX | float | Read/Write | 33 | 0 | 0 | Weak magnetic current value, default 0 |
| 0X2009 | CAN_ID | uint8 | Settings | 20 | 0 | 1 | Motor index, marking the joint position of the motor |
| 0X200a | CAN_MASTER | uint8 | Settings | 127 | 0 | 1 | id of this object |
| 0X200b | CAN_TIMEOUT | uint8 | Settings | 127 | 0 | 0 | can host id |
| 0X200c | status2 | uint32 | Read/Write | 100000 | 0 | 0 | can timeout threshold. The default value is 0 |
| 0X200d | status3 | int16 | Read/Write | 1500 | 0 | 800 | Reserved parameter |
| 0X200e | Status4 | uint32 | Read/Write | 1000000 | 1000 | 20000 | Reserved parameter |
| 0X200f | Status5 | float | Read/Write | 64 | 1 | 7.75 | Reserved parameter |
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
| 0X201a | limit_a | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201b | fault1 | float | Read/Write | 100 | 0 | 0 | Reserved parameter |
| 0X201c | fault2 | float | Read/Write | 27 | 0 | 0 | High speed segment offset |
| 0X201d | fault3 | float | Read/Write | 27 | 0 | 0 | The low end is offset |
| 0X201e | fault4 | float | Read/Write | 150 | 0 | (blank) | Velocity-mode acceleration |
| 0X201f | fault5 | float | Read/Write | 20 | 0 | (blank) | PP mode speed |
| 0X2020 | fault6 | float | Read/Write | 1000 | 0 | (blank) | PP mode acceleration |
| 0X2021 | fault7 | float | Read/Write | 100 | 0 | 0 | Zero marker |
| 0X2022 | baud | uint8 | Read/Write | 10 | 0 | 0 | Baud rate flag |
| 0X2023 | zero_sta | uint8 | Read/Write | (blank) | (blank) | 0 | Zero point flag |
| 0X2024 | position_offset | uint8 | Read/Write | 27 | 0 | 0 | Position offset |
| 0x2025 | protocol_1 | uint8 | Read/Write | (blank) | (blank) | 0 | Protocol flag |
| 0x2026 | damper | uint8 | Read/Write | 0 | 20 | 0 | Damping switch — **Max/Min order as printed is inverted vs. every other row (Max=0 < Min=20); confirmed by direct page image, not an extraction error. Flagged, not corrected.** |
| 0x2027 | add_offset | float | Read/Write | -7 | 7 | 0 | Position offset parameter — **Max/Min also appear inverted here (-7 then 7); transcribed verbatim.** |
| 0X3000 | timeUse0 | uint16 | Read only | | | 5 | |
| 0X3001 | timeUse1 | uint16 | Read only | | | 0 | |
| 0X3002 | timeUse2 | uint16 | Read only | | | 10 | |
| 0X3003 | timeUse3 | uint16 | Read only | | | 0 | |
| 0X3004 | encoderRaw | int16 | Read only | | | 11396 | Magnetic encoder sampling value |
| 0X3005 | mcuTemp | int16 | Read only | | | 337 | mcu internal temperature, *10 |
| 0X3006 | motorTemp | int16 | Read only | | | 333 | Motor ntc temperature, *10 |
| 0X3007 | vBus(mv) | uint16 | Read only | | | 24195 | Bus voltage |
| 0X3008 | adc1Offset | int32 | Read only | | | 2084 | adc sampling channel 1 zero current bias |
| 0X3009 | adc2Offset | int32 | Read only | | | 2084 | adc sampling channel 2 zero current bias |
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
| 0X3022 | faultSta | uint32 | Read only | | | 0 | Fault status value (see Fault/error codes section) |
| 0X3023 | warnSta | uint32 | Read only | | | 0 | Warning status value |
| 0X3024 | drv_fault | uint16 | Read only | | | 0 | "The driver chip fault value is 1" — see Table 11, p.31 [30] |
| 0X3025 | drv_temp | int16 | Read only | | | 48 | "The driver chip fault value is 2" — see Table 12, p.31 [30] (name/remark mismatch: "drv_temp" paired with "driver chip fault value is 2" remark — transcribed as printed) |
| 0X3026 | Uq | float | Read only | | | 0 | Q-axis voltage |
| 0X3027 | as_angle | float | Read only | | | 0 | Magnetic encoder initial angle |
| 0X3028 | cs_angle | float | Read only | | | 0 | Differential magnetic encoder initial angle |
| 0X3029 | chasu_angle | float | Read only | | | 0 | Differential angle |
| 0X302a | v_bus | float | Read only | | | 0 | Motor voltage |
| 0X302b | ElecOffset | float | Read only | | | 24.195 | Electrical angle offset |
| 0X302c | torque_fdb | float | Read only | | | 0 | Torque feedback value, nm |
| 0X302d | rated_i | float | Read only | | | 8 | Rated current of motor |
| 0X302e | MechPos_init | float | Read only | | | 27 | Motor retention parameters |
| 0X302f | instep | float | Read only | | | 0 | Motor protection parameters |
| 0X3030 | status | uint8 | Read only | | | 0 | Reserved parameters |
| 0X3031 | cmdlocref | float | Read only | | | 0 | Position setpoint |
| 0X3032 | vel_max | float | Read only | | | 0 | Motor speed setpoint |
| 0X3033 | fault1 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3034 | fault2 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3035 | fault3 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3036 | fault4 | float | Read only | | | 0 | Motor position determination parameters |
| 0X3037 | fault5 | float | Read only | | | 0 | Log failure |
| 0X3038 | fault6 | uint32 | Read only | | | 0 | Log failure |
| 0X3039 | fault7 | uint32 | Read only | | | 0 | Log failure |
| 0X303a | fault8 | uint32 | Read only | | | 0 | Log failure |
| 0X303b | mcOverTemp | int16 | Read only | | | 0 | Over-temperature threshold |
| 0X303c | Kt_Nm/Amp | float | Read only | | | 0 | Torque coefficient |
| 0X303d | Tqcali_Type | uint8 | Read only | | | 0 | Motor type |
| 0X303e | theta_mech_1 | float | Read only | | | 0 | Type 2 low-speed angle |
| 0X303f | adc0ffset_1 | uint32 | Read only | | | 2053 | ADC sampling channel 1 zero current offset |
| 0X3040 | adc0ffset_2 | uint32 | Read only | | | 2057 | ADC sampling channel 2 zero current offset |
| 0X3041 | can_status | uint8 | Read only | | | 1 | CAN status |
| 0X3042 | position | float | Read only | | | 16.558939 | Initialization position |
| 0X3043 | chasu_angle_init | float | Read only | | | 2.106539 | Motor position determination parameters |
| 0X3044 | chasu_angle_out | float | Read only | | | 18.958853 | Motor position determination parameters |
| 0X3045 | motormechinit | float | Read only | | | 16.558939 | Motor position determination parameters |
| 0X3046 | mech_angle_init2 | float | Read only | | | 1.839882 | Motor position determination parameters |
| 0X3047 | mech_angle_rotat | int16 | Read only | | | 0 | Motor position determination parameters |
| 0X3048 | coder_reg | uint16 | Read only | | | 1 | System parameters |
| 0X3049 | pos_cnt1 | uint16 | Read only | | | 0 | System parameters |

Note: the names `fault1`–`fault7` are reused twice in this table (0X201b–0X2021, and 0X3033–0X3039/0X303a) for unrelated "Reserved parameter" / "Log failure" / position-determination values — neither set is the fault bitmask (that is `faultSta` at 0X3022, documented separately below). Do not conflate.

Separately, the read/write index table at 0x7xxx (section 4.1.13, listed in the TX/Type-17-18 section above) is a *different* indexing namespace used specifically by Type 17/18 (private protocol) and MIT Commands 14/15 — it is not the same index space as the 0X0000–0X3049 flash table, even though some names overlap (e.g. both have an entry conceptually related to `loc_kp`, `cur_kp`, etc., at different numeric indices: flash table 0X2016 `loc_kp` vs. read/write table... note the read/write table's `loc_kp` was not matched to an explicit index name in the extracted rows — see Open Questions).

## Fault/error codes

### 0x3022 `faultSta` bit list — p.30 [29], section 3.3.7 "Motor fault instructions"
"Function code 0x3022 indicates the fault code, where"
- "bit14: motor blocking overload algorithm protection"
- "bit7: Encoder uncalibrated: Motor uncalibrated encoder"
- "bit3: Overvoltage fault: the motor voltage exceeds the protection voltage by 60V"
- "bit2: Undervoltage fault: the motor voltage is lower than the protection voltage of 12V"
- "bit1: Driver chip failure: Motor driver chip failure reported"
- "bit0: Motor overtemperature fault: motor thermistor temperature exceeds 145 degrees"

Note: this list (6 bits) is shorter than the fault-bit lists given for the Type 21 CAN frame and the MIT fault-status response (11 bits each, including bits 16, 9, 8, 5, 4 not mentioned here). Same underlying fault word (0x3022/faultSta), three different enumerations of which bits are documented — not reconciled; see Open Questions.

### Communication Type 21 fault-feedback bit list — p.45–46 [44–45] (full list under TX/RX frame definitions above)
Bits documented: 16, 14, 9, 8, 7, 5, 4, 3, 2, 1, 0 (fault, Byte0~3) plus bit 0 of Byte4~7 (warning: "motor overtemperature warning, the default is 135°C").

### MIT Fault Status Response bit list — p.77–78 [76–77] (full list under RX frame definitions above)
Identical bit numbering to Type 21's fault list (16, 14, 9, 8, 7, 5, 4, 3, 2, 1, 0).

### Table 11: Fault Status Register 1 Field Descriptions (function code 0x3024, `drv_fault`) — p.31 [30], rendered table, transcribed from page image
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

### Table 12: Fault Status Register 2 Field Descriptions (function code 0x3025, `drv_temp`) — p.31 [30], rendered table, transcribed from page image
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

(Table titles/numbers "Table 11"/"Table 12" and the underlying field names (DRV8353xS-style register names) indicate the gate-driver IC is a TI DRV8353-family part; this is stated by the table content itself, not an inference added here.)

## OPEN QUESTIONS (ambiguous or not confidently extractable)
1. **Default/factory CAN ID**: never explicitly stated as "default = N" anywhere in the document. Flash table shows 0X2009 `CAN_ID` "Current value (for reference)" = 1, Max = 20, Min = 0 (p.23 [22]) — unclear whether this is a genuine factory default or an example unit's current reading, and the Max=20 ceiling is unusually low for an 8-bit CAN ID field (vs. CAN_MASTER/CAN_TIMEOUT at 0X200a/0X200b whose Max=127). Did not infer a default.
2. **Host ID convention**: 0xFD recurs as the example "host CAN_ID" / "Host ID" in multiple worked examples (pp.44–45 [43–44], 51 [50], 75–78 [74–77]) but is never stated to be a required/reserved/fixed host address. Not clear if any host ID is valid or 0xFD is mandatory.
3. **0x2026 `damper` and 0x2027 `add_offset` rows**: Max/Min values appear swapped relative to every other row in the flash parameter table (damper: Max=0, Min=20; add_offset: Max=-7, Min=7). Confirmed via direct page-image inspection that this is how the table is actually printed (p.24 [23]), not a text-extraction artifact. Not corrected — flagged as a likely source-document error.
4. **0x200b `CAN_TIMEOUT` vs 0x200c `status2` remarks**: as printed, CAN_TIMEOUT's remark is "can host id" and status2's remark is "can timeout threshold. The default value is 0" — these remarks look like they belong to each other's rows (CAN_TIMEOUT should logically have the "timeout threshold" remark). Confirmed via page-image inspection this is exactly how it is printed (p.23 [22]). Not corrected.
5. **Type 17 (single parameter read) reply table, bit23~16 field**: source sentence is truncated mid-thought — "...01 indicates that the master can_ID" (p.45 [44]) — meaning of value "01" vs "00" in that sub-field is not completed in the manual text.
6. **Type 21 fault-feedback frame's ID-field row**: the Size/Description row text order ("bit23~8: motor | bit7~0: identifies the main CAN_ID") is reversed compared to the pattern used by every other frame table in the document. Could not confirm whether this is a genuine difference in this frame's ID layout or a typo, since no worked numeric example is given for Type 21 (unlike Types 1, 2, 17, 18, 24). Did not infer which is correct.
7. **Fault bit numbering is documented three different ways for what appears to be the same underlying fault word** (0x3022 `faultSta` text description with 6 bits, p.30 [29]; Type 21 CAN frame with 11 bits, p.45–46 [44–45]; MIT Fault Status Response with the same 11 bits, p.77–78 [76–77]). The two 11-bit lists agree with each other; the 6-bit list is a subset using slightly different wording for shared bits (e.g. "motor blocking overload algorithm protection" at bit14 vs. "Motor stall and overload algorithm protection" at Bit14). Not merged into one canonical table — presented as transcribed per source location.
8. **bit20 double-definition** in the Type 2 / Type 24 / version-read feedback frame fault field: source text literally assigns bit20 to both "Uncalibrated" and "Gridlock overload fault" in sequence ("bit21: uncalibrated bit20: Uncalibrated bit20: Gridlock overload fault bit19: ..."), appearing identically in all three locations it's printed (pp.42 [41], 43–44 [42–43], 47 [46]). Could not determine from text whether one of "bit21: uncalibrated" / "bit20: Uncalibrated" is a duplicate/typo for a different bit number. Not resolved.
9. **0X3024 `drv_fault` and 0X3025 `drv_temp` names vs. remarks**: flash table remarks read "The driver chip fault value is 1" / "...is 2" (p.26 [25]) which seem to describe *which fault register* (1 or 2) rather than describe the parameter names themselves (`drv_fault`, `drv_temp` — the latter name doesn't obviously match "fault" at all). Transcribed as printed; relationship between name and remark not clarified by the text.
10. **Read/write parameter list (0x7xxx, section 4.1.13) vs. flash table (0X0000–0X3049, section 3.3.4)**: the manual states these are used by different mechanisms (0x7xxx by Type 17/18 and MIT Commands 14/15; the flash table by the host-computer GUI) but never explicitly cross-maps every name between the two tables (e.g. it's not stated in the text whether flash-table 0X2016 `loc_kp` and read/write-table entries are the same physical register accessed through two different index schemes, or genuinely separate values). Treated as two separate tables per the document's own section structure; no cross-reference inferred.
11. **Company name**: manual uses both "Lingfoot" (legal/after-sales sections, pp.3–4) and "Lingzu"/"Lingzu Times" (hardware/firmware sections, pp.15, 22) for what appears to be the same manufacturer. Not resolved — both forms quoted as printed.
12. **0X0000 `Name` and 0X0001 `BarCode` "Current value" cells**: rendered as a string of unreadable/non-ASCII placeholder glyphs in the extracted text (p.22 [21]); could not determine the actual printed value (likely a placeholder/blank field or a font-encoding artifact in the source PDF itself, not an extraction failure on our end — the glyphs are genuinely not standard text).
13. **Driver chip fault code tables (Table 11, Table 12, p.31 [30])**: these did not appear in the plain-text (`pdftotext`) extraction at all — the page only yielded surrounding prose ("Function code 0x3024 is driver chip fault code 1. The specific faults are as follows" followed by nothing). They were recovered only by rendering the page as an image and reading the table visually. Flagging this extraction-method gap in case other image-only tables exist elsewhere in the document that were not independently visually re-verified beyond the specific pages checked in this pass (pp.23, 24, 31, 41, 42, 68).
