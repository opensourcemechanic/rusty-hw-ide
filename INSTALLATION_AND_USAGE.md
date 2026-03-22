# Installation and Usage Guide

## ✅ Build Menu Now Working!

The **Build → Compile** and **Build → Upload** menu items are now connected and will show status messages.

---

## 🔧 Installing Arduino Compilation Tools (Windows)

### **Option 1: Arduino IDE (Easiest)**
1. **Download Arduino IDE:**
   - Go to: https://www.arduino.cc/en/software
   - Download "Windows Win 10 and newer" installer
   - Run installer and follow prompts

2. **Tools are automatically installed to:**
   ```
   C:\Users\<YourUsername>\AppData\Local\Arduino15\packages\arduino\tools\
   ```

3. **Add to PATH (Step-by-Step):**

   **Step 1: Find the exact paths**
   - Open File Explorer
   - Navigate to: `C:\Users\<YourUsername>\AppData\Local\Arduino15\packages\arduino\tools\`
     - Replace `<YourUsername>` with your actual Windows username (e.g., `brian`)
     - If you can't see `AppData`, enable hidden folders: View → Show → Hidden items
   
   - Look for these folders:
     - `avr-gcc\` → Open it → Find the version folder (e.g., `7.3.0-atmel3.6.1-arduino7`) → Open `bin`
     - `avrdude\` → Open it → Find the version folder (e.g., `6.3.0-arduino17`) → Open `bin`
   
   - Copy the full paths (example):
     ```
     C:\Users\brian\AppData\Local\Arduino15\packages\arduino\tools\avr-gcc\7.3.0-atmel3.6.1-arduino7\bin
     C:\Users\brian\AppData\Local\Arduino15\packages\arduino\tools\avrdude\6.3.0-arduino17\bin
     ```

   **Step 2: Open Environment Variables**
   - Press `Win + R` to open Run dialog
   - Type: `sysdm.cpl` and press Enter
   - Click the **"Advanced"** tab
   - Click **"Environment Variables..."** button at the bottom

   **Step 3: Edit PATH Variable**
   - In the **"User variables"** section (top half), find and select **"Path"**
   - Click **"Edit..."** button
   - Click **"New"** button
   - Paste the first path: `C:\Users\brian\AppData\Local\Arduino15\packages\arduino\tools\avr-gcc\7.3.0-atmel3.6.1-arduino7\bin`
   - Click **"New"** again
   - Paste the second path: `C:\Users\brian\AppData\Local\Arduino15\packages\arduino\tools\avrdude\6.3.0-arduino17\bin`
   - Click **"OK"** on all dialogs

   **Step 4: Verify (IMPORTANT - Restart PowerShell first!)**
   - Close any open PowerShell/Command Prompt windows
   - Open a **NEW** PowerShell window
   - Test:
     ```powershell
     avr-gcc --version
     # Should output: avr-gcc (GCC) 7.3.0
     
     avrdude -?
     # Should output: avrdude version 6.3
     ```
   
   **If commands not found:**
   - Double-check the paths are correct (version numbers might differ)
   - Make sure you restarted PowerShell after editing PATH
   - Try logging out and back in to Windows

### **Option 2: WinAVR (Alternative)**
1. Download from: https://sourceforge.net/projects/winavr/
2. Install to default location: `C:\WinAVR-20100110\`
3. Installer should add to PATH automatically

### **Option 3: Scoop Package Manager**
```powershell
# Install Scoop if you don't have it
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
irm get.scoop.sh | iex

# Install AVR tools
scoop install avr-gcc
scoop install avrdude
```

---

## 🔍 Verify Installation

Open PowerShell and check:

```powershell
# Check AVR-GCC
avr-gcc --version
# Should show: avr-gcc (GCC) 7.3.0 or similar

# Check AVRDUDE
avrdude -?
# Should show: avrdude version 6.3 or similar
```

---

## 🚀 How to Compile and Upload

### **Current Status (Temporary):**
Right now, the IDE shows placeholder messages:
- **Build → Compile** → Shows: "Compilation feature coming soon - install avr-gcc and avrdude"
- **Build → Upload** → Shows: "Upload feature coming soon - install avrdude"

### **What's Needed for Full Functionality:**

The IDE has the backend code ready in `hw_hal` crate, but it needs to be connected to the UI. Here's what needs to be implemented:

1. **Create Project from Current File**
   - Generate temporary `.ino` or `.cpp` file
   - Create build directory

2. **Invoke Compilation Manager**
   - Use `hw_hal::platforms::avr::AVRPlatform`
   - Call `compile()` method with project path
   - Display compilation output in a dialog

3. **Invoke Upload Manager**
   - Detect connected Arduino board
   - Use `hw_hal::platforms::avr::AVRPlatform`
   - Call `upload()` method with firmware path and port
   - Display upload progress

---

## 📋 Manual Compilation (Until Full Integration)

For now, you can compile Arduino sketches manually:

### **For Arduino Uno:**

```powershell
# Navigate to your sketch directory
cd C:\Users\brian\Documents\Arduino\MySketch

# Compile
avr-gcc -mmcu=atmega328p -DF_CPU=16000000UL -Os -o sketch.elf sketch.cpp
avr-objcopy -O ihex sketch.elf sketch.hex

# Upload (replace COM3 with your port)
avrdude -p atmega328p -c arduino -P COM3 -b 115200 -U flash:w:sketch.hex:i
```

### **For WeMos D1 Mini (ESP8266):**

```powershell
# Install ESP8266 tools via Arduino IDE or PlatformIO
# Then use esptool.py

python -m esptool --port COM3 write_flash 0x00000 firmware.bin
```

---

## 🔌 Finding Your COM Port

### **Method 1: Device Manager**
1. Connect Arduino/WeMos to USB
2. Open Device Manager (Win+X → Device Manager)
3. Expand "Ports (COM & LPT)"
4. Look for "Arduino Uno (COM3)" or "USB-SERIAL CH340 (COM9)"

### **Method 2: PowerShell**
```powershell
Get-WmiObject Win32_SerialPort | Select-Object Name, DeviceID
```

### **Method 3: IDE Hardware Detection**
- Click **Hardware → Detect Hardware**
- The IDE will scan and show available devices

---

## 🎯 Testing the Current Build Menu

1. **Run the IDE:**
   ```powershell
   cd C:\Users\brian\CascadeProjects\win\rusty_hw
   cargo run --bin hw_ide
   ```

2. **Test Build Menu:**
   - Click **Build → Compile** (should show message in status bar)
   - Click **Build → Upload** (should show message in status bar)
   - Press **Ctrl+B** (compile shortcut - should show message)
   - Press **Ctrl+U** (upload shortcut - should show message)

3. **Test File Menu:**
   - Click **File → New File** (creates blank editor)
   - Click **File → Open File** (opens file dialog)
   - Click **File → Save** (saves current file)

---

## 🛠️ Next Steps for Full Compilation Support

To enable actual compilation and upload, these changes are needed in `hw_ide_core/src/main.rs`:

1. **Replace placeholder compile handler:**
   ```rust
   if self.menu_bar.compile_clicked {
       self.create_project_from_current_file();
       self.compile_project();
       self.menu_bar.reset_action_flags();
   }
   ```

2. **Replace placeholder upload handler:**
   ```rust
   if self.menu_bar.upload_clicked {
       if self.current_project.is_none() {
           self.create_project_from_current_file();
       }
       self.upload_firmware();
       self.menu_bar.reset_action_flags();
   }
   ```

3. **Add these methods to `HardwareIDE` impl:**
   - `create_project_from_current_file()`
   - `compile_project()`
   - `upload_firmware()`
   - `show_compilation_dialog()`
   - `show_upload_dialog()`

These methods already exist in the old directory and can be ported over.

---

## 📝 Summary

**✅ Working Now:**
- Build menu items trigger action flags
- Status bar shows placeholder messages
- Keyboard shortcuts (Ctrl+B, Ctrl+U) work
- File operations (New, Open, Save) work
- Line numbers display correctly

**⏳ Needs Implementation:**
- Connect UI to compilation backend
- Create project from current file
- Show compilation output dialog
- Show upload progress dialog
- Handle compilation errors gracefully

**🔧 Tools to Install:**
- `avr-gcc` (AVR C/C++ compiler)
- `avrdude` (AVR programmer/uploader)
- Optional: Arduino IDE (includes both)

Once you install the tools and verify with `avr-gcc --version`, let me know if you want me to implement the full compilation integration!
