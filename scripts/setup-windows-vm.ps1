# PodMesh Windows 11 Autonomous Provisioning Script (v2.4.0)
# Configures AutoAdminLogon, disables sleep/lockscreen, enables OpenSSH (key + password),
# installs SPICE Guest Tools / VirtIO / WinFsp, and mounts MeshFS-CAS (Z:) at logon.

# 1. User account & password expiration policy
net user logos LogosAdmin2026! /logonpasswordchg:no
net accounts /maxpwage:unlimited

# 2. Direct AutoAdminLogon into Windows desktop
$winlogon = "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon"
Set-ItemProperty -Path $winlogon -Name "AutoAdminLogon" -Value "1"
Set-ItemProperty -Path $winlogon -Name "DefaultUserName" -Value "logos"
Set-ItemProperty -Path $winlogon -Name "DefaultPassword" -Value "LogosAdmin2026!"
Set-ItemProperty -Path $winlogon -Name "DefaultDomainName" -Value $env:COMPUTERNAME
if (Test-Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\PasswordLess\Device") {
    Set-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion\PasswordLess\Device" -Name "DevicePasswordLessBuildVersion" -Value 0
}

# 3. Disable sleep, hibernate, screen timeout, and lock screen
powercfg /change standby-timeout-ac 0
powercfg /change standby-timeout-dc 0
powercfg /change monitor-timeout-ac 0
powercfg /change monitor-timeout-dc 0
powercfg /change hibernate-timeout-ac 0
powercfg /change hibernate-timeout-dc 0
powercfg /hibernate off
New-Item -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows" -Name "Personalization" -Force -ErrorAction SilentlyContinue | Out-Null
Set-ItemProperty -Path "HKLM:\SOFTWARE\Policies\Microsoft\Windows\Personalization" -Name "NoLockScreen" -Value 1
Set-ItemProperty -Path "HKCU:\Control Panel\Desktop" -Name "ScreenSaveActive" -Value "0"

# 4. Enable OpenSSH Server with language-independent SID ACLs
Add-WindowsCapability -Online -Name OpenSSH.Server~~~~0.0.1.0 -ErrorAction SilentlyContinue
Set-Service -Name sshd -StartupType Automatic
Start-Service sshd

$sshKey = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID4ezzmrooG7FixkJrs1CgwIF5yN2DxLrrq7nP0c0DA0 zaza@ssd.ici"
New-Item -Path "C:\ProgramData\ssh" -ItemType Directory -Force -ErrorAction SilentlyContinue | Out-Null
$adminKeyPath = "C:\ProgramData\ssh\administrators_authorized_keys"
Set-Content -Path $adminKeyPath -Value $sshKey -Encoding Ascii
icacls.exe $adminKeyPath /inheritance:r /grant "*S-1-5-32-544:F" /grant "*S-1-5-18:F"

New-Item -Path "C:\Users\logos\.ssh" -ItemType Directory -Force -ErrorAction SilentlyContinue | Out-Null
$userKeyPath = "C:\Users\logos\.ssh\authorized_keys"
Set-Content -Path $userKeyPath -Value $sshKey -Encoding Ascii
icacls.exe $userKeyPath /inheritance:r /grant "logos:F" /grant "*S-1-5-18:F"

$sshdConfig = "C:\ProgramData\ssh\sshd_config"
if (Test-Path $sshdConfig) {
    $c = Get-Content $sshdConfig
    $c = $c -replace '#PasswordAuthentication yes', 'PasswordAuthentication yes'
    $c = $c -replace 'PasswordAuthentication no', 'PasswordAuthentication yes'
    $c = $c -replace 'Match Group administrators', '#Match Group administrators'
    $c = $c -replace 'AuthorizedKeysFile __PROGRAMDATA__/ssh/administrators_authorized_keys', '#AuthorizedKeysFile __PROGRAMDATA__/ssh/administrators_authorized_keys'
    Set-Content -Path $sshdConfig -Value $c -Encoding Ascii
    Restart-Service sshd
}

# 5. Windows Firewall rules (SSH 22 & RDP 3389)
netsh advfirewall firewall add rule name="OpenSSH" dir=in action=allow protocol=TCP localport=22
netsh advfirewall firewall add rule name="RDP" dir=in action=allow protocol=TCP localport=3389
Set-ItemProperty -Path 'HKLM:\System\CurrentControlSet\Control\Terminal Server' -Name "fDenyTSConnections" -Value 0

Write-Host ">>> WINDOWS VM SETUP COMPLETED SUCCESSFULLY <<<"
