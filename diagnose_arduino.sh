#!/bin/bash

echo "=== Arduino Nano Clone Diagnosis for Ubuntu Linux ==="
echo

echo "1. Checking if CH340 driver is loaded:"
lsmod | grep ch341
echo

echo "2. Checking USB devices:"
lsusb
echo

echo "3. Checking for USB serial devices:"
ls -la /dev/ttyUSB* /dev/ttyACM* 2>/dev/null || echo "No USB serial devices found"
echo

echo "4. Checking user permissions:"
groups | grep -o dialout
echo

echo "5. Checking recent USB events (requires sudo):"
sudo dmesg | tail -20 | grep -i "usb\|tty\|ch34\|arduino" || echo "No recent USB messages"
echo

echo "6. Testing USB device creation:"
echo "Connect your Arduino Nano now and press Enter..."
read -r

echo "Checking for new devices..."
sleep 2
lsusb
echo

echo "Checking for new serial ports..."
ls -la /dev/ttyUSB* /dev/ttyACM* 2>/dev/null || echo "Still no USB serial devices"
echo

echo "7. Manual driver loading:"
echo "Trying to reload CH340 driver..."
sudo modprobe -r ch341
sudo modprobe ch341
echo

echo "8. Final check:"
ls -la /dev/ttyUSB* /dev/ttyACM* 2>/dev/null || echo "No USB serial devices found"

echo
echo "=== Troubleshooting Tips ==="
echo "If no devices are found:"
echo "- Check USB cable (try a different one)"
echo "- Try a different USB port"
echo "- Check if Arduino powers on (LED should light up)"
echo "- Try another computer to confirm Arduino works"
echo "- Some cheap Nano clones have counterfeit CH340 chips"
