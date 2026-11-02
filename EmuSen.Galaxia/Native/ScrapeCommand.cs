using System;
using System.Collections.Generic;
using System.IO;
using System.Text;
using System.Text.Json;

namespace EmuSen.Galaxia.Native
{
    // A refusal of the scrape command, with the scraper's status and its words, already redacted.
    public sealed class ScrapeCommandException(long status, string words) : Exception(words)
    {
        public long Status { get; } = status;
    }

    // The scrape command as data through emusen_scraping_command: a window's choice, plan and run are the emusen.scrape module's - see EmuSen_LibraryTools.md §9.
    public static unsafe class ScrapeCommand
    {
        public const long Busy = -1164;
        public const long StoreInUse = -1167;

        private static readonly Lazy<(nint Command, nint LastError)> Exports = new(() =>
            (PlatformLibrary.Export("emusen_scraping_command"), PlatformLibrary.Export("emusen_scrape_last_error")));

        // Whether the command can be asked here: the platform library is loaded and has the call.
        public static bool Available => Exports.Value.Command != 0 && Exports.Value.LastError != 0;

        // Why the platform library is or is not in use, for the words of a window that cannot ask.
        public static string Report => PlatformLibrary.Report;

        // One request answered, by the length-query idiom; a refusal is a ScrapeCommandException.
        public static string Ask(string request)
        {
            GalaxiaScope.Refuse("A scrape run");
            if (!Available) throw new ScrapeCommandException(-256, $"{PlatformLibrary.FileName} is not in use ({PlatformLibrary.Report})");
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)Exports.Value.Command;
            byte[] text = Encoding.UTF8.GetBytes(request);
            byte[] buffer = new byte[16384];
            while (true)
            {
                long length;
                fixed (byte* r = text)
                fixed (byte* b = buffer)
                    length = call(r, (nuint)text.Length, b, (nuint)buffer.Length);
                if (length < 0) throw new ScrapeCommandException(length, LastError());
                if (length <= buffer.Length) return Encoding.UTF8.GetString(buffer, 0, (int)length);
                buffer = new byte[length];
            }
        }

        // A request of the words after `scrape`, the paths chosen by hand, and whether ls looks up each kind's source.
        public static string Request(IEnumerable<string> args, IEnumerable<string>? paths = null, bool sources = true, long? since = null)
        {
            using var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                w.WriteStartArray("args");
                foreach (string a in args) w.WriteStringValue(a);
                w.WriteEndArray();
                if (paths is not null)
                {
                    w.WriteStartArray("paths");
                    foreach (string p in paths) w.WriteStringValue(p);
                    w.WriteEndArray();
                }
                if (!sources) w.WriteBoolean("sources", false);
                if (since is long s) w.WriteNumber("since", s);
                w.WriteEndObject();
            }
            return Encoding.UTF8.GetString(stream.ToArray());
        }

        private static string LastError()
        {
            var last = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)Exports.Value.LastError;
            byte[] buffer = new byte[1024];
            long n;
            fixed (byte* b = buffer) n = last(b, (nuint)buffer.Length);
            if (n > buffer.Length)
            {
                buffer = new byte[n];
                fixed (byte* b = buffer) n = last(b, (nuint)buffer.Length);
            }
            return Encoding.UTF8.GetString(buffer, 0, (int)Math.Clamp(n, 0, buffer.Length));
        }
    }
}
