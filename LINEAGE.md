# LINEAGE — `podmesh-windows-mount`

- **2026-10-28 (`v2.4.1` Bootstrap)**:
  Created as part of the **PodMesh v2.4.x Universal Multi-Storage Architecture**, extending the MeshFS 4 MiB CAS fabric beyond Linux, Proxmox VE, TrueNAS, and Synology to native Windows 11 / Windows Server drive mounts via WinFsp.
- **2026-10-29 (`v2.4.1` Live Iron Qualification on `LOGOSTEST` VM 111)**:
  Qualified end-to-end on Windows 11 Pro 24H2 (`LOGOSTEST`, VMID 111 on `gbs-p4`), validating SPICE Guest Tools, Auto-Logon, OpenSSH Server, WinFsp 2.0 kernel driver installation, and live mounting of `MeshFS-CAS (Z:)` in Windows Explorer against the MeshFS S3 WORM Gateway.
