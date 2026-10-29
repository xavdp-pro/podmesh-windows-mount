# LAW — Architectural & Operational Invariants of `podmesh-windows-mount` (v2.4.0)

1. **Kernel FUSE Equivalence via WinFsp**:
   - Under Windows 11 / Windows Server, filesystem mounting MUST operate through the signed kernel driver **WinFsp** (`WinFsp.Launcher` service) paired with the VFS caching dispatcher (`--vfs-cache-mode=full` and `--s3-force-path-style=true`).
2. **Windows Interactive Session Token Visibility (`Session 1` vs `Session 0`)**:
   - Windows isolates drive letters per Logon Session (`\Sessions\<ID>\DosDevices`). A drive mounted inside an elevated SSH session (`Session 0`) is invisible to the graphical desktop shell (`explorer.exe` in `Session 1`).
   - Therefore, persistent desktop mounts MUST be registered under the interactive user token (`/RU "<user>" /IT /RL LIMITED /SC ONLOGON`) via `scripts/run-hidden.vbs` so that both the user desktop (`Ce PC > MeshFS-CAS (Z:)`) and user applications see the volume immediately on boot.
3. **Language-Independent Windows Security Identifiers (SID)**:
   - Provisioning scripts MUST never hardcode localized group names (`Administrators` vs `Administrateurs`). All ACL operations (`icacls.exe`) MUST use universal well-known SIDs: `*S-1-5-32-544` (Administrators) and `*S-1-5-18` (`NT AUTHORITY\SYSTEM`).
4. **Deterministic Auto-Logon & Zero-Sleep Test Readiness**:
   - On autonomous Windows test nodes (`LOGOSTEST`), `DevicePasswordLessBuildVersion` MUST be set to `0` alongside `AutoAdminLogon=1`, `NoLockScreen=1`, and all `powercfg` timeouts set to `0`.
5. **Bounded Retry & Fail-Closed Health Check (Rule 38)**:
   - `podmesh-windows-mount.exe status <endpoint>` queries `/healthz` with a strict timeout and returns deterministic exit codes (`0` healthy, non-zero on unreachable gateway).
