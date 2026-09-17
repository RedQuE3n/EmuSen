using System.Text;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Mars's display processor against angrylion, with parallel-rdp checking angrylion, one primitive at a time - see Mars_RdpDifferential.md.
    public class MarsRdpDifferentialTests
    {
        private const uint Framebuffer = 0x0010_0000;
        private const int Bits4 = 0, Bits8 = 1, Bits16 = 2, Bits32 = 3;
        private const ulong FillCycle = (0x2FUL << 56) | (3UL << 52);
        private const ulong SyncFull = 0x29UL << 56;

        // An artefact is a reference behaviour that is not a claim about hardware, a dispute is the two references differing - see §4.
        private sealed record Case(string Name, uint Image, int Size, int Width, ulong[] Commands, string? Artefact = null, string? Dispute = null, bool CrossChecked = true)
        {
            public (uint Address, byte[] Bytes)[] Uploads { get; init; } = Array.Empty<(uint, byte[])>();
        }

        // Uploaded holds the pages the upload cache restored, which the reference reports only if drawing changed them.
        private sealed record Drawn(byte[] Rdram, byte[] Hidden, byte[] TextureMemory, IReadOnlyDictionary<uint, byte[]> Uploaded);

        private sealed record Replay(IReadOnlyList<RdpReferenceSync> Reference, IReadOnlyList<Drawn> Mars, int AgreementExit, string AgreementLog,
            IReadOnlyDictionary<string, int> DisputeExits);

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
        public void Mars_draws_what_the_reference_draws(string name)
        {
            Replay? replay = Replayed.Value;
            if (replay is null) return;

            int index = Array.FindIndex(Cases, c => c.Name == name);
            RdpReferenceSync reference = replay.Reference[index];

            string differences = Differences(Cases[index], reference, replay.Mars[index]);

            if (Cases[index].Artefact is string artefact)
            {
                Assert.True(differences.Length > 0, $"{name}: expected to differ because {artefact}, and did not");
                return;
            }

            Assert.True(differences.Length == 0, $"{name}:\n{differences}{Messages(reference)}");
        }

        // A case the two references disagree on grades Mars against a model in dispute, so it is named rather than trusted - see §4.
        [Fact]
        public void The_two_reference_rasterizers_agree_on_every_case()
        {
            Replay? replay = Replayed.Value;
            if (replay is null) return;

            Assert.True(replay.AgreementExit == 0, Tail(replay.AgreementLog));
        }

        public static TheoryData<string> DisputedNames()
        {
            var data = new TheoryData<string>();
            foreach (Case c in Cases.Where(c => c.Dispute is not null)) data.Add(c.Name);
            return data;
        }

        // Kept as a measurement: if either reference changes its answer, the dispute on the man page is out of date.
        [Theory]
        [MemberData(nameof(DisputedNames))]
        public void A_recorded_dispute_between_the_references_still_stands(string name)
        {
            Replay? replay = Replayed.Value;
            if (replay is null) return;

            Assert.True(replay.DisputeExits[name] != 0, $"{name}: parallel-rdp now agrees with angrylion, so the recorded dispute is stale");
        }

        private static Replay? ReplayAll()
        {
            if (RdpReference.Grader is null || RdpReference.CrossCheck is null) return null;

            string directory = Path.Combine(AppContext.BaseDirectory, "mars-rdp-differential");
            System.IO.Directory.CreateDirectory(directory);

            string graded = Write(Path.Combine(directory, "cases.rdp"), Cases);
            string agreed = Write(Path.Combine(directory, "cases-cross-checked.rdp"), Cases.Where(c => c.CrossChecked && c.Dispute is null));

            IReadOnlyList<RdpReferenceSync> reference = RdpReference.Replay(graded);
            (int exit, string log) = RdpReference.Agree(agreed);

            var disputes = new Dictionary<string, int>();
            foreach ((Case c, int i) in Cases.Select((c, i) => (c, i)).Where(p => p.c.Dispute is not null))
            {
                disputes[c.Name] = RdpReference.Agree(Write(Path.Combine(directory, $"cases-dispute-{i}.rdp"), new[] { c })).Exit;
            }

            return new Replay(reference, ReplayMars(), exit, log, disputes);
        }

        private static string Write(string path, IEnumerable<Case> cases)
        {
            var dump = new RdpDump();
            foreach (Case c in cases)
            {
                foreach ((uint address, byte[] bytes) in c.Uploads) dump.Upload(address, bytes);
                dump.Reset();
                foreach (ulong[] command in Split(c.Commands)) dump.Command(command);
                dump.Sync();
            }

            File.WriteAllBytes(path, dump.Finish());
            return path;
        }

        // One display processor for every case, as in the references, with memory restored from the same upload cache where they flush.
        private static IReadOnlyList<Drawn> ReplayMars()
        {
            var bus = new MarsBus(expansionPak: true);
            var cache = new byte[bus.Rdram.Length];
            var drawn = new List<Drawn>();
            var uploaded = new Dictionary<uint, byte[]>();

            foreach (Case c in Cases)
            {
                foreach ((uint address, byte[] bytes) in c.Uploads)
                {
                    bytes.CopyTo(cache, address);
                    uploaded = new Dictionary<uint, byte[]>(uploaded);
                    for (uint page = address & ~(uint)(RdpReference.PageSize - 1); page < address + bytes.Length; page += RdpReference.PageSize)
                    {
                        uploaded[page] = cache.AsSpan((int)page, RdpReference.PageSize).ToArray();
                    }
                }

                cache.CopyTo(bus.Rdram, 0);
                Array.Clear(bus.RdramHidden);

                foreach (ulong word in c.Commands) bus.Dp.Processor.Accept(word);
                drawn.Add(new Drawn((byte[])bus.Rdram.Clone(), (byte[])bus.RdramHidden.Clone(), (byte[])bus.Dp.Processor.TextureMemory.Clone(), uploaded));
            }

            return drawn;
        }

        private static IEnumerable<ulong[]> Split(ulong[] commands)
        {
            for (int at = 0; at < commands.Length;)
            {
                int length = EmuSen.Cores.Nintendo.Mars.Rdp.Rdp.Length(EmuSen.Cores.Nintendo.Mars.Rdp.Rdp.Id(commands[at]));
                yield return commands[at..(at + length)];
                at += length;
            }
        }

        // Every page either side touched, RDRAM and then hidden RDRAM, byte for byte, reported as pixels of the case's colour image.
        private static string Differences(Case c, RdpReferenceSync reference, Drawn mars)
        {
            var report = new StringBuilder();
            int count = 0;
            int bytes = Math.Max(1, (1 << c.Size) / 2);

            Compare(reference.Pages, mars.Uploaded, mars.Rdram, 1, "");
            Compare(reference.HiddenPages, new Dictionary<uint, byte[]>(), mars.Hidden, 2, "hidden ");

            for (int i = 0; i < 0x1000; i++)
            {
                if (reference.TextureMemory[i] != mars.TextureMemory[i] && count++ < 12)
                {
                    report.AppendLine($"  texture memory {i:X3}: Mars {mars.TextureMemory[i]:X2}, reference {reference.TextureMemory[i]:X2}");
                }
            }

            if (count > 12) report.AppendLine($"  ...{count} differing bytes in all");
            return report.ToString();

            void Compare(IReadOnlyDictionary<uint, byte[]> expectedPages, IReadOnlyDictionary<uint, byte[]> unchangedPages, byte[] actual, int scale, string label)
            {
                var pages = new SortedSet<uint>(expectedPages.Keys);
                pages.UnionWith(unchangedPages.Keys);
                for (uint page = 0; page < actual.Length; page += RdpReference.PageSize)
                {
                    if (actual.AsSpan((int)page, RdpReference.PageSize).IndexOfAnyExcept((byte)0) >= 0) pages.Add(page);
                }

                foreach (uint page in pages)
                {
                    byte[] expected = expectedPages.TryGetValue(page, out byte[]? p) ? p
                        : unchangedPages.TryGetValue(page, out byte[]? u) ? u : new byte[RdpReference.PageSize];

                    for (int i = 0; i < RdpReference.PageSize; i++)
                    {
                        if (expected[i] == actual[page + i]) continue;

                        if (count++ < 12)
                        {
                            long pixel = ((long)(page + i) * scale - (c.Image & ~(uint)(bytes - 1))) / bytes;
                            long x = ((pixel % c.Width) + c.Width) % c.Width, y = (long)Math.Floor((double)pixel / c.Width);
                            report.AppendLine($"  {label}{page + i:X6} pixel ({x}, {y}): Mars {actual[page + i]:X2}, reference {expected[i]:X2}");
                        }
                    }
                }
            }
        }

        private static string Messages(RdpReferenceSync reference) =>
            reference.Messages.Count == 0 ? string.Empty : "reference said: " + string.Join("; ", reference.Messages.Distinct());

        private static string Tail(string log) => string.Join('\n', log.Split('\n').TakeLast(20));

        private static Case[] BuildCases()
        {
            var cases = new List<Case>();

            void Fill(string name, ulong scissor, ulong rectangle, int size = Bits16, int width = 32, uint image = Framebuffer, uint color = 0xF0F0_0F0F,
                string? artefact = null, string? dispute = null, bool crossChecked = true) =>
                cases.Add(new Case(name, image, size, width, new[] { ColorImage(size, width, image), scissor, FillCycle, FillColor(color), rectangle, SyncFull },
                    artefact, dispute, crossChecked));

            ulong wholeScreen = Scissor(0, 0, 32 * 4, 32 * 4);

            Fill("rectangle inside the scissor", wholeScreen, Rectangle(16, 16, 40, 32));
            Fill("scissor inside the rectangle", Scissor(16, 16, 40, 32), Rectangle(0, 0, 124, 124));
            Fill("rectangle and scissor with the same corners", Scissor(16, 16, 40, 32), Rectangle(16, 16, 40, 32));
            Fill("one pixel", wholeScreen, Rectangle(20, 20, 20, 20));
            Fill("right edge left of the left edge", wholeScreen, Rectangle(40, 32, 16, 16));
            Fill("rectangle past the image's right side", Scissor(0, 0, 160, 32), Rectangle(112, 8, 140, 12),
                artefact: "the reference clamps each span to the image's width, a validation workaround in its fork (angrylion-rdp-plus 9a1965c)");

            foreach (uint fraction in new uint[] { 1, 2, 3 })
            {
                Fill($"left edge at +{fraction}/4", wholeScreen, Rectangle(16 + fraction, 16, 40, 32));
                Fill($"right edge at +{fraction}/4", wholeScreen, Rectangle(16, 16, 40 + fraction, 32));
                Fill($"top edge at +{fraction}/4", wholeScreen, Rectangle(16, 16 + fraction, 40, 32));
                Fill($"bottom edge at +{fraction}/4", wholeScreen, Rectangle(16, 16, 40, 32 + fraction));
                Fill($"scissor left at +{fraction}/4", Scissor(16 + fraction, 0, 128, 128), Rectangle(8, 8, 60, 24));
                Fill($"scissor right at +{fraction}/4", Scissor(0, 0, 40 + fraction, 128), Rectangle(8, 8, 60, 24));
                Fill($"scissor top at +{fraction}/4", Scissor(0, 16 + fraction, 128, 128), Rectangle(8, 8, 60, 24));
                Fill($"scissor bottom at +{fraction}/4", Scissor(0, 0, 128, 32 + fraction), Rectangle(8, 8, 60, 60));
            }

            Fill("every edge fractional", Scissor(9, 10, 118, 107), Rectangle(13, 6, 101, 111));

            Fill("rectangle starting on the scissor's right edge", Scissor(0, 0, 40, 128), Rectangle(40, 8, 60, 24));
            Fill("rectangle starting a quarter left of the scissor's right edge", Scissor(0, 0, 40, 128), Rectangle(39, 8, 60, 24));
            Fill("rectangle ending on the scissor's left edge", Scissor(40, 0, 128, 128), Rectangle(8, 8, 40, 24));
            Fill("rectangle ending a quarter left of the scissor's left edge", Scissor(40, 0, 128, 128), Rectangle(8, 8, 39, 24));
            Fill("rectangle wholly right of the scissor", Scissor(0, 0, 40, 128), Rectangle(48, 8, 60, 24));
            Fill("rectangle wholly left of the scissor", Scissor(40, 0, 128, 128), Rectangle(8, 8, 32, 24));
            Fill("empty scissor", Scissor(40, 0, 40, 128), Rectangle(8, 8, 60, 24),
                dispute: "parallel-rdp draws the scissor's one column where angrylion draws nothing");
            Fill("rectangle starting on the scissor's bottom edge", Scissor(0, 0, 128, 32), Rectangle(8, 32, 60, 48));
            Fill("rectangle ending on the scissor's top edge", Scissor(0, 32, 128, 128), Rectangle(8, 8, 60, 32));
            Fill("right edge a quarter left of the left edge in one pixel", wholeScreen, Rectangle(42, 16, 41, 32));
            Fill("rectangle below a fractional scissor bottom in the same row", Scissor(0, 0, 128, 33), Rectangle(8, 35, 60, 60));

            Fill("32-bit image", wholeScreen, Rectangle(8, 8, 40, 20), size: Bits32, width: 16, color: 0x1234_5678);
            Fill("8-bit image at an odd address", wholeScreen, Rectangle(8, 8, 40, 20), size: Bits8, width: 16, image: Framebuffer + 1, color: 0x1234_5678);
            Fill("16-bit image at an odd address", wholeScreen, Rectangle(8, 8, 40, 20), image: Framebuffer + 1);
            Fill("16-bit image of odd width", wholeScreen, Rectangle(0, 0, 40, 8), width: 31);
            Fill("4-bit image", wholeScreen, Rectangle(8, 8, 40, 20), size: Bits4, crossChecked: false);

            Fill("scissor keeping even rows", Scissor(0, 0, 128, 128, field: true, keepOdd: false), Rectangle(8, 8, 40, 40));
            Fill("scissor keeping odd rows", Scissor(0, 0, 128, 128, field: true, keepOdd: true), Rectangle(8, 8, 40, 40));

            // The image is 32 wide and the scissor stops at column 31, so no span reaches the reference's width clamp - see §4.3.
            ulong inside = Scissor(0, 0, 124, 124);

            void Triangle(string name, ulong scissor, ulong[] triangle, int size = Bits16, int width = 32, uint color = 0xF0F0_0F0F) =>
                cases.Add(new Case(name, Framebuffer, size, width,
                    new[] { ColorImage(size, width, Framebuffer), scissor, FillCycle, FillColor(color) }.Concat(triangle).Append(SyncFull).ToArray()));

            Triangle("triangle with its major edge on the left", inside, Vertices(4, 2, 20, 10, 6, 26));
            Triangle("triangle with its major edge on the right", inside, Vertices(24, 2, 8, 12, 22, 28));
            Triangle("triangle with a flat top", inside, Vertices(4, 4, 26, 4, 14, 24));
            Triangle("triangle with a flat bottom", inside, Vertices(14, 3, 4, 22, 27, 22));
            Triangle("triangle with fractional vertices", inside, Vertices(3.25, 2.5, 21.75, 9.75, 7.5, 25.25));
            Triangle("triangle thinner than a pixel", inside, Vertices(10, 1, 10.5, 15, 10.25, 29));
            Triangle("triangle with a shallow edge", inside, Vertices(1, 10, 30, 12, 2, 14));
            Triangle("triangle with a steep edge", inside, Vertices(15, 0, 16, 30, 17.5, 30.5));
            Triangle("triangle starting above the image", inside, Vertices(12, -9, 28, 14, 4, 20));
            Triangle("triangle starting left of the image", inside, Vertices(-10, 4, 18, 8, 6, 26));
            Triangle("triangle cut by the scissor on every side", Scissor(24, 20, 96, 88), Vertices(1, 1, 30, 6, 12, 30));
            Triangle("triangle under a fractional scissor", Scissor(25, 22, 97, 91), Vertices(1, 1, 30, 6, 12, 30));
            Triangle("triangle under an interlaced scissor", Scissor(0, 0, 124, 124, field: true, keepOdd: true), Vertices(4, 2, 20, 10, 6, 26));
            Triangle("triangle in a 32-bit image", inside, Vertices(4, 2, 20, 10, 6, 26), size: Bits32, color: 0x1234_5678);
            Triangle("triangle in an 8-bit image", inside, Vertices(4, 2, 20, 10, 6, 26), size: Bits8, color: 0x1234_5678);
            Triangle("triangle with its major-edge flag inverted", inside, Vertices(4, 2, 20, 10, 6, 26, invertMajor: true));
            Triangle("triangle with its middle vertex above its top", inside, Vertices(4, 2, 20, 10, 6, 26, swapTopAndMiddle: true));
            Triangle("shaded triangle in the fill cycle", inside, Vertices(4, 2, 20, 10, 6, 26, id: 0x0C));
            Triangle("textured depth-tested triangle in the fill cycle", inside, Vertices(24, 2, 8, 12, 22, 28, id: 0x0B));
            Triangle("shaded textured depth-tested triangle in the fill cycle", inside, Vertices(24, 2, 8, 12, 22, 28, id: 0x0F));
            Triangle("triangle reaching past x 1024", inside, Vertices(4, 2, 1100, 10, 6, 26));
            Triangle("edge whose step has its low bit set", inside,
                Edges(majorOnLeft: false, yh: 0, ym: 0, yl: 64, xh: (9 << 16) - 82, dxhdy: 12, xm: 2 << 16, dxmdy: 0, xl: 2 << 16, dxldy: 0));

            var random = new Random(0x4D41_5253);
            for (int n = 0; n < 60; n++)
            {
                double Coordinate() => random.Next(-24, 150) / 4.0;
                ulong scissor = n % 3 == 0 ? Scissor((uint)random.Next(0, 60), (uint)random.Next(0, 60), (uint)random.Next(64, 125), (uint)random.Next(64, 125)) : inside;
                Triangle($"random triangle {n}", scissor, Vertices(Coordinate(), Coordinate(), Coordinate(), Coordinate(), Coordinate(), Coordinate()));
            }

            cases.AddRange(OneCycleCases());
            cases.AddRange(ShadeAndDepthCases());
            cases.AddRange(TextureCases());
            return cases.ToArray();
        }

        private static ulong ColorImage(int size, int width, uint address, int format = 0) =>
            (0x3FUL << 56) | ((ulong)format << 53) | ((ulong)size << 51) | ((ulong)(width - 1) << 32) | address;

        // Flat primitives drawn in the one-cycle mode, with every mode that needs neither texture, depth, chroma key nor noise - see Mars_RdpCoverage.md §6.
        private static IEnumerable<Case> OneCycleCases()
        {
            var cases = new List<Case>();
            ulong inside = Scissor(0, 0, 124, 124);
            ulong primitive = Combine(subA: 8, subB: 8, mul: 16, add: 3, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 3);
            ulong blend = Combine(subA: 3, subB: 5, mul: 10, add: 5, alphaSubA: 3, alphaSubB: 5, alphaMul: 3, alphaAdd: 5);
            ulong[] triangle = Vertices(3.25, 2.5, 27.75, 9.75, 7.5, 28.25);
            ulong[] other = Vertices(28.5, 1.25, 4.75, 17.5, 24.25, 29.75);

            void Draw(string name, ulong modes, ulong combine, ulong[] shapes, int size = Bits16, int format = 0, uint background = 0, bool fill = true,
                string? dispute = null, uint k4 = 0x0B5, uint k5 = 0x13C, uint blendColor = 0x7755_AA80, uint deltaZ = 0x0040)
            {
                var commands = new List<ulong> { ColorImage(size, 32, Framebuffer, format), inside };
                if (fill) commands.AddRange(new[] { FillCycle, FillColor(background), Rectangle(0, 0, 124, 124) });
                commands.AddRange(new[]
                {
                    modes, combine, Color(0x3A, 0xC864_2A9F, 0x5A), Color(0x3B, 0x3C90_D071), Color(0x39, blendColor), Color(0x38, 0x2266_EE40),
                    (0x2CUL << 56) | (0xA00UL << 9) | ((ulong)k4 << 9) | k5, (0x2EUL << 56) | (0x1234UL << 16) | deltaZ,
                });
                commands.AddRange(shapes);
                commands.Add(SyncFull);
                cases.Add(new Case(name, Framebuffer, size, 32, commands.ToArray(), Dispute: dispute));
            }

            Draw("one-cycle rectangle in the primitive colour", Modes(), primitive, new[] { Rectangle(9, 14, 99, 77) });
            Draw("one-cycle triangle in the primitive colour", Modes(), primitive, triangle);
            Draw("anti-aliased triangle keeping its coverage", Modes(antialias: true, imageRead: true), primitive, triangle, background: 0x7BDE_7BDF);
            Draw("anti-aliased triangle blended over memory", Modes(antialias: true, imageRead: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0), blend, triangle, background: 0x7BDE_7BDF);
            Draw("blend against memory alpha", Modes(antialias: true, imageRead: true, m1a: 0, m1b: 0, m2a: 1, m2b: 1), blend, triangle, background: 0x7BDF_7BDF);
            Draw("blend against memory alpha with a primitive depth", Modes(antialias: true, imageRead: true, m1a: 0, m1b: 0, m2a: 1, m2b: 1, zSource: true), blend, triangle, background: 0x7BDF_7BDF);
            Draw("two anti-aliased triangles wrapping coverage", Modes(antialias: true, imageRead: true, cvgDest: 1, m2a: 1), blend, triangle.Concat(other).ToArray());
            Draw("two anti-aliased triangles clamping coverage", Modes(antialias: true, imageRead: true, cvgDest: 0, m2a: 1), blend, triangle.Concat(other).ToArray());
            Draw("coverage zapped", Modes(antialias: true, imageRead: true, cvgDest: 2), primitive, triangle.Concat(other).ToArray());
            Draw("coverage saved", Modes(antialias: true, imageRead: true, cvgDest: 3), primitive, triangle.Concat(other).ToArray(), background: 0x0001_0000);
            Draw("colour only where coverage wrapped", Modes(antialias: true, imageRead: true, colorOnCvg: true, m2a: 1), blend, triangle.Concat(other).ToArray());
            Draw("force blend", Modes(forceBlend: true, m1a: 2, m1b: 1, m2a: 3, m2b: 2), blend, triangle);
            Draw("alpha compared against the blend alpha", Modes(alphaCompare: true), blend, triangle);
            Draw("coverage times alpha", Modes(antialias: true, cvgTimesAlpha: true), blend, triangle);
            Draw("alpha taken from coverage", Modes(antialias: true, alphaCvgSelect: true, m1b: 0, m2a: 1), blend, triangle);
            Draw("magic square dither", Modes(rgbDither: 0, alphaDither: 0), blend, triangle);
            Draw("bayer dither with inverted alpha dither", Modes(rgbDither: 1, alphaDither: 1), blend, triangle);
            Draw("the previous pixel's combined colour", Modes(), Combine(subA: 0, subB: 3, mul: 15, add: 3, alphaSubA: 0, alphaSubB: 3, alphaMul: 6, alphaAdd: 3), triangle,
                dispute: "angrylion's one-cycle combined input is the previous pixel's result, which parallel-rdp does not reproduce");
            Draw("key centre, key scale and convert constants", Modes(), Combine(subA: 5, subB: 6, mul: 6, add: 7, alphaSubA: 6, alphaSubB: 3, alphaMul: 5, alphaAdd: 4), triangle);
            Draw("negative convert constants", Modes(), Combine(subA: 3, subB: 7, mul: 15, add: 5, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 3), triangle,
                k4: 0x1C5, k5: 0x1F0);
            Draw("alpha dither without colour dither, at the compare threshold", Modes(rgbDither: 3, alphaDither: 0, alphaCompare: true),
                Combine(subA: 3, subB: 8, mul: 16, add: 3, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 5), triangle, blendColor: 0x7755_AA74);
            Draw("forced blend against memory alpha with a steep primitive depth", Modes(antialias: true, imageRead: true, forceBlend: true, m1a: 0, m1b: 0, m2a: 1, m2b: 1, zSource: true),
                blend, triangle, background: 0x7BDF_7BDF, deltaZ: 0x8000);
            Draw("one-cycle 32-bit image", Modes(antialias: true, imageRead: true, m2a: 1), blend, triangle, size: Bits32, background: 0x1234_5678);
            Draw("one-cycle 8-bit image", Modes(antialias: true, imageRead: true, m2a: 1), blend, triangle, size: Bits8, background: 0x1234_5678);
            // Recorded from the run that found it; the dispute depends on pixel values as well as on these modes - see Mars_RdpCoverage.md §6.
            Draw("8-bit image forcing a blend against memory colour", 0x2F0000C0_F050620CUL, 0x3C35666A_33CDE8F4UL,
                new[]
                {
                    0x08000084_00310013UL, 0x000F0000_000046F1UL, 0x0016CD98_FFFFEDE0UL, 0x00178666_FFFEF777UL,
                    0x08800065_00440000UL, 0x00200000_FFFC26CAUL, 0x001DC000_FFFED4E9UL, 0x001DC000_000021E2UL,
                },
                size: Bits8, fill: false, dispute: "in an 8-bit image, a forced blend with memory colour as exactly one input differs in green bytes");
            Draw("one-cycle 16-bit intensity image", Modes(antialias: true, imageRead: true, m2a: 1), blend, triangle, format: 4, background: 0x1234_5678);
            Draw("one-cycle 4-bit image", Modes(antialias: true), primitive, triangle, size: Bits4, fill: false);

            var random = new Random(0x434F_5652);
            for (int n = 0; n < 150; n++)
            {
                double Coordinate() => random.Next(-8, 136) / 4.0;
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                int size = Pick(Bits16, Bits16, Bits16, Bits32, Bits8);
                int m1a = random.Next(4), m2a = random.Next(4);
                bool forceBlend = Coin();

                // The other disputed combination, which has its one named case above.
                if (size == Bits8 && (m1a == 1) != (m2a == 1)) forceBlend = false;

                ulong modes = Modes(
                    rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3), m1a: m1a, m1b: random.Next(4), m2a: m2a, m2b: random.Next(4),
                    forceBlend: forceBlend, alphaCvgSelect: Coin(), cvgTimesAlpha: Coin(), cvgDest: random.Next(4), colorOnCvg: Coin(), imageRead: Coin(),
                    antialias: Coin(), zSource: Coin(), alphaCompare: random.Next(4) == 0);
                // No combined input: the references disagree about it in this mode, so it has its one named case - see Mars_RdpCoverage.md §6.
                ulong combine = Combine(
                    subA: Pick(3, 4, 5, 6, 9), subB: Pick(3, 4, 5, 6, 7, 10), mul: Pick(3, 4, 5, 6, 10, 11, 12, 14, 15, 20), add: Pick(3, 4, 5, 6, 7),
                    alphaSubA: Pick(3, 4, 5, 6, 7), alphaSubB: Pick(3, 4, 5, 6, 7), alphaMul: Pick(3, 4, 5, 6, 7), alphaAdd: Pick(3, 4, 5, 6, 7));

                var shapes = new List<ulong>(Vertices(Coordinate(), Coordinate(), Coordinate(), Coordinate(), Coordinate(), Coordinate()));
                if (Coin()) shapes.AddRange(Vertices(Coordinate(), Coordinate(), Coordinate(), Coordinate(), Coordinate(), Coordinate()));
                if (random.Next(4) == 0) shapes.Add(Rectangle((uint)random.Next(0, 60), (uint)random.Next(0, 60), (uint)random.Next(60, 124), (uint)random.Next(60, 124)));

                Draw($"random one-cycle case {n}", modes, combine, shapes.ToArray(), size: size, format: size == Bits16 && random.Next(4) == 0 ? 4 : 0,
                    background: (uint)random.Next() * 2 + (uint)random.Next(2), fill: random.Next(5) != 0);
            }

            return cases;
        }

        private static ulong Modes(int rgbDither = 3, int alphaDither = 3, int m1a = 0, int m1b = 0, int m2a = 0, int m2b = 0, bool forceBlend = false,
            bool alphaCvgSelect = false, bool cvgTimesAlpha = false, int cvgDest = 0, bool colorOnCvg = false, bool imageRead = false, bool antialias = false,
            bool zSource = false, bool alphaCompare = false, int zMode = 0, bool zCompare = false, bool zUpdate = false,
            bool perspective = false, bool biLerp0 = false, bool? biLerp1 = null)
        {
            ulong blender = (ulong)(uint)((m1a << 30) | (m1a << 28) | (m1b << 26) | (m1b << 24) | (m2a << 22) | (m2a << 20) | (m2b << 18) | (m2b << 16));
            return (0x2FUL << 56) | ((ulong)rgbDither << 38) | ((ulong)alphaDither << 36) | blender
                | (forceBlend ? 1UL << 14 : 0) | (alphaCvgSelect ? 1UL << 13 : 0) | (cvgTimesAlpha ? 1UL << 12 : 0) | ((ulong)cvgDest << 8)
                | (colorOnCvg ? 1UL << 7 : 0) | (imageRead ? 1UL << 6 : 0) | (antialias ? 1UL << 3 : 0) | (zSource ? 1UL << 2 : 0) | (alphaCompare ? 1UL : 0)
                | ((ulong)zMode << 10) | (zCompare ? 1UL << 4 : 0) | (zUpdate ? 1UL << 5 : 0)
                | (perspective ? 1UL << 51 : 0) | (biLerp0 ? 1UL << 43 : 0) | ((biLerp1 ?? biLerp0) ? 1UL << 42 : 0);
        }

        // Both cycles given the same inputs, since the one-cycle mode reads the second.
        private static ulong Combine(int subA, int subB, int mul, int add, int alphaSubA, int alphaSubB, int alphaMul, int alphaAdd)
        {
            // A selector wider than its field would silently spill into its neighbour's.
            if (subA > 15 || subB > 15 || mul > 31 || add > 7 || alphaSubA > 7 || alphaSubB > 7 || alphaMul > 7 || alphaAdd > 7)
                throw new ArgumentOutOfRangeException(nameof(subA), "a combiner selector does not fit its field");

            ulong high = (ulong)((subA << 20) | (mul << 15) | (alphaSubA << 12) | (alphaMul << 9) | (subA << 5) | mul);
            ulong low = (ulong)(uint)((subB << 28) | (subB << 24) | (alphaSubA << 21) | (alphaMul << 18) | (add << 15) | (alphaSubB << 12) | (alphaAdd << 9) | (add << 6) | (alphaSubB << 3) | alphaAdd);
            return (0x3CUL << 56) | (high << 32) | low;
        }

        private static ulong Color(uint id, uint rgba, uint extra = 0) => ((ulong)id << 56) | ((ulong)extra << 32) | rgba;

        private const uint DepthBuffer = 0x0012_0000;

        // Shaded and depth-tested primitives in the one-cycle mode, over a depth buffer the case clears itself - see Mars_RdpDepth.md §6.
        private static IEnumerable<Case> ShadeAndDepthCases()
        {
            var cases = new List<Case>();
            ulong inside = Scissor(0, 0, 124, 124);
            ulong shade = Combine(subA: 8, subB: 8, mul: 16, add: 4, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 4);
            ulong primitive = Combine(subA: 8, subB: 8, mul: 16, add: 3, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 3);

            var a = new Vertex(3.25, 2.5, 250, 20, 60, 255, 0x08000);
            var b = new Vertex(27.75, 9.75, 10, 240, 90, 128, 0x30000);
            var c = new Vertex(7.5, 28.25, 40, 70, 230, 12, 0x1C000);
            var d = new Vertex(28.5, 1.25, 200, 200, 30, 90, 0x3F000);
            var e = new Vertex(4.75, 17.5, 30, 160, 250, 200, 0x02000);
            var f = new Vertex(24.25, 29.75, 120, 10, 10, 255, 0x12000);

            void Draw(string name, ulong modes, ulong combine, ulong[] shapes, uint depthClear = 0xFFFC_FFFC, uint background = 0x7BDE_7BDF,
                int size = Bits16, uint deltaZ = 0x0040, uint primitiveZ = 0x1234)
            {
                var commands = new List<ulong>
                {
                    ColorImage(size, 32, Framebuffer), inside, FillCycle, FillColor(background), Rectangle(0, 0, 124, 124),
                    ColorImage(Bits16, 32, DepthBuffer), FillColor(depthClear), Rectangle(0, 0, 124, 124), ColorImage(size, 32, Framebuffer),
                    (0x3EUL << 56) | DepthBuffer,
                    modes, combine, Color(0x3A, 0xC864_2A9F, 0x5A), Color(0x3B, 0x3C90_D071), Color(0x39, 0x7755_AA80), Color(0x38, 0x2266_EE40),
                    (0x2EUL << 56) | ((ulong)primitiveZ << 16) | deltaZ,
                };
                commands.AddRange(shapes);
                commands.Add(SyncFull);
                cases.Add(new Case(name, Framebuffer, size, 32, commands.ToArray()));
            }

            Draw("Gouraud-shaded triangle", Modes(), shade, Shaded(0x0C, a, b, c));
            Draw("anti-aliased shaded triangle, corrected at its edges", Modes(antialias: true, imageRead: true, m2a: 1), shade, Shaded(0x0C, a, b, c));
            Draw("shade alpha as the blend weight", Modes(antialias: true, forceBlend: true, m1b: 2, m2a: 1, m2b: 0), shade, Shaded(0x0C, a, b, c));
            Draw("depth-tested triangle over a cleared depth buffer", Modes(zCompare: true, zUpdate: true), shade, Shaded(0x0D, a, b, c));
            Draw("two triangles crossing in depth", Modes(zCompare: true, zUpdate: true), shade, Shaded(0x0D, a, b, c).Concat(Shaded(0x0D, d, e, f)).ToArray());
            Draw("two anti-aliased triangles crossing in depth", Modes(antialias: true, imageRead: true, m2a: 1, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a, b, c).Concat(Shaded(0x0D, d, e, f)).ToArray());
            Draw("interpenetrating depth mode", Modes(antialias: true, imageRead: true, m2a: 1, zMode: 1, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a, b, c).Concat(Shaded(0x0D, d, e, f)).ToArray());
            Draw("transparent depth mode", Modes(antialias: true, imageRead: true, m2a: 1, zMode: 2, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a, b, c).Concat(Shaded(0x0D, d, e, f)).ToArray());
            // Decal mode passes nothing over a cleared buffer, so the surface under the decal is drawn opaque first.
            ulong decal = Modes(antialias: true, imageRead: true, zMode: 3, zCompare: true, zUpdate: true);
            Draw("decal depth mode over the same triangle", Modes(antialias: true, imageRead: true, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a, b, c).Append(decal).Concat(Shaded(0x0D, a with { R = 0, G = 0 }, b with { R = 0 }, c with { B = 0 })).ToArray());
            Draw("depth-only triangle with flat colour", Modes(zCompare: true, zUpdate: true), primitive, Shaded(0x09, a, b, c).Concat(Shaded(0x09, d, e, f)).ToArray());
            Draw("depth written without comparing", Modes(zUpdate: true), shade, Shaded(0x0D, a, b, c).Concat(Shaded(0x0D, d, e, f)).ToArray());
            Draw("primitive depth for rectangles", Modes(zSource: true, zCompare: true, zUpdate: true), primitive,
                new[] { Rectangle(8, 8, 90, 70), Color(0x2E, 0x0800_0100), Rectangle(40, 30, 120, 110) }, primitiveZ: 0x2000);
            Draw("blend shifts from stored depth slopes", Modes(antialias: true, imageRead: true, m1a: 0, m1b: 0, m2a: 1, m2b: 1, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a, b, c).Concat(Shaded(0x0D, d, e, f)).ToArray());
            Draw("depth buffer cleared to a low-exponent value", Modes(zCompare: true, zUpdate: true), shade, Shaded(0x0D, a with { Z = 0x0100 }, b with { Z = 0x0200 }, c with { Z = 0x0080 }),
                depthClear: 0x0400_0400);
            Draw("depth buffer holding the coplanar slope", Modes(antialias: true, imageRead: true, m2a: 1, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a with { Z = 0x0100 }, b with { Z = 0x0100 }, c with { Z = 0x0100 }), depthClear: 0x0103_0103);
            Draw("shaded triangle in a 32-bit image", Modes(antialias: true, imageRead: true, m2a: 1), shade, Shaded(0x0C, a, b, c), size: Bits32, background: 0x1234_5678);
            Draw("steep depth slope", Modes(zCompare: true, zUpdate: true), shade, Shaded(0x0D, a with { Z = 0 }, b with { Z = 0x3FFFF }, c with { Z = 0x100 }));
            Draw("depth beyond its range", Modes(antialias: true, imageRead: true, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a with { Z = 0x48000 }, b with { Z = 0x70000 }, c with { Z = -0x1000 }));
            Draw("depth beyond its range, transparent mode", Modes(antialias: true, imageRead: true, zMode: 2, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a with { Z = 0x48000 }, b with { Z = 0x70000 }, c with { Z = 0x50000 }));
            Draw("decal behind the stored surface", Modes(antialias: true, imageRead: true, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a with { Z = 0x10000 }, b with { Z = 0x10000 }, c with { Z = 0x10000 }).Append(decal)
                    .Concat(Shaded(0x0D, d with { Z = 0x10000 }, e with { Z = 0x18000 }, f with { Z = 0x10040 })).ToArray());
            Draw("stored slope widened at low precision", Modes(zMode: 3, zCompare: true, zUpdate: true), shade,
                Shaded(0x0D, a with { Z = 0x4030 }, b with { Z = 0x3FD0 }, c with { Z = 0x4010 }), depthClear: 0x0400_0400);

            var random = new Random(0x5A44_4550);
            for (int n = 0; n < 150; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;
                Vertex Point() => new(random.Next(-8, 136) / 4.0, random.Next(-8, 136) / 4.0, random.Next(256), random.Next(256), random.Next(256), random.Next(256),
                    random.Next(0x40000));

                int size = Pick(Bits16, Bits16, Bits16, Bits32);
                ulong modes = Modes(
                    rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3), m1a: random.Next(4), m1b: random.Next(4), m2a: random.Next(4), m2b: random.Next(4),
                    forceBlend: Coin(), alphaCvgSelect: Coin(), cvgTimesAlpha: Coin(), cvgDest: random.Next(4), colorOnCvg: Coin(), imageRead: Coin(),
                    antialias: Coin(), zSource: random.Next(4) == 0, alphaCompare: random.Next(4) == 0, zMode: random.Next(4), zCompare: Coin(), zUpdate: Coin());
                ulong combine = Combine(
                    subA: Pick(3, 4, 5, 6, 9), subB: Pick(3, 4, 5, 6, 7, 10), mul: Pick(3, 4, 5, 6, 10, 11, 12, 14, 15, 20), add: Pick(3, 4, 5, 6, 7),
                    alphaSubA: Pick(3, 4, 5, 6, 7), alphaSubB: Pick(3, 4, 5, 6, 7), alphaMul: Pick(3, 4, 5, 6, 7), alphaAdd: Pick(3, 4, 5, 6, 7));

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 4); t < count; t++) shapes.AddRange(Shaded(Pick(0x08, 0x09, 0x0C, 0x0D), Point(), Point(), Point()));

                Draw($"random shade and depth case {n}", modes, combine, shapes.ToArray(), size: size,
                    depthClear: Coin() ? 0xFFFC_FFFC : (uint)random.Next(0x10000) * 0x10001, background: (uint)random.Next() * 2 + (uint)random.Next(2),
                    deltaZ: (uint)random.Next(0x10000), primitiveZ: (uint)random.Next(0x8000));
            }

            return cases;
        }

        private readonly record struct Vertex(double X, double Y, double R, double G, double B, double A, double Z, double S = 0, double T = 0, double W = 1);

        private const uint TextureImage = 0x0020_0000;

        // Point-sampled textures in the one-cycle mode: every format, tile shift, mask, mirror and clamp, tile and block loads - see Mars_RdpTextures.md §7.
        private static IEnumerable<Case> TextureCases()
        {
            var cases = new List<Case>();
            ulong inside = Scissor(0, 0, 124, 124);
            ulong texel0 = Combine(subA: 8, subB: 8, mul: 16, add: 1, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1);
            byte[] image = Pattern(0x5445_5831, 0x1000);

            void Draw(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, int size = Bits16, string? dispute = null, bool crossChecked = true,
                (uint, byte[])[]? uploads = null)
            {
                var commands = new List<ulong>
                {
                    ColorImage(size, 32, Framebuffer), inside, FillCycle, FillColor(0x7BDE_7BDF), Rectangle(0, 0, 124, 124),
                    ColorImage(Bits16, 32, DepthBuffer), FillColor(0xFFFC_FFFC), Rectangle(0, 0, 124, 124), ColorImage(size, 32, Framebuffer),
                    (0x3EUL << 56) | DepthBuffer, (0x2CUL << 56) | 0x0B89_1A3C_0156B3CUL & 0x00FF_FFFF_FFFF_FFFF,
                };
                commands.AddRange(setup);
                commands.AddRange(new[] { modes, combine, Color(0x3A, 0xC864_2A9F, 0x5A), Color(0x3B, 0x3C90_D071), Color(0x39, 0x7755_AA80) });
                commands.AddRange(shapes);
                commands.Add(SyncFull);
                cases.Add(new Case(name, Framebuffer, size, 32, commands.ToArray(), Dispute: dispute, CrossChecked: crossChecked)
                {
                    Uploads = new[] { (TextureImage, image) }.Concat(uploads ?? Array.Empty<(uint, byte[])>()).ToArray(),
                });
            }

            // A 4-bit tile loads from an 8-bit image unless told otherwise, since a 4-bit image stops the load in both references - see Mars_RdpTextures.md §7.
            ulong[] Loaded(int format, int size, uint width = 16, uint height = 16, int tile = 0, int memory = 0, int shiftS = 0, int shiftT = 0,
                int maskS = 0, int maskT = 0, bool mirrorS = false, bool mirrorT = false, bool clampS = false, bool clampT = false, uint offset = 0, int? imageSize = null,
                int palette = 0, uint column = 0, uint row = 0)
            {
                int slots = format == 1 ? (int)(width + 1) / 2 : size switch { 0 => (int)(width + 3) / 4, 1 => (int)(width + 1) / 2, _ => (int)width };
                int line = ((slots + 3) / 4) & 0x1FF;
                return new[]
                {
                    TextureImageCommand(format, imageSize ?? Math.Max(size, Bits8), (int)width, TextureImage + offset),
                    TileCommand(tile, format, size, line, memory, palette, clampS, mirrorS, maskS, shiftS, clampT, mirrorT, maskT, shiftT),
                    LoadCommand(0x34, tile, column << 2, row << 2, (column + width - 1) << 2, (row + height - 1) << 2),
                };
            }

            ulong[] rectangle = TextureRectangle(false, 0, 12, 10, 100, 90, 0, 0, 0x0400, 0x0400);
            ulong rectModes = Modes(biLerp0: true);

            foreach (var (label, format, size) in new[]
            {
                ("RGBA16", 0, 2), ("RGBA32", 0, 3), ("RGBA4", 0, 0), ("RGBA8", 0, 1), ("YUV16", 1, 2), ("CI4", 2, 0), ("CI8", 2, 1), ("CI16", 2, 2),
                ("IA4", 3, 0), ("IA8", 3, 1), ("IA16", 3, 2), ("I4", 4, 0), ("I8", 4, 1), ("I16", 4, 2),
            })
            {
                Draw($"point-sampled {label} texture rectangle", rectModes, texel0, Loaded(format, size), rectangle);
            }

            Draw("point-sampled 4-bit format 5 texture rectangle", rectModes, texel0, Loaded(5, 0), rectangle,
                dispute: "parallel-rdp samples nothing from texture formats 5 to 7, which angrylion reads as intensity");
            Draw("load from a 4-bit texture image", rectModes, texel0, Loaded(0, 0, imageSize: Bits4), Array.Empty<ulong>(), crossChecked: false);
            Draw("8-bit YUV texture rectangle", rectModes, texel0, Loaded(1, 1), rectangle, crossChecked: false);
            Draw("32-bit intensity-alpha texture rectangle", rectModes, texel0, Loaded(3, 3), rectangle, crossChecked: false);

            Draw("flipped texture rectangle", rectModes, texel0, Loaded(0, 2), TextureRectangle(true, 0, 12, 10, 100, 90, 0x20, 0x40, 0x0300, 0x0500));
            Draw("YUV converted without bilinear filtering", Modes(), texel0, Loaded(1, 2), rectangle);
            Draw("tile shift, mask and mirror", rectModes, texel0, Loaded(0, 2, shiftS: 1, shiftT: 12, maskS: 3, maskT: 2, mirrorS: true), rectangle);
            Draw("tile clamped against its size", rectModes, texel0,
                Loaded(0, 2, clampS: true, clampT: true, maskS: 4, maskT: 4).Append(TileSizeCommand(0, 9, 13, 37, 29)).ToArray(),
                TextureRectangle(false, 0, 4, 4, 120, 120, -0x0100, -0x0080, 0x0300, 0x0280));
            Draw("block load", rectModes, texel0,
                new[] { TextureImageCommand(0, 2, 16, TextureImage), TileCommand(0, 0, 2, 4, 0), (0x33UL << 56) | (0UL << 44) | (3UL << 32) | (15UL << 12) | 0x0400 }, rectangle);
            Draw("tile larger than texture memory", rectModes, texel0, Loaded(0, 2, width: 64, height: 40, memory: 0x80), rectangle);
            Draw("32-bit tile reaching the second half of texture memory", rectModes, texel0, Loaded(0, 3, width: 37, height: 38, memory: 0x38), rectangle,
                dispute: "angrylion folds a 32-bit tile's rows past the first half of texture memory back into it, and parallel-rdp does not");
            Draw("YUV tile reaching the second half of texture memory", rectModes, texel0, Loaded(1, 2, width: 40, height: 44, memory: 0xC0), rectangle);
            Draw("second tile at an offset in texture memory", rectModes, texel0, Loaded(3, 1, tile: 3, memory: 0x40, offset: 0x200),
                TextureRectangle(false, 3, 12, 10, 100, 90, 0, 0, 0x0400, 0x0400));
            Draw("next pixel's texel", rectModes, Combine(subA: 1, subB: 2, mul: 3, add: 2, alphaSubA: 1, alphaSubB: 2, alphaMul: 3, alphaAdd: 2), Loaded(0, 2),
                TextureRectangle(false, 0, 12, 10, 100, 90, 0x10, 0x30, 0x0200, 0x0300));
            Draw("next pixel's texel with the second cycle unfiltered", Modes(biLerp0: true, biLerp1: false),
                Combine(subA: 1, subB: 2, mul: 3, add: 2, alphaSubA: 1, alphaSubB: 2, alphaMul: 3, alphaAdd: 2), Loaded(0, 2),
                TextureRectangle(false, 0, 12, 10, 100, 90, 0x10, 0x30, 0x0200, 0x0300),
                dispute: "parallel-rdp filters the one-cycle mode's next-pixel texel by the second cycle's settings, and angrylion by the first's");
            Draw("texture rectangle in a 32-bit image", rectModes, texel0, Loaded(0, 3), rectangle, size: Bits32);

            var p = new Vertex(3.25, 2.5, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var q = new Vertex(27.75, 9.75, 10, 240, 90, 128, 0x30000, S: 15, T: 2, W: 0.55);
            var r = new Vertex(7.5, 28.25, 40, 70, 230, 12, 0x1C000, S: 4, T: 15, W: 0.8);
            Draw("affine textured triangle", rectModes, texel0, Loaded(0, 2), Shaded(0x0A, p with { W = 1 }, q with { W = 1 }, r with { W = 1 }));
            Draw("perspective textured triangle", Modes(perspective: true, biLerp0: true), texel0, Loaded(0, 2), Shaded(0x0A, p, q, r));
            Draw("perspective textured, shaded, depth-tested triangle", Modes(perspective: true, biLerp0: true, zCompare: true, zUpdate: true, antialias: true, imageRead: true, m2a: 1),
                Combine(subA: 1, subB: 8, mul: 4, add: 7, alphaSubA: 1, alphaSubB: 7, alphaMul: 4, alphaAdd: 7), Loaded(0, 2), Shaded(0x0F, p, q, r));
            Draw("textured triangle with texel coordinates out of range", Modes(perspective: true, biLerp0: true), texel0, Loaded(3, 2, maskS: 4, mirrorT: true, maskT: 3),
                Shaded(0x0B, p with { S = -20, T = 40 }, q with { S = 300, W = 0.1 }, r with { T = -500, W = 1.5 }));
            Draw("next pixel's texel across a triangle's rows", Modes(perspective: true, biLerp0: true),
                Combine(subA: 2, subB: 1, mul: 3, add: 1, alphaSubA: 2, alphaSubB: 1, alphaMul: 3, alphaAdd: 1), Loaded(0, 2), Shaded(0x0A, p, q, r));

            // Each case below exists because a breakage of the rule it names survived every case above - see Mars_RdpTextures.md §7.5.
            Draw("texel alpha of an RGBA16 tile as the multiplier", rectModes,
                Combine(subA: 1, subB: 8, mul: 8, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1), Loaded(0, 2), rectangle);
            Draw("4-bit colour-indexed tile with a palette number", rectModes, texel0, Loaded(2, 0, palette: 11), rectangle);
            Draw("4-bit YUV texture rectangle", rectModes, texel0, Loaded(1, 0), rectangle, crossChecked: false);
            Draw("32-bit YUV texture rectangle", rectModes, Combine(subA: 1, subB: 8, mul: 8, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1),
                Loaded(1, 3), rectangle, crossChecked: false);
            Draw("tile clamped exactly at its corner", rectModes, texel0,
                Loaded(0, 2, clampS: true, clampT: true, maskS: 4, maskT: 4).Append(TileSizeCommand(0, 11, 7, 36, 30)).ToArray(),
                TextureRectangle(false, 0, 8, 8, 120, 120, 228, 180, 0x0080, 0x0080));
            Draw("mirror at bit ten for a mask past ten", rectModes, texel0,
                Loaded(0, 2, width: 64, height: 40, maskS: 12, maskT: 12, mirrorS: true, mirrorT: true).Append(TileSizeCommand(0, 0xFFC, 0xFFC, 0xFFF, 0xFFF)).ToArray(),
                TextureRectangle(false, 0, 8, 8, 120, 120, unchecked((short)0xB500), unchecked((short)0xB500), 0x0400, 0x0400));
            Draw("perspective triangle past the divider's range", Modes(perspective: true, biLerp0: true), texel0, Loaded(0, 2, maskS: 4, maskT: 4),
                Shaded(0x0A, p with { S = -3000, T = 2500, W = 0.1 }, q with { S = 3000, T = -2000, W = 0.1 }, r with { S = 0, T = 0, W = 0.12 }));
            Draw("perspective triangle with a w of one", Modes(perspective: true, biLerp0: true), texel0, Loaded(0, 2, maskS: 4, maskT: 4),
                Shaded(0x0A, p with { S = -2000, T = 0, W = 1.0 / 0x7FFF }, q with { S = 2000, T = -2000, W = 1.0 / 0x7FFF }, r with { S = 0, T = 2000, W = 1.0 / 0x7FFF }));
            Draw("perspective triangle with w in the seventh reciprocal segment", Modes(perspective: true, biLerp0: true), texel0, Loaded(0, 2, maskS: 4, maskT: 4),
                Shaded(0x0A, p with { S = 0, T = 0, W = 17920.5 / 0x7FFF }, q with { S = 1000, T = 400, W = 17920.5 / 0x7FFF }, r with { S = 300, T = 1000, W = 17920.5 / 0x7FFF }));
            Draw("next pixel's texel alpha as its only reader", rectModes,
                Combine(subA: 1, subB: 8, mul: 9, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 7), Loaded(0, 2),
                TextureRectangle(false, 0, 12, 10, 100, 90, 0x10, 0x30, 0x0200, 0x0300));
            Draw("textured triangle clipped far from its major edge", rectModes, texel0, Loaded(0, 2, maskS: 4, maskT: 4),
                Shaded(0x0A, p with { X = -1900, Y = 2, S = 0, T = 0, W = 1 }, q with { X = 30, Y = 6, S = 1000, T = 37, W = 1 }, r with { X = 28, Y = 29, S = 971, T = 999, W = 1 }));
            Draw("texture rectangle with fractional steps", rectModes, texel0, Loaded(0, 2), TextureRectangle(false, 0, 8, 8, 120, 120, 0x0013, 0x0027, 0x015F, 0x01E7));
            Draw("tile loaded from an odd column and row", rectModes, texel0, Loaded(0, 2, memory: 0x60, column: 3, row: 1), TextureRectangle(false, 0, 8, 8, 120, 120, 3 << 5, 1 << 5, 0x0400, 0x0400));
            Draw("32-bit tile loaded from an odd column", rectModes, texel0, Loaded(0, 3, memory: 0x20, column: 3, row: 1), TextureRectangle(false, 0, 8, 8, 120, 120, 3 << 5, 1 << 5, 0x0400, 0x0400));
            Draw("tile load whose only row has no sub-row", rectModes, texel0,
                new[] { TextureImageCommand(0, 2, 16, TextureImage + 0x40), TileCommand(0, 0, 2, 4, 0x1F0), LoadCommand(0x34, 0, 0, 3, 15 << 2, 3) }, rectangle,
                dispute: "angrylion loads one step of a tile whose only row has no sub-row inside it, and parallel-rdp the whole row");
            Draw("block load from a column past 2047", rectModes, texel0,
                new[] { TextureImageCommand(0, 2, 16, TextureImage), TileCommand(0, 0, 2, 4, 0x1C0), LoadCommand(0x33, 0, 0x900, 120, 0x90F, 0x0400) }, rectangle,
                dispute: "angrylion takes a block load's left column as signed when placing the image pointer, and parallel-rdp does not");
            Draw("tile loaded past the end of RDRAM", rectModes, texel0, Loaded(0, 2, memory: 0x140, offset: 0x5F_FFF0), rectangle);
            Draw("tile loaded across the end of addressable memory", rectModes, texel0, Loaded(0, 2, memory: 0x100, offset: 0xDF_FFF8), rectangle,
                dispute: "angrylion wraps an image read past sixteen megabytes to address zero, and parallel-rdp reads zero",
                uploads: new[] { (0x0000_0000u, Pattern(0x5445_5833, 0x40)) });

            // Random cases keep to what both references model: formats 0 to 4, YUV only at 16 bits, 32 bits only as RGBA and within half of texture memory.
            var random = new Random(0x5445_5832);
            for (int n = 0; n < 150; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                int format = random.Next(5), tile = random.Next(8);
                int size = format == 1 ? Bits16 : format == 0 ? random.Next(4) : random.Next(3);
                uint width = (uint)random.Next(1, 48), height = (uint)random.Next(1, 48);
                int memory = random.Next(0x200);
                if (size == Bits32)
                {
                    uint line = (width + 3) / 4;
                    height = Math.Min(height, 0x100 / line);
                    memory = random.Next((int)(0x100 - line * height) + 1);
                }

                var setup = new List<ulong>(Loaded(format, size, width, height, tile, memory, random.Next(16), random.Next(16), random.Next(16), random.Next(16),
                    Coin(), Coin(), Coin(), Coin(), (uint)random.Next(0x100) * 4));
                if (Coin()) setup.Add(TileSizeCommand(tile, (uint)random.Next(0x100), (uint)random.Next(0x100), (uint)random.Next(0x400), (uint)random.Next(0x400)));

                ulong modes = Modes(
                    rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3), m1a: random.Next(4), m1b: random.Next(4), m2a: random.Next(4), m2b: random.Next(4),
                    forceBlend: random.Next(4) == 0, cvgDest: random.Next(4), imageRead: Coin(), antialias: Coin(), alphaCompare: random.Next(4) == 0,
                    zMode: random.Next(4), zCompare: Coin(), zUpdate: Coin(), perspective: Coin(), biLerp0: random.Next(4) != 0);
                ulong combine = Combine(
                    subA: Pick(1, 2, 3, 4, 5, 6), subB: Pick(1, 2, 3, 4, 5, 7), mul: Pick(1, 2, 3, 4, 8, 9, 10, 11, 15, 20), add: Pick(1, 2, 3, 4, 5, 7),
                    alphaSubA: Pick(1, 2, 3, 4, 7), alphaSubB: Pick(1, 2, 3, 4, 7), alphaMul: Pick(1, 2, 3, 4, 7), alphaAdd: Pick(1, 2, 3, 4, 7));

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 3); t < count; t++)
                {
                    if (Coin())
                    {
                        uint left = (uint)random.Next(0, 100), top = (uint)random.Next(0, 100);
                        shapes.AddRange(TextureRectangle(Coin(), tile, left, top, left + (uint)random.Next(4, 60), top + (uint)random.Next(4, 60),
                            (short)random.Next(-0x400, 0x800), (short)random.Next(-0x400, 0x800), (short)random.Next(-0x800, 0x800), (short)random.Next(-0x800, 0x800)));
                    }
                    else
                    {
                        Vertex Point() => new(random.Next(-8, 136) / 4.0, random.Next(-8, 136) / 4.0, random.Next(256), random.Next(256), random.Next(256), random.Next(256),
                            random.Next(0x40000), random.Next(-16, 64), random.Next(-16, 64), 0.25 + 0.75 * random.NextDouble());
                        shapes.AddRange(Shaded(Pick(0x0A, 0x0B, 0x0E, 0x0F) | (tile << 16), Point(), Point(), Point()));
                    }
                }

                Draw($"random texture case {n}", modes, combine, setup.ToArray(), shapes.ToArray(), size: Pick(Bits16, Bits16, Bits32));
            }

            return cases;
        }

        private static byte[] Pattern(int seed, int length)
        {
            var bytes = new byte[length];
            new Random(seed).NextBytes(bytes);
            return bytes;
        }

        private static ulong TextureImageCommand(int format, int size, int width, uint address) =>
            (0x3DUL << 56) | ((ulong)format << 53) | ((ulong)size << 51) | ((ulong)(width - 1) << 32) | address;

        private static ulong TileCommand(int tile, int format, int size, int line, int memory, int palette = 0, bool clampS = false, bool mirrorS = false,
            int maskS = 0, int shiftS = 0, bool clampT = false, bool mirrorT = false, int maskT = 0, int shiftT = 0) =>
            (0x35UL << 56) | ((ulong)(uint)format << 53) | ((ulong)(uint)size << 51) | ((ulong)(uint)line << 41) | ((ulong)(uint)memory << 32) | ((ulong)(uint)tile << 24)
            | ((ulong)(uint)palette << 20) | (clampT ? 1UL << 19 : 0) | (mirrorT ? 1UL << 18 : 0) | ((ulong)(uint)maskT << 14) | ((ulong)(uint)shiftT << 10)
            | (clampS ? 1UL << 9 : 0) | (mirrorS ? 1UL << 8 : 0) | ((ulong)(uint)maskS << 4) | (uint)shiftS;

        private static ulong LoadCommand(ulong id, int tile, uint sl, uint tl, uint sh, uint th) =>
            (id << 56) | ((ulong)sl << 44) | ((ulong)tl << 32) | ((ulong)tile << 24) | ((ulong)sh << 12) | th;

        private static ulong TileSizeCommand(int tile, uint sl, uint tl, uint sh, uint th) => LoadCommand(0x32, tile, sl, tl, sh, th);

        private static ulong[] TextureRectangle(bool flip, int tile, uint left, uint top, uint right, uint bottom, short s, short t, short dsdx, short dtdy) => new[]
        {
            ((flip ? 0x25UL : 0x24UL) << 56) | ((ulong)right << 44) | ((ulong)bottom << 32) | ((ulong)tile << 24) | ((ulong)left << 12) | top,
            ((ulong)(ushort)s << 48) | ((ulong)(ushort)t << 32) | ((ulong)(ushort)dsdx << 16) | (ushort)dtdy,
        };

        // Edges from Vertices, and each attribute as a plane through the three vertices, taken at the major edge's first row - see Mars_RdpDepth.md §1.
        private static ulong[] Shaded(int id, Vertex v1, Vertex v2, Vertex v3)
        {
            ulong[] words = Vertices(v1.X, v1.Y, v2.X, v2.Y, v3.X, v3.Y, id & 0x3F);
            words[0] |= (ulong)((id >> 16) & 7) << 48;
            var sorted = new[] { v1, v2, v3 }.OrderBy(v => v.Y).ToArray();
            Vertex top = sorted[0], bottom = sorted[2];

            double major = bottom.Y > top.Y ? (bottom.X - top.X) / (bottom.Y - top.Y) : 0;
            double rowTop = Math.Floor(top.Y);
            double xh = top.X + major * (rowTop - top.Y);
            double area = (v2.X - v1.X) * (v3.Y - v1.Y) - (v3.X - v1.X) * (v2.Y - v1.Y);

            (int Value, int Dx, int De, int Dy) Plane(Func<Vertex, double> attribute, double scale)
            {
                double dx = 0, dy = 0;
                if (Math.Abs(area) > 1e-9)
                {
                    dx = ((attribute(v2) - attribute(v1)) * (v3.Y - v1.Y) - (attribute(v3) - attribute(v1)) * (v2.Y - v1.Y)) / area;
                    dy = ((attribute(v3) - attribute(v1)) * (v2.X - v1.X) - (attribute(v2) - attribute(v1)) * (v3.X - v1.X)) / area;
                }

                double value = attribute(top) + dx * (xh - top.X) + dy * (rowTop - top.Y);
                return (Fixed(value * scale), Fixed(dx * scale), Fixed((dy + dx * major) * scale), Fixed(dy * scale));
            }

            int at = 4;
            if ((id & 4) != 0)
            {
                var channels = new[] { Plane(v => v.R, 65536), Plane(v => v.G, 65536), Plane(v => v.B, 65536), Plane(v => v.A, 65536) };
                ulong Pack(Func<(int Value, int Dx, int De, int Dy), int> part, bool high) =>
                    channels.Aggregate(0UL, (word, channel) => (word << 16) | (ushort)(high ? part(channel) >> 16 : part(channel)));

                words[at++] = Pack(p => p.Value, true);
                words[at++] = Pack(p => p.Dx, true);
                words[at++] = Pack(p => p.Value, false);
                words[at++] = Pack(p => p.Dx, false);
                words[at++] = Pack(p => p.De, true);
                words[at++] = Pack(p => p.Dy, true);
                words[at++] = Pack(p => p.De, false);
                words[at++] = Pack(p => p.Dy, false);
            }

            if ((id & 2) != 0)
            {
                var channels = new[] { Plane(v => v.S * v.W * 32, 65536), Plane(v => v.T * v.W * 32, 65536), Plane(v => v.W * 0x7FFF, 65536), (0, 0, 0, 0) };
                ulong Pack(Func<(int Value, int Dx, int De, int Dy), int> part, bool high) =>
                    channels.Aggregate(0UL, (word, channel) => (word << 16) | (ushort)(high ? part(channel) >> 16 : part(channel)));

                words[at++] = Pack(p => p.Value, true);
                words[at++] = Pack(p => p.Dx, true);
                words[at++] = Pack(p => p.Value, false);
                words[at++] = Pack(p => p.Dx, false);
                words[at++] = Pack(p => p.De, true);
                words[at++] = Pack(p => p.Dy, true);
                words[at++] = Pack(p => p.De, false);
                words[at++] = Pack(p => p.Dy, false);
            }

            if ((id & 1) != 0)
            {
                var depth = Plane(v => v.Z, 8192);
                words[^2] = ((ulong)(uint)depth.Value << 32) | (uint)depth.Dx;
                words[^1] = ((ulong)(uint)depth.De << 32) | (uint)depth.Dy;
            }

            return words;
        }

        private static int Fixed(double value) => (int)Math.Clamp(Math.Round(value), int.MinValue, int.MaxValue);

        private static ulong Scissor(uint left, uint top, uint right, uint bottom, bool field = false, bool keepOdd = false) =>
            (0x2DUL << 56) | ((ulong)left << 44) | ((ulong)top << 32) | ((ulong)right << 12) | bottom
            | (field ? 1UL << 25 : 0) | (keepOdd ? 1UL << 24 : 0);

        private static ulong FillColor(uint color) => (0x37UL << 56) | color;

        // A plain triangle setup from three vertices; any well-formed words are a valid test, since both sides read the same ones.
        private static ulong[] Vertices(double x1, double y1, double x2, double y2, double x3, double y3, int id = 0x08,
            bool invertMajor = false, bool swapTopAndMiddle = false)
        {
            var sorted = new[] { (X: x1, Y: y1), (X: x2, Y: y2), (X: x3, Y: y3) }.OrderBy(v => v.Y).ToArray();
            var (top, middle, bottom) = (sorted[0], sorted[1], sorted[2]);

            double major = bottom.Y > top.Y ? (bottom.X - top.X) / (bottom.Y - top.Y) : 0;
            double upper = middle.Y > top.Y ? (middle.X - top.X) / (middle.Y - top.Y) : 0;
            double lower = bottom.Y > middle.Y ? (bottom.X - middle.X) / (bottom.Y - middle.Y) : 0;

            int yh = (int)Math.Floor(top.Y * 4), ym = (int)Math.Floor(middle.Y * 4), yl = (int)Math.Floor(bottom.Y * 4);
            double rowTop = Math.Floor(top.Y);

            double xh = top.X + major * (rowTop - top.Y);
            double xm = top.X + upper * (rowTop - top.Y);
            double xl = middle.X + lower * (ym / 4.0 - middle.Y);

            bool majorOnLeft = middle.X > top.X + major * (middle.Y - top.Y);
            if (invertMajor) majorOnLeft = !majorOnLeft;
            if (swapTopAndMiddle) (yh, ym) = (ym, yh);

            var words = new ulong[EmuSen.Cores.Nintendo.Mars.Rdp.Rdp.Length((uint)id)];
            words[0] = ((ulong)id << 56) | (majorOnLeft ? 1UL << 55 : 0)
                | ((ulong)(uint)(yl & 0x3FFF) << 32) | ((ulong)(uint)(ym & 0x3FFF) << 16) | (uint)(yh & 0x3FFF);
            words[1] = Edge(xl, lower);
            words[2] = Edge(xh, major);
            words[3] = Edge(xm, upper);

            for (int i = 4; i < words.Length; i++) words[i] = 0x0123_4567_89AB_CDEFUL * (ulong)i;
            return words;
        }

        private static ulong[] Edges(bool majorOnLeft, int yh, int ym, int yl, int xh, int dxhdy, int xm, int dxmdy, int xl, int dxldy) => new[]
        {
            (0x08UL << 56) | (majorOnLeft ? 1UL << 55 : 0) | ((ulong)(uint)(yl & 0x3FFF) << 32) | ((ulong)(uint)(ym & 0x3FFF) << 16) | (uint)(yh & 0x3FFF),
            ((ulong)(uint)xl << 32) | (uint)dxldy,
            ((ulong)(uint)xh << 32) | (uint)dxhdy,
            ((ulong)(uint)xm << 32) | (uint)dxmdy,
        };

        private static ulong Edge(double x, double slope) =>
            ((ulong)(uint)(int)Math.Round(x * 65536) << 32) | (uint)(int)Math.Round(Math.Clamp(slope, -8192, 8191) * 65536);

        private static ulong Rectangle(uint left, uint top, uint right, uint bottom) =>
            (0x36UL << 56) | ((ulong)right << 44) | ((ulong)bottom << 32) | ((ulong)left << 12) | top;
    }
}
