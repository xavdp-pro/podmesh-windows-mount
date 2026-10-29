# INTENT — `podmesh-windows-mount` (v2.4.0)

## Why `podmesh-windows-mount` Exists

`podmesh-windows-mount` is the **Native Windows FUSE Drive Mount Utility** for the PodMesh and MeshFS v2.4.0 ecosystem.

While Linux nodes mount MeshFS directly via kernel FUSE (`fuse3`), Windows workstations and servers require a kernel-mode filesystem proxy to project the deduplicated 4 MiB Content-Addressable Storage (CAS) pool as a native local drive letter (e.g., `Z:\` / `MeshFS-CAS (Z:)`).

`podmesh-windows-mount` bridges Windows 11 / Windows Server directly to the MeshFS S3 WORM Gateway (`podmesh-meshfs-s3`) using **WinFsp** (*Windows File System Proxy*):

1. **Native Drive Letter Projection (`MeshFS-CAS (Z:)`)**:
   Mounts any MeshFS bucket as a first-class Windows volume accessible in Windows Explorer (`Ce PC`), PowerShell, CMD, and native Win32 applications without SMB/CIFS protocol overhead.
2. **4 MiB CAS Deduplication & WORM Compliance**:
   Every file written to `Z:\` is automatically chunked into 4 MiB SHA-256 blocks by the MeshFS Gateway, deduplicated across Proxmox VE, TrueNAS, Synology, and Linux nodes, and protected by fail-closed WORM retention locks.
3. **Interactive Session 1 Token Alignment**:
   Solves the Windows Session 0 / UAC drive-letter isolation trap by providing a deterministic scheduled-task and hidden VBS launcher (`scripts/run-hidden.vbs`) that mounts the FUSE drive directly inside the interactive user token (`explorer.exe`).
4. **Zero-Dependency Rust Binary**:
   Compiled via MinGW (`x86_64-pc-windows-gnu`) into a standalone 1.2 MiB PE32+ executable (`dist/podmesh-windows-mount.exe`).
