Set WshShell = CreateObject("WScript.Shell")
WshShell.Run """C:\Program Files\PodMesh\podmesh-windows-mount.exe"" mount Z: http://192.168.10.179:9000 truenas-backup", 0, False
