using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Systems
{
    // The packs' code in emusen-systems, reached through the platform library under the systems table's switch; each caller keeps its C# as failover - see EmuSen_Backend.md §13.
    public static unsafe class SystemCode
    {
        private const int Absent = -1030, BadArgument = -1033, Refused = -1034;

        // Every export the packs' code takes; SystemTable refuses a library without one, so the table and the code are chosen together.
        public static readonly IReadOnlyList<string> Exports = new[]
        {
            "emusen_systems_media", "emusen_systems_media_read", "emusen_systems_media_copy", "emusen_systems_media_delete", "emusen_systems_media_format",
        };

        // The library's code is in use: the switch chose the table.
        public static bool Active => SystemTable.Active;

        // Whether a text crosses to the library as it is; text that does not stays on the C#.
        public static bool Takes(string? text) => PlatformText.Crosses(text);

        public sealed record MediumModule(string Id, int Length, int Capacity, string Unit);

        private static readonly Lazy<IReadOnlyList<MediumModule>> Modules = new(() =>
        {
            var media = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)Export("emusen_systems_media");
            using var document = JsonDocument.Parse(Bytes((o, l) => media(o, l), 1024));
            return document.RootElement.EnumerateArray().Select(m => new MediumModule(m.GetProperty("id").GetString()!, m.GetProperty("length").GetInt32(), m.GetProperty("capacity").GetInt32(), m.GetProperty("unit").GetString()!)).ToArray();
        });

        // A medium's module facts as the library states them, whichever the switch chose.
        public static MediumModule Module(string medium) => Modules.Value.First(m => m.Id == medium);

        private static nint Export(string name)
        {
            nint export = PlatformLibrary.Export(name);
            return export != 0 ? export : throw new InvalidOperationException($"{PlatformLibrary.FileName} has no {name}");
        }

        private delegate long Call(byte* output, nuint length);

        // The length-query idiom, with a first buffer of the size the answer usually is.
        private static byte[] Bytes(Call call, int first)
        {
            byte[] buffer = new byte[first];
            long length;
            fixed (byte* b = buffer) length = call(b, (nuint)buffer.Length);
            if (length < 0) throw new SystemCodeFault((int)length, LastError());
            if (length == buffer.Length) return buffer;
            if (length < buffer.Length) return buffer[..(int)length];
            buffer = new byte[length];
            fixed (byte* b = buffer) length = call(b, (nuint)buffer.Length);
            if (length < 0) throw new SystemCodeFault((int)length, LastError());
            return buffer;
        }

        internal static string LastError()
        {
            nint export = PlatformLibrary.Export("emusen_platform_last_error");
            if (export == 0) return "no words";
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
            long n = call(null, 0);
            var bytes = new byte[Math.Max(0, n)];
            fixed (byte* b = bytes) call(b, (nuint)bytes.Length);
            return Encoding.UTF8.GetString(bytes);
        }

        // A status the library answered with, and its words.
        private sealed class SystemCodeFault(int status, string words) : Exception(words)
        {
            public int Status { get; } = status;
        }

        public static MediumListing MediaRead(string medium, ReadOnlySpan<byte> image)
        {
            byte[] id = Encoding.UTF8.GetBytes(medium);
            var read = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)Export("emusen_systems_media_read");
            string json;
            fixed (byte* i = id)
            fixed (byte* m = image)
            {
                byte* idp = i, imagep = m;
                nuint idn = (nuint)id.Length, imagen = (nuint)image.Length;
                json = Encoding.UTF8.GetString(Guard(() => Bytes((o, l) => read(idp, idn, imagep, imagen, o, l), 1024)));
            }
            using var document = JsonDocument.Parse(json);
            JsonElement r = document.RootElement;
            var saves = r.GetProperty("saves").EnumerateArray().Select(s => new SaveEntry(s.GetProperty("id").GetString()!, s.GetProperty("title").GetString()!, s.GetProperty("units").GetInt32(),
                s.GetProperty("game").ValueKind == JsonValueKind.Null ? null : s.GetProperty("game").GetString(), s.GetProperty("damaged").GetBoolean())).ToArray();
            return new MediumListing(r.GetProperty("formatted").GetBoolean(), saves, r.GetProperty("free").GetInt32(),
                r.GetProperty("problem").ValueKind == JsonValueKind.Null ? null : r.GetProperty("problem").GetString());
        }

        public static MediumEdit MediaCopy(string medium, ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to)
        {
            byte[] id = Encoding.UTF8.GetBytes(medium), name = Encoding.UTF8.GetBytes(save);
            var copy = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, long>)Export("emusen_systems_media_copy");
            fixed (byte* i = id)
            fixed (byte* f = from)
            fixed (byte* n = name)
            fixed (byte* t = to)
            {
                byte* ip = i, fp = f, np = n, tp = t;
                nuint il = (nuint)id.Length, fl = (nuint)from.Length, nl = (nuint)name.Length, tl = (nuint)to.Length;
                int size = Math.Max(1, to.Length);
                return Edit(() => Bytes((o, l) => copy(ip, il, fp, fl, np, nl, tp, tl, o, l), size));
            }
        }

        public static MediumEdit MediaDelete(string medium, ReadOnlySpan<byte> image, string save)
        {
            byte[] id = Encoding.UTF8.GetBytes(medium), name = Encoding.UTF8.GetBytes(save);
            var delete = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, byte*, nuint, long>)Export("emusen_systems_media_delete");
            fixed (byte* i = id)
            fixed (byte* m = image)
            fixed (byte* n = name)
            {
                byte* ip = i, mp = m, np = n;
                nuint il = (nuint)id.Length, ml = (nuint)image.Length, nl = (nuint)name.Length;
                int size = Math.Max(1, image.Length);
                return Edit(() => Bytes((o, l) => delete(ip, il, mp, ml, np, nl, o, l), size));
            }
        }

        public static byte[]? MediaFormat(string medium)
        {
            byte[] id = Encoding.UTF8.GetBytes(medium);
            var format = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)Export("emusen_systems_media_format");
            fixed (byte* i = id)
            {
                byte* ip = i;
                nuint il = (nuint)id.Length;
                try { return Bytes((o, l) => format(ip, il, o, l), Module(medium).Length); }
                catch (SystemCodeFault fault) when (fault.Status == Absent) { return null; }
            }
        }

        private static MediumEdit Edit(Func<byte[]> call)
        {
            try { return MediumEdit.Done(Guard(call)); }
            catch (SystemCodeFault fault) when (fault.Status == Refused) { return MediumEdit.Refused(fault.Message); }
        }

        // The C# module's exception where a damaged directory reached outside the image; any other status is a defect, said with its words.
        private static T Guard<T>(Func<T> call)
        {
            try { return call(); }
            catch (SystemCodeFault fault) when (fault.Status == BadArgument) { throw new ArgumentOutOfRangeException(); }
            catch (SystemCodeFault fault) when (fault.Status != Refused) { throw new InvalidOperationException($"{PlatformLibrary.FileName}: {fault.Message} ({fault.Status})"); }
        }
    }
}
