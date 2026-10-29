# dump_audio_bt.ps1
# Diagnostics script to dump Bluetooth devices and Audio endpoints before and during a call.
# Usage: powershell -ExecutionPolicy Bypass -File .\dump_audio_bt.ps1 [-OutputJson]

param (
    [switch]$OutputJson
)

$timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  SAATHI / VoidRelay Audio & Bluetooth Diagnostics ($timestamp)" -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

# 1. Bluetooth Devices & Services
Write-Host "`n--- [1] Bluetooth Devices & Services ---" -ForegroundColor Yellow

$btDevices = Get-PnpDevice -Class "Bluetooth" -ErrorAction SilentlyContinue | Select-Object FriendlyName, Status, InstanceId, Problem

$btDevices | Format-Table -Property FriendlyName, Status, Problem, InstanceId -AutoSize

# Check specifically for Handsfree / Headset profiles
$hfpDevices = $btDevices | Where-Object { 
    $_.FriendlyName -match "Hands-Free|Headset|Handsfree|Telephony|Gateway" 
}

if ($hfpDevices) {
    Write-Host "[+] Found Hands-Free / Telephony related Bluetooth devices:" -ForegroundColor Green
    $hfpDevices | ForEach-Object { Write-Host "    - $($_.FriendlyName) (Status: $($_.Status))" -ForegroundColor Green }
} else {
    Write-Host "[-] No Bluetooth Handsfree/Headset services detected currently." -ForegroundColor Yellow
}

# 2. Windows MMDevice Audio Endpoints
Write-Host "`n--- [2] Windows Audio Render Endpoints (Speakers/Outputs) ---" -ForegroundColor Yellow

$renderEndpoints = @()
$renderKeys = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\*' -ErrorAction SilentlyContinue
foreach ($key in $renderKeys) {
    $id = $key.PSChildName
    $propKey = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render\$id\Properties"
    if (Test-Path $propKey) {
        $props = Get-ItemProperty $propKey -ErrorAction SilentlyContinue
        $name = $props.'{a45c254e-df1c-4efd-8020-67d146a850e0},2'
        $desc = $props.'{b3f8fa53-0004-438e-9003-51a46e139bfc},6'
        if (-not $desc) { $desc = $props.'{a45c254e-df1c-4efd-8020-67d146a850e0},14' }
        $fullLabel = if ($desc) { "$name ($desc)" } else { $name }
        $state = $key.DeviceState # 1 = Active, 2 = Disabled, 4 = NotPresent, 8 = Unplugged
        $stateStr = switch ($state) {
            1 { "Active" }
            2 { "Disabled" }
            4 { "NotPresent" }
            8 { "Unplugged" }
            default { "Unknown ($state)" }
        }
        $renderEndpoints += [PSCustomObject]@{
            Flow      = "Render"
            Name      = $name
            Device    = $desc
            Label     = $fullLabel
            State     = $stateStr
            Endpoint  = $id
        }
    }
}
$renderEndpoints | Format-Table -Property State, Name, Device, Endpoint -AutoSize

Write-Host "`n--- [3] Windows Audio Capture Endpoints (Microphones/Inputs) ---" -ForegroundColor Yellow

$captureEndpoints = @()
$captureKeys = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture\*' -ErrorAction SilentlyContinue
foreach ($key in $captureKeys) {
    $id = $key.PSChildName
    $propKey = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture\$id\Properties"
    if (Test-Path $propKey) {
        $props = Get-ItemProperty $propKey -ErrorAction SilentlyContinue
        $name = $props.'{a45c254e-df1c-4efd-8020-67d146a850e0},2'
        $desc = $props.'{b3f8fa53-0004-438e-9003-51a46e139bfc},6'
        if (-not $desc) { $desc = $props.'{a45c254e-df1c-4efd-8020-67d146a850e0},14' }
        $fullLabel = if ($desc) { "$name ($desc)" } else { $name }
        $state = $key.DeviceState
        $stateStr = switch ($state) {
            1 { "Active" }
            2 { "Disabled" }
            4 { "NotPresent" }
            8 { "Unplugged" }
            default { "Unknown ($state)" }
        }
        $captureEndpoints += [PSCustomObject]@{
            Flow      = "Capture"
            Name      = $name
            Device    = $desc
            Label     = $fullLabel
            State     = $stateStr
            Endpoint  = $id
        }
    }
}
$captureEndpoints | Format-Table -Property State, Name, Device, Endpoint -AutoSize

# 3. Phone Bluetooth Endpoint Check
Write-Host "`n--- [4] Phone Hands-Free Audio Endpoints Summary ---" -ForegroundColor Yellow
$phoneRender = $renderEndpoints | Where-Object { $_.Label -match "Hands-Free|OnePlus|Phone|Cell" }
$phoneCapture = $captureEndpoints | Where-Object { $_.Label -match "Hands-Free|OnePlus|Phone|Cell" }

if ($phoneRender) {
    Write-Host "[+] Phone Render Endpoints detected in Windows:" -ForegroundColor Green
    $phoneRender | ForEach-Object { Write-Host "    - $($_.Label) [$($_.State)]" -ForegroundColor Green }
} else {
    Write-Host "[-] No Phone Render Endpoints found in Windows." -ForegroundColor Red
}

if ($phoneCapture) {
    Write-Host "[+] Phone Capture Endpoints detected in Windows:" -ForegroundColor Green
    $phoneCapture | ForEach-Object { Write-Host "    - $($_.Label) [$($_.State)]" -ForegroundColor Green }
} else {
    Write-Host "[-] No Phone Capture Endpoints found in Windows." -ForegroundColor Red
}

# 4. Voicemeeter Devices Check
Write-Host "`n--- [5] Voicemeeter Status ---" -ForegroundColor Yellow
$vmRender = $renderEndpoints | Where-Object { $_.Name -match "Voicemeeter" }
$vmCapture = $captureEndpoints | Where-Object { $_.Name -match "Voicemeeter" }

if ($vmRender -and $vmCapture) {
    Write-Host "[+] Voicemeeter VAIO Virtual Endpoints are AVAILABLE." -ForegroundColor Green
} else {
    Write-Host "[-] Voicemeeter Endpoints NOT detected. Install Voicemeeter if needed." -ForegroundColor Yellow
}

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Diagnostics Complete. Run this during a live call to compare!" -ForegroundColor Cyan
Write-Host "============================================================`n" -ForegroundColor Cyan

if ($OutputJson) {
    $result = [PSCustomObject]@{
        Timestamp        = $timestamp
        BluetoothDevices = $btDevices
        RenderEndpoints  = $renderEndpoints
        CaptureEndpoints = $captureEndpoints
        PhoneRender      = $phoneRender
        PhoneCapture     = $phoneCapture
        VoicemeeterReady = ($vmRender.Count -gt 0 -and $vmCapture.Count -gt 0)
    }
    return ($result | ConvertTo-Json -Depth 4)
}
