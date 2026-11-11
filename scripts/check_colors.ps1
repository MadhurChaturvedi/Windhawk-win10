$dwm = Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\DWM" -ErrorAction SilentlyContinue
Write-Output "=== DWM ==="
if ($dwm) {
    Write-Output "ColorizationColor: 0x$($dwm.ColorizationColor.ToString('X8'))"
    Write-Output "AccentColor: 0x$($dwm.AccentColor.ToString('X8'))"
    Write-Output "ColorPrevalence: $($dwm.ColorPrevalence)"
}

$pers = Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize" -ErrorAction SilentlyContinue
Write-Output "=== Personalize ==="
if ($pers) {
    Write-Output "ColorPrevalence: $($pers.ColorPrevalence)"
    Write-Output "SystemUsesLightTheme: $($pers.SystemUsesLightTheme)"
    Write-Output "AppsUseLightTheme: $($pers.AppsUseLightTheme)"
}

$acc = Get-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent" -ErrorAction SilentlyContinue
Write-Output "=== Accent ==="
if ($acc) {
    Write-Output "AccentColorMenu: 0x$($acc.AccentColorMenu.ToString('X8'))"
    Write-Output "StartColorMenu: 0x$($acc.StartColorMenu.ToString('X8'))"
    Write-Output "AccentPalette length: $($acc.AccentPalette.Length)"
}

