# podmesh-windows-mount (v2.4.0)

> **Native Windows FUSE Drive Mount Utility (`MeshFS-CAS (Z:)`) for the PodMesh 4 MiB Content-Addressable Storage Ecosystem.**  
> *Powered by Rust (`x86_64-pc-windows-gnu`) + WinFsp Kernel FUSE Driver + MeshFS S3 WORM Gateway.*

---

## Overview

`podmesh-windows-mount` enables any Windows 11 or Windows Server machine to mount a **MeshFS** distributed storage volume directly as a local drive letter (`Z:\`) without SMB/CIFS protocol translation.

```text
+-----------------------------------------------------------------------+
|                    Windows 11 / Windows Server                        |
|                                                                       |
|   Windows Explorer ("Ce PC > MeshFS-CAS (Z:)") / Win32 Apps           |
|                                 |                                     |
|                                 v                                     |
|            WinFsp Kernel Driver (Fsp.sys / WinFsp.Launcher)           |
|                                 |                                     |
|                                 v                                     |
|      podmesh-windows-mount.exe (v2.4.0) + VFS Full Cache Engine       |
+---------------------------------|-------------------------------------+
                                  |  HTTP S3 REST API (:9000)
                                  v
+-----------------------------------------------------------------------+
|             MeshFS S3 WORM Gateway (podmesh-meshfs-s3)                |
|        4 MiB SHA-256 CAS Pool + FastCDC Deduplication + WORM Lock     |
+-----------------------------------------------------------------------+
```

---

## Repository Structure

| Path | Description |
| :--- | :--- |
| `src/main.rs` | Pure Rust CLI and WinFsp / VFS mount dispatcher |
| `dist/podmesh-windows-mount.exe` | Pre-compiled standalone Windows x64 executable (1.2 MiB) |
| `scripts/setup-windows-vm.ps1` | Autonomous Windows 11 setup (Auto-Logon, No-Sleep, OpenSSH, Firewall) |
| `scripts/run-hidden.vbs` | Session 1 non-elevated background launcher for Windows Explorer visibility |
| `AI_INSTALL_AND_CONFIG.md` | Deterministic headless deployment guide for AI agents |
| `INTENT.md` / `LAW.md` / `LINEAGE.md` | Canonical SHAPER PHOENIX architectural specifications |

---

## CLI Usage

```cmd
:: Mount bucket 'truenas-backup' as drive Z: (foreground)
podmesh-windows-mount.exe mount Z: http://192.168.10.179:9000 truenas-backup

:: Mount bucket in background
podmesh-windows-mount.exe daemon Z: http://192.168.10.179:9000 truenas-backup

:: Unmount drive Z:
podmesh-windows-mount.exe unmount Z:

:: Check MeshFS Gateway health and deduplication telemetry
podmesh-windows-mount.exe status http://192.168.10.179:9000
```

---

## Cross-Compiling from Linux

```bash
sudo apt-get install -y mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

---

## License

Licensed under the **GNU Affero General Public License v3.0 or later (`AGPL-3.0-or-later`)**. See [`LICENSE`](LICENSE) and [`DEDICATION.md`](DEDICATION.md).
