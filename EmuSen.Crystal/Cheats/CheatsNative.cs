using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Services;
using EmuSen.Crystal.Systems;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Cheats
{
    // The cheat files and the database's calls into emusen-cheats, under the file services' switch; what cannot cross stays on the C# - see EmuSen_Backend.md §14.3.
    public static unsafe class CheatsNative
    {
        public static bool Active => ServiceLibrary.Active;

        // A codec the library names by id, or null; a C# codec of no id keeps the read on the C#.
        private static bool Named(ICheatCodeCodec? codec, out string? id)
        {
            id = codec is NativeCheatCodec native ? native.Id : null;
            return codec is null || id is not null;
        }

        public static bool TakesParse(string text, ICheatCodeCodec? codec, string space, ICheatCodeCodec? patch) =>
            SystemCode.Active && Named(codec, out _) && Named(patch, out _) && PlatformText.Crosses(text) && PlatformText.Crosses(space);

        public static ChtParseResult Parse(string text, ICheatCodeCodec? codec, string space, ICheatCodeCodec? patch)
        {
            Named(codec, out string? codecId);
            Named(patch, out string? patchId);
            byte[] request = ServiceLibrary.Request(w => { w.WriteString("text", text); ServiceLibrary.WriteText(w, "codec", codecId); w.WriteString("space", space); ServiceLibrary.WriteText(w, "patch", patchId); });
            JsonDocument d;
            try { d = ServiceLibrary.CallJson("emusen_cheats_parse", request, Math.Max(1024, text.Length * 2)); }
            catch (ServiceFault fault) when (fault.Status == ServiceFault.BadArgument) { throw new ArgumentOutOfRangeException(); }
            using (d)
            {
                var cheats = d.RootElement.GetProperty("cheats").EnumerateArray().Select(c => new ChtCheat
                {
                    Description = c.GetProperty("desc").GetString()!,
                    Enabled = c.GetProperty("enabled").GetBoolean(),
                    Kind = Enum.Parse<CheatKind>(c.GetProperty("kind").GetString()!),
                    Writes = c.GetProperty("writes").EnumerateArray().Select(ReadWrite).ToArray(),
                }).ToList();
                return new ChtParseResult { Cheats = cheats, Skipped = d.RootElement.GetProperty("skipped").GetInt32() };
            }
        }

        public static CheatWrite ReadWrite(JsonElement w) => new()
        {
            Space = w.GetProperty("space").GetString()!,
            Address = w.GetProperty("address").GetInt32(),
            Value = w.GetProperty("value").GetUInt32(),
            Width = w.GetProperty("width").GetInt32(),
            Type = (CheatWriteType)w.GetProperty("type").GetInt32(),
            BitPosition = w.GetProperty("bit").ValueKind == JsonValueKind.Null ? null : w.GetProperty("bit").GetInt32(),
            BigEndian = w.GetProperty("big_endian").GetBoolean(),
            RepeatCount = w.GetProperty("repeat").GetInt32(),
            RepeatAddAddress = w.GetProperty("repeat_address").GetInt32(),
            RepeatAddValue = w.GetProperty("repeat_value").GetUInt32(),
        };

        private static void Write(Utf8JsonWriter j, CheatWrite w)
        {
            j.WriteStartObject();
            j.WriteString("space", w.Space ?? "");
            j.WriteNumber("address", w.Address);
            j.WriteNumber("value", w.Value);
            j.WriteNumber("width", w.Width);
            j.WriteNumber("type", (int)w.Type);
            if (w.BitPosition is { } bit) j.WriteNumber("bit", bit); else j.WriteNull("bit");
            j.WriteBoolean("big_endian", w.BigEndian);
            j.WriteNumber("repeat", w.RepeatCount);
            j.WriteNumber("repeat_address", w.RepeatAddAddress);
            j.WriteNumber("repeat_value", w.RepeatAddValue);
            j.WriteEndObject();
        }

        public static bool TakesWrite(IReadOnlyList<ChtCheat> cheats) =>
            cheats.All(c => c.Description is not null && c.Writes is not null && PlatformText.Crosses(c.Description) && c.Writes.All(w => PlatformText.Crosses(w.Space)));

        public static string Write(IReadOnlyList<ChtCheat> cheats)
        {
            byte[] request = ServiceLibrary.Request(j =>
            {
                j.WriteStartArray("cheats");
                foreach (ChtCheat c in cheats)
                {
                    j.WriteStartObject();
                    j.WriteString("desc", c.Description);
                    j.WriteBoolean("enabled", c.Enabled);
                    j.WriteString("kind", c.Kind.ToString());
                    j.WriteStartArray("writes");
                    foreach (CheatWrite w in c.Writes) Write(j, w);
                    j.WriteEndArray();
                    j.WriteEndObject();
                }
                j.WriteEndArray();
            });
            return Encoding.UTF8.GetString(ServiceLibrary.Call("emusen_cheats_write", request, 512 + cheats.Count * 512));
        }

        // A scanned database in the library, freed when the C# lets go of it.
        public sealed class Database : SafeHandle
        {
            private Database() : base(0, ownsHandle: true) { }

            public override bool IsInvalid => handle == 0;

            protected override bool ReleaseHandle()
            {
                ((delegate* unmanaged[Cdecl]<nint, void>)PlatformLibrary.Export("emusen_cheats_database_free"))(handle);
                return true;
            }

            public static Database Open(string directory)
            {
                byte[] d = Encoding.UTF8.GetBytes(directory);
                var db = new Database();
                fixed (byte* p = d) db.SetHandle(((delegate* unmanaged[Cdecl]<byte*, nuint, nint>)PlatformLibrary.Export("emusen_cheats_database_open"))(p, (nuint)d.Length));
                return db;
            }

            private byte[] Ask(string name, byte[]? request, int first)
            {
                bool added = false;
                DangerousAddRef(ref added);
                try
                {
                    nint h = handle;
                    if (request is null) return Length((o, l) => ((delegate* unmanaged[Cdecl]<nint, byte*, nuint, long>)PlatformLibrary.Export(name))(h, o, l), first);
                    fixed (byte* r = request)
                    {
                        byte* rp = r;
                        nuint rl = (nuint)request.Length;
                        return Length((o, l) => ((delegate* unmanaged[Cdecl]<nint, byte*, nuint, byte*, nuint, long>)PlatformLibrary.Export(name))(h, rp, rl, o, l), first);
                    }
                }
                finally { if (added) DangerousRelease(); }
            }

            private delegate long Query(byte* output, nuint length);

            private static byte[] Length(Query query, int first)
            {
                byte[] buffer = new byte[first];
                long n;
                fixed (byte* b = buffer) n = query(b, (nuint)buffer.Length);
                if (n < 0) throw new ServiceFault((int)n, ServiceLibrary.LastError());
                if (n <= buffer.Length) return buffer[..(int)n];
                buffer = new byte[n];
                fixed (byte* b = buffer) n = query(b, (nuint)buffer.Length);
                if (n < 0) throw new ServiceFault((int)n, ServiceLibrary.LastError());
                return buffer;
            }

            public IReadOnlyList<CheatDatabaseEntry> Entries()
            {
                using JsonDocument d = JsonDocument.Parse(Ask("emusen_cheats_database_entries", null, 1 << 16));
                return d.RootElement.EnumerateArray().Select(e => new CheatDatabaseEntry { System = e[0].GetString()!, Game = e[1].GetString()!, Path = e[2].GetString()! }).ToArray();
            }

            public IReadOnlyList<(string System, int Count)> Systems()
            {
                using JsonDocument d = JsonDocument.Parse(Ask("emusen_cheats_database_query", ServiceLibrary.Request(w => w.WriteString("op", "systems")), 4096));
                return d.RootElement.EnumerateArray().Select(e => (e[0].GetString()!, e[1].GetInt32())).ToList();
            }

            public IReadOnlyList<int> Games(string system, string? filter)
            {
                using JsonDocument d = JsonDocument.Parse(Ask("emusen_cheats_database_query", ServiceLibrary.Request(w => { w.WriteString("op", "games"); w.WriteString("system", system); ServiceLibrary.WriteText(w, "filter", filter); }), 4096));
                return d.RootElement.EnumerateArray().Select(e => e.GetInt32()).ToArray();
            }

            public IReadOnlyList<int> Find(string text, int limit)
            {
                using JsonDocument d = JsonDocument.Parse(Ask("emusen_cheats_database_query", ServiceLibrary.Request(w => { w.WriteString("op", "find"); w.WriteString("text", text); w.WriteNumber("limit", limit); }), 1024));
                return d.RootElement.EnumerateArray().Select(e => e.GetInt32()).ToArray();
            }

            public CheatPrunePlan Plan(IReadOnlyCollection<string> keep)
            {
                using JsonDocument d = JsonDocument.Parse(Ask("emusen_cheats_prune_plan", ServiceLibrary.Request(w => ServiceLibrary.WriteList(w, "keep", keep)), 4096));
                JsonElement r = d.RootElement;
                string? reason = ServiceLibrary.Text(r, "reason");
                return new CheatPrunePlan
                {
                    Removing = r.GetProperty("removing").EnumerateArray().Select(p => new PrunedSystem { System = p.GetProperty("system").GetString()!, Path = p.GetProperty("path").GetString()!, Files = p.GetProperty("files").GetInt32(), Bytes = p.GetProperty("bytes").GetInt64() }).ToList(),
                    Keeping = ServiceLibrary.Words(r, "keeping"),
                    CanApply = reason is null,
                    Reason = reason,
                };
            }
        }

        public static (int Removed, IReadOnlyList<string> Failed) Apply(string directory, CheatPrunePlan plan)
        {
            byte[] request = ServiceLibrary.Request(w =>
            {
                w.WriteString("directory", directory);
                w.WriteStartArray("removing");
                foreach (PrunedSystem s in plan.Removing) { w.WriteStartObject(); w.WriteString("system", s.System); w.WriteString("path", s.Path); w.WriteEndObject(); }
                w.WriteEndArray();
            });
            using JsonDocument d = ServiceLibrary.CallJson("emusen_cheats_prune_apply", request, 1024);
            return (d.RootElement.GetProperty("removed").GetInt32(), ServiceLibrary.Words(d.RootElement, "failed"));
        }

        // An archive installed by the library; null where its form is the C#'s, nothing having been written.
        public static CheatDatabaseInstallResult? Install(byte[] zip, string target)
        {
            byte[] t = Encoding.UTF8.GetBytes(target);
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)PlatformLibrary.Export("emusen_cheats_install");
            byte[] buffer = new byte[256];
            long n;
            fixed (byte* z = zip)
            fixed (byte* tp = t)
            fixed (byte* b = buffer) n = call(z, (nuint)zip.Length, tp, (nuint)t.Length, b, (nuint)buffer.Length);
            if (n == ServiceFault.NotSupported) return null;
            if (n == ServiceFault.Access) throw new UnauthorizedAccessException(ServiceLibrary.LastError());
            if (n == ServiceFault.NotFound) throw new DirectoryNotFoundException(ServiceLibrary.LastError());
            if (n < 0) throw new IOException(ServiceLibrary.LastError());
            using JsonDocument d = JsonDocument.Parse(buffer.AsMemory(0, (int)n));
            return new CheatDatabaseInstallResult { Installed = d.RootElement.GetProperty("installed").GetInt32(), Skipped = d.RootElement.GetProperty("skipped").GetInt32() };
        }
    }
}
