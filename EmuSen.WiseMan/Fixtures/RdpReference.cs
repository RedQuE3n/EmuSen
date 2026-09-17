using System.Diagnostics;

namespace EmuSen.WiseMan.Fixtures
{
    // An RDPDUMP2 stream: memory resets, commands as 32-bit halves, and a sync marking each comparison point - see Mars_RdpDifferential.md §2.
    public sealed class RdpDump
    {
        public const int RdramSize = 0x80_0000;
        public const int HiddenSize = 0x40_0000;

        private const uint UpdateDramFlush = 7;
        private const uint RdpCommand = 2;
        private const uint SignalComplete = 5;
        private const uint EndOfFile = 6;
        private const uint UpdateHiddenDramFlush = 9;

        private readonly MemoryStream _stream = new();
        private readonly BinaryWriter _writer;

        public RdpDump()
        {
            _writer = new BinaryWriter(_stream);
            _writer.Write("RDPDUMP2"u8);
            _writer.Write(RdramSize);
            _writer.Write(HiddenSize);
        }

        // Nothing is ever uploaded, so a flush returns both memories to zero.
        public void Reset()
        {
            _writer.Write(UpdateDramFlush);
            _writer.Write(UpdateHiddenDramFlush);
        }

        public void Command(ulong[] words)
        {
            _writer.Write(RdpCommand);
            _writer.Write((uint)(words[0] >> 56) & 0x3F);
            _writer.Write((uint)words.Length * 2);

            foreach (ulong word in words)
            {
                _writer.Write((uint)(word >> 32));
                _writer.Write((uint)word);
            }
        }

        public void Sync() => _writer.Write(SignalComplete);

        public byte[] Finish()
        {
            _writer.Write(EndOfFile);
            _writer.Flush();
            return _stream.ToArray();
        }
    }

    // What angrylion drew by one sync: changed RDRAM pages in console byte order, changed hidden pages, and what it printed - see Mars_RdpDifferential.md §2.
    public sealed record RdpReferenceSync(IReadOnlyDictionary<uint, byte[]> Pages, IReadOnlyDictionary<uint, byte[]> HiddenPages, IReadOnlyList<string> Messages);

    // The two instruments build-probe.sh rdp leaves in the cache, absent on a machine that never built them - see Mars_RdpDifferential.md §3.
    public static class RdpReference
    {
        public const int PageSize = 4096;

        private static readonly TimeSpan Patience = TimeSpan.FromMinutes(2);

        public static string Directory => Path.Combine(CacheHome, "emusen", "probe", "rdp");

        public static string? Grader => Existing(Path.Combine(Directory, "rdp-reference"));

        public static string? CrossCheck => Existing(Path.Combine(Directory, "rdp-validate-dump"));

        public static IReadOnlyList<RdpReferenceSync> Replay(string dump)
        {
            string output = dump + ".ref";
            (int exit, string log) = Run(Grader!, dump, output);
            if (exit != 0) throw new InvalidOperationException($"rdp-reference exited {exit}: {log}");

            return Parse(File.ReadAllBytes(output));
        }

        // Zero when parallel-rdp matched angrylion in RDRAM, hidden RDRAM and texture memory at every sync.
        public static (int Exit, string Log) Agree(string dump) => Run(CrossCheck!, dump, "--sync-only");

        // The tool writes host-order words and runs only on little-endian hosts, so each word is reversed into the console's order.
        private static IReadOnlyList<RdpReferenceSync> Parse(byte[] data)
        {
            if (data.Length < 12 || !data.AsSpan(0, 8).SequenceEqual("RDPREF01"u8)) throw new InvalidDataException("not an RDPREF01 file");

            var syncs = new List<RdpReferenceSync>();
            Dictionary<uint, byte[]>? pages = null;
            Dictionary<uint, byte[]>? hidden = null;
            List<string>? messages = null;
            int at = 8;

            while (true)
            {
                uint tag = BitConverter.ToUInt32(data, at);
                at += 4;

                if (tag is 0 or 1 && pages is not null) syncs.Add(new RdpReferenceSync(pages, hidden!, messages!));
                if (tag == 0) return syncs;

                switch (tag)
                {
                    case 1:
                        at += 4;
                        pages = new Dictionary<uint, byte[]>();
                        hidden = new Dictionary<uint, byte[]>();
                        messages = new List<string>();
                        break;

                    case 2:
                    case 3:
                        uint offset = BitConverter.ToUInt32(data, at);
                        if (tag == 2) pages![offset] = ConsoleOrder(data.AsSpan(at + 4, PageSize));
                        else hidden![offset] = data.AsSpan(at + 4, PageSize).ToArray();
                        at += 4 + PageSize;
                        break;

                    case 4:
                        int length = BitConverter.ToInt32(data, at);
                        messages!.Add(System.Text.Encoding.UTF8.GetString(data, at + 4, length));
                        at += 4 + length;
                        break;

                    default:
                        throw new InvalidDataException($"unknown RDPREF01 tag {tag}");
                }
            }
        }

        private static byte[] ConsoleOrder(ReadOnlySpan<byte> page)
        {
            var swapped = new byte[page.Length];
            for (int i = 0; i < page.Length; i += 4)
            {
                swapped[i] = page[i + 3];
                swapped[i + 1] = page[i + 2];
                swapped[i + 2] = page[i + 1];
                swapped[i + 3] = page[i];
            }

            return swapped;
        }

        private static (int Exit, string Log) Run(string tool, params string[] arguments)
        {
            var start = new ProcessStartInfo(tool)
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                WorkingDirectory = Directory,
            };
            foreach (string argument in arguments) start.ArgumentList.Add(argument);

            using Process process = Process.Start(start)!;
            Task<string> output = process.StandardOutput.ReadToEndAsync();
            Task<string> error = process.StandardError.ReadToEndAsync();

            if (!process.WaitForExit(Patience))
            {
                process.Kill(entireProcessTree: true);
                throw new TimeoutException($"{Path.GetFileName(tool)} ran past {Patience}");
            }

            return (process.ExitCode, output.Result + error.Result);
        }

        private static string CacheHome =>
            Environment.GetEnvironmentVariable("XDG_CACHE_HOME") is { Length: > 0 } cache
                ? cache
                : Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache");

        private static string? Existing(string path) => File.Exists(path) ? path : null;
    }
}
