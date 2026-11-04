using System;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Library
{
    // Every error a frontend shows or swallows, one plain-text file a day in the log folder - see EmuSen_Settings_Reference.md §4.70.
    public static class ErrorLog
    {
        public const string Prefix = "emusen_";
        public const int KeepDays = 14;
        public const long MaxBytes = 8L << 20;

        public const string WriteCheck = ".emusen-write-check";

        public static string FallbackWords(string configured, string root) =>
            $"the log folder {configured} cannot be made or written on this machine, so the log is kept in {root}";

        public static string? DirectoryOverride { get; set; }

        // Set by a frontend at start, so nothing it writes carries a credential to disk.
        public static Func<string, string>? Redactor { get; set; }

        public static string DefaultRoot => GalaxiaNative.Directory(GalaxiaDirectory.LogDefault)!;

        // The folder the per-game logs and crash reports share: the LogDirectory setting when it can be made, else ~/.config/EmuSen/Logs.
        public static string Root() => GalaxiaNative.LogRoot(DirectoryOverride);

        // Where a log goes when the setting names no usable folder: the default when it names none, the home's own Logs when it names one this machine cannot make or write.
        public static string FallbackRoot(string? configured) => string.IsNullOrWhiteSpace(configured) ? DefaultRoot : DataStore.Logs;

        // A settings file copied from another machine can name a folder this one cannot make or write.
        public static bool Usable(string? directory) =>
            GalaxiaNative.LogUsable(directory);

        public static string PathFor(DateTime day) => GalaxiaNative.LogPath(Root(), $"{day:yyyyMMdd}");

        public static string? Error(string area, string message, Exception? fault = null, string? context = null) =>
            Write("ERROR", area, message, fault, context);

        public static string? Warning(string area, string message, string? context = null) =>
            Write("WARN", area, message, null, context);

        private static string? Write(string level, string area, string message, Exception? fault, string? context) =>
            WriteNative(DateTime.Now, level, area, message, fault, context);

        // The time is formatted here, by this machine's calendar, and the redactor runs here; the library shapes the entry and owns the file; a lone surrogate is written as U+FFFD - see EmuSen_RustPlatform.md §11.4, §21.6.
        internal static string? WriteNative(DateTime now, string level, string area, string message, Exception? fault, string? context)
        {
            try
            {
                string stamp = $"{now:yyyy-MM-dd HH:mm:ss.fff}";
                string entry = GalaxiaNative.LogFormat(stamp, level, area, message, context, fault?.ToString());
                string text = Redactor is { } redact ? redact(entry) : entry;
                string path = GalaxiaNative.LogPath(GalaxiaNative.LogRoot(DirectoryOverride), $"{now:yyyyMMdd}");
                return GalaxiaNative.LogAppend(path, stamp, new DateTimeOffset(now).ToUnixTimeMilliseconds(), text) ? path : null;
            }
            catch
            {
                return null;
            }
        }

        // Tests only: the next write prunes again.
        public static void ResetForTests()
        {
            if (GalaxiaNative.Loaded) GalaxiaNative.LogReset();
        }
    }
}
