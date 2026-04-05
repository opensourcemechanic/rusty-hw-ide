# Arduino Upload Troubleshooting Guide

## Problem: Upload Fails with Nano Clone

Your Arduino Nano clone (FTDI/CH340) needs specific settings for successful upload.

## Fixed Settings

✅ **Baud Rate**: 57600 (not 9600 or 115200)  
✅ **Programmer**: stk500v1 for FTDI, arduino for CH340  
✅ **Port**: /dev/ttyUSB0 (your detected port)

## Upload Configuration Options

### 1. FTDI Chips (Your Arduino)
```bash
Programmer: stk500v1
Baud Rate: 57600
Port: /dev/ttyUSB0
MCU: atmega328p
```

### 2. CH340 Chips (Alternative)
```bash
Programmer: arduino  
Baud Rate: 57600
Port: /dev/ttyUSB0
MCU: atmega328p
```

### 3. USBasp (Alternative)
```bash
Programmer: usbasp
Baud Rate: 57600
Port: /dev/ttyUSB0
MCU: atmega328p
```

## Common Upload Errors & Solutions

### Error: "avrdude: stk500_recv(): programmer is not responding"
**Solution**: Try `stk500v1` programmer instead of `arduino`

### Error: "avrdude: stk500_getsync(): not in sync"
**Solution**: 
- Check baud rate is 57600
- Try different programmer
- Verify Arduino is in bootloader mode

### Error: "Permission denied"
**Solution**: 
```bash
sudo usermod -a -G dialout $USER
# Then logout and login again
```

### Error: "Device not found"
**Solution**:
```bash
# Check if device is still there
ls -la /dev/ttyUSB0
# If not, reseat USB cable or try different port
```

## Manual Upload Command

If IDE upload fails, try manual command:

```bash
avrdude -v -patmega328p -cstk500v1 -P/dev/ttyUSB0 -b57600 -Uflash:w:sketch.hex:i
```

## Bootloader Issues

If upload still fails, the bootloader might be corrupted:

### Option 1: Reburn Bootloader
- Use another Arduino as ISP programmer
- Select "Tools > Burn Bootloader" in Arduino IDE

### Option 2: ISP Programming
- Connect USBasp programmer
- Use `usbasp` programmer setting
- Upload directly to chip (bypasses bootloader)

## Testing Upload

Create a simple test sketch:

```cpp
void setup() {
  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  digitalWrite(LED_BUILTIN, HIGH);
  delay(1000);
  digitalWrite(LED_BUILTIN, LOW);
  delay(1000);
}
```

If LED blinks after upload, it worked!

## Next Steps

1. Try the updated IDE with correct 57600 baud rate
2. If still fails, try `stk500v1` programmer
3. Check that Arduino's power LED is on
4. Try a different USB cable or port

The IDE now automatically detects your FTDI Nano and uses the correct settings.
