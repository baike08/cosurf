# Native Module Build Guide

## 📦 Overview

CoSurf uses a Rust native module (`cosurf-native.node`) for high-performance operations. This guide explains how to build and manage the native module.

## 🔧 Build Scripts

### 1. `build-native.ps1` - Build Native Module Only

Builds the Rust code and automatically copies the `.dll` file to `.node`.

```powershell
# Using npm script
pnpm build:native

# Or directly
powershell -ExecutionPolicy Bypass -File scripts/build-native.ps1
```

**What it does:**
- ✅ Compiles Rust code in release mode
- ✅ Locates the compiled `cosurf_native.dll`
- ✅ Stops Electron if running (to avoid file locks)
- ✅ Copies DLL to `native/cosurf-native.node`
- ✅ Displays build information

---

### 2. `dev-with-native.ps1` - Smart Development Mode

Automatically checks if native module needs rebuild, then starts dev server.

```powershell
# Using npm script
pnpm dev:full

# Or directly
powershell -ExecutionPolicy Bypass -File scripts/dev-with-native.ps1
```

**What it does:**
- 🔍 Checks if `.node` file exists
- 🔍 Compares timestamps of `.dll` and `.node` files
- 🔄 Rebuilds only if necessary
- 🚀 Starts development server

---

### 3. `dev:native` - Quick Rebuild & Restart

Rebuilds native module and immediately starts dev server.

```powershell
pnpm dev:native
```

---

## 📝 Usage Scenarios

### Scenario 1: First Time Setup

```powershell
# Build native module
pnpm build:native

# Start development
pnpm dev
```

---

### Scenario 2: After Modifying Rust Code

```powershell
# Option A: Rebuild only
pnpm build:native
pnpm dev

# Option B: Smart rebuild (recommended)
pnpm dev:full
```

---

### Scenario 3: Daily Development (No Rust Changes)

```powershell
# Just start dev server (no rebuild)
pnpm dev
```

The smart script will detect that `.node` is up-to-date and skip rebuilding.

---

### Scenario 4: Force Rebuild

```powershell
# Clean and rebuild
cargo clean
pnpm build:native
```

---

## 🛠️ Manual Build Steps

If you prefer manual control:

```powershell
# 1. Build Rust code
cd d:\coding-harness\CoSurf
cargo build --release

# 2. Stop Electron (if running)
Stop-Process -Name "electron" -Force

# 3. Copy DLL to .node
Copy-Item target\release\cosurf_native.dll native\cosurf-native.node -Force

# 4. Start dev server
pnpm dev
```

---

## 📂 File Locations

| File | Location | Description |
|------|----------|-------------|
| Source Code | `native/src/` | Rust source files |
| Compiled DLL | `target/release/cosurf_native.dll` | Windows library output |
| Node Module | `native/cosurf-native.node` | Electron loads this file |
| Build Script | `scripts/build-native.ps1` | Automated build script |

---

## ⚠️ Common Issues

### Issue 1: ".node file is locked"

**Cause:** Electron is still running and has loaded the old `.node` file.

**Solution:**
```powershell
# Stop Electron
Stop-Process -Name "electron" -Force

# Rebuild
pnpm build:native
```

The `build-native.ps1` script handles this automatically.

---

### Issue 2: "Cannot find cosurf_native.dll"

**Cause:** Rust compilation failed or output to wrong directory.

**Solution:**
```powershell
# Check build output
cargo build --release

# Verify DLL exists
Get-ChildItem target\release\cosurf_native.dll
```

---

### Issue 3: Changes not reflected after rebuild

**Cause:** Electron cached the old module.

**Solution:**
1. Stop Electron completely
2. Rebuild: `pnpm build:native`
3. Restart: `pnpm dev`

---

## 🎯 Best Practices

1. **Use `pnpm dev:full` for daily development**
   - Automatically detects when rebuild is needed
   - Saves time by skipping unnecessary builds

2. **Use `pnpm build:native` when debugging Rust code**
   - Clear separation between build and run steps
   - Easier to identify build errors

3. **Always stop Electron before manual rebuilds**
   - Prevents file lock issues
   - Ensures clean module loading

4. **Check timestamps if unsure**
   ```powershell
   Get-ChildItem target\release\cosurf_native.dll | Select-Object LastWriteTime
   Get-ChildItem native\cosurf-native.node | Select-Object LastWriteTime
   ```

---

## 📊 Build Performance

| Operation | Time (approx.) |
|-----------|----------------|
| Clean build | 60-90 seconds |
| Incremental build | 30-50 seconds |
| File copy | < 1 second |
| Total (smart rebuild) | 30-90 seconds |

---

## 🔗 Related Documentation

- [N-API Documentation](https://napi.rs/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Electron Native Modules](https://www.electronjs.org/docs/latest/tutorial/using-native-node-modules)
