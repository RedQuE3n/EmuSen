using System.IO.Compression;
using System.Text;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Services;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The cheat database's installer reads every zip form .NET's reader reads, and refuses the rest in .NET's words: each form made here by hand, installed by emusen-cheats and by .NET's own reader, outcome and files compared - see EmuSen_Backend.md §40.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class CheatArchiveFormsTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenArchiveForms_" + Guid.NewGuid().ToString("N"));

        public CheatArchiveFormsTests()
        {
            Assert.True(ServiceLibrary.Ready, ServiceLibrary.Report);
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // One entry as a zip holds it: its stored bytes are made from Data by Method unless Stored gives them.
        public sealed record Entry(string Name, byte[] Data)
        {
            public ushort Method { get; init; } = 8;
            public ushort Flags { get; init; }
            public byte[]? Stored { get; init; }
            public bool Zip64Extra { get; init; }
            public bool Descriptor { get; init; }
            public bool DescriptorSignature { get; init; } = true;
            public ushort Disk { get; init; }
            public uint Time { get; init; } = (46u << 25) | (7u << 21) | (14u << 16) | (12u << 11);
            public uint External { get; init; } = 0x81A40000u;
            public byte[]? RawName { get; init; }
        }

        // How the archive's end is written: plainly, as Zip64, or with fields a reader must refuse.
        public sealed record Ending
        {
            public bool Zip64 { get; init; }
            public ushort Disk { get; init; }
            public ushort CentralDisk { get; init; }
            public int? Count { get; init; }
            public int? CountOnDisk { get; init; }
            public long? CentralOffset { get; init; }
            public int Comment { get; init; }
        }

        private static byte[] Deflate(byte[] data)
        {
            var o = new MemoryStream();
            using (var d = new DeflateStream(o, CompressionLevel.Optimal, leaveOpen: true)) d.Write(data);
            return o.ToArray();
        }

        public static byte[] Zip(IReadOnlyList<Entry> entries, Ending? ending = null)
        {
            ending ??= new Ending();
            var o = new MemoryStream();
            var w = new BinaryWriter(o);
            var central = new MemoryStream();
            var c = new BinaryWriter(central);
            foreach (Entry e in entries)
            {
                byte[] stored = e.Stored ?? (e.Method == 8 ? Deflate(e.Data) : e.Data);
                byte[] name = e.RawName ?? Encoding.UTF8.GetBytes(e.Name);
                uint crc = Crc32(e.Data);
                long local = o.Position;
                byte[] extra = e.Zip64Extra ? Zip64Extra(e.Data.Length, stored.Length, local) : Array.Empty<byte>();
                ushort flags = (ushort)(e.Flags | (e.Descriptor ? 8 : 0));
                w.Write(0x04034B50u); w.Write((ushort)(e.Zip64Extra ? 45 : 20)); w.Write(flags); w.Write(e.Method); w.Write(e.Time);
                w.Write(e.Descriptor ? 0u : crc);
                w.Write(e.Descriptor ? 0u : e.Zip64Extra ? 0xFFFFFFFFu : (uint)stored.Length);
                w.Write(e.Descriptor ? 0u : e.Zip64Extra ? 0xFFFFFFFFu : (uint)e.Data.Length);
                w.Write((ushort)name.Length); w.Write((ushort)(e.Zip64Extra ? Zip64Extra(e.Data.Length, stored.Length, null).Length : 0));
                w.Write(name);
                if (e.Zip64Extra) w.Write(Zip64Extra(e.Data.Length, stored.Length, null));
                w.Write(stored);
                if (e.Descriptor)
                {
                    if (e.DescriptorSignature) w.Write(0x08074B50u);
                    w.Write(crc);
                    if (e.Zip64Extra) { w.Write((ulong)stored.Length); w.Write((ulong)e.Data.Length); }
                    else { w.Write((uint)stored.Length); w.Write((uint)e.Data.Length); }
                }
                c.Write(0x02014B50u); c.Write((ushort)(3 << 8 | 45)); c.Write((ushort)(e.Zip64Extra ? 45 : 20)); c.Write(flags); c.Write(e.Method); c.Write(e.Time); c.Write(crc);
                c.Write(e.Zip64Extra ? 0xFFFFFFFFu : (uint)stored.Length);
                c.Write(e.Zip64Extra ? 0xFFFFFFFFu : (uint)e.Data.Length);
                c.Write((ushort)name.Length); c.Write((ushort)extra.Length); c.Write((ushort)0);
                c.Write(e.Disk); c.Write((ushort)0); c.Write(e.External);
                c.Write(e.Zip64Extra ? 0xFFFFFFFFu : (uint)local);
                c.Write(name);
                if (e.Zip64Extra) c.Write(extra);
            }
            long start = o.Position;
            w.Write(central.ToArray());
            long size = o.Position - start;
            int count = ending.Count ?? entries.Count;
            if (ending.Zip64)
            {
                long record = o.Position;
                w.Write(0x06064B50u); w.Write(44UL); w.Write((ushort)45); w.Write((ushort)45); w.Write((uint)ending.Disk); w.Write((uint)ending.CentralDisk);
                w.Write((ulong)(ending.CountOnDisk ?? count)); w.Write((ulong)count); w.Write((ulong)size); w.Write((ulong)(ending.CentralOffset ?? start));
                w.Write(0x07064B50u); w.Write((uint)ending.Disk); w.Write((ulong)record); w.Write(1u);
                w.Write(0x06054B50u); w.Write((ushort)0xFFFF); w.Write((ushort)0xFFFF); w.Write((ushort)0xFFFF); w.Write((ushort)0xFFFF); w.Write(0xFFFFFFFFu); w.Write(0xFFFFFFFFu);
            }
            else
            {
                w.Write(0x06054B50u); w.Write(ending.Disk); w.Write(ending.CentralDisk); w.Write((ushort)(ending.CountOnDisk ?? count)); w.Write((ushort)count);
                w.Write((uint)size); w.Write((uint)(ending.CentralOffset ?? start));
            }
            w.Write((ushort)ending.Comment);
            w.Write(new byte[ending.Comment]);
            return o.ToArray();
        }

        // A Zip64 extra field holding the two sizes, and the local header's offset where one is given.
        private static byte[] Zip64Extra(long length, long compressed, long? offset)
        {
            var o = new MemoryStream();
            var w = new BinaryWriter(o);
            w.Write((ushort)1); w.Write((ushort)(offset is null ? 16 : 24)); w.Write((ulong)length); w.Write((ulong)compressed);
            if (offset is { } at) w.Write((ulong)at);
            return o.ToArray();
        }

        private static uint Crc32(byte[] data)
        {
            uint crc = 0xFFFFFFFF;
            foreach (byte b in data)
            {
                crc ^= b;
                for (int k = 0; k < 8; k++) crc = (crc & 1) != 0 ? crc >> 1 ^ 0xEDB88320 : crc >> 1;
            }
            return ~crc;
        }

        // A Deflate64 stream of one fixed-code block: literals, then matches Deflate64 alone can say, a length past 258 and a distance past 32 KiB.
        public static (byte[] Data, byte[] Stored) Deflate64()
        {
            var r = new Random(64);
            var data = new List<byte>();
            var bits = new BitWriter();
            bits.Put(1, 1);
            bits.Put(1, 2);
            for (int i = 0; i < 40_000; i++)
            {
                byte b = (byte)r.Next(256);
                data.Add(b);
                bits.Literal(b);
            }
            void Match(int length, int distance)
            {
                for (int i = 0; i < length; i++) data.Add(data[^distance]);
                bits.Code(0xC5, 8);
                bits.Put((uint)(length - 3), 16);
                (int code, int @base, int extra) = distance > 49152 ? (31, 49153, 14) : distance > 32768 ? (30, 32769, 14) : (0, 1, 0);
                bits.Code((uint)code, 5);
                bits.Put((uint)(distance - @base), extra);
            }
            Match(1000, 40);
            Match(300, 33_000);
            Match(60_000, 39_999);
            bits.Code(0, 7);
            return (data.ToArray(), bits.Bytes());
        }

        private sealed class BitWriter
        {
            private readonly List<byte> _bytes = new();
            private int _bit;

            public void Put(uint value, int count)
            {
                for (int i = 0; i < count; i++)
                {
                    if (_bit == 0) _bytes.Add(0);
                    if ((value >> i & 1) != 0) _bytes[^1] |= (byte)(1 << _bit);
                    _bit = (_bit + 1) & 7;
                }
            }

            // A Huffman code, written from its top bit.
            public void Code(uint code, int length)
            {
                for (int i = length - 1; i >= 0; i--) Put(code >> i & 1, 1);
            }

            public void Literal(byte b)
            {
                if (b < 144) Code(0x30u + b, 8);
                else Code(0x190u + (uint)(b - 144), 9);
            }

            public byte[] Bytes() => _bytes.ToArray();
        }

        private static readonly byte[] Text = Encoding.ASCII.GetBytes("cheats = 1\ncheat0_desc = \"Lives\"\ncheat0_code = \"7E0DBE:63\"\n");

        // Each form, by name; every form .NET reads is read, and every refusal is .NET's.
        public static IEnumerable<(string Form, byte[] Zip)> Forms()
        {
            Entry Plain(string name, ushort method = 8) => new(name, Text) { Method = method };
            yield return ("stored and deflate", Zip([Plain("cht/A/One.cht", 0), Plain("cht/A/Two.cht")]));
            yield return ("data descriptors with their signature", Zip([Plain("cht/A/One.cht") with { Descriptor = true }, Plain("cht/A/Two.cht", 0) with { Descriptor = true }]));
            yield return ("data descriptors without their signature", Zip([Plain("cht/A/One.cht") with { Descriptor = true, DescriptorSignature = false }, Plain("cht/A/Two.cht") with { Descriptor = true, DescriptorSignature = false }]));
            yield return ("Zip64 extra fields", Zip([Plain("cht/A/One.cht") with { Zip64Extra = true }, Plain("cht/A/Two.cht", 0) with { Zip64Extra = true }]));
            yield return ("a Zip64 end", Zip([Plain("cht/A/One.cht"), Plain("cht/A/Two.cht", 0)], new Ending { Zip64 = true }));
            yield return ("Zip64 throughout, with descriptors", Zip([Plain("cht/A/One.cht") with { Zip64Extra = true, Descriptor = true }], new Ending { Zip64 = true }));
            var (data, stored) = Deflate64();
            yield return ("Deflate64", Zip([new Entry("cht/A/Big.cht", data) { Method = 9, Stored = stored }, Plain("cht/A/Two.cht")]));
            foreach (ushort method in new ushort[] { 1, 6, 12, 14, 93, 95, 98, 99 })
                yield return ($"method {method}", Zip([Plain("cht/A/One.cht") with { Method = method, Stored = Text }, Plain("cht/A/Two.cht")]));
            yield return ("an encrypted entry", Zip([Plain("cht/A/One.cht") with { Flags = 1 }, Plain("cht/A/Two.cht")]));
            yield return ("an encrypted stored entry", Zip([Plain("cht/A/One.cht", 0) with { Flags = 1 }, Plain("cht/A/Two.cht")]));
            yield return ("a name in UTF-8 flagged", Zip([Plain("cht/é/Ünï.cht") with { Flags = 0x800 }]));
            yield return ("a name of bytes that are not UTF-8", Zip([Plain("x") with { RawName = new byte[] { (byte)'c', (byte)'h', (byte)'t', (byte)'/', 0xE9, 0xFF, 0xC3, (byte)'.', (byte)'c', (byte)'h', (byte)'t' } }]));
            yield return ("a name holding a NUL", Zip([Plain("cht/A/One.cht"), Plain("cht/A/N\0ul.cht"), Plain("cht/A/Two.cht")]));
            yield return ("a name holding a NUL that is no .cht", Zip([Plain("cht/A/N\0ul.txt"), Plain("cht/A/Two.cht")]));
            yield return ("an entry on another disk", Zip([Plain("cht/A/One.cht") with { Disk = 1 }, Plain("cht/A/Two.cht")]));
            yield return ("a spanned archive", Zip([Plain("cht/A/One.cht")], new Ending { Disk = 1, CentralDisk = 1 }));
            yield return ("an end on a disk of its own", Zip([Plain("cht/A/One.cht")], new Ending { Disk = 1 }));
            yield return ("a count on this disk short of the whole", Zip([Plain("cht/A/One.cht"), Plain("cht/A/Two.cht")], new Ending { CountOnDisk = 1 }));
            yield return ("a Zip64 spanned archive", Zip([Plain("cht/A/One.cht")], new Ending { Zip64 = true, Disk = 2, CentralDisk = 2 }));
            yield return ("a count past the directory", Zip([Plain("cht/A/One.cht")], new Ending { Count = 3, CountOnDisk = 3 }));
            yield return ("a count short of the directory", Zip([Plain("cht/A/One.cht"), Plain("cht/A/Two.cht")], new Ending { Count = 1, CountOnDisk = 1 }));
            yield return ("a Zip64 count past the directory", Zip([Plain("cht/A/One.cht")], new Ending { Zip64 = true, Count = 2, CountOnDisk = 2 }));
            yield return ("a directory past the end", Zip([Plain("cht/A/One.cht")], new Ending { CentralOffset = 1 << 20 }));
            yield return ("a directory starting inside an entry", Zip([Plain("cht/A/One.cht")], new Ending { CentralOffset = 3 }));
            yield return ("a comment at the end", Zip([Plain("cht/A/One.cht")], new Ending { Comment = 300 }));
            yield return ("no end", Zip([Plain("cht/A/One.cht")])[..^22]);
            yield return ("an end and nothing else", Zip(Array.Empty<Entry>()));
            yield return ("nothing", Array.Empty<byte>());
            yield return ("ten bytes", new byte[10]);
            byte[] local = Zip([Plain("cht/A/One.cht"), Plain("cht/A/Two.cht")]);
            local[0] = 0;
            yield return ("a damaged local header", local);
            byte[] cut = Zip([new Entry("cht/A/One.cht", Text) { Method = 0, Stored = Text.Concat(new byte[100]).ToArray() }]);
            yield return ("an entry whose stored bytes claim past the end", cut[..^140]);
        }

        [Fact]
        public void Every_form_installs_as_dotnet_installs_it()
        {
            var differences = new List<string>();
            int i = 0;
            foreach (var (form, zip) in Forms())
            {
                string net = Path.Combine(_root, $"net{i}", "Database"), rust = Path.Combine(_root, $"rust{i}", "Database");
                i++;
                string expected = DotNetCheatInstall.Said(() => DotNetCheatInstall.Install(zip, net)) + "\n" + DotNetCheatInstall.Tree(net);
                string actual = DotNetCheatInstall.Said(() => CheatDatabaseInstaller.Install(new MemoryStream(zip), rust)) + "\n" + DotNetCheatInstall.Tree(rust);
                if (expected != actual) differences.Add($"{form}:\n  .NET: {expected.Replace("\n", "\n        ")}\n  Rust: {actual.Replace("\n", "\n        ")}");
            }
            Assert.True(differences.Count == 0, $"{differences.Count} of {i} forms differ:\n" + string.Join("\n", differences));
        }
    }
}
