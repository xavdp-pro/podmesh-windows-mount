# AGENTS — `podmesh-windows-mount` (v2.4.1)

## Instructions for Autonomous AI Agents & Human Operators

1. **Understand the Two Mount Modes Before Diagnosing `Z:\` over SSH**:
   - **Pure Userland Mode (Default & Encouraged)**: Configured via `install-userland` (`Session 1`, `/RL LIMITED`). In this mode, `Z:\` is visible to the human in `explorer.exe` (`Session 1`), **NOT** to `sshd` (`Session 0`). Do **NOT** assume `Z:\` is broken simply because `Test-Path Z:\` returns `False` over SSH! Instead, check `Get-Process podmesh-windows-mount, rclone` (`SessionId == 1`) and `curl.exe -s -X POST http://127.0.0.1:5572/vfs/stats`.
   - **Global Mode (Under the Hood)**: Configured via `install-global` (`SYSTEM` + `-o FileSecurity=D:P(A;;FA;;;WD)`). Use this mode only when an AI Agent over SSH (`Session 0`) or a Windows Service must directly read/write `Z:\` (`\GLOBAL??\Z:`) at the same time as `explorer.exe`.
2. **Never Kill `rclone.exe` with `taskkill /F`**:
   - Always use `podmesh-windows-mount.exe unmount Z:` so the VFS writeback cache drains (`uploadsInProgress == 0`) before unmounting.
3. **Companion Lab Repository**:
   - Refer to `https://github.com/xavdp-pro/podmesh-windows-mount-lab` (`RUNBOOK_WINDOWS11_SPICE_AUTOLOGON_WINFSP.md` and `CONTRE_REGARD_WINDOWS_MESHFS_V2_4_0.md`) for full operational runbooks and qualification evidence.
