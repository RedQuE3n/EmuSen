using System.Text;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Mars's video interface against angrylion's, one scanned frame at a time - see Mars_Video.md §3.
    public class MarsViDifferentialTests
    {
        private const uint Framebuffer = 0x0020_0000;

        private const int Control = 0, Origin = 1, Width = 2, VerticalSync = 6, HorizontalStart = 9, VerticalStart = 10, ScaleX = 12, ScaleY = 13;
        private const int CurrentLine = 4;

        private const int Blank = 0, Rgba5551 = 2, Rgba8888 = 3;
        private const int FetchAlways = 0, FetchAsNeeded = 1, ResampleOnly = 2, Replicate = 3;

        // The control bits of the passes past the filter; bit 2, the gamma dither, is not among them - see Mars_VideoPasses.md §3.1.
        private const uint GammaOn = 1 << 3, DivotOn = 1 << 4, DitherFilter = 1 << 16;

        private const uint NtscSync = 525, PalSync = 625;
        private const uint NtscLeft = 108, NtscTop = 34, PalLeft = 128, PalTop = 44;

        // Each case scans one or more frames through its own registers; the interface keeps its raster between them - see §2.4.
        private sealed record Case(string Name, uint[][] Scans)
        {
            public (uint Address, byte[] Bytes)[] Uploads { get; init; } = Array.Empty<(uint, byte[])>();

            // The hidden bits are indexed by sixteen-bit word rather than by byte, on both sides - see Mars_VideoFilter.md §1.
            public (uint Index, byte[] Bytes)[] Hidden { get; init; } = Array.Empty<(uint, byte[])>();
        }

        private sealed record Replay(IReadOnlyList<RdpReferenceSync> Reference, IReadOnlyList<IReadOnlyList<byte[]>> Mars);

        private static readonly Case[] Cases = BuildCases();

        private static readonly Lazy<Replay?> Replayed = new(ReplayAll);

        public static TheoryData<string> CaseNames()
        {
            var data = new TheoryData<string>();
            foreach (Case c in Cases) data.Add(c.Name);
            return data;
        }

        [Theory]
        [MemberData(nameof(CaseNames))]
        public void Mars_scans_out_what_the_reference_scans_out(string name)
        {
            Replay? replay = Replayed.Value;
            if (replay is null) return;

            int index = Array.FindIndex(Cases, c => c.Name == name);
            string differences = Differences(replay.Reference[index].Frames, replay.Mars[index]);

            Assert.True(differences.Length == 0, $"{name}:\n{differences}");
        }

        private static Replay? ReplayAll()
        {
            if (RdpReference.Grader is null) return null;

            string directory = Path.Combine(AppContext.BaseDirectory, "mars-vi-differential");
            System.IO.Directory.CreateDirectory(directory);

            string path = Path.Combine(directory, "cases.rdp");
            var dump = new RdpDump();
            (uint, byte[])[]? uploaded = null;
            (uint, byte[])[]? hidden = null;

            foreach (Case c in Cases)
            {
                // Both caches persist across cases, so a case that carries the memory before it need not send it again.
                if (!ReferenceEquals(c.Uploads, uploaded))
                {
                    foreach ((uint address, byte[] bytes) in c.Uploads) dump.Upload(address, bytes);
                    uploaded = c.Uploads;
                }

                if (!ReferenceEquals(c.Hidden, hidden))
                {
                    foreach ((uint index, byte[] bytes) in c.Hidden) dump.UploadHidden(index, bytes);
                    hidden = c.Hidden;
                }

                dump.Reset();
                foreach (uint[] registers in c.Scans)
                {
                    for (int i = 0; i < registers.Length; i++) dump.ViRegister(i, registers[i]);
                    dump.UpdateScreen();
                }

                dump.Sync();
            }

            File.WriteAllBytes(path, dump.Finish());

            return new Replay(RdpReference.Replay(path), ReplayMars());
        }

        // One interface for every case, as in the reference, with memory restored from the same upload cache where the dump flushes.
        private static IReadOnlyList<IReadOnlyList<byte[]>> ReplayMars()
        {
            var bus = new MemoryBus(expansionPak: true);
            var cache = new byte[bus.Rdram.Length];
            var hidden = new byte[bus.RdramHidden.Length];
            var scanned = new List<IReadOnlyList<byte[]>>();

            foreach (Case c in Cases)
            {
                foreach ((uint address, byte[] bytes) in c.Uploads) bytes.CopyTo(cache, address);
                foreach ((uint index, byte[] bytes) in c.Hidden) bytes.CopyTo(hidden, index);
                cache.CopyTo(bus.Rdram, 0);
                hidden.CopyTo(bus.RdramHidden, 0);

                var frames = new List<byte[]>();
                foreach (uint[] registers in c.Scans)
                {
                    for (int i = 0; i < registers.Length; i++) bus.Write32(MemoryMap.ViBase + (uint)i * 4, registers[i]);
                    if (bus.Vi.Scan()) frames.Add(bus.Vi.Frame.ToArray());
                }

                scanned.Add(frames);
            }

            return scanned;
        }

        // The first pixels that differ, as a column and row of the raster, with what each side put there.
        private static string Differences(IReadOnlyList<RdpReferenceFrame> reference, IReadOnlyList<byte[]> mars)
        {
            var report = new StringBuilder();

            if (reference.Count != mars.Count)
            {
                return $"  the reference scanned {reference.Count} frames and Mars {mars.Count}\n";
            }

            for (int frame = 0; frame < reference.Count; frame++)
            {
                RdpReferenceFrame theirs = reference[frame];
                byte[] ours = mars[frame];

                if (theirs.Pixels.Length != ours.Length)
                {
                    report.AppendLine($"  frame {frame} is {theirs.Width}x{theirs.Height} in the reference and {ours.Length / 4 / theirs.Width} rows in Mars");
                    continue;
                }

                int count = 0;
                for (int pixel = 0; pixel < ours.Length && count < 8; pixel += 4)
                {
                    if (theirs.Pixels.AsSpan(pixel, 4).SequenceEqual(ours.AsSpan(pixel, 4))) continue;

                    int at = pixel / 4;
                    report.AppendLine($"  frame {frame} ({at % theirs.Width},{at / theirs.Width}): reference {Pixel(theirs.Pixels, pixel)}, Mars {Pixel(ours, pixel)}");
                    count++;
                }
            }

            return report.ToString();
        }

        private static string Pixel(byte[] bytes, int at) => $"{bytes[at]:X2}{bytes[at + 1]:X2}{bytes[at + 2]:X2}/{bytes[at + 3]:X2}";

        private static uint Start(uint from, uint to) => ((from & 0x3FF) << 16) | (to & 0x3FF);

        private static uint Scale(uint step, uint bias) => ((bias & 0xFFF) << 16) | (step & 0xFFF);

        private static uint Mode(int type, int antialias, bool serrate = false) =>
            (uint)(type & 3) | ((uint)(antialias & 3) << 8) | (serrate ? 1u << 6 : 0);

        // A picture 320 wide and 240 tall, stretched to the whole raster.
        private static uint[] Ntsc(int type, int antialias, uint width = 320, uint stepX = 512, uint stepY = 1024, bool serrate = false)
        {
            var registers = new uint[14];
            registers[Control] = Mode(type, antialias, serrate);
            registers[Origin] = Framebuffer;
            registers[Width] = width;
            registers[VerticalSync] = NtscSync;
            registers[HorizontalStart] = Start(NtscLeft, NtscLeft + 640);
            registers[VerticalStart] = Start(NtscTop, NtscTop + 224 * 2);
            registers[ScaleX] = Scale(stepX, 0);
            registers[ScaleY] = Scale(stepY, 0);
            return registers;
        }

        private static uint[] With(uint[] registers, int index, uint value)
        {
            var copy = (uint[])registers.Clone();
            copy[index] = value;
            return copy;
        }

        private static Case[] BuildCases()
        {
            var cases = new List<Case>();
            (uint, byte[])[] uploads = { (Framebuffer, Picture()) };
            (uint, byte[])[] mixed = { (Framebuffer / 2, Hidden(-1)) };

            void Scan(string name, params uint[][] scans) => cases.Add(new Case(name, scans) { Uploads = uploads, Hidden = mixed });

            Scan("a sixteen-bit picture replicated", Ntsc(Rgba5551, Replicate));
            Scan("a sixteen-bit picture resampled", Ntsc(Rgba5551, ResampleOnly));
            Scan("a thirty-two-bit picture replicated", Ntsc(Rgba8888, Replicate));
            Scan("a thirty-two-bit picture resampled", Ntsc(Rgba8888, ResampleOnly));

            Scan("a blank signal", Ntsc(Blank, Replicate));
            Scan("two blank signals in a row", Ntsc(Blank, Replicate), Ntsc(Blank, Replicate));
            Scan("a blank signal between two pictures", Ntsc(Rgba5551, Replicate), Ntsc(Blank, Replicate), Ntsc(Rgba5551, Replicate));

            // A blank signal clears the raster and every line's count before it scans, which only a narrower picture after it shows - see §2.2.
            Scan("a blank signal narrower than the picture before it",
                Ntsc(Rgba5551, Replicate), With(Ntsc(Blank, Replicate), HorizontalStart, Start(NtscLeft + 40, NtscLeft + 40 + 200)));
            Scan("a blank signal leaves no line held",
                Ntsc(Rgba5551, Replicate), Ntsc(Blank, Replicate),
                With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 60 * 2)),
                With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 60 * 2)),
                With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 60 * 2)));
            Scan("no frame buffer", With(Ntsc(Rgba5551, Replicate), Origin, 0));

            Scan("a picture that starts left of the raster", With(Ntsc(Rgba5551, Replicate), HorizontalStart, Start(40, 40 + 640)));
            Scan("a picture that runs past the raster's right", With(Ntsc(Rgba5551, Replicate), HorizontalStart, Start(NtscLeft + 200, NtscLeft + 200 + 640)));
            Scan("a picture that starts above the raster", With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(8, 8 + 224 * 2)));
            Scan("a picture of no width", With(Ntsc(Rgba5551, Replicate), HorizontalStart, Start(NtscLeft, NtscLeft)));

            Scan("a scale with a bias", With(With(Ntsc(Rgba5551, ResampleOnly), ScaleX, Scale(512, 0x220)), ScaleY, Scale(1024, 0x180)));
            Scan("rows repeated", With(Ntsc(Rgba5551, ResampleOnly), ScaleY, Scale(256, 0)));
            Scan("columns skipped", With(Ntsc(Rgba5551, ResampleOnly), ScaleX, Scale(0xC00, 0)));
            Scan("a step of no size", With(With(Ntsc(Rgba5551, ResampleOnly), ScaleX, Scale(0, 0)), ScaleY, Scale(0, 0)));

            Scan("a wider frame buffer than the picture", With(Ntsc(Rgba5551, Replicate), Width, 640));
            Scan("a narrower frame buffer than the picture", With(Ntsc(Rgba5551, Replicate), Width, 64));

            var pal = Ntsc(Rgba5551, Replicate);
            pal[VerticalSync] = PalSync;
            pal[HorizontalStart] = Start(PalLeft, PalLeft + 640);
            pal[VerticalStart] = Start(PalTop, PalTop + 240 * 2);
            Scan("a PAL picture", pal);

            uint[] serrate = Ntsc(Rgba5551, Replicate, serrate: true);
            Scan("an interlaced picture, both fields", With(serrate, CurrentLine, 0), With(serrate, CurrentLine, 1));

            Scan("a vertical sync too short to show anything", With(Ntsc(Rgba5551, Replicate), VerticalSync, 20));
            Scan("a last line that wraps past its field", With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 600 * 2)));
            Scan("a picture taller than the frame", With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 280 * 2)));

            // An interlaced field holds its own lines and fades the other's, which only a later frame can show - see §2.4.
            Scan("an interlaced picture, then a shorter one",
                With(serrate, CurrentLine, 0), With(serrate, CurrentLine, 1),
                With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 88 * 2)),
                With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 88 * 2)));

            // The raster holds a line for two frames after nothing writes it, so the fade needs three scans to show - see §2.4.
            uint[] shown = Ntsc(Rgba5551, Replicate);
            uint[] none = With(shown, HorizontalStart, Start(NtscLeft, NtscLeft));
            Scan("a picture, then nothing, three times over", shown, none, none, none, none);

            // A line above the picture but past the active lines is darkened only where the picture lies, and no border clear reaches it - see §2.4.
            uint[] tall = With(Ntsc(Rgba5551, Replicate), VerticalStart, Start(NtscTop, NtscTop + 480 * 2));
            uint[] high = With(With(With(Ntsc(Rgba5551, Replicate), VerticalSync, 234),
                VerticalStart, Start(NtscTop + 240, NtscTop + 240 + 20 * 2)), HorizontalStart, Start(NtscLeft + 40, NtscLeft + 40 + 200));
            Scan("a line above the picture and past the active lines", tall, high, high, high);

            Scan("a shorter picture after a taller one", shown, With(shown, VerticalStart, Start(NtscTop, NtscTop + 100 * 2)), shown);
            Scan("a narrower picture after a wider one", shown, With(shown, HorizontalStart, Start(NtscLeft + 40, NtscLeft + 40 + 400)), shown);

            AntiAliased(cases, uploads, mixed);
            Passes(cases, uploads, mixed);

            var random = new Random(0x5649_4449);
            for (int n = 0; n < 80; n++)
            {
                int type = (n & 1) == 0 ? Rgba5551 : Rgba8888;
                int antialias = (n >> 1) & 3;
                uint left = (uint)random.Next(60, 200);
                uint top = (uint)random.Next(10, 80);
                uint[] registers = Ntsc(type, antialias,
                    width: (uint)random.Next(16, 640),
                    stepX: (uint)random.Next(0, 0x1000),
                    stepY: (uint)random.Next(0, 0x1000));

                registers[HorizontalStart] = Start(left, left + (uint)random.Next(16, 640));
                registers[VerticalStart] = Start(top, top + (uint)random.Next(16, 480));
                registers[ScaleX] = Scale(registers[ScaleX] & 0xFFF, (uint)random.Next(0, 0x400));
                registers[ScaleY] = Scale(registers[ScaleY] & 0xFFF, (uint)random.Next(0, 0x400));

                // Half the random cases also turn on a random set of the four passes past the filter - see Mars_VideoPasses.md §4.
                if ((n & 1) == 1)
                {
                    uint passes = 0;
                    if (random.Next(2) == 0) passes |= DitherFilter;
                    if (random.Next(2) == 0) passes |= DivotOn;
                    if (random.Next(2) == 0) passes |= GammaOn;
                    registers[Control] |= passes;
                }

                cases.Add(new Case($"random {n}", new[] { registers }) { Uploads = uploads, Hidden = mixed });
            }

            return cases.ToArray();
        }

        // The dither filter, divot and gamma: each alone, each where it must do nothing, and all of them at once - see Mars_VideoPasses.md §4.
        private static void Passes(List<Case> cases, (uint, byte[])[] uploads, (uint, byte[])[] mixed)
        {
            void Scan(string name, params uint[][] scans) => cases.Add(new Case(name, scans) { Uploads = uploads, Hidden = mixed });

            uint[] Pass(uint[] registers, uint bits) => With(registers, Control, registers[Control] | bits);

            Scan("the dither filter on a sixteen-bit picture", Pass(Ntsc(Rgba5551, Replicate), DitherFilter));
            Scan("the dither filter on a thirty-two-bit picture", Pass(Ntsc(Rgba8888, Replicate), DitherFilter));
            Scan("the dither filter under anti-aliasing", Pass(Ntsc(Rgba5551, FetchAsNeeded), DitherFilter));
            Scan("the dither filter with no whole pixel to filter", Pass(Ntsc(Rgba8888, FetchAsNeeded), DitherFilter));
            // The artefact only reaches the row a resampled pixel mixes with, so a replicated scan cannot show it - see §1.1.
            Scan("the dither filter after a row read twice", Pass(With(Ntsc(Rgba5551, ResampleOnly), ScaleY, Scale(0x200, 0)), DitherFilter));
            Scan("the dither filter after a row read twice, thirty-two bit", Pass(With(Ntsc(Rgba8888, ResampleOnly), ScaleY, Scale(0x200, 0)), DitherFilter));
            Scan("the dither filter resampled", Pass(Ntsc(Rgba5551, ResampleOnly), DitherFilter));

            Scan("divot on a sixteen-bit picture", Pass(Ntsc(Rgba5551, FetchAsNeeded), DivotOn));
            Scan("divot on a thirty-two-bit picture", Pass(Ntsc(Rgba8888, FetchAsNeeded), DivotOn));
            Scan("divot with a fractional step", Pass(With(With(Ntsc(Rgba5551, FetchAsNeeded), ScaleX, Scale(0x2AB, 0x80)), ScaleY, Scale(0x155, 0x40)), DivotOn));
            Scan("divot where no pixel is whole", Pass(Ntsc(Rgba5551, Replicate), DivotOn));
            Scan("divot and the dither filter together", Pass(Ntsc(Rgba5551, FetchAsNeeded), DivotOn | DitherFilter));

            Scan("gamma", Pass(Ntsc(Rgba5551, Replicate), GammaOn));
            Scan("gamma on a thirty-two-bit picture", Pass(Ntsc(Rgba8888, Replicate), GammaOn));
            Scan("gamma resampled", Pass(Ntsc(Rgba5551, ResampleOnly), GammaOn));
            Scan("gamma under anti-aliasing", Pass(Ntsc(Rgba5551, FetchAsNeeded), GammaOn));

            // The gamma dither reads a noise neither reference models as hardware, so no case sets bit 2 - see §3.1.
            Scan("every pass at once", Pass(Ntsc(Rgba5551, FetchAsNeeded), DitherFilter | DivotOn | GammaOn));
            Scan("every pass at once, thirty-two bit", Pass(Ntsc(Rgba8888, FetchAsNeeded), DitherFilter | DivotOn | GammaOn));

            void Bits(string name, int coverage, uint passes, params uint[][] scans) =>
                cases.Add(new Case(name, scans)
                {
                    Uploads = new[] { (Framebuffer, Picture(coverage)) },
                    Hidden = new[] { (Framebuffer / 2, Hidden(coverage * 3)) },
                });

            // Divot leaves a row of whole pixels alone, which is only visible against a picture that has nothing else - see §2.
            Bits("divot where every pixel is whole", 1, DivotOn, Pass(Ntsc(Rgba5551, FetchAsNeeded), DivotOn));
            Bits("divot where every pixel is partly covered", 0, DivotOn, Pass(Ntsc(Rgba5551, FetchAsNeeded), DivotOn));
            Bits("the dither filter where every pixel is whole", 1, DitherFilter, Pass(Ntsc(Rgba5551, FetchAsNeeded), DitherFilter));
        }

        // Every case that needs a coverage: the filter itself, the neighbourhoods it can see, and the fetch bug - see Mars_VideoFilter.md §4.
        private static void AntiAliased(List<Case> cases, (uint, byte[])[] uploads, (uint, byte[])[] mixed)
        {
            void Scan(string name, params uint[][] scans) => cases.Add(new Case(name, scans) { Uploads = uploads, Hidden = mixed });

            Scan("a sixteen-bit picture anti-aliased", Ntsc(Rgba5551, FetchAsNeeded));
            Scan("a thirty-two-bit picture anti-aliased", Ntsc(Rgba8888, FetchAsNeeded));
            Scan("a sixteen-bit picture in the other anti-alias mode", Ntsc(Rgba5551, FetchAlways));
            Scan("a thirty-two-bit picture in the other anti-alias mode", Ntsc(Rgba8888, FetchAlways));

            // A step short of a whole line makes the row after the repeat fetch its own line for the one below - see §3.
            Scan("a row read twice under anti-aliasing", With(Ntsc(Rgba5551, FetchAsNeeded), ScaleY, Scale(0x200, 0)));
            Scan("every row read again under anti-aliasing", With(Ntsc(Rgba5551, FetchAsNeeded), ScaleY, Scale(0, 0)));
            Scan("a row read twice, thirty-two bit", With(Ntsc(Rgba8888, FetchAsNeeded), ScaleY, Scale(0x200, 0)));
            Scan("a step of three quarters of a line", With(Ntsc(Rgba5551, FetchAsNeeded), ScaleY, Scale(0x300, 0)));

            Scan("a narrow frame buffer anti-aliased", With(Ntsc(Rgba5551, FetchAsNeeded), Width, 64));
            Scan("a wide frame buffer anti-aliased", With(Ntsc(Rgba5551, FetchAsNeeded), Width, 640));
            Scan("an anti-aliased picture at the start of memory", With(Ntsc(Rgba5551, FetchAsNeeded), Origin, 0x100));
            Scan("an anti-aliased picture pulled in at the left", With(Ntsc(Rgba5551, FetchAsNeeded), HorizontalStart, Start(40, 40 + 640)));
            Scan("an anti-aliased picture cut at the right", With(Ntsc(Rgba5551, FetchAsNeeded), HorizontalStart, Start(NtscLeft + 200, NtscLeft + 200 + 640)));
            Scan("an interlaced picture anti-aliased", With(Ntsc(Rgba5551, FetchAsNeeded, serrate: true), CurrentLine, 1));

            // The raster keeps each pixel's coverage, which only a frame that darkens over it can show - see §2.3.
            uint[] shown = Ntsc(Rgba5551, FetchAsNeeded);
            Scan("an anti-aliased picture, then nothing", shown, With(shown, HorizontalStart, Start(NtscLeft, NtscLeft)));

            void Bits(string name, int coverage, params uint[][] scans) =>
                cases.Add(new Case(name, scans)
                {
                    Uploads = new[] { (Framebuffer, Picture(coverage)) },
                    Hidden = new[] { (Framebuffer / 2, Hidden(coverage * 3)) },
                });

            // With no whole pixel anywhere the filter has only the centre to work from, and must leave it alone - see §2.2.
            Bits("no pixel whole", 0, Ntsc(Rgba5551, FetchAsNeeded));
            Bits("no pixel whole, thirty-two bit", 0, Ntsc(Rgba8888, FetchAsNeeded));
            Bits("every pixel whole", 1, Ntsc(Rgba5551, FetchAsNeeded));
            Bits("every pixel whole, thirty-two bit", 1, Ntsc(Rgba8888, FetchAsNeeded));

            // Only the hidden bits are held down, so a pixel is whole exactly where the word's own bit says so - see §1.
            cases.Add(new Case("whole pixels scattered through a picture", new[] { Ntsc(Rgba5551, FetchAsNeeded) })
            {
                Uploads = uploads,
                Hidden = new[] { (Framebuffer / 2, Hidden(3)) },
            });
        }

        // A frame buffer with no two neighbouring pixels alike, so a step in either direction shows.
        private static byte[] Picture(int coverage = -1)
        {
            var bytes = new byte[640 * 480 * 4];
            uint state = 0x1357_9BDF;
            for (int i = 0; i < bytes.Length; i++)
            {
                state = state * 1664525 + 1013904223;
                bytes[i] = (byte)(state >> 24);
            }

            if (coverage < 0) return bytes;

            // The coverage a pixel carries itself: one bit of a sixteen-bit word, three of a thirty-two-bit one - see Mars_VideoFilter.md §1.
            for (int i = 1; i < bytes.Length; i += 2) bytes[i] = (byte)((bytes[i] & ~1) | coverage);
            for (int i = 3; i < bytes.Length; i += 4) bytes[i] = (byte)((bytes[i] & ~0xE0) | (coverage * 0xE0));

            return bytes;
        }

        // One byte a sixteen-bit word, carrying two coverage bits the processor cannot reach - see Mars_VideoFilter.md §1.
        private static byte[] Hidden(int value)
        {
            var bytes = new byte[640 * 480 * 2];
            uint state = 0x2468_ACE0;
            for (int i = 0; i < bytes.Length; i++)
            {
                state = state * 1664525 + 1013904223;
                bytes[i] = value < 0 ? (byte)(state >> 30) : (byte)value;
            }

            return bytes;
        }
    }
}
