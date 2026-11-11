Add-Type @"
using System;
using System.Runtime.InteropServices;
public class ColorTester {
    [DllImport("user32.dll", CharSet = CharSet.Auto)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
        uint fuFlags, uint uTimeout, out UIntPtr lpdwResult
    );
    [DllImport("shell32.dll")]
    public static extern void SHChangeNotify(int eventId, uint flags, IntPtr item1, IntPtr item2);

    [DllImport("dwmapi.dll", EntryPoint = "#127")]
    public static extern int DwmGetColorizationParameters(out DWMCOLORIZATIONPARAMS parameters);

    [DllImport("dwmapi.dll", EntryPoint = "#131")]
    public static extern int DwmSetColorizationParameters(ref DWMCOLORIZATIONPARAMS parameters, uint unknown);

    public struct DWMCOLORIZATIONPARAMS {
        public uint Color;
        public uint Afterglow;
        public uint ColorBalance;
        public uint AfterglowBalance;
        public uint BlurBalance;
        public uint GlassReflectionIntensity;
        public uint OpaqueBlend;
    }

    public static void Notify() {
        UIntPtr res;
        SendMessageTimeout((IntPtr)0xffff, 0x001A, UIntPtr.Zero, "ImmersiveColorSet", 2, 1000, out res);
        SendMessageTimeout((IntPtr)0xffff, 0x001A, UIntPtr.Zero, "TraySettings", 2, 1000, out res);
        SendMessageTimeout((IntPtr)0xffff, 0x001A, UIntPtr.Zero, "ShellState", 2, 1000, out res);
        SHChangeNotify(0x08000000, 0x0000, IntPtr.Zero, IntPtr.Zero);
    }
}
"@

# Let's test with a Royal Purple / Indigo color: R=139 (0x8B), G=92 (0x5C), B=246 (0xF6)
$r = 139
$g = 92
$b = 246

# ABGR format (0xFF BB GG RR)
$abgr = [uint32](0xFF000000 -bor ($b -shl 16) -bor ($g -shl 8) -bor $r)
# ARGB format (0xFF RR GG BB)
$argb = [uint32](0xC4000000 -bor ($r -shl 16) -bor ($g -shl 8) -bor $b)

Write-Output "ABGR: 0x$($abgr.ToString('X8')), ARGB: 0x$($argb.ToString('X8'))"

# 1. Enable ColorPrevalence on Taskbar
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize" -Name "ColorPrevalence" -Value 1 -Type DWord
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\DWM" -Name "ColorPrevalence" -Value 1 -Type DWord

# 2. Set AccentColor in DWM
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\DWM" -Name "AccentColor" -Value $abgr -Type DWord
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\DWM" -Name "ColorizationColor" -Value $argb -Type DWord

# 3. Set Accent in Explorer
if (!(Test-Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent")) {
    New-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent" -Force | Out-Null
}
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent" -Name "AccentColorMenu" -Value $abgr -Type DWord
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent" -Name "StartColorMenu" -Value $abgr -Type DWord

# Create a 32-byte AccentPalette based on the chosen color
$palette = New-Object byte[] 32
for ($i = 0; $i -lt 8; $i++) {
    $factor = 0.5 + ($i * 0.1) # vary brightness
    $pr = [Math]::Min(255, [int]($r * $factor))
    $pg = [Math]::Min(255, [int]($g * $factor))
    $pb = [Math]::Min(255, [int]($b * $factor))
    $palette[$i * 4 + 0] = [byte]$pr
    $palette[$i * 4 + 1] = [byte]$pg
    $palette[$i * 4 + 2] = [byte]$pb
    $palette[$i * 4 + 3] = 255
}
Set-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\Accent" -Name "AccentPalette" -Value $palette -Type Binary

# 4. Notify Shell & Taskbar
[ColorTester]::Notify()

Write-Output "Applied purple taskbar color!"

