# Arduino Nano Clone Setup for Ubuntu Linux

## Problem
Your Arduino Nano clone isn't being detected on Ubuntu Linux, but it worked on Windows.

## Root Cause
The CH340/CH341 USB-to-serial chip driver needs to be loaded on Ubuntu.

## Quick Fix

1. **Load the CH340 driver:**
   ```bash
   sudo modprobe ch341
   ```

2. **Verify driver is loaded:**
   ```bash
   lsmod | grep ch341
   ```

3. **Connect Arduino and check:**
   ```bash
   lsusb | grep -i "1a86\|ch340"
   ls -la /dev/ttyUSB*
   ```

## If Still Not Working

### Run the diagnostic script:
```bash
cd /home/bri/src/rusty-hw-ide
./diagnose_arduino.sh
```

### Common Issues:

1. **USB Cable Problems:**
   - Try a different USB cable (some charge-only cables don't work)
   - Try a different USB port

2. **Driver Issues:**
   ```bash
   # Reload driver
   sudo modprobe -r ch341
   sudo modprobe ch341
   
   # Check kernel messages
   dmesg | tail -10
   ```

3. **Permissions:**
   ```bash
   # Ensure you're in dialout group
   sudo usermod -a -G dialout $USER
   # Then logout and login again
   ```

4. **Hardware Issues:**
   - Check if Arduino powers on (LED should light up)
   - Try on another computer to confirm it works
   - Some cheap clones have counterfeit CH340 chips

## Expected Result

When working correctly, you should see:
```bash
$ lsusb
Bus XXX Device XXX: ID 1a86:7523 QinHeng Electronics CH340 serial converter

$ ls -la /dev/ttyUSB*
crw-rw---- 1 root dialout 188, 0 /dev/ttyUSB0
```

## Testing with Your IDE

Once the device appears as `/dev/ttyUSB0`, run:
```bash
cd /home/bri/src/rusty-hw-ide
cargo run --bin hw_ide
```

The IDE should now detect your Arduino Nano clone.

## Permanent Solution

To automatically load the driver on boot:
```bash
echo "ch341" | sudo tee -a /etc/modules-load.d/ch341.conf
```

## Alternative Drivers

If CH340 doesn't work, some clones use FTDI:
```bash
sudo modprobe ftdi_sio
```
