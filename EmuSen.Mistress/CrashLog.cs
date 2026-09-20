using System;
using System.IO;
using System.Threading.Tasks;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress
{
    // Whatever ends the process or halts the machine, written whole to a file beside the logs - see EmuSen_Settings_Reference.md §4.27.
    public static class CrashLog
    {
        public static string? DirectoryOverride;

        public static void Install()
        {
            AppDomain.CurrentDomain.UnhandledException += (_, e) => Write("unhandled", e.ExceptionObject as Exception);
            TaskScheduler.UnobservedTaskException += (_, e) => Write("unobserved task", e.Exception);
        }

        // Best effort: a report that cannot be written must not become a second fault.
        public static string? Write(string kind, Exception? fault, string? context = null)
        {
            try
            {
                string root = DirectoryOverride ?? LogRoot();
                Directory.CreateDirectory(root);
                string path = Path.Combine(root, $"crash_{DateTime.Now:yyyyMMdd_HHmmss_fff}.txt");
                File.WriteAllText(path, $"{kind} at {DateTime.Now:O}\n{context}\n\n{fault}\n");
                return path;
            }
            catch
            {
                return null;
            }
        }

        private static string LogRoot()
        {
            string? configured = AppSettings.Load().LogDirectory;
            return string.IsNullOrWhiteSpace(configured)
                ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen", "Logs")
                : configured;
        }
    }
}
