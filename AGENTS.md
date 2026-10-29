# AGENTS — Instructions for Human-AI Tandems (`podmesh-windows-mount`)

## Repository Scope
This repository contains the pure Rust source code, pre-compiled Windows PE32+ binary (`dist/podmesh-windows-mount.exe`), and autonomous Windows 11 provisioning scripts for mounting **MeshFS** as a native FUSE drive (`Z:\`) on Windows via **WinFsp**.

## Companion Lab Repository
All live qualification evidence, screenshots, QMP scancode injection scripts, and the Windows 11 SPICE + AutoLogon + WinFsp Runbook are maintained in the companion private lab repository:
- **GitHub**: `https://github.com/xavdp-pro/podmesh-windows-mount-lab`
- **Local Path**: `/home/zaza/Bureau/SHAPER-PHOENIX/LAB/REPOSITORIES/podmesh-windows-mount-lab`

## Cross-Compilation Command (from Linux)
```bash
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/podmesh-windows-mount.exe dist/
```
