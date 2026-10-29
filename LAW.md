# LAW — `podmesh-windows-mount` (v2.4.1)

## Non-Negotiable Invariants

1. **Userland-First Default (`install-userland` / `daemon`)**:
   - By default, the FUSE drive (`Z:\`) MUST be mounted in **Pure Userland** inside the user's interactive session (`Session 1`, `/RL LIMITED`) without requiring `SYSTEM` or Administrator privilege escalation at runtime.
2. **Explicit Under-the-Hood Global Mode (`install-global` / `--global`)**:
   - Because Windows NT isolates userland DOS devices (`\Sessions\0\DosDevices\<LUID>\Z:`) from `Session 0` (`sshd` / services), `podmesh-windows-mount` keeps under the hood an explicit **Global Mode (`install-global`)** (`SYSTEM` + `-o FileSecurity=D:P(A;;FA;;;WD)` -> `\GLOBAL??\Z:`) for when an AI Agent over SSH (`Session 0`) or a system service requires direct access to `Z:\` alongside `explorer.exe`.
3. **Native Windowless Supervision (`CREATE_NO_WINDOW`)**:
   - Background execution MUST use native Win32 process creation flags (`CREATE_NO_WINDOW = 0x08000000`) and a built-in Rust watchdog loop (`supervise`), with zero reliance on fragile `.vbs` wrappers.
4. **Graceful Cache Drain Before Unmount**:
   - Unmounting (`unmount`) MUST verify that the VFS writeback cache (`127.0.0.1:5572/vfs/stats`) has `uploadsInProgress == 0` and `uploadsQueued == 0` before calling `core/quit`, preventing silent truncation of in-flight 4 MiB CAS chunks.
5. **Single SPICE Guest Agent Service (`spice-agent`)**:
   - On Proxmox VE Windows 11 guests using `vga: qxl`, `tablet: 1` MUST be enabled on the hypervisor and only the modern `spice-agent` service may own `\\.\Global\com.redhat.spice.0` (legacy `vdservice` must be removed).
