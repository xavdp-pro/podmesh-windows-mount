# PodMesh Windows Mount (`podmesh-windows-mount` v2.4.1)

Native Windows 11 FUSE Drive Mount Utility (`Z:\`) for the **PodMesh MeshFS 4 MiB Content-Addressable Storage (CAS)** engine, powered by **WinFsp 2.0**.

---

## 🏛️ Architectural Philosophy: Pure Userland by Default, Global Mode Under the Hood

In accordance with **Shaper OS / PodMesh Sovereignty Laws**, `podmesh-windows-mount` enforces a **Userland-First** security posture while providing an explicit, documented **Global Mode (`--global` / `install-global`)** when cross-session visibility (`sshd` Session 0 + `explorer.exe` Session 1) is required.

| Mode | Command | Windows NT Namespace | Privileges Required | Visible in `explorer.exe` (Session 1 GUI) | Visible in `sshd` (Session 0 SSH) & Services | Recommended Use Case |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Pure Userland (DEFAULT & ENCOURAGED)** | `daemon` / `install-userland` | `\Sessions\0\DosDevices\<User-LUID>\Z:` | **None** (`/RL LIMITED`, standard user token) | ✅ **Yes** (Native Drive `Z:`) | ❌ No (Isolated from Session 0 by Windows NT) | **Default desktop & workstation usage.** Zero privilege escalation; drive only exists inside the user's interactive session. |
| **2. Global System Mode (Under the Hood)** | `daemon --global` / `install-global` | `\GLOBAL??\Z:` (`FileSecurity=D:P(A;;FA;;;WD)`) | **Administrator / `SYSTEM`** (`/RL HIGHEST`) | ✅ **Yes** (Full Read/Write) | ✅ **Yes** (`Test-Path Z:\ == True` in SSH) | **AI-Agent SSH automation & CI/CD** where `sshd` (Session 0) and the GUI user (Session 1) must inspect and mutate `Z:\` simultaneously. |

> [!IMPORTANT]
> **Critical Note for Humans and AI Agents (`Session 0` vs `Session 1` Isolation):**
> When `Z:` is mounted in the default **Pure Userland** mode (`install-userland`), Windows NT intentionally isolates the DOS device letter `Z:` to the interactive user's Logon Session (`Session 1`). Therefore, running `dir Z:\` or `Test-Path Z:\` from an **OpenSSH (`sshd`) session (`Session 0`)** will return `Drive not found` even though `Z:\` is healthy and mounted in `explorer.exe`!
> - To verify a Pure Userland mount from SSH without switching modes, check the process and RC API:
>   `curl.exe -s -X POST http://127.0.0.1:5572/vfs/stats`
> - If an AI Agent or service connecting via `sshd` (`Session 0`) needs direct filesystem access to `Z:\`, explicitly activate **Global Mode**:
>   `podmesh-windows-mount.exe install-global Z: http://192.168.10.179:9000 truenas-backup`

---

## ✨ Key Features in v2.4.1

1. **Zero-VBScript Native Windowless Supervision (`CREATE_NO_WINDOW`)**:
   - `podmesh-windows-mount.exe daemon` spawns a native Rust background watchdog (`supervise`) using the Win32 `CREATE_NO_WINDOW` (`0x08000000`) flag. No fragile `.vbs` wrapper is needed, and no console window flashes on the user's desktop.
   - If the network blips at boot or the underlying FUSE worker exits unexpectedly, the `supervise` loop automatically restarts the mount after 3 seconds.
2. **Graceful VFS Cache Drain Before Unmount (`unmount`)**:
   - Instead of terminating the FUSE process abruptly, `podmesh-windows-mount.exe unmount Z:` queries the local RC endpoint (`http://127.0.0.1:5572/vfs/stats`), waits until `uploadsInProgress == 0` and `uploadsQueued == 0`, and then issues a clean `core/quit` shutdown.
3. **Externalized Credentials Hygiene**:
   - Reads `MESHFS_ACCESS_KEY` and `MESHFS_SECRET_KEY` from environment variables, `%USERPROFILE%\.meshfs\credentials.env`, or `C:\ProgramData\PodMesh\credentials.env`.

---

## 🚀 Quick Start (CLI Reference)

```powershell
# 1. Default & Recommended: Configure persistent Pure Userland auto-mount at logon (Session 1)
& "C:\Program Files\PodMesh\podmesh-windows-mount.exe" install-userland Z: http://192.168.10.179:9000 truenas-backup
schtasks /Run /TN "PodMesh-MeshFS-Drive"

# 2. Optional (Under the Hood): Configure Global Mode so sshd (Session 0) + Explorer (Session 1) both see Z:
& "C:\Program Files\PodMesh\podmesh-windows-mount.exe" install-global Z: http://192.168.10.179:9000 truenas-backup
schtasks /Run /TN "PodMesh-MeshFS-Drive"

# 3. Graceful Unmount (Drains VFS writeback cache before detaching Z:)
& "C:\Program Files\PodMesh\podmesh-windows-mount.exe" unmount Z:

# 4. Check MeshFS Gateway Health & Dedup Ratio
& "C:\Program Files\PodMesh\podmesh-windows-mount.exe" status http://192.168.10.179:9000
```

---

## 🔗 Companion Laboratory Repository

All live qualification reports, SPICE/XRDP cursor fixes (`tablet: 1`), and SHAPER Council counter-reviews are maintained in:
- `https://github.com/xavdp-pro/podmesh-windows-mount-lab` (branch `v2.4.x`)
