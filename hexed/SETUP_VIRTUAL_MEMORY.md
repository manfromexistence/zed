# CRITICAL: Setup Virtual Memory for Low-RAM Systems

## ⚠️ DO THIS FIRST BEFORE BUILDING

To build Zed on a system with <8GB RAM, you MUST configure a large pagefile (virtual memory).

## Step-by-Step Instructions

### Method 1: GUI (Recommended)

1. Press `Win + Pause/Break` (or right-click "This PC" → Properties)
2. Click "Advanced system settings"
3. Under "Performance", click "Settings"
4. Go to "Advanced" tab
5. Under "Virtual Memory", click "Change"
6. **Uncheck** "Automatically manage paging file size for all drives"
7. Select your **fastest drive** (preferably SSD/NVMe with Windows installed)
8. Select "Custom size"
9. Enter:
   - **Initial size**: `16384` (16 GB)
   - **Maximum size**: `32768` (32 GB)
10. Click "Set"
11. Click "OK" on all dialogs
12. **RESTART YOUR COMPUTER** (required!)

### Method 2: PowerShell (Admin)

```powershell
# Run PowerShell as Administrator
wmic pagefileset where name="C:\\pagefile.sys" set InitialSize=16384,MaximumSize=32768
```

Then restart your computer.

## Requirements

- **Free disk space**: At least 35-40 GB on the drive with the pagefile
- **Drive type**: SSD strongly recommended (HDD will be VERY slow but may work)
- **Restart**: You MUST restart after changing pagefile settings

## Why This Matters

The linker needs to load hundreds of compiled libraries into memory simultaneously. With <8GB physical RAM, the linker will use virtual memory (pagefile). Windows' default pagefile is too small for a project this size.

With a 16-32GB pagefile:
- The linker can use up to 32GB total memory (RAM + pagefile)
- Linking will be slower but should succeed
- Expect the linking phase to take 10-30 minutes

## After Setup

Once you've configured virtual memory and restarted:

```bash
# Try the standard build
just run

# Or try with Cranelift (even better for low memory)
just setup-cranelift
just run-cranelift
```

## Verification

To check your current pagefile settings:

```powershell
wmic pagefile list /format:list
```

Or:

```powershell
Get-CimInstance -ClassName Win32_PageFileUsage
```

You should see a pagefile of 16-32 GB.
