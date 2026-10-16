using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;

namespace EmuSen.WiseMan.Galaxia
{
    // The links a test makes: symbolic links wherever the system allows them, and Windows' junctions - see EmuSen_RustPlatform.md §27.
    internal static class Links
    {
        private static readonly Lazy<bool> Probe = new(() =>
        {
            if (!OperatingSystem.IsWindows()) return true;
            string link = Path.Combine(Path.GetTempPath(), "EmuSenLink_" + Guid.NewGuid().ToString("N"));
            try
            {
                File.CreateSymbolicLink(link, link + ".target");
                File.Delete(link);
                return true;
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return false; }
        });

        // Windows makes a symbolic link only with its privilege or in developer mode, which a CI runner has and a desktop may not.
        public static bool Allowed => Probe.Value;

        // A folder and all under it, its links removed first and each alone: Directory.Delete refuses a junction inside a recursive delete on Windows.
        public static void DeleteTree(string root)
        {
            if (!Directory.Exists(root)) return;
            if (!OperatingSystem.IsWindows())
            {
                Directory.Delete(root, recursive: true);
                return;
            }
            var folders = new Stack<string>(new[] { root });
            while (folders.Count > 0)
            {
                foreach (string entry in Directory.GetFileSystemEntries(folders.Pop()))
                {
                    FileAttributes attributes = File.GetAttributes(entry);
                    if ((attributes & FileAttributes.ReparsePoint) == 0)
                    {
                        if ((attributes & FileAttributes.Directory) != 0) folders.Push(entry);
                    }
                    else if ((attributes & FileAttributes.Directory) != 0) Directory.Delete(entry);
                    else File.Delete(entry);
                }
            }
            Directory.Delete(root, recursive: true);
        }

        // A Windows junction, which needs no privilege, made as mklink makes one.
        public static void Junction(string link, string target)
        {
            var start = new ProcessStartInfo("cmd.exe") { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
            foreach (string argument in new[] { "/C", "mklink", "/J", link, target }) start.ArgumentList.Add(argument);
            using Process made = Process.Start(start)!;
            string said = made.StandardOutput.ReadToEnd() + made.StandardError.ReadToEnd();
            made.WaitForExit();
            if (made.ExitCode != 0) throw new IOException($"mklink /J {link} {target}: {said}");
        }
    }
}
