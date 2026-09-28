using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress.Input
{
    // What types into a big-screen text row: Steam's keyboard over a real field, the field alone, or Mistress's own keyboard - see EmuSen_Settings_Reference.md §4.79.
    public enum KeyboardKind { Steam, Field, EmuSen }

    // The window a big-screen text row lives in, which knows the setting and whether a pad or the keyboard chose the row.
    public interface IDeviceKeyboardHost
    {
        KeyboardKind KeyboardFor(Avalonia.Controls.TextBox box);
    }

    // Opens a URL the way the platform does; tests replace it so that nothing is ever launched.
    public interface IUrlLauncher
    {
        void Open(string url);
    }

    // The On-Screen Keyboard setting, the environment that says Steam is there, and the request for Steam's keyboard - see EmuSen_Settings_Reference.md §4.79.
    public static class DeviceKeyboard
    {
        public const string SteamKeyboardUrl = "steam://open/keyboard";

        public static readonly (string Value, string Text)[] Choices =
        [
            (AppSettings.OnScreenKeyboardAutomatic, "Automatic"), (AppSettings.OnScreenKeyboardSteam, "Steam"), (AppSettings.OnScreenKeyboardEmuSen, "EmuSen's"),
        ];

        // The process's own environment, a check for a running Steam client, and the platform's URL opener; the test harness swaps all three.
        public static Func<string, string?> Environment { get; set; } = System.Environment.GetEnvironmentVariable;
        public static Func<bool> SteamRunning { get; set; } = SteamProcessRunning;
        public static IUrlLauncher Launcher { get; set; } = new PlatformUrlLauncher();

        // Automatic: the field alone for a physical keyboard, Steam or not (Q165); for a pad, Steam's keyboard under Steam, else Mistress's.
        public static KeyboardKind Choose(string? setting, bool underSteam, bool padInUse) => setting switch
        {
            AppSettings.OnScreenKeyboardSteam => KeyboardKind.Steam,
            AppSettings.OnScreenKeyboardEmuSen => KeyboardKind.EmuSen,
            _ => !padInUse ? KeyboardKind.Field : underSteam ? KeyboardKind.Steam : KeyboardKind.EmuSen,
        };

        // Game Mode's session, a process Steam launched, a Steam Deck, or a Steam client running (Desktop Mode with Steam open).
        public static bool UnderSteam(Func<string, string?> environment, Func<bool> steamRunning) =>
            Views.MainWindow.InGameModeSession(environment) || LaunchedBySteam(environment) || environment("SteamDeck") == "1" || steamRunning();

        // Steam gives what it launches its game's id, and Proton's compatibility variables to what runs under Proton.
        public static bool LaunchedBySteam(Func<string, string?> environment) =>
            new[] { "SteamGameId", "SteamAppId", "STEAM_COMPAT_APP_ID", "STEAM_COMPAT_DATA_PATH", "STEAM_COMPAT_CLIENT_INSTALL_PATH" }
                .Any(name => !string.IsNullOrEmpty(environment(name)));

        public static KeyboardKind ChooseNow(string? setting, bool padInUse) => Choose(setting, UnderSteam(Environment, SteamRunning), padInUse);

        // Asks Steam for its keyboard; it types into whichever window has the focus.
        public static void AskSteam() => Launcher.Open(SteamKeyboardUrl);

        // pgrep -x matches the process name exactly, so a command line that merely mentions steam does not count.
        private static bool SteamProcessRunning()
        {
            if (!OperatingSystem.IsLinux()) return Process.GetProcessesByName("steam").Length > 0;
            try
            {
                using Process? p = Process.Start(new ProcessStartInfo("pgrep", ["-x", "steam"]) { RedirectStandardOutput = true, UseShellExecute = false });
                if (p is null) return false;
                if (!p.WaitForExit(2000)) { p.Kill(); return false; }
                return p.ExitCode == 0;
            }
            catch (Exception) { return false; }
        }
    }

    // xdg-open first, as the desktop opens any link; the steam command if xdg-open is missing or fails; the shell elsewhere.
    public sealed class PlatformUrlLauncher : IUrlLauncher
    {
        public void Open(string url) => _ = Task.Run(() =>
        {
            if (!OperatingSystem.IsLinux())
            {
                Run(new ProcessStartInfo(url) { UseShellExecute = true });
                return;
            }
            if (!Run(new ProcessStartInfo("xdg-open", [url]) { UseShellExecute = false })) Run(new ProcessStartInfo("steam", [url]) { UseShellExecute = false });
        });

        private static bool Run(ProcessStartInfo start)
        {
            try
            {
                using Process? p = Process.Start(start);
                if (p is null) return false;
                // Steam may keep the process open while it hands the link to the running client; only a quick failure counts.
                return !p.WaitForExit(10000) || p.ExitCode == 0;
            }
            catch (Exception) { return false; }
        }
    }

    // Records what would have been opened and opens nothing, for a harness that must never launch a program.
    public sealed class RecordingUrlLauncher : IUrlLauncher
    {
        private readonly List<string> _opened = new();

        public IReadOnlyList<string> Opened { get { lock (_opened) return _opened.ToList(); } }

        public void Open(string url) { lock (_opened) _opened.Add(url); }

        public void Clear() { lock (_opened) _opened.Clear(); }
    }
}
