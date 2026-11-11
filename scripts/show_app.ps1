Add-Type -ReferencedAssemblies System.Drawing, System.Windows.Forms @"
using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public class WindowCapturer {
    [DllImport("user32.dll", SetLastError = true)]
    public static extern IntPtr OpenDesktop(string lpszDesktop, uint dwFlags, bool fInherit, uint dwDesiredAccess);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool SetThreadDesktop(IntPtr hDesktop);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool CloseDesktop(IntPtr hDesktop);

    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport("user32.dll")]
    public static extern bool EnumDesktopWindows(IntPtr hDesktop, EnumWindowsProc lpfn, IntPtr lParam);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdcBlt, uint nFlags);

    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct STARTUPINFO {
        public Int32 cb;
        public string lpReserved;
        public string lpDesktop;
        public string lpTitle;
        public Int32 dwX;
        public Int32 dwY;
        public Int32 dwXSize;
        public Int32 dwYSize;
        public Int32 dwXCountChars;
        public Int32 dwYCountChars;
        public Int32 dwFillAttribute;
        public Int32 dwFlags;
        public Int16 wShowWindow;
        public Int16 cbReserved2;
        public IntPtr lpReserved2;
        public IntPtr hStdInput;
        public IntPtr hStdOutput;
        public IntPtr hStdError;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct PROCESS_INFORMATION {
        public IntPtr hProcess;
        public IntPtr hThread;
        public Int32 dwProcessId;
        public Int32 dwThreadId;
    }

    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
    public static extern bool CreateProcess(
        string lpApplicationName,
        string lpCommandLine,
        IntPtr lpProcessAttributes,
        IntPtr lpThreadAttributes,
        bool bInheritHandles,
        uint dwCreationFlags,
        IntPtr lpEnvironment,
        string lpCurrentDirectory,
        ref STARTUPINFO lpStartupInfo,
        out PROCESS_INFORMATION lpProcessInformation
    );

    public static string LaunchAndCapture(string exePath, string outputPath) {
        STARTUPINFO si = new STARTUPINFO();
        si.cb = Marshal.SizeOf(si);
        si.lpDesktop = @"WinSta0\Default";
        PROCESS_INFORMATION pi = new PROCESS_INFORMATION();
        bool ok = CreateProcess(exePath, null, IntPtr.Zero, IntPtr.Zero, false, 0, IntPtr.Zero, null, ref si, out pi);
        if (!ok) {
            return "Failed to launch process: " + Marshal.GetLastWin32Error();
        }

        IntPtr foundHwnd = IntPtr.Zero;
        IntPtr hDesk = OpenDesktop("Default", 0, false, 0x01FF);
        SetThreadDesktop(hDesk);

        // Wait up to 6 seconds for window to appear
        for (int i = 0; i < 30; i++) {
            Thread.Sleep(200);
            EnumDesktopWindows(hDesk, (hWnd, lParam) => {
                uint pid;
                GetWindowThreadProcessId(hWnd, out pid);
                if (pid == pi.dwProcessId && IsWindowVisible(hWnd)) {
                    var sb = new StringBuilder(256);
                    GetWindowText(hWnd, sb, 256);
                    if (sb.ToString().Contains("Windhawk")) {
                        foundHwnd = hWnd;
                        return false;
                    }
                }
                return true;
            }, IntPtr.Zero);
            if (foundHwnd != IntPtr.Zero) break;
        }

        if (foundHwnd == IntPtr.Zero) {
            CloseDesktop(hDesk);
            return "Process launched with PID " + pi.dwProcessId + " but window not found within timeout.";
        }

        ShowWindow(foundHwnd, 9); // SW_RESTORE
        SetForegroundWindow(foundHwnd);
        Thread.Sleep(1000);

        RECT rect;
        GetWindowRect(foundHwnd, out rect);
        int width = rect.Right - rect.Left;
        int height = rect.Bottom - rect.Top;

        if (width <= 0 || height <= 0) {
            CloseDesktop(hDesk);
            return "Invalid window dimensions: " + width + "x" + height;
        }

        try {
            using (Bitmap bmp = new Bitmap(width, height)) {
                using (Graphics g = Graphics.FromImage(bmp)) {
                    IntPtr hdc = g.GetHdc();
                    // PW_RENDERFULLCONTENT = 2
                    bool printSuccess = PrintWindow(foundHwnd, hdc, 2);
                    g.ReleaseHdc(hdc);
                    if (!printSuccess) {
                        g.CopyFromScreen(rect.Left, rect.Top, 0, 0, new Size(width, height), CopyPixelOperation.SourceCopy);
                    }
                }
                bmp.Save(outputPath, ImageFormat.Png);
            }
        } catch (Exception ex) {
            CloseDesktop(hDesk);
            return "Error saving bitmap: " + ex.Message;
        }

        CloseDesktop(hDesk);
        return "SUCCESS: PID " + pi.dwProcessId + ", Window: " + width + "x" + height + ", Saved to: " + outputPath;
    }
}
"@

$targetDir = "C:\Users\madhur\.gemini\antigravity\brain\430ce6f1-5b42-4275-9d0e-1a2bedf74313"
if (!(Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

$exe = "D:\Windhawk-win10\windhawk-win10\target\release\windhawk-win10.exe"
$outPng = "$targetDir\windhawk_gui_preview.png"

# Kill any existing windhawk-win10 instances first so we launch clean
Get-Process -Name "windhawk-win10" -ErrorAction SilentlyContinue | Stop-Process -Force

$result = [WindowCapturer]::LaunchAndCapture($exe, $outPng)
Write-Output $result

