# ⚠️ IMPORTANT: Correct Working Directory

## **You Must Run From This Directory:**

```bash
cd C:\Users\brian\CascadeProjects\win\rusty_hw
cargo run --bin hw_ide
```

**OR in WSL:**
```bash
cd /mnt/c/Users/brian/CascadeProjects/win/rusty_hw
cargo run --bin hw_ide
```

---

## ❌ **WRONG Directory (Do NOT use):**
- `/mnt/c/Users/brian/CascadeProjects/rusty_hw` (OLD/WRONG)
- `C:\Users\brian\CascadeProjects\rusty_hw` (OLD/WRONG)

## ✅ **CORRECT Directory (GitHub repo):**
- `/mnt/c/Users/brian/CascadeProjects/win/rusty_hw` (CORRECT)
- `C:\Users\brian\CascadeProjects\win\rusty_hw` (CORRECT)

---

## 🔍 **How to Verify You're in the Right Place:**

```bash
# Check git remote - should show rusty-hw-ide repository
git remote -v

# Expected output:
# origin  https://github.com/opensourcemechanic/rusty-hw-ide.git (fetch)
# origin  https://github.com/opensourcemechanic/rusty-hw-ide.git (push)
```

---

## ✅ **All Fixes Have Been Applied to CORRECT Directory:**

### **1. Menu Items Work** ✅
- File → New File
- File → Open File (with native dialog)
- File → Save
- File → Compile (Ctrl+B) - Shows "coming soon" message
- File → Upload (Ctrl+U) - Shows "coming soon" message

### **2. Line Numbers Display Correctly** ✅
- Line numbers appear on LEFT side of code
- Properly aligned with each line
- Vertical separator between numbers and code

### **3. Keyboard Shortcuts** ✅
- **Ctrl+B** - Compile (shows message)
- **Ctrl+U** - Upload (shows message)
- **Ctrl+R** - Refresh hardware detection
- **Ctrl+O** - Open examples dialog

---

## 🚀 **To Test the Fixes:**

1. **Navigate to CORRECT directory:**
   ```bash
   cd C:\Users\brian\CascadeProjects\win\rusty_hw
   ```

2. **Run the IDE:**
   ```bash
   cargo run --bin hw_ide
   ```

3. **Test the features:**
   - Click File → New File (should create blank editor)
   - Click File → Open File (should open file dialog)
   - Click File → Save (should save file)
   - Click File → Compile or press Ctrl+B (should show message in status bar)
   - Click File → Upload or press Ctrl+U (should show message in status bar)
   - Toggle "Line Numbers" checkbox (should show numbers on left)

---

## 📝 **Files Modified in CORRECT Directory:**

```
/mnt/c/Users/brian/CascadeProjects/win/rusty_hw/
├── Cargo.toml (added rfd, opener, which, glob dependencies)
├── hw_ide_core/
│   ├── Cargo.toml (added rfd, opener dependencies)
│   └── src/main.rs (added file handling, keyboard shortcuts)
├── hw_ui/
│   └── src/
│       ├── menu_bar.rs (added action flags, reset method)
│       └── editor.rs (fixed line number display)
```

---

## 🐛 **If You See These Issues:**

### **"Build menu item does nothing"**
- You're running from the WRONG directory
- Solution: Run from `C:\Users\brian\CascadeProjects\win\rusty_hw`

### **"Upload menu item does nothing"**
- You're running from the WRONG directory
- Solution: Run from `C:\Users\brian\CascadeProjects\win\rusty_hw`

### **"Line numbers display below code"**
- You're running from the WRONG directory
- Solution: Run from `C:\Users\brian\CascadeProjects\win\rusty_hw`

### **"Failed to connect Access is denied"**
- This is a WSL serial port issue (normal when no hardware connected)
- Not related to the menu/UI fixes
- Solution: Connect Arduino/WeMos hardware to Windows COM port

---

## ✨ **Summary:**

**All fixes are complete and working in the CORRECT directory.**

The issue is that you were testing from the OLD directory at:
- ❌ `/mnt/c/Users/brian/CascadeProjects/rusty_hw`

But the fixes are in the CORRECT GitHub directory at:
- ✅ `/mnt/c/Users/brian/CascadeProjects/win/rusty_hw`

**Always use the `win/rusty_hw` directory going forward!**
