using System;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Galaxia.Models;

namespace EmuSen.Galaxia.Library
{
    // Every error a frontend shows or swallows, one plain-text file a day in the log folder - see EmuSen_Settings_Reference.md §4.70.
    public static class ErrorLog
    {
        public const string Prefix = "emusen_";
        public const int KeepDays = 14;
        public const long MaxBytes = 8L << 20;

        private static readonly object Gate = new();
        private static bool _pruned;

        public static string? DirectoryOverride { get; set; }

        // Set by a frontend at start, so nothing it writes carries a credential to disk.
        public static Func<string, string>? Redactor { get; set; }

        public static string DefaultRoot => Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen", "Logs");

        // The folder the per-game logs and crash reports share: the LogDirectory setting when it can be made, else ~/.config/EmuSen/Logs.
        public static string Root()
        {
            if (DirectoryOverride is { } set) return set;
            string? configured = null;
            try { configured = AppSettings.Load().LogDirectory; } catch { }
            return Usable(configured) ? configured! : DefaultRoot;
        }

        // A settings file copied from another machine can name a folder this one cannot make.
        public static bool Usable(string? directory)
        {
            if (string.IsNullOrWhiteSpace(directory)) return false;
            try
            {
                Directory.CreateDirectory(directory);
                return true;
            }
            catch
            {
                return false;
            }
        }

        public static string PathFor(DateTime day) => Path.Combine(Root(), $"{Prefix}{day:yyyyMMdd}.log");

        public static string? Error(string area, string message, Exception? fault = null, string? context = null) =>
            Write("ERROR", area, message, fault, context);

        public static string? Warning(string area, string message, string? context = null) =>
            Write("WARN", area, message, null, context);

        // Best effort: a log that cannot be written must never become a second fault.
        private static string? Write(string level, string area, string message, Exception? fault, string? context)
        {
            try
            {
                DateTime now = DateTime.Now;
                var entry = new StringBuilder();
                entry.Append($"{now:yyyy-MM-dd HH:mm:ss.fff} {level} [{area}] {OneLine(message)}\n");
                if (!string.IsNullOrWhiteSpace(context)) entry.Append($"    context: {OneLine(context)}\n");
                if (fault is not null) entry.Append(Indent(fault.ToString()));
                string text = Redactor is { } redact ? redact(entry.ToString()) : entry.ToString();

                lock (Gate)
                {
                    string path = PathFor(now);
                    Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                    Prune(Path.GetDirectoryName(path)!, now);
                    long size = File.Exists(path) ? new FileInfo(path).Length : 0;
                    if (size >= MaxBytes) return null;
                    if (size + text.Length >= MaxBytes) text += $"{now:yyyy-MM-dd HH:mm:ss.fff} WARN [log] this file reached {MaxBytes >> 20} MB; nothing more is written to it today\n";
                    File.AppendAllText(path, text);
                    return path;
                }
            }
            catch
            {
                return null;
            }
        }

        private static void Prune(string root, DateTime now)
        {
            if (_pruned) return;
            _pruned = true;
            foreach (string file in Directory.GetFiles(root, Prefix + "*.log").Where(f => File.GetLastWriteTime(f) < now.AddDays(-KeepDays)))
                try { File.Delete(file); } catch { }
        }

        private static string OneLine(string text) => text.ReplaceLineEndings(" ⏎ ");

        private static string Indent(string text) => string.Concat(text.ReplaceLineEndings("\n").Split('\n').Select(l => "    " + l + "\n"));

        // Tests only: the next write prunes again.
        public static void ResetForTests() => _pruned = false;
    }
}
