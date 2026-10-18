using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Crystal.Cheats;
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
            "emusen_systems_cheat_codecs", "emusen_systems_cheat_decode", "emusen_systems_cheat_encode",
            "emusen_systems_code", "emusen_systems_recognise", "emusen_systems_route", "emusen_systems_openvgdb", "emusen_systems_header",
            "emusen_systems_disassemble", "emusen_systems_board_name", "emusen_systems_view",
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

        // A codec as the library names it: its name, kind and space; and each system's pair of codecs by id.
        public sealed record CodecFacts(string Id, string Name, CheatCodeKind Kind, string? Space);

        public sealed record CodecPair(string System, string? Auto, string? Explicit);

        private static readonly Lazy<(IReadOnlyList<CodecFacts> Codecs, IReadOnlyList<CodecPair> Pairs)> CodecTable = new(() =>
        {
            var codecs = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)Export("emusen_systems_cheat_codecs");
            using var document = JsonDocument.Parse(Bytes((o, l) => codecs(o, l), 2048));
            JsonElement r = document.RootElement;
            return (r.GetProperty("codecs").EnumerateArray().Select(c => new CodecFacts(c.GetProperty("id").GetString()!, c.GetProperty("name").GetString()!,
                    Enum.Parse<CheatCodeKind>(c.GetProperty("kind").GetString()!), c.GetProperty("space").GetString())).ToArray(),
                r.GetProperty("systems").EnumerateArray().Select(p => new CodecPair(p.GetProperty("id").GetString()!, p.GetProperty("auto").GetString(), p.GetProperty("explicit").GetString())).ToArray());
        });

        public static IReadOnlyList<CodecFacts> Codecs => CodecTable.Value.Codecs;

        public static IReadOnlyList<CodecPair> CodecPairs => CodecTable.Value.Pairs;

        // A pack's codec: the library's under the codec's id, the C# codec as its failover, or the C# alone under the switch's 0.
        public static ICheatCodeCodec Codec(string id, ICheatCodeCodec managed) => Active ? new NativeCheatCodec(id, managed) : managed;

        // A code's answers from the library's codec, as JSON.
        internal static JsonDocument CheatDecode(string codec, string code)
        {
            byte[] id = Encoding.UTF8.GetBytes(codec), text = Encoding.UTF8.GetBytes(code);
            var decode = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)Export("emusen_systems_cheat_decode");
            fixed (byte* i = id)
            fixed (byte* t = text)
            {
                byte* ip = i, tp = t;
                nuint il = (nuint)id.Length, tl = (nuint)text.Length;
                return JsonDocument.Parse(Guard(() => Bytes((o, l) => decode(ip, il, tp, tl, o, l), 1024)));
            }
        }

        // A Game Genie code for a patch, from the library's encoder of the codec named; the SNES's and the Genesis's.
        public static string CheatEncode(string codec, int address, uint value)
        {
            byte[] id = Encoding.UTF8.GetBytes(codec);
            var encode = (delegate* unmanaged[Cdecl]<byte*, nuint, int, uint, byte*, nuint, long>)Export("emusen_systems_cheat_encode");
            fixed (byte* i = id)
            {
                byte* ip = i;
                nuint il = (nuint)id.Length;
                return Encoding.UTF8.GetString(Guard(() => Bytes((o, l) => encode(ip, il, address, value, o, l), 16)));
            }
        }

        // Which systems have code of each kind, as the library states it.
        public sealed record SystemCodeFacts(string Id, bool Recognises, bool OpenVgdb, bool Header);

        private static readonly Lazy<IReadOnlyList<SystemCodeFacts>> CodeTable = new(() =>
        {
            var code = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)Export("emusen_systems_code");
            using var document = JsonDocument.Parse(Bytes((o, l) => code(o, l), 1024));
            return document.RootElement.EnumerateArray().Select(c => new SystemCodeFacts(c.GetProperty("id").GetString()!, c.GetProperty("recognises").GetBoolean(),
                c.GetProperty("openvgdb").GetBoolean(), c.GetProperty("header").GetBoolean())).ToArray();
        });

        public static IReadOnlyList<SystemCodeFacts> Code => CodeTable.Value;

        private static Recognition? RecognitionOf(JsonElement r) => r.ValueKind == JsonValueKind.Null ? null
            : new Recognition(r.GetProperty("system").GetString()!, r.GetProperty("serial").GetString(), r.GetProperty("region").GetString());

        // A system's recogniser over a probe, reading as many of the file's first bytes as the recogniser asks for, as the C# recognisers read them.
        public static Recognition? Recognise(string system, IMediaProbe probe)
        {
            byte[] id = Encoding.UTF8.GetBytes(system), head = Array.Empty<byte>();
            var recognise = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, ulong, ulong*, byte*, nuint, long>)Export("emusen_systems_recognise");
            ulong asked = 0;
            while (true)
            {
                ulong need = 0;
                byte[] answer;
                fixed (byte* i = id)
                fixed (byte* h = head)
                {
                    byte* ip = i, hp = h;
                    nuint il = (nuint)id.Length, hl = (nuint)head.Length;
                    ulong a = asked;
                    ulong* n = &need;
                    answer = Guard(() => Bytes((o, l) => recognise(ip, il, hp, hl, a, n, o, l), 256));
                }
                if (need == 0)
                {
                    using var document = JsonDocument.Parse(answer);
                    return RecognitionOf(document.RootElement);
                }
                if (need <= asked || need > int.MaxValue) throw new InvalidOperationException($"{PlatformLibrary.FileName}: the recogniser of {system} asked for {need} bytes after {asked}");
                head = probe.Read((int)need);
                asked = need;
            }
        }

        // The decision among candidates, each claim by the candidate's place; null where a text cannot cross, for the C# to decide.
        public static Routed? Route(string fileName, string? folder, IReadOnlyList<SystemEntry> candidates, IReadOnlyList<(int At, Recognition Claim)> claims)
        {
            if (!Takes(fileName) || !Takes(folder) || candidates.Any(c => !Takes(c.Id) || !Takes(c.Name) || !Takes(c.EsdeSystem))) return null;
            var stream = new System.IO.MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                w.WriteString("file", fileName);
                if (folder is null) w.WriteNull("folder"); else w.WriteString("folder", folder);
                w.WriteStartArray("candidates");
                foreach (SystemEntry c in candidates) { w.WriteStartObject(); w.WriteString("id", c.Id); w.WriteString("name", c.Name); w.WriteString("esde", c.EsdeSystem); w.WriteEndObject(); }
                w.WriteEndArray();
                w.WriteStartArray("claims");
                foreach (var (at, claim) in claims)
                {
                    w.WriteStartObject();
                    w.WriteNumber("at", at);
                    w.WriteString("system", claim.SystemId);
                    if (claim.Serial is null) w.WriteNull("serial"); else w.WriteString("serial", claim.Serial);
                    if (claim.Region is null) w.WriteNull("region"); else w.WriteString("region", claim.Region);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                w.WriteEndObject();
            }
            byte[] request = stream.ToArray();
            var route = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, long>)Export("emusen_systems_route");
            byte[] answer;
            fixed (byte* r = request)
            {
                byte* rp = r;
                nuint rl = (nuint)request.Length;
                answer = Guard(() => Bytes((o, l) => route(rp, rl, o, l), 1024));
            }
            using var document = JsonDocument.Parse(answer);
            JsonElement d = document.RootElement;
            string? system = d.GetProperty("system").GetString();
            return new Routed(system is null ? null : candidates.First(c => c.Id == system), RecognitionOf(d.GetProperty("recognition")), d.GetProperty("reason").GetString());
        }

        // The bytes OpenVGDB hashed for the system's file; the file itself where they are its own.
        public static byte[] OpenVgdbBytes(string system, byte[] file)
        {
            byte[] id = Encoding.UTF8.GetBytes(system);
            var openvgdb = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)Export("emusen_systems_openvgdb");
            fixed (byte* i = id)
            fixed (byte* f = file)
            {
                byte* ip = i, fp = f;
                nuint il = (nuint)id.Length, fl = (nuint)file.Length;
                int size = Math.Max(1, file.Length);
                return Guard(() =>
                {
                    try { return Bytes((o, l) => openvgdb(ip, il, fp, fl, o, l), size); }
                    catch (SystemCodeFault fault) when (fault.Status == Absent) { return file; }
                });
            }
        }

        // A header reader's answer from the image's first 4 KiB and its length.
        private static JsonDocument Header(string system, byte[] image)
        {
            byte[] id = Encoding.UTF8.GetBytes(system);
            var header = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, ulong, byte*, nuint, long>)Export("emusen_systems_header");
            fixed (byte* i = id)
            fixed (byte* h = image)
            {
                byte* ip = i, hp = h;
                nuint il = (nuint)id.Length, hl = (nuint)Math.Min(image.Length, 4096);
                ulong length = (ulong)image.Length;
                return JsonDocument.Parse(Guard(() => Bytes((o, l) => header(ip, il, hp, hl, length, o, l), 512)));
            }
        }

        // A refusal as the C# reader threw it.
        private static void HeaderRefused(JsonElement r)
        {
            if (!r.TryGetProperty("refused", out JsonElement kind)) return;
            string words = r.GetProperty("words").GetString()!;
            throw kind.GetString() == "unsupported" ? new NotSupportedException(words) : new System.IO.InvalidDataException(words);
        }

        public static Nes.NesHeader NesHeader(byte[] image)
        {
            using JsonDocument document = Header("nes", image);
            JsonElement r = document.RootElement;
            HeaderRefused(r);
            return new Nes.NesHeader(r.GetProperty("mapper").GetInt32(), r.GetProperty("prg_banks").GetInt32(), r.GetProperty("chr_banks").GetInt32(), r.GetProperty("battery").GetBoolean(),
                r.GetProperty("trainer").GetBoolean(), r.GetProperty("nes20").GetBoolean(), (Nes.NesHeaderTrust)r.GetProperty("trust").GetInt32(), (Nes.NesMirroring)r.GetProperty("mirroring").GetInt32());
        }

        public static GameBoy.GbCartridge GbHeader(byte[] image)
        {
            using JsonDocument document = Header("gb", image);
            JsonElement r = document.RootElement;
            HeaderRefused(r);
            return new GameBoy.GbCartridge(r.GetProperty("type").GetByte(), r.GetProperty("board").GetString()!, r.GetProperty("rom_banks").GetInt32(), r.GetProperty("ram_length").GetInt32(),
                r.GetProperty("battery").GetBoolean(), r.GetProperty("timer").GetBoolean(), r.GetProperty("rumble").GetBoolean(), r.GetProperty("cgb_flag").GetByte());
        }

        // One instruction as a disassembler of the library's gives it: its length, mnemonic, operands and the address it names.
        public sealed record Instruction(int Length, string Mnemonic, string Operands, Debugger.StaticReferenceKind? Kind, long Target);

        public static Instruction Disassemble(string processor, ReadOnlySpan<byte> code, int pc, int operand)
        {
            byte[] id = Encoding.UTF8.GetBytes(processor);
            var disassemble = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, int, int, byte*, nuint, long>)Export("emusen_systems_disassemble");
            byte[] answer;
            fixed (byte* i = id)
            fixed (byte* c = code)
            {
                byte* ip = i, cp = c;
                nuint il = (nuint)id.Length, cl = (nuint)code.Length;
                answer = Guard(() => Bytes((o, l) => disassemble(ip, il, cp, cl, pc, operand, o, l), 256));
            }
            using var document = JsonDocument.Parse(answer);
            JsonElement r = document.RootElement, reference = r.GetProperty("reference");
            return new Instruction(r.GetProperty("length").GetInt32(), r.GetProperty("mnemonic").GetString()!, r.GetProperty("operands").GetString()!,
                reference.ValueKind == JsonValueKind.Null ? null : Enum.Parse<Debugger.StaticReferenceKind>(reference.GetProperty("kind").GetString()!),
                reference.ValueKind == JsonValueKind.Null ? 0 : reference.GetProperty("target").GetInt64());
        }

        // The board an iNES mapper number means, as the NES pack names it; null for a number it names none for.
        public static string? BoardName(int mapper)
        {
            var board = (delegate* unmanaged[Cdecl]<int, byte*, nuint, long>)Export("emusen_systems_board_name");
            return Guard(() =>
            {
                try { return Encoding.UTF8.GetString(Bytes((o, l) => board(mapper, o, l), 64)); }
                catch (SystemCodeFault fault) when (fault.Status == Absent) { return null; }
            });
        }

        // A pack's views: the library's fronting the C# views, or the C# alone under the switch's 0.
        public static ISystemDebugViews Views(string system, ISystemDebugViews managed) => Active ? new NativeDebugViews(system, managed) : managed;

        // One view's answer, reading memory in the rounds the view asks for; null where the system has no such view.
        internal static (JsonDocument Answer, byte[] Payload)? View(byte[] request, IReadOnlyList<Debugger.IDebugMemorySpace> spaces)
        {
            var view = (delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, byte*, nuint, long>)Export("emusen_systems_view");
            var tape = new List<byte>();
            for (int round = 0; ; round++)
            {
                byte[] taped = tape.ToArray();
                byte[]? envelope;
                fixed (byte* r = request)
                fixed (byte* t = taped)
                {
                    byte* rp = r, tp = t;
                    nuint rl = (nuint)request.Length, tl = (nuint)taped.Length;
                    envelope = Guard(() =>
                    {
                        try { return Bytes((o, l) => view(rp, rl, tp, tl, o, l), 4096); }
                        catch (SystemCodeFault fault) when (fault.Status == Absent) { return null; }
                    });
                }
                if (envelope is null) return null;
                int length = BitConverter.ToInt32(envelope, 0);
                var answer = JsonDocument.Parse(envelope.AsMemory(4, length));
                if (!answer.RootElement.TryGetProperty("need", out JsonElement need)) return (answer, envelope[(4 + length)..]);
                if (round >= 64) throw new InvalidOperationException($"{PlatformLibrary.FileName}: a view asked for memory in more than 64 rounds");
                foreach (JsonElement run in need.EnumerateArray())
                {
                    string name = run.GetProperty("space").GetString()!;
                    Debugger.IDebugMemorySpace space = spaces.First(s => s.Name == name);
                    int at = run.GetProperty("address").GetInt32(), count = run.GetProperty("length").GetInt32();
                    for (int a = 0; a < count; a++) tape.Add(space.Read(at + a));
                }
                answer.Dispose();
            }
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
