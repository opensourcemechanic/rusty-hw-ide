# WSL USB Pass-through Setup for Arduino

## Method 1: USBIPD (Windows Host + WSL)

1. Install usbipd-win on Windows (as Administrator):
   ```powershell
   winget install --interactive --exact dorssel.usbipd-win
   ```

2. List available USB devices:
   ```powershell
   usbipd list
   ```

3. Find your Arduino (look for CH340 or similar VID:PID)

4. Attach the USB device to WSL:
   ```powershell
   usbipd bind --busid <BUS_ID>  # Replace with actual bus ID
   usbipd attach --wsl --busid <BUS_ID>
   ```

5. In WSL, load the USB driver:
   ```bash
   sudo modprobe usbip_core
   sudo modprobe usbip_host
   sudo modprobe vhci_hcd
   ```

6. Check if device is visible:
   ```bash
   lsusb
   dmesg | tail -10
   ```

## Method 2: Windows COM Port Forwarding (Simpler)

1. In Windows, find Arduino COM port (Device Manager)
2. Configure .wslconfig for COM port forwarding:
   ```ini
   [wsl2]
   com1=\\\\.\\COM8  # Replace with your Arduino COM port
   ```

3. Restart WSL:
   ```powershell
   wsl --shutdown
   wsl
   ```

4. Test access in WSL:
   ```bash
   ls -la /dev/ttyS*
   ```

## Method 3: Run on Windows Directly

Since your code works on Windows, consider running it there instead of WSL.

## Testing

After setup, test with:
```bash
cd /home/bri/src/rusty-hw-ide
cargo run --bin hw_ide
```

Or test port access:
```bash
rustc test_port_access.rs -o test_port_access && ./test_port_access
```
