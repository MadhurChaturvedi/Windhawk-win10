Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class DwmTest {
    [DllImport("dwmapi.dll")]
    public static extern int DwmSetColorizationColor(uint crColorization, bool fOpaqueBlend);
    public static int SetColor(uint color) {
        return DwmSetColorizationColor(color, false);
    }
}
"@
$res = [DwmTest]::SetColor(0xC48B5CF6)
Write-Output "DwmSetColorizationColor returned: $res"

