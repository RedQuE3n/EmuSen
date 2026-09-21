using System;
using System.Diagnostics;
using System.IO;

namespace EmuSen.Mistress.Input
{
    // Steam's on-screen keyboard, asked for by its URL when a pad puts the focus in a text box - see EmuSen_Settings_Reference.md §4.30.
    public static class SteamKeyboard
    {
        public const string Url = "steam://open/keyboard";

        // Replaced by tests; the real one hands the URL to the running Steam client.
        public static Action<string> Launcher { get; set; } = Launch;
        public static Func<string, string?> Environment { get; set; } = System.Environment.GetEnvironmentVariable;
        public static Func<bool> ClientRunning { get; set; } = SteamIsRunning;

        // Only a Steam that is already running is asked, because the same command would otherwise start one.
        public static bool IsAvailable =>
            Environment("SteamDeck") == "1" || !string.IsNullOrEmpty(Environment("SteamGameId")) || ClientRunning();

        public static bool Show()
        {
            if (!IsAvailable) return false;

            try { Launcher(Url); return true; }
            catch (Exception) { return false; }
        }

        private static void Launch(string url) =>
            Process.Start(new ProcessStartInfo("steam", url) { UseShellExecute = false, RedirectStandardOutput = true, RedirectStandardError = true })?.Dispose();

        private static bool SteamIsRunning()
        {
            try
            {
                string file = Path.Combine(System.Environment.GetFolderPath(System.Environment.SpecialFolder.UserProfile), ".steam", "steam.pid");
                return File.Exists(file) && int.TryParse(File.ReadAllText(file).Trim(), out int pid) && Directory.Exists($"/proc/{pid}");
            }
            catch (Exception) { return false; }
        }
    }
}
