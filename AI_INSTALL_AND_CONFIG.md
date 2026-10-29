# AI-First Installation & Configuration Guide (`podmesh-windows-mount` v2.4.1)

This runbook is designed for **autonomous AI Agents** and **Human System Administrators** deploying and operating the native Windows 11 FUSE drive (`Z:\`) backed by **PodMesh MeshFS 4 MiB CAS**.

---

## 1. Core Doctrine: Pure Userland by Default vs. Global Mode Under the Hood

### Mode A: Pure Userland Mode (DEFAULT & STRONGLY ENCOURAGED)
- **Why we encourage Pure Userland**: Following the Shaper OS userland-first security principle, a filesystem mount should run with the minimum necessary privileges (`Session 1`, `/RL LIMITED`, standard user token) without escalating to `SYSTEM` or Administrator at runtime.
- **How to configure**:
  ```powershell
  & "C:\Program Files\PodMesh\podmesh-windows-mount.exe" install-userland Z: http://192.168.10.179:9000 truenas-backup
  schtasks /Run /TN "PodMesh-MeshFS-Drive"
  ```
- **Crucial Behavior for AI Agents over SSH (`Session 0`)**:
  - In Pure Userland mode, Windows NT creates the drive letter inside the interactive user's local Logon Session namespace (`\Sessions\0\DosDevices\<User-LUID>\Z:`).
  - Consequently, `explorer.exe` and all GUI applications in `Session 1` see and use `Z:\` normally, **but an SSH session (`sshd` running in `Session 0`) will see `Test-Path Z:\ == False`**.
  - **How an AI Agent over SSH verifies a Pure Userland mount**:
    1. Check that both `podmesh-windows-mount` (supervisor) and `rclone` are running in `SessionId == 1`:
       `Get-Process podmesh-windows-mount, rclone | Select-Object Name, Id, SessionId`
    2. Query the local VFS Remote Control health endpoint:
       `curl.exe -s -X POST http://127.0.0.1:5572/vfs/stats`

---

### Mode B: Global Administrator/SYSTEM Mode (Kept Under the Hood for SSH / Service Access)
- **When to use**: When an AI Agent connected via `sshd` (`Session 0`), a CI/CD runner, or a Windows Service must directly read/write `Z:\` (`dir Z:\`, `Get-ChildItem Z:\`) at the exact same time as the human user in `explorer.exe` (`Session 1`).
- **How it works under the hood**:
  - Runs the `supervise` loop under `NT AUTHORITY\SYSTEM` (`/RU "SYSTEM" /RL HIGHEST /SC ONSTART`) with the WinFsp FUSE option `-o FileSecurity=D:P(A;;FA;;;WD)`.
  - WinFsp registers the symbolic link in the Windows NT global namespace **`\GLOBAL??\Z:`**, making `Z:\` visible and writable across **all sessions** (`Session 0` SSH + `Session 1` GUI + UAC Elevated Admin).
- **How to activate Global Mode**:
  ```powershell
  & "C:\Program Files\PodMesh\podmesh-windows-mount.exe" unmount Z:
  & "C:\Program Files\PodMesh\podmesh-windows-mount.exe" install-global Z: http://192.168.10.179:9000 truenas-backup
  schtasks /Run /TN "PodMesh-MeshFS-Drive"
  ```
- **How to switch back to Pure Userland Mode**:
  ```powershell
  & "C:\Program Files\PodMesh\podmesh-windows-mount.exe" unmount Z:
  & "C:\Program Files\PodMesh\podmesh-windows-mount.exe" install-userland Z: http://192.168.10.179:9000 truenas-backup
  schtasks /Run /TN "PodMesh-MeshFS-Drive"
  ```

---

## 2. Credentials Configuration (`credentials.env`)

Create `%USERPROFILE%\.meshfs\credentials.env` (for Pure Userland mode) and `C:\ProgramData\PodMesh\credentials.env` (for Global `SYSTEM` mode):
```ini
MESHFS_ACCESS_KEY=meshfs-admin
MESHFS_SECRET_KEY=meshfs-secret-2026-safe-key
```

---

## 3. Graceful Unmount & Zero Data Loss Guarantee

Never run `taskkill /F /IM rclone.exe` while writes are in flight. Always use:
```powershell
& "C:\Program Files\PodMesh\podmesh-windows-mount.exe" unmount Z:
```
This command:
1. Signals the `supervise` watchdog loop to stop restarting.
2. Polls `http://127.0.0.1:5572/vfs/stats` until `uploadsInProgress == 0` and `uploadsQueued == 0`.
3. Sends a clean `POST http://127.0.0.1:5572/core/quit` to flush and detach the WinFsp volume.
