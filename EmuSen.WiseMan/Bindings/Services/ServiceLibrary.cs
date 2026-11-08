using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Native;

namespace EmuSen.WiseMan.Bindings.Services
{
    // A status the file services' library answered with, and its words.
    public sealed class ServiceFault(int status, string words) : Exception(words)
    {
        public const int Parse = -1025, Absent = -1030, NotFound = -1031, Access = -1032, BadArgument = -1033, Refused = -1034, NotSupported = -256, Io = -1024;

        public int Status { get; } = status;
    }

    // The file services, emusen-firmware, emusen-savemedia and emusen-cheats with emusen-net for the download, reached through the platform library; there is no C# twin since 2026-11-04 - see EmuSen_Backend.md §15, §16, §40.1.
    public static unsafe class ServiceLibrary
    {
        // Every export the services take; a library without one gives no service, and Report says why.
        public static readonly IReadOnlyList<string> Exports = new[]
        {
            "emusen_firmware_folders", "emusen_firmware_find", "emusen_firmware_locate",
            "emusen_cheats_parse", "emusen_cheats_write", "emusen_cheats_database_open", "emusen_cheats_database_free", "emusen_cheats_database_entries",
            "emusen_cheats_database_query", "emusen_cheats_prune_plan", "emusen_cheats_prune_apply",
            "emusen_net_download_start", "emusen_net_download_free", "emusen_net_download_progress", "emusen_net_download_wait",
            "emusen_net_download_cancel", "emusen_net_download_body", "emusen_net_download_failure",
        };

        private static readonly Lazy<(bool Ready, string Report)> Loaded = new(() =>
        {
            RetiredSwitch.Note("EMUSEN_SERVICES_NATIVE", "the file services");
            if (!PlatformLibrary.Available) return (false, PlatformLibrary.Report);
            if (Exports.FirstOrDefault(e => PlatformLibrary.Export(e) == 0) is { } missing) return (false, $"{PlatformLibrary.FileName} has no {missing}");
            return (true, $"{PlatformLibrary.FileName}, the file services");
        });

        // Whether the library gives the services; without it every call throws with Report's words.
        public static bool Ready => Loaded.Value.Ready;

        public static string Report => Loaded.Value.Report;

        // A text as it crosses, each lone surrogate as U+FFFD.
        public static string Mend(string text) => PlatformText.Mend(text);

        // A request written as one JSON object.
        public static byte[] Request(Action<Utf8JsonWriter> write)
        {
            var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                write(w);
                w.WriteEndObject();
            }
            return stream.ToArray();
        }

        public static void WriteList(Utf8JsonWriter w, string name, IEnumerable<string> items)
        {
            w.WriteStartArray(name);
            foreach (string item in items) w.WriteStringValue(Mend(item));
            w.WriteEndArray();
        }

        public static void WriteText(Utf8JsonWriter w, string name, string? text)
        {
            if (text is null) w.WriteNull(name); else w.WriteString(name, Mend(text));
        }

        private static nint Export(string name)
        {
            if (!Ready) throw new InvalidOperationException($"The file services are emusen_platform's, and it gives none: {Report}");
            nint export = PlatformLibrary.Export(name);
            return export != 0 ? export : throw new InvalidOperationException($"{PlatformLibrary.FileName} has no {name}");
        }

        // One call taking a request and answering by the length-query idiom, first with a buffer of the size the answer usually is; what the call queued is reported after it.
        public static byte[] Call(string name, byte[] request, int first)
        {
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)Export(name);
            try
            {
                byte[] buffer = new byte[Math.Max(1, first)];
                long length;
                fixed (byte* r = request)
                fixed (byte* b = buffer) length = call(r, (nuint)request.Length, b, (nuint)buffer.Length);
                if (length < 0) throw new ServiceFault((int)length, LastError());
                if (length <= buffer.Length) return length == buffer.Length ? buffer : buffer[..(int)length];
                buffer = new byte[length];
                fixed (byte* r = request)
                fixed (byte* b = buffer) length = call(r, (nuint)request.Length, b, (nuint)buffer.Length);
                if (length < 0) throw new ServiceFault((int)length, LastError());
                return buffer;
            }
            finally { Drain(); }
        }

        public static JsonDocument CallJson(string name, byte[] request, int first = 1024) => JsonDocument.Parse(Call(name, request, first));

        // One call answering with a status alone.
        public static int CallStatus(string name, byte[] request)
        {
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, int>)Export(name);
            try
            {
                int status;
                fixed (byte* r = request) status = call(r, (nuint)request.Length);
                if (status < 0) throw new ServiceFault(status, LastError());
                return status;
            }
            finally { Drain(); }
        }

        public static string LastError()
        {
            nint export = PlatformLibrary.Export("emusen_platform_last_error");
            if (export == 0) return "no words";
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
            long n = call(null, 0);
            var bytes = new byte[Math.Max(0, n)];
            fixed (byte* b = bytes) call(b, (nuint)bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }

        // What the library queued where the C# calls ConfigDiagnostics.Report, reported here in its order - see EmuSen_RustPlatform.md §3.5.
        private static void Drain()
        {
            nint export = PlatformLibrary.Export("emusen_platform_diagnostics");
            if (export == 0) return;
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
            long n = call(null, 0);
            if (n <= 0) return;
            var bytes = new byte[n];
            fixed (byte* b = bytes) n = call(b, (nuint)bytes.Length);
            foreach (string message in Encoding.UTF8.GetString(bytes, 0, (int)n).Split('\0', StringSplitOptions.RemoveEmptyEntries)) ConfigDiagnostics.Report(message);
        }

        public static string? Text(JsonElement e, string name) => e.GetProperty(name).ValueKind == JsonValueKind.Null ? null : e.GetProperty(name).GetString();

        public static string[] Words(JsonElement e, string name) => e.GetProperty(name).EnumerateArray().Select(w => w.GetString()!).ToArray();
    }
}
