# AI-First Installation & Configuration Guide (`podmesh-windows-mount` v2.4.0)

This guide provides the deterministic, zero-UI procedure for an AI agent to provision a Windows 11 machine (`LOGOSTEST`) over SSH and mount a MeshFS 4 MiB CAS bucket as a native local FUSE drive (`Z:\`).

---

## 1. Prerequisites on the Windows Target

Download and stage in `C:\Program Files\PodMesh\`:
1. **WinFsp 2.0 MSI**: `https://github.com/winfsp/winfsp/releases/download/v2.0/winfsp-2.0.23075.msi`
2. **Rclone Windows AMD64**: `https://downloads.rclone.org/v1.68.1/rclone-v1.68.1-windows-amd64.zip` (`rclone.exe`)
3. **PodMesh Windows Mount Binary**: `dist/podmesh-windows-mount.exe`
4. **Hidden Interactive Session Launcher**: `scripts/run-hidden.vbs`

---

## 2. Silent Installation via SSH

```cmd
copy "C:\Program Files\PodMesh\winfsp.msi" C:\winfsp.msi
msiexec.exe /i C:\winfsp.msi /qn /norestart
del C:\winfsp.msi
sc query WinFsp.Launcher
```

---

## 3. Health Verification Against MeshFS Gateway

```cmd
"C:\Program Files\PodMesh\podmesh-windows-mount.exe" status http://192.168.10.179:9000
```

Expected JSON output:
```json
{"cas_chunk_count":3,"dedup_ratio":1.9999959,"service":"meshfs-s3","status":"ok","version":"2.3.0"}
```

---

## 4. Persistent Interactive FUSE Mount (`MeshFS-CAS (Z:)`)

To ensure the mounted drive `Z:` is visible inside the graphical Windows Explorer (`Ce PC`) as well as user applications upon Auto-Logon:

```cmd
schtasks /Create /RU "logos" /IT /RL LIMITED /SC ONLOGON /TN "PodMesh-MeshFS-Drive" /TR "wscript.exe \"C:\Program Files\PodMesh\run-hidden.vbs\"" /F
schtasks /Run /TN "PodMesh-MeshFS-Drive"
```
