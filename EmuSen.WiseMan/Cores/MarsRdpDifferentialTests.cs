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
            cases.AddRange(FilterCases());
            cases.AddRange(LodCases());
            cases.AddRange(TwoCycleCases());
            cases.AddRange(CopyCases());
            cases.AddRange(ChromaKeyCases());
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
            bool perspective = false, bool biLerp0 = false, bool? biLerp1 = null, bool palette = false, bool paletteIa = false, bool sampleFour = false, bool midTexel = false,
            bool lod = false, bool sharpen = false, bool detail = false, int cycle = 0, int? m1a1 = null, int? m1b1 = null, int? m2a1 = null, int? m2b1 = null,
            bool convertOne = false, bool keyEnabled = false)
        {
            ulong blender = (ulong)(uint)((m1a << 30) | ((m1a1 ?? m1a) << 28) | (m1b << 26) | ((m1b1 ?? m1b) << 24) | (m2a << 22) | ((m2a1 ?? m2a) << 20)
                | (m2b << 18) | ((m2b1 ?? m2b) << 16));
            return (0x2FUL << 56) | ((ulong)rgbDither << 38) | ((ulong)alphaDither << 36) | blender
                | (forceBlend ? 1UL << 14 : 0) | (alphaCvgSelect ? 1UL << 13 : 0) | (cvgTimesAlpha ? 1UL << 12 : 0) | ((ulong)cvgDest << 8)
                | (colorOnCvg ? 1UL << 7 : 0) | (imageRead ? 1UL << 6 : 0) | (antialias ? 1UL << 3 : 0) | (zSource ? 1UL << 2 : 0) | (alphaCompare ? 1UL : 0)
                | ((ulong)zMode << 10) | (zCompare ? 1UL << 4 : 0) | (zUpdate ? 1UL << 5 : 0)
                | (perspective ? 1UL << 51 : 0) | (biLerp0 ? 1UL << 43 : 0) | ((biLerp1 ?? biLerp0) ? 1UL << 42 : 0)
                | (palette ? 1UL << 47 : 0) | (paletteIa ? 1UL << 46 : 0) | (sampleFour ? 1UL << 45 : 0) | (midTexel ? 1UL << 44 : 0)
                | (lod ? 1UL << 48 : 0) | (sharpen ? 1UL << 49 : 0) | (detail ? 1UL << 50 : 0) | ((ulong)cycle << 52) | (convertOne ? 1UL << 41 : 0) | (keyEnabled ? 1UL << 40 : 0);
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

        // Each cycle's colour A, B, C, D and alpha A, B, C, D, in that order.
        private static ulong CombineCycles(int[] first, int[] second)
        {
            int[] widths = { 15, 15, 31, 7, 7, 7, 7, 7 };
            if (first.Concat(second).Select((v, i) => v > widths[i % 8]).Any(tooWide => tooWide))
                throw new ArgumentOutOfRangeException(nameof(first), "a combiner selector does not fit its field");

            ulong high = (ulong)((first[0] << 20) | (first[2] << 15) | (first[4] << 12) | (first[6] << 9) | (second[0] << 5) | second[2]);
            ulong low = (ulong)(uint)((first[1] << 28) | (second[1] << 24) | (second[4] << 21) | (second[6] << 18) | (first[3] << 15) | (first[5] << 12)
                | (first[7] << 9) | (second[3] << 6) | (second[5] << 3) | second[7]);
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

        private static byte[] TexturePattern => Pattern(0x5445_5831, 0x1000);

        // A case that clears a colour and a depth image, runs its texture setup and modes, and draws - see Mars_RdpTextures.md §7.
        private static Case Textured(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, int size = Bits16, string? dispute = null,
            bool crossChecked = true, (uint, byte[])[]? uploads = null, uint primitive = 0x5A)
        {
            var commands = new List<ulong>
            {
                ColorImage(size, 32, Framebuffer), Scissor(0, 0, 124, 124), FillCycle, FillColor(0x7BDE_7BDF), Rectangle(0, 0, 124, 124),
                ColorImage(Bits16, 32, DepthBuffer), FillColor(0xFFFC_FFFC), Rectangle(0, 0, 124, 124), ColorImage(size, 32, Framebuffer),
                (0x3EUL << 56) | DepthBuffer, (0x2CUL << 56) | 0x0B89_1A3C_0156B3CUL & 0x00FF_FFFF_FFFF_FFFF,
            };
            commands.AddRange(setup);
            commands.AddRange(new[] { modes, combine, Color(0x3A, 0xC864_2A9F, primitive), Color(0x3B, 0x3C90_D071), Color(0x39, 0x7755_AA80) });
            commands.AddRange(shapes);
            commands.Add(SyncFull);
            return new Case(name, Framebuffer, size, 32, commands.ToArray(), Dispute: dispute, CrossChecked: crossChecked)
            {
                Uploads = new[] { (TextureImage, TexturePattern) }.Concat(uploads ?? Array.Empty<(uint, byte[])>()).ToArray(),
            };
        }

        // A 4-bit tile loads from an 8-bit image unless told otherwise, since a 4-bit image stops the load in both references - see Mars_RdpTextures.md §7.
        private static ulong[] Loaded(int format, int size, uint width = 16, uint height = 16, int tile = 0, int memory = 0, int shiftS = 0, int shiftT = 0,
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

        // Point-sampled textures in the one-cycle mode: every format, tile shift, mask, mirror and clamp, tile and block loads - see Mars_RdpTextures.md §7.
        private static IEnumerable<Case> TextureCases()
        {
            var cases = new List<Case>();
            ulong texel0 = Combine(subA: 8, subB: 8, mul: 16, add: 1, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1);

            void Draw(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, int size = Bits16, string? dispute = null, bool crossChecked = true,
                (uint, byte[])[]? uploads = null) => cases.Add(Textured(name, modes, combine, setup, shapes, size, dispute, crossChecked, uploads));

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

        // Four-texel sampling in the one-cycle mode: the bilinear filter, the mid-texel, and palette lookup after a palette load - see Mars_RdpFiltering.md §5.
        private static IEnumerable<Case> FilterCases()
        {
            var cases = new List<Case>();
            ulong texel0 = Combine(subA: 8, subB: 8, mul: 16, add: 1, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1);
            ulong bothTexels = Combine(subA: 1, subB: 2, mul: 3, add: 2, alphaSubA: 1, alphaSubB: 2, alphaMul: 3, alphaAdd: 2);

            void Draw(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, int size = Bits16, string? dispute = null, bool crossChecked = true,
                (uint, byte[])[]? uploads = null) => cases.Add(Textured(name, modes, combine, setup, shapes, size, dispute, crossChecked, uploads));

            // A palette is loaded through a tile of its own, whose 4-bit size and memory 0x100 put entry n's four banks at word 0x400 + 4n - see Mars_RdpFiltering.md §4.2.
            ulong[] Palette(int tile = 7, uint entries = 256, int imageSize = Bits16, uint offset = 0x800, int memory = 0x100, int tileSize = Bits4, uint rows = 1) => new[]
            {
                TextureImageCommand(0, imageSize, 16, TextureImage + offset),
                TileCommand(tile, 0, tileSize, 0, memory),
                LoadCommand(0x30, tile, 0, 0, (entries - 1) << 2, (rows - 1) << 2),
            };

            ulong[] With(ulong[] first, params ulong[][] rest) => rest.Aggregate(first, (all, next) => all.Concat(next).ToArray());

            ulong[] stretched = TextureRectangle(false, 0, 12, 10, 100, 90, 0x07, 0x0B, 0x0155, 0x0123);
            ulong[] centred = TextureRectangle(false, 0, 12, 10, 100, 90, 0x10, 0x10, 0x0400, 0x0400);
            ulong filtered = Modes(biLerp0: true, sampleFour: true);

            foreach (var (label, format, size) in new[]
            {
                ("RGBA16", 0, 2), ("RGBA32", 0, 3), ("RGBA4", 0, 0), ("RGBA8", 0, 1), ("YUV16", 1, 2), ("CI4", 2, 0), ("CI8", 2, 1), ("CI16", 2, 2),
                ("IA4", 3, 0), ("IA8", 3, 1), ("IA16", 3, 2), ("I4", 4, 0), ("I8", 4, 1), ("I16", 4, 2),
            })
            {
                Draw($"bilinear-filtered {label} texture rectangle", filtered, texel0, Loaded(format, size), stretched);
            }

            Draw("bilinear-filtered 4-bit YUV texture rectangle", filtered, texel0, Loaded(1, 0), stretched, crossChecked: false);
            Draw("bilinear-filtered 8-bit YUV texture rectangle", filtered, texel0, Loaded(1, 1), stretched, crossChecked: false);
            Draw("bilinear-filtered 32-bit YUV texture rectangle", filtered, texel0, Loaded(1, 3), stretched, crossChecked: false);
            Draw("bilinear-filtered tile wrapping at its mask", filtered, texel0, Loaded(0, 2, maskS: 4, maskT: 4), TextureRectangle(false, 0, 8, 8, 120, 120, -0x0107, 0x0033, 0x0265, 0x01F3));
            Draw("bilinear-filtered tile mirrored", filtered, texel0, Loaded(0, 2, maskS: 4, maskT: 4, mirrorS: true, mirrorT: true),
                TextureRectangle(false, 0, 8, 8, 120, 120, -0x0107, 0x0033, 0x0265, 0x01F3));
            Draw("bilinear-filtered YUV tile mirrored", filtered, texel0, Loaded(1, 2, maskS: 4, maskT: 4, mirrorS: true, mirrorT: true),
                TextureRectangle(false, 0, 8, 8, 120, 120, -0x0107, 0x0033, 0x0265, 0x01F3));
            Draw("bilinear-filtered tile clamped against its size", filtered, texel0,
                Loaded(0, 2, clampS: true, clampT: true, maskS: 4, maskT: 4).Append(TileSizeCommand(0, 9, 13, 37, 29)).ToArray(),
                TextureRectangle(false, 0, 4, 4, 120, 120, -0x0107, -0x0083, 0x0305, 0x0287));
            Draw("mid-texel filter", Modes(biLerp0: true, sampleFour: true, midTexel: true), texel0, Loaded(0, 2), centred);
            Draw("mid-texel filter of a YUV tile", Modes(biLerp0: true, sampleFour: true, midTexel: true), texel0, Loaded(1, 2), centred);
            Draw("mid-texel mode away from the mid-texel", Modes(biLerp0: true, sampleFour: true, midTexel: true), texel0, Loaded(0, 2), stretched);
            Draw("four texels converted without bilinear filtering", Modes(sampleFour: true), texel0, Loaded(1, 2), stretched);
            Draw("four texels of an RGBA tile converted without bilinear filtering", Modes(sampleFour: true), texel0, Loaded(0, 2), stretched);
            Draw("bilinear-filtered next pixel's texel", filtered, bothTexels, Loaded(0, 2), stretched);

            var p = new Vertex(3.25, 2.5, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var q = new Vertex(27.75, 9.75, 10, 240, 90, 128, 0x30000, S: 15, T: 2, W: 0.55);
            var r = new Vertex(7.5, 28.25, 40, 70, 230, 12, 0x1C000, S: 4, T: 15, W: 0.8);
            Draw("bilinear-filtered perspective triangle", Modes(perspective: true, biLerp0: true, sampleFour: true), texel0, Loaded(0, 2), Shaded(0x0A, p, q, r));
            Draw("bilinear-filtered triangle with texel coordinates out of range", Modes(perspective: true, biLerp0: true, sampleFour: true), texel0,
                Loaded(3, 2, maskS: 4, mirrorT: true, maskT: 3), Shaded(0x0B, p with { S = -20, T = 40 }, q with { S = 300, W = 0.1 }, r with { T = -500, W = 1.5 }));

            Draw("palette load", Modes(), texel0, Palette(), Array.Empty<ulong>());
            Draw("palette load of sixteen entries", Modes(), texel0, Palette(entries: 16), Array.Empty<ulong>());
            Draw("palette load into the lower half of texture memory", Modes(), texel0, Palette(memory: 0x20), Array.Empty<ulong>());
            Draw("palette load into an 8-bit tile", Modes(), texel0, Palette(tileSize: Bits8), Array.Empty<ulong>());
            Draw("palette load from an 8-bit image", Modes(), texel0, Palette(imageSize: Bits8), Array.Empty<ulong>());
            Draw("palette load from a 32-bit image", Modes(), texel0, Palette(imageSize: Bits32), Array.Empty<ulong>());
            Draw("palette load from an odd address", Modes(), texel0, Palette(offset: 0x801), Array.Empty<ulong>());
            Draw("palette load of two rows", Modes(), texel0, Palette(rows: 2), Array.Empty<ulong>(), crossChecked: false);

            Draw("8-bit colour-indexed tile through an RGBA16 palette", Modes(biLerp0: true, palette: true), texel0, With(Loaded(2, 1), Palette()), stretched);
            Draw("4-bit colour-indexed tile through an IA16 palette", Modes(biLerp0: true, palette: true, paletteIa: true), texel0, With(Loaded(2, 0, palette: 5), Palette()), stretched);
            Draw("16-bit colour-indexed tile through a palette", Modes(biLerp0: true, palette: true), texel0, With(Loaded(2, 2), Palette()), stretched);
            Draw("bilinear-filtered 8-bit colour-indexed tile through a palette", Modes(biLerp0: true, sampleFour: true, palette: true), texel0, With(Loaded(2, 1), Palette()), stretched);
            Draw("bilinear-filtered 4-bit colour-indexed tile through a palette", Modes(biLerp0: true, sampleFour: true, palette: true, paletteIa: true), texel0,
                With(Loaded(2, 0, palette: 9), Palette()), stretched);
            Draw("bilinear-filtered 16-bit colour-indexed tile through a palette", Modes(biLerp0: true, sampleFour: true, palette: true), texel0, With(Loaded(2, 2), Palette()), stretched);
            Draw("palette lookup without bilinear filtering", Modes(sampleFour: true, palette: true), texel0, With(Loaded(2, 1), Palette()), stretched);
            Draw("palette lookup through a YUV tile", Modes(biLerp0: true, sampleFour: true, palette: true), texel0, With(Loaded(1, 2), Palette()), stretched,
                dispute: "angrylion indexes a palette by a YUV tile's bytes, and parallel-rdp samples nothing through a palette from a YUV tile");
            Draw("palette lookup through a mirrored tile", Modes(biLerp0: true, sampleFour: true, palette: true), texel0,
                With(Loaded(2, 1, maskS: 4, maskT: 4, mirrorS: true, mirrorT: true), Palette()), TextureRectangle(false, 0, 8, 8, 120, 120, -0x0107, 0x0033, 0x0265, 0x01F3));

            // Each case below exists because a breakage of the rule it names survived, or was caught only by a random case - see Mars_RdpFiltering.md §5.5.
            Draw("bilinear filter at the mid-texel with the mid-texel bit clear", filtered, texel0, Loaded(0, 2), centred);
            Draw("bilinear-filtered tile taller than 256 rows", filtered, texel0, Loaded(0, 2, width: 4, height: 512, maskS: 2, maskT: 9),
                With(TextureRectangle(false, 0, 8, 8, 120, 48, 0x0005, (254 << 5) + 0x13, 0x0155, 0x0100),
                    TextureRectangle(false, 0, 8, 64, 120, 112, 0x0005, (510 << 5) + 0x13, 0x0155, 0x0100)));
            Draw("palette lookup through a 4-bit YUV tile", Modes(biLerp0: true, sampleFour: true, palette: true), texel0, With(Loaded(1, 0, palette: 6), Palette()), stretched,
                crossChecked: false);
            Draw("bilinear-filtered lookup through a palette loaded from an odd address", Modes(biLerp0: true, sampleFour: true, palette: true), texel0,
                With(Loaded(2, 1), Palette(offset: 0x801)), stretched);
            Draw("lookup of one texel through a palette loaded from an odd address", Modes(biLerp0: true, palette: true), texel0,
                With(Loaded(2, 1), Palette(offset: 0x801)), stretched);
            Draw("4-bit tile in the upper half read through a palette", Modes(biLerp0: true, sampleFour: true, palette: true), texel0,
                With(Loaded(2, 0, memory: 0x180), Palette(entries: 16)), stretched);
            Draw("16-bit tile in the upper half read through a palette", Modes(biLerp0: true, sampleFour: true, palette: true), texel0,
                With(Loaded(2, 2, memory: 0x180), Palette(entries: 16)), stretched);
            Draw("texel alpha through an RGBA16 palette as the multiplier", Modes(biLerp0: true, palette: true),
                Combine(subA: 1, subB: 8, mul: 8, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1), With(Loaded(2, 1), Palette()), stretched);
            Draw("palette load through an all-zero tile", Modes(), texel0, Palette(tile: 5, entries: 16, offset: 0x200, memory: 0), Array.Empty<ulong>());
            Draw("palette load through a 32-bit tile", Modes(), texel0, Palette(tileSize: Bits32), Array.Empty<ulong>(), crossChecked: false);

            // Random cases keep to what both references model, as the point-sampled ones do, and read a palette only from a tile that is not YUV.
            var random = new Random(0x4649_4C54);
            for (int n = 0; n < 120; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                bool palette = random.Next(3) == 0;
                int format = palette ? Pick(0, 2, 3, 4) : random.Next(5), tile = random.Next(7);
                int size = format == 1 ? Bits16 : format == 0 ? random.Next(4) : random.Next(3);
                uint width = (uint)random.Next(1, 48), height = (uint)random.Next(1, 48);
                int memory = random.Next(0x200);
                // A 32-bit tile keeps within half of texture memory, and a tile read through a palette keeps below the palette.
                if (size == Bits32 || palette)
                {
                    uint slots = size switch { 0 => (width + 3) / 4, 1 => (width + 1) / 2, _ => width };
                    uint line = (slots + 3) / 4;
                    height = Math.Min(height, 0x100 / line);
                    memory = random.Next((int)(0x100 - line * height) + 1);
                }

                var setup = new List<ulong>(Loaded(format, size, width, height, tile, memory, random.Next(16), random.Next(16), random.Next(16), random.Next(16),
                    Coin(), Coin(), Coin(), Coin(), (uint)random.Next(0x100) * 4, palette: random.Next(16)));
                if (Coin()) setup.Add(TileSizeCommand(tile, (uint)random.Next(0x100), (uint)random.Next(0x100), (uint)random.Next(0x400), (uint)random.Next(0x400)));
                if (palette) setup.AddRange(Palette(entries: (uint)random.Next(1, 257), offset: 0x800 + (uint)random.Next(0x300) * 2));

                bool midTexel = random.Next(3) == 0;
                ulong modes = Modes(
                    rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3), m1a: random.Next(4), m1b: random.Next(4), m2a: random.Next(4), m2b: random.Next(4),
                    forceBlend: random.Next(4) == 0, cvgDest: random.Next(4), imageRead: Coin(), antialias: Coin(), alphaCompare: random.Next(4) == 0,
                    zMode: random.Next(4), zCompare: Coin(), zUpdate: Coin(), perspective: Coin(), biLerp0: random.Next(4) != 0,
                    palette: palette, paletteIa: Coin(), sampleFour: palette ? Coin() : random.Next(6) != 0, midTexel: midTexel);
                ulong combine = Combine(
                    subA: Pick(1, 2, 3, 4, 5, 6), subB: Pick(1, 2, 3, 4, 5, 7), mul: Pick(1, 2, 3, 4, 8, 9, 10, 11, 15, 20), add: Pick(1, 2, 3, 4, 5, 7),
                    alphaSubA: Pick(1, 2, 3, 4, 7), alphaSubB: Pick(1, 2, 3, 4, 7), alphaMul: Pick(1, 2, 3, 4, 7), alphaAdd: Pick(1, 2, 3, 4, 7));

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 3); t < count; t++)
                {
                    uint left = (uint)random.Next(0, 100), top = (uint)random.Next(0, 100);
                    if (midTexel && Coin())
                    {
                        short Centre() => (short)(random.Next(-64, 256) * 32 + 0x10);
                        shapes.AddRange(TextureRectangle(Coin(), tile, left, top, left + (uint)random.Next(4, 60), top + (uint)random.Next(4, 60), Centre(), Centre(),
                            (short)(Pick(1, 2, -1) * 0x400), (short)(Pick(1, 2, -1) * 0x400)));
                    }
                    else if (Coin())
                    {
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

                Draw($"random filter case {n}", modes, combine, setup.ToArray(), shapes.ToArray(), size: Pick(Bits16, Bits16, Bits32));
            }

            return cases;
        }

        // A chain of tiles from tile `first`, each half the last and shifted one further, placed one after another in texture memory.
        private static ulong[] Mipmaps(int first = 0, int levels = 4, uint width = 32, int format = 0, int size = Bits16, bool wrap = true)
        {
            var commands = new List<ulong>();
            int memory = 0;
            for (int level = 0; level < levels; level++)
            {
                uint side = Math.Max(width >> level, 1);
                int mask = wrap ? System.Numerics.BitOperations.Log2(side) : 0;
                commands.AddRange(Loaded(format, size, side, side, (first + level) & 7, memory, shiftS: level, shiftT: level, maskS: mask, maskT: mask,
                    offset: (uint)(level * 0x100)));
                uint slots = format == 1 ? (side + 1) / 2 : size switch { 0 => (side + 3) / 4, 1 => (side + 1) / 2, _ => side };
                memory += (int)((slots + 3) / 4 * side);
            }

            return commands.ToArray();
        }

        // The level of detail in the one-cycle mode: the tile a pixel's texel movement picks, and the fraction the combiner reads - see Mars_RdpLod.md §4.
        private static IEnumerable<Case> LodCases()
        {
            var cases = new List<Case>();
            ulong texel0 = Combine(subA: 8, subB: 8, mul: 16, add: 1, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1);
            ulong fraction = Combine(subA: 1, subB: 8, mul: 13, add: 3, alphaSubA: 1, alphaSubB: 7, alphaMul: 0, alphaAdd: 7);
            ulong bothTexels = Combine(subA: 1, subB: 2, mul: 13, add: 2, alphaSubA: 1, alphaSubB: 2, alphaMul: 0, alphaAdd: 2);
            const string Measured = "angrylion measures the one-cycle mode's level of detail along the span from the next pixel, and parallel-rdp from the pixel across x and down y, as in the two-cycle mode";

            void Draw(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, string? dispute = null, bool crossChecked = true, int minLevel = 0,
                int primitiveFraction = 0x5A) =>
                cases.Add(Textured(name, modes, combine, setup, shapes, dispute: dispute, crossChecked: crossChecked, primitive: (uint)((minLevel << 8) | primitiveFraction)));

            ulong[] With(ulong[] first, params ulong[][] rest) => rest.Aggregate(first, (all, next) => all.Concat(next).ToArray());

            var p = new Vertex(2.25, 1.5, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var q = new Vertex(29.75, 6.75, 10, 240, 90, 128, 0x30000, S: 180, T: 20, W: 0.35);
            var r = new Vertex(6.5, 29.25, 40, 70, 230, 12, 0x1C000, S: 30, T: 260, W: 0.7);
            ulong[] shrinking = Shaded(0x0A | (3 << 19), p, q, r);
            ulong perspective = Modes(perspective: true, biLerp0: true, lod: true);

            Draw("mipmapped perspective triangle", perspective, texel0, Mipmaps(), shrinking, dispute: Measured);
            Draw("mipmapped affine triangle", Modes(biLerp0: true, lod: true), texel0, Mipmaps(), Shaded(0x0A | (3 << 19), p with { W = 1 }, q with { W = 1 }, r with { W = 1 }), dispute: Measured);
            Draw("mipmapped triangle from tile 6", perspective, texel0, Mipmaps(first: 6), Shaded(0x0A | (6 << 16) | (3 << 19), p, q, r), dispute: Measured);
            Draw("mipmapped triangle past its maximum level", perspective, texel0, Mipmaps(), Shaded(0x0A | (1 << 19), p, q, r));
            Draw("mipmapped triangle with no maximum level", perspective, texel0, Mipmaps(), Shaded(0x0A, p, q, r));
            Draw("level-of-detail fraction as the multiplier", Modes(perspective: true, biLerp0: true), fraction, Mipmaps(), shrinking, dispute: Measured);
            Draw("level-of-detail fraction with the level bit set", perspective, fraction, Mipmaps(), shrinking, dispute: Measured);
            Draw("level-of-detail fraction below the minimum level", perspective, fraction, Mipmaps(), Shaded(0x0A | (3 << 19), p, q with { S = 12 }, r with { T = 14 }),
                minLevel: 20, dispute: Measured);
            Draw("detail texture", Modes(perspective: true, biLerp0: true, lod: true, detail: true), fraction, Mipmaps(), shrinking, minLevel: 9, dispute: Measured);
            Draw("sharpened texture", Modes(perspective: true, biLerp0: true, lod: true, sharpen: true), fraction, Mipmaps(), shrinking, minLevel: 9, dispute: Measured);
            Draw("magnified detail texture", Modes(perspective: true, biLerp0: true, lod: true, detail: true), fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p, q with { S = 12 }, r with { T = 14 }), minLevel: 9, dispute: Measured);
            Draw("next pixel's texel from a mipmapped triangle", perspective, bothTexels, Mipmaps(), shrinking, dispute: Measured);
            Draw("mipmapped triangle past the divider's range", perspective, fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { S = -3000, T = 2500, W = 0.1 }, q with { S = 3000, T = -2000, W = 0.1 }, r with { S = 0, T = 0, W = 0.12 }));
            Draw("mipmapped triangle moving past a quarter of the coordinate range", Modes(biLerp0: true, lod: true), fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { S = -20000, W = 1 }, q with { S = 20000, T = 900, W = 1 }, r with { T = -9000, W = 1 }));
            Draw("texture rectangle with the level bit set", Modes(biLerp0: true, lod: true), bothTexels, Mipmaps(), TextureRectangle(false, 0, 8, 8, 120, 120, 0, 0, 0x1C00, 0x0A00));

            // Spans of six, seven and eight pixels and longer ones, whose last pixels measure against the pixel before or the next row - see Mars_RdpLod.md §2.
            foreach (uint span in new uint[] { 6, 7, 8, 9 })
            {
                Draw($"level of detail at the end of a {span + 1}-pixel span", Modes(biLerp0: true, lod: true, sharpen: true), bothTexels, Mipmaps(),
                    With(TextureRectangle(false, 0, 8, 8, 8 + (span + 1) * 4, 60, 0x0013, 0x0027, 0x0D33, 0x0A00),
                        TextureRectangle(true, 0, 64, 64, 64 + (span + 1) * 4, 112, 0x0013, 0x0027, 0x0B11, 0x0C00)), dispute: Measured);
            }

            Draw("level of detail across a triangle's rows", perspective, bothTexels, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { X = 1.25 }, q with { X = 9.5, Y = 14.75 }, r with { X = 2.75 }));

            // Movement along one axis at a time, fast enough to saturate the level and slow enough not to, is what separates the two references here - see Mars_RdpLod.md §4.2.
            Draw("level of detail from fast movement along x alone", Modes(biLerp0: true, lod: true), fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { S = p.X * 9, T = 5, W = 1 }, q with { S = q.X * 9, T = 5, W = 1 }, r with { S = r.X * 9, T = 5, W = 1 }));
            Draw("level of detail from slow movement along x in spans of at most seven pixels", Modes(biLerp0: true, lod: true), fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { X = 1, Y = 1, S = 1.5, T = 5, W = 1 }, q with { X = 7, Y = 1, S = 10.5, T = 5, W = 1 }, r with { X = 1, Y = 29, S = 1.5, T = 5, W = 1 }));
            Draw("level of detail from slow movement along x at a long span's end", Modes(biLerp0: true, lod: true), fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { S = p.X * 1.5, T = 5, W = 1 }, q with { S = q.X * 1.5, T = 5, W = 1 }, r with { S = r.X * 1.5, T = 5, W = 1 }), dispute: Measured);
            Draw("level of detail from movement down the rows alone", Modes(biLerp0: true, lod: true), fraction, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { T = p.Y * 9, S = 5, W = 1 }, q with { T = q.Y * 9, S = 5, W = 1 }, r with { T = r.Y * 9, S = 5, W = 1 }), dispute: Measured);

            // Each case below exists because a breakage of the rule it names survived, or was caught only by a random case - see Mars_RdpLod.md §4.4.
            ulong shown = Combine(subA: 6, subB: 8, mul: 13, add: 7, alphaSubA: 6, alphaSubB: 7, alphaMul: 0, alphaAdd: 7);
            ulong texel1 = Combine(subA: 8, subB: 8, mul: 16, add: 2, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 2);
            Draw("level-of-detail fraction read without a texel", Modes(perspective: true, biLerp0: true), shown, Mipmaps(), shrinking, dispute: Measured);
            Draw("magnified texture rectangle's fraction", Modes(biLerp0: true, lod: true), shown, Mipmaps(), TextureRectangle(false, 0, 8, 8, 120, 120, 0, 0, 0x0200, 0x0100));
            Draw("magnified and sharpened texture rectangle's fraction", Modes(biLerp0: true, lod: true, sharpen: true), shown, Mipmaps(),
                TextureRectangle(false, 0, 8, 8, 120, 120, 0, 0, 0x0200, 0x0100), minLevel: 5);
            Draw("level-of-detail fraction as the alpha multiplier, through alpha compare", Modes(perspective: true, biLerp0: true, lod: true, alphaCompare: true), shown,
                Mipmaps(), shrinking, dispute: Measured);
            Draw("sharpened fraction at the ends of a rectangle's spans, last row included", Modes(biLerp0: true, lod: true, sharpen: true), shown, Mipmaps(),
                TextureRectangle(false, 0, 8, 8, 120, 120, 0x0013, 0x0027, 0x0633, 0x0100), dispute: Measured);
            Draw("texel 1's level at the end of spans of every length", Modes(perspective: true, biLerp0: true, lod: true), texel1, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { X = 2, Y = 2, S = 5, T = 3, W = 1 }, q with { X = 2, Y = 29, S = 5, T = 3, W = 1 }, r with { X = 29, Y = 29, S = 5 + 27 * 2.3, T = 3, W = 0.3 }),
                dispute: Measured);
            Draw("level of detail with s under the divider's range and not moving", Modes(perspective: true, biLerp0: true, lod: true), shown, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { S = -3000, T = 3, W = 0.1 }, q with { S = -3000, T = 3, W = 0.1 }, r with { S = -3000, T = 3, W = 0.1 }));
            Draw("level of detail where the divider under- and overflows at some pixels", Modes(perspective: true, biLerp0: true, lod: true), shown, Mipmaps(),
                Shaded(0x0A | (3 << 19), p with { S = -2500, T = 3, W = 0.15 }, q with { S = 40, T = 2600, W = 0.2 }, r with { S = 10, T = 10, W = 1 }));
            Draw("movement with bit 13 set and a low level", Modes(biLerp0: true, lod: true), shown, Mipmaps(),
                Shaded(0x0A | (7 << 19), p with { S = p.X * 264, T = 3, W = 1 }, q with { S = q.X * 264, T = 3, W = 1 }, r with { S = r.X * 264, T = 3, W = 1 }));

            // Random cases are graded against angrylion alone, since nearly any that reads an unsaturated level meets the dispute - see Mars_RdpLod.md §4.2.
            var random = new Random(0x4C4F_4431);
            for (int n = 0; n < 120; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                int format = random.Next(5), size = format == 1 ? Bits16 : random.Next(3);
                int first = random.Next(8), levels = random.Next(1, 5), maxLevel = random.Next(8);
                ulong[] setup = Mipmaps(first, levels, (uint)Pick(8, 16, 32), format, size, Coin());

                ulong modes = Modes(
                    rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3), m1a: random.Next(4), m1b: random.Next(4), m2a: random.Next(4), m2b: random.Next(4),
                    forceBlend: random.Next(4) == 0, cvgDest: random.Next(4), imageRead: Coin(), antialias: Coin(), alphaCompare: random.Next(4) == 0,
                    zMode: random.Next(4), zCompare: Coin(), zUpdate: Coin(), perspective: Coin(), biLerp0: random.Next(4) != 0, sampleFour: Coin(),
                    lod: random.Next(4) != 0, sharpen: random.Next(3) == 0, detail: random.Next(3) == 0);
                ulong combine = Combine(
                    subA: Pick(1, 2, 3, 4, 5), subB: Pick(1, 2, 3, 4, 7), mul: Pick(1, 2, 8, 9, 13, 13, 14, 15), add: Pick(1, 2, 3, 4, 7),
                    alphaSubA: Pick(1, 2, 3, 7), alphaSubB: Pick(1, 2, 3, 7), alphaMul: Pick(0, 0, 1, 2, 6, 7), alphaAdd: Pick(1, 2, 3, 7));

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 3); t < count; t++)
                {
                    if (random.Next(3) == 0)
                    {
                        uint left = (uint)random.Next(0, 100), top = (uint)random.Next(0, 100);
                        shapes.AddRange(TextureRectangle(Coin(), first, left, top, left + (uint)random.Next(4, 60), top + (uint)random.Next(4, 60),
                            (short)random.Next(-0x400, 0x800), (short)random.Next(-0x400, 0x800), (short)random.Next(-0x7FFF, 0x7FFF), (short)random.Next(-0x7FFF, 0x7FFF)));
                    }
                    else
                    {
                        int reach = Pick(64, 256, 1024, 8000);
                        Vertex Point() => new(random.Next(-8, 136) / 4.0, random.Next(-8, 136) / 4.0, random.Next(256), random.Next(256), random.Next(256), random.Next(256),
                            random.Next(0x40000), random.Next(-reach, reach), random.Next(-reach, reach), 0.05 + 0.95 * random.NextDouble());
                        shapes.AddRange(Shaded(Pick(0x0A, 0x0B, 0x0E, 0x0F) | (first << 16) | (maxLevel << 19), Point(), Point(), Point()));
                    }
                }

                Draw($"random level-of-detail case {n}", modes, combine, setup, shapes.ToArray(), crossChecked: false, minLevel: random.Next(32), primitiveFraction: random.Next(256));
            }

            return cases;
        }

        // The two-cycle mode: both combiner and blender cycles, the swapped texels, convert-one, two tiles, and the pipelining between pixels - see Mars_RdpTwoCycle.md §5.
        private static IEnumerable<Case> TwoCycleCases()
        {
            var cases = new List<Case>();

            void Draw(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, string? dispute = null, bool crossChecked = true, int size = Bits16) =>
                cases.Add(Textured(name, modes, combine, setup, shapes, size, dispute, crossChecked));

            int[] Pass(int colour, int alpha) => new[] { 8, 8, 16, colour, 7, 7, 7, alpha };
            ulong[] With(ulong[] first, params ulong[][] rest) => rest.Aggregate(first, (all, next) => all.Concat(next).ToArray());

            var p = new Vertex(2.25, 1.5, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var q = new Vertex(29.75, 6.75, 10, 240, 90, 20, 0x30000, S: 40, T: 6, W: 0.55);
            var r = new Vertex(6.5, 29.25, 40, 70, 230, 140, 0x1C000, S: 5, T: 44, W: 0.8);
            ulong[] triangle = Shaded(0x0F, p, q, r);
            ulong[] other = Shaded(0x0F, new Vertex(28.5, 1.25, 30, 200, 120, 90, 0x10000, S: 3, T: 9, W: 0.7), new Vertex(4.75, 17.5, 220, 40, 10, 250, 0x2C000, S: 35, T: 2, W: 1.0),
                new Vertex(24.25, 29.75, 90, 90, 250, 30, 0x0C000, S: 12, T: 40, W: 0.6));
            ulong[] twoTiles = With(Loaded(0, 2), Loaded(3, 1, tile: 1, memory: 0x80, offset: 0x300));
            ulong twoCycle = Modes(cycle: 1, biLerp0: true);
            const string Combined = "angrylion's first cycle reads the previous pixel's result as its combined input, and at a row's start the result for the pixel past the last row's end, where parallel-rdp reads zero";
            const string PreviousSlope = "angrylion shifts the first blend's memory-alpha weights by the previous pixel's stored depth slope, and parallel-rdp by the pixel's own";

            Draw("two-cycle shaded triangle passing shade through both cycles", twoCycle, CombineCycles(Pass(4, 4), Pass(0, 0)), Array.Empty<ulong>(), Shaded(0x0C, p, q, r));
            Draw("two-cycle textured triangle, texel times shade then times primitive", twoCycle,
                CombineCycles(new[] { 1, 8, 4, 7, 1, 7, 4, 7 }, new[] { 0, 8, 3, 7, 0, 7, 3, 7 }), Loaded(0, 2), triangle);
            Draw("first cycle reading the combined input", twoCycle,
                CombineCycles(new[] { 0, 4, 11, 1, 0, 4, 4, 1 }, new[] { 0, 8, 5, 3, 0, 7, 5, 7 }), Loaded(0, 2), triangle, dispute: Combined);
            Draw("two tiles, the second read as texel 1", twoCycle,
                CombineCycles(new[] { 2, 1, 10, 1, 2, 1, 3, 1 }, Pass(0, 0)), twoTiles, triangle);
            Draw("second cycle reading its texels swapped", twoCycle,
                CombineCycles(Pass(4, 4), new[] { 1, 2, 3, 2, 1, 2, 3, 2 }), twoTiles, triangle);
            Draw("convert-one: the second texel converts the first", Modes(cycle: 1, biLerp0: true, biLerp1: false, convertOne: true),
                CombineCycles(Pass(1, 1), Pass(1, 1)), With(Loaded(1, 2), Loaded(0, 2, tile: 1, memory: 0x80, offset: 0x300)), triangle);
            Draw("convert-one with the second cycle filtering four texels", Modes(cycle: 1, biLerp0: true, biLerp1: true, convertOne: true, sampleFour: true),
                CombineCycles(Pass(1, 1), Pass(1, 1)), With(Loaded(1, 2), Loaded(0, 2, tile: 1, memory: 0x80, offset: 0x300)), triangle);
            Draw("convert-one with the second cycle point-sampled and filtered", Modes(cycle: 1, biLerp0: true, biLerp1: true, convertOne: true),
                CombineCycles(Pass(1, 1), Pass(1, 1)), With(Loaded(1, 2), Loaded(0, 2, tile: 1, memory: 0x80, offset: 0x300)), triangle);
            Draw("second texel converted without convert-one", Modes(cycle: 1, biLerp0: true, biLerp1: false),
                CombineCycles(Pass(4, 4), Pass(1, 1)), With(Loaded(0, 2), Loaded(1, 2, tile: 1, memory: 0x80, offset: 0x300)), triangle);
            Draw("convert-one converting a first texel already converted", Modes(cycle: 1, biLerp0: false, biLerp1: false, convertOne: true),
                CombineCycles(Pass(1, 1), Pass(1, 1)), With(Loaded(1, 2), Loaded(0, 2, tile: 1, memory: 0x80, offset: 0x300)), triangle);
            Draw("convert-one with the second cycle taking four texels unfiltered", Modes(cycle: 1, biLerp0: true, biLerp1: false, convertOne: true, sampleFour: true),
                CombineCycles(Pass(1, 1), Pass(1, 1)), With(Loaded(1, 2), Loaded(0, 2, tile: 1, memory: 0x80, offset: 0x300)), triangle);
            Draw("convert-one filtering four texels at their centres", Modes(cycle: 1, biLerp0: true, biLerp1: true, convertOne: true, sampleFour: true, midTexel: true),
                CombineCycles(Pass(1, 1), Pass(1, 1)), With(Loaded(1, 2), Loaded(0, 2, tile: 1, memory: 0x80, offset: 0x300)),
                TextureRectangle(false, 0, 12, 10, 100, 90, 0x10, 0x10, 0x0400, 0x0400));

            ulong blendModes = Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0, m1a1: 0, m1b1: 3, m2a1: 2, m2b1: 2, forceBlend: true);
            ulong texelShade = CombineCycles(new[] { 1, 8, 4, 7, 1, 7, 4, 7 }, Pass(0, 0));
            Draw("both blend cycles, forced", blendModes, texelShade, Loaded(0, 2), With(other, triangle));
            Draw("both blend cycles, the second dividing", Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0, m1a1: 0, m1b1: 0, m2a1: 3, m2b1: 0,
                antialias: true, imageRead: true, zCompare: true, zUpdate: true), texelShade, Loaded(0, 2), With(other, triangle));
            Draw("first blend undivided, its weights not summing to one", Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 3, m1a1: 0, m1b1: 0, m2a1: 3, m2b1: 0,
                antialias: true, imageRead: true, zCompare: true, zUpdate: true), texelShade, Loaded(0, 2), With(other, triangle));
            Draw("first blend weighed by memory alpha", Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 1, m1a1: 0, m1b1: 0, m2a1: 3, m2b1: 0,
                antialias: true, imageRead: true, zCompare: true, zUpdate: true, forceBlend: true), texelShade, Loaded(0, 2), With(other, triangle), dispute: PreviousSlope);
            Draw("first blend's previous slope after a primitive without depth test", Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 1, m1a1: 0, m1b1: 0, m2a1: 3,
                m2b1: 0, antialias: true, imageRead: true, forceBlend: true), CombineCycles(Pass(1, 6), Pass(0, 0)), Loaded(0, 2), With(other, new[] { Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 1,
                m2b: 1, m1a1: 0, m1b1: 0, m2a1: 3, m2b1: 0, antialias: true, imageRead: true, zCompare: true, zUpdate: true, forceBlend: true) }, TextureRectangle(false, 0, 12, 10, 100, 90, 0, 0, 0x0400, 0x0400)), dispute: PreviousSlope);
            Draw("second blend weighed by memory alpha", Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 3, m2b: 0, m1a1: 0, m1b1: 0, m2a1: 1, m2b1: 1,
                antialias: true, imageRead: true, zCompare: true, zUpdate: true, forceBlend: true), texelShade, Loaded(0, 2), With(other, triangle));
            Draw("second blend reading shade alpha", Modes(cycle: 1, biLerp0: true, m1a: 0, m1b: 0, m2a: 2, m2b: 0, m1a1: 0, m1b1: 2, m2a1: 1, m2b1: 0, forceBlend: true),
                texelShade, Loaded(0, 2), With(other, Shaded(0x0F, p with { A = 0 }, q with { A = 255 }, r with { A = 30 })),
                dispute: "angrylion's second blend reads the next pixel's shade alpha, and parallel-rdp the pixel's own");
            Draw("two-cycle alpha compare", Modes(cycle: 1, biLerp0: true, alphaCompare: true),
                CombineCycles(new[] { 1, 8, 4, 7, 1, 7, 4, 7 }, new[] { 0, 8, 3, 7, 3, 7, 4, 7 }), Loaded(0, 2), triangle);
            Draw("two-cycle alpha compare on alpha from coverage", Modes(cycle: 1, biLerp0: true, alphaCompare: true, alphaCvgSelect: true, antialias: true),
                CombineCycles(Pass(4, 4), Pass(0, 0)), Array.Empty<ulong>(), Shaded(0x0C, p, q, r));
            Draw("two-cycle alpha compare on coverage times alpha", Modes(cycle: 1, biLerp0: true, alphaCompare: true, alphaCvgSelect: true, cvgTimesAlpha: true, antialias: true),
                CombineCycles(Pass(4, 4), Pass(0, 0)), Array.Empty<ulong>(), Shaded(0x0C, p, q, r));
            Draw("two-cycle alpha compare, dithered", Modes(cycle: 1, biLerp0: true, alphaCompare: true, alphaDither: 0),
                CombineCycles(Pass(4, 4), Pass(0, 0)), Array.Empty<ulong>(), Shaded(0x0C, p, q, r));
            Draw("two-cycle coverage times alpha", Modes(cycle: 1, biLerp0: true, alphaCvgSelect: true, cvgTimesAlpha: true, antialias: true, imageRead: true, cvgDest: 0),
                CombineCycles(Pass(4, 4), Pass(0, 0)), Array.Empty<ulong>(), With(other, Shaded(0x0C, p, q, r)));
            Draw("two-cycle depth-tested, anti-aliased triangles", Modes(cycle: 1, biLerp0: true, antialias: true, imageRead: true, zCompare: true, zUpdate: true, zMode: 1,
                m1a: 0, m1b: 0, m2a: 1, m2b: 0, m1a1: 0, m1b1: 0, m2a1: 1, m2b1: 0), texelShade, Loaded(0, 2), With(triangle, other));

            ulong trilinear = CombineCycles(new[] { 2, 1, 13, 1, 2, 1, 0, 1 }, new[] { 0, 8, 4, 7, 0, 7, 4, 7 });
            Draw("two-cycle mipmaps, trilinear", Modes(cycle: 1, perspective: true, lod: true, biLerp0: true), trilinear, Mipmaps(), Shaded(0x0F | (3 << 19), p, q, r));
            Draw("two-cycle detail texture", Modes(cycle: 1, perspective: true, lod: true, detail: true, biLerp0: true), trilinear, Mipmaps(), Shaded(0x0F | (3 << 19), p, q, r));
            Draw("two-cycle sharpened texture", Modes(cycle: 1, perspective: true, lod: true, sharpen: true, biLerp0: true), trilinear, Mipmaps(), Shaded(0x0F | (3 << 19), p, q, r));
            Draw("two-cycle mipmapped texture rectangle", Modes(cycle: 1, lod: true, biLerp0: true), trilinear, Mipmaps(), TextureRectangle(false, 0, 8, 8, 120, 120, 0, 0, 0x0B00, 0x0900));
            ulong[] slow = Shaded(0x0F | (3 << 19), p with { W = 1 }, q with { W = 1, S = 140 }, r with { W = 1, T = 90 });
            Draw("second texel past a row's end, from the next row", Modes(cycle: 1, lod: true, biLerp0: true), CombineCycles(new[] { 2, 0, 3, 0, 2, 0, 3, 0 }, new[] { 2, 0, 10, 0, 2, 0, 3, 0 }),
                Mipmaps(), slow, dispute: Combined);
            Draw("past a row's end, alpha compare on the first cycle's texel 0", Modes(cycle: 1, lod: true, biLerp0: true, alphaCompare: true),
                CombineCycles(new[] { 2, 0, 3, 0, 1, 0, 3, 6 }, Pass(0, 0)), Mipmaps(), slow, dispute: Combined);
            Draw("past a row's end, alpha compare on the first cycle's texel 1", Modes(cycle: 1, lod: true, biLerp0: true, alphaCompare: true),
                CombineCycles(new[] { 4, 0, 7, 0, 2, 0, 3, 0 }, Pass(0, 0)), Mipmaps(), slow, dispute: Combined);
            Draw("past a row's end, alpha compare on the first cycle's fraction", Modes(cycle: 1, lod: true, biLerp0: true, alphaCompare: true),
                CombineCycles(new[] { 2, 0, 3, 0, 4, 0, 0, 6 }, Pass(0, 0)), Mipmaps(), slow, dispute: Combined);
            Draw("past a row's end without the enable bit", twoCycle, CombineCycles(new[] { 2, 0, 3, 0, 2, 0, 3, 0 }, new[] { 2, 0, 10, 0, 2, 0, 3, 0 }),
                Mipmaps(), slow, dispute: Combined);
            Draw("past a row's end, the fraction read without the enable bit", twoCycle, CombineCycles(new[] { 2, 0, 13, 0, 2, 0, 3, 0 }, new[] { 2, 0, 10, 0, 2, 0, 3, 0 }),
                Mipmaps(), slow, dispute: Combined);
            Draw("first cycle reading the fraction without the enable bit", Modes(cycle: 1, perspective: true, biLerp0: true),
                CombineCycles(new[] { 6, 8, 13, 7, 7, 7, 7, 7 }, Pass(0, 0)), Mipmaps(), Shaded(0x0F | (3 << 19), p, q, r));
            Draw("second tile without the enable bit, the fraction read", twoCycle, CombineCycles(Pass(4, 4), new[] { 8, 8, 16, 1, 7, 7, 0, 7 }), twoTiles, triangle);
            Draw("two-cycle palette lookup", Modes(cycle: 1, biLerp0: true, sampleFour: true, palette: true), CombineCycles(new[] { 2, 1, 10, 1, 2, 1, 3, 1 }, Pass(0, 0)),
                With(Loaded(2, 1), Loaded(2, 1, tile: 1, memory: 0x60, offset: 0x200), new[] { TextureImageCommand(0, 2, 16, TextureImage + 0x800), TileCommand(7, 0, 0, 0, 0x100),
                    LoadCommand(0x30, 7, 0, 0, 255 << 2, 0) }), triangle);
            Draw("fill rectangle in the two-cycle mode", Modes(cycle: 1, rgbDither: 0), CombineCycles(Pass(3, 3), new[] { 0, 5, 12, 5, 0, 5, 3, 7 }), Array.Empty<ulong>(),
                new[] { Rectangle(8, 12, 100, 88) });

            // Random cases draw from both cycles' selectors and every mode built so far, but never the three disputed inputs: combined in the first cycle, memory alpha in the first blend, shade alpha in the second.
            var random = new Random(0x5457_4F43);
            for (int n = 0; n < 120; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                int format = Pick(0, 0, 2, 3, 4), size = format == 0 ? Pick(1, 2, 2) : random.Next(3);
                ulong[] setup = Mipmaps(0, random.Next(1, 5), (uint)Pick(8, 16, 32), format, size, Coin());

                int[] Cycle(bool combined) => new[]
                {
                    Pick(combined ? 0 : 6, 1, 2, 3, 4, 5, 6, 8), Pick(combined ? 0 : 8, 1, 2, 3, 4, 5, 7, 8), Pick(combined ? 0 : 16, 1, 2, 3, 4, combined ? 7 : 16, 8, 9, 10, 11, 13, 14, 16),
                    Pick(combined ? 0 : 6, 1, 2, 3, 4, 5, 6, 7), Pick(combined ? 0 : 7, 1, 2, 3, 4, 6, 7), Pick(combined ? 0 : 7, 1, 2, 3, 4, 7), Pick(0, 1, 2, 3, 4, 6, 7),
                    Pick(combined ? 0 : 7, 1, 2, 3, 4, 6, 7),
                };

                ulong modes = Modes(
                    cycle: 1, rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3), m1a: random.Next(4), m1b: random.Next(4), m2a: random.Next(4), m2b: Pick(0, 2, 3),
                    m1a1: random.Next(4), m1b1: Pick(0, 1, 3), m2a1: random.Next(4), m2b1: random.Next(4), forceBlend: random.Next(3) == 0, alphaCvgSelect: random.Next(4) == 0,
                    cvgTimesAlpha: random.Next(4) == 0, cvgDest: random.Next(4), colorOnCvg: random.Next(4) == 0, imageRead: Coin(), antialias: Coin(),
                    alphaCompare: random.Next(4) == 0, zMode: random.Next(4), zCompare: Coin(), zUpdate: Coin(), perspective: Coin(), biLerp0: random.Next(4) != 0,
                    biLerp1: random.Next(4) != 0, convertOne: random.Next(4) == 0, sampleFour: random.Next(3) == 0, lod: Coin(), sharpen: random.Next(4) == 0,
                    detail: random.Next(4) == 0);

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 3); t < count; t++)
                {
                    if (random.Next(3) == 0)
                    {
                        uint left = (uint)random.Next(0, 100), top = (uint)random.Next(0, 100);
                        shapes.AddRange(TextureRectangle(Coin(), 0, left, top, left + (uint)random.Next(4, 60), top + (uint)random.Next(4, 60),
                            (short)random.Next(-0x400, 0x800), (short)random.Next(-0x400, 0x800), (short)random.Next(-0x1800, 0x1800), (short)random.Next(-0x1800, 0x1800)));
                    }
                    else
                    {
                        int reach = Pick(32, 128, 512);
                        Vertex Point() => new(random.Next(-8, 136) / 4.0, random.Next(-8, 136) / 4.0, random.Next(256), random.Next(256), random.Next(256), random.Next(256),
                            random.Next(0x40000), random.Next(-reach, reach), random.Next(-reach, reach), 0.2 + 0.8 * random.NextDouble());
                        shapes.AddRange(Shaded(Pick(0x08, 0x09, 0x0C, 0x0D, 0x0A, 0x0B, 0x0E, 0x0F) | (random.Next(4) << 19), Point(), Point(), Point()));
                    }
                }

                Draw($"random two-cycle case {n}", modes, CombineCycles(Cycle(combined: false), Cycle(combined: true)), setup, shapes.ToArray(), size: Pick(Bits16, Bits16, Bits32));
            }

            return cases;
        }

        // The copy mode: four texels a step written to the colour image as bytes, with no combiner or blender - see Mars_RdpCopy.md §5.
        private static IEnumerable<Case> CopyCases()
        {
            var cases = new List<Case>();
            ulong texel0 = Combine(subA: 8, subB: 8, mul: 16, add: 1, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1);

            void Draw(string name, ulong modes, ulong[] setup, ulong[] shapes, int size = Bits16, string? dispute = null, bool crossChecked = true) =>
                cases.Add(Textured(name, modes, texel0, setup, shapes, size, dispute, crossChecked));

            ulong[] With(ulong[] first, params ulong[][] rest) => rest.Aggregate(first, (all, next) => all.Concat(next).ToArray());

            ulong copy = Modes(cycle: 2);
            const string Torn = "angrylion writes a right-major copy span backwards by bytes from the starting pixel's first byte, so each group lands half a pixel off and the span's end pixels are written in halves, where parallel-rdp writes whole pixels";
            ulong[] Rect(uint left, uint top, uint right, uint bottom, short ds = 0x1000, short dt = 0x0400, bool flip = false, int tile = 0) =>
                TextureRectangle(flip, tile, left, top, right, bottom, 0, 0, ds, dt);

            ulong[] wide = Rect(12, 10, 100, 90);
            var p = new Vertex(2.25, 1.5, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var q = new Vertex(29.75, 6.75, 10, 240, 90, 20, 0x30000, S: 40, T: 6, W: 0.55);
            var r = new Vertex(6.5, 29.25, 40, 70, 230, 140, 0x1C000, S: 5, T: 44, W: 0.8);

            Draw("copy a sixteen-bit texture rectangle", copy, Loaded(0, 2), wide);
            Draw("copy a rectangle whose span is not four pixels wide", copy, Loaded(0, 2), Rect(12, 10, 40, 60));
            Draw("copy a flipped rectangle", copy, Loaded(0, 2), Rect(12, 10, 100, 90, 0x0400, 0x1000, flip: true));
            Draw("copy a rectangle stepping four texels a pixel", copy, Loaded(0, 2), Rect(12, 10, 100, 90, 0x4000, 0x1000));
            Draw("copy with alpha compare on the texel's alpha bit", Modes(cycle: 2, alphaCompare: true), Loaded(0, 2), wide);

            Draw("copy into an eight-bit image", copy, Loaded(0, 1), wide, size: Bits8);
            Draw("copy an eight-bit intensity-alpha texture into an eight-bit image", copy, Loaded(3, 1), wide, size: Bits8,
                dispute: "angrylion doubles the high nibble of an eight-bit intensity-alpha texel where the copy mode makes a byte of it, and parallel-rdp takes the byte whole");
            Draw("copy into an eight-bit image with alpha compare", Modes(cycle: 2, alphaCompare: true), Loaded(0, 1), wide, size: Bits8,
                dispute: "angrylion tests a copied byte against the alpha threshold in an eight-bit image, and parallel-rdp tests alpha only in a sixteen-bit one");
            Draw("copy into a four-bit image", copy, Loaded(0, 2), wide, size: Bits4, crossChecked: false);
            Draw("copy into a thirty-two-bit image", copy, Loaded(0, 2), wide, size: Bits32, crossChecked: false);
            Draw("copy a sixteen-bit texture into an eight-bit image", copy, Loaded(0, 2), wide, size: Bits8);
            Draw("copy an eight-bit texture into a sixteen-bit image", copy, Loaded(0, 1), wide);

            Draw("copy a four-bit colour-indexed texture", copy, Loaded(2, 0, palette: 5), wide, size: Bits8,
                dispute: "angrylion puts the tile's palette number in the high nibble of a copied four-bit colour-indexed texel, and parallel-rdp doubles the index nibble");
            Draw("copy a four-bit intensity-alpha texture", copy, Loaded(3, 0), wide, size: Bits8,
                dispute: "angrylion folds a copied four-bit intensity-alpha texel into intensity and alpha bits, and parallel-rdp doubles the nibble as it does for intensity");
            Draw("copy a four-bit intensity texture", copy, Loaded(4, 0), wide, size: Bits8);
            Draw("copy a thirty-two-bit texture", copy, Loaded(0, 3), wide);
            Draw("copy a YUV texture", copy, Loaded(1, 2), wide,
                dispute: "angrylion reads a copied YUV tile's chroma a further step along in texture memory, where parallel-rdp's copy path has no YUV case");

            ulong[] palette = new[] { TextureImageCommand(0, 2, 16, TextureImage + 0x800), TileCommand(7, 0, 0, 0, 0x100), LoadCommand(0x30, 7, 0, 0, 255 << 2, 0) };
            Draw("copy through a palette", Modes(cycle: 2, palette: true), With(Loaded(2, 1), palette), wide);
            Draw("copy a four-bit index through a palette", Modes(cycle: 2, palette: true), With(Loaded(2, 0, palette: 3), palette), wide);
            Draw("copy through an intensity-alpha palette", Modes(cycle: 2, palette: true, paletteIa: true), With(Loaded(2, 1), palette), wide);
            Draw("copy through a palette loaded from an odd address", Modes(cycle: 2, palette: true),
                With(Loaded(2, 1), new[] { TextureImageCommand(0, 2, 16, TextureImage + 0x801), TileCommand(7, 0, 0, 0, 0x100),
                    LoadCommand(0x30, 7, 0, 0, 255 << 2, 0) }), wide);
            Draw("copy a YUV tile through a palette", Modes(cycle: 2, palette: true), With(Loaded(1, 2), palette), wide,
                dispute: "angrylion reads a copied YUV tile's chroma a further step along in texture memory, where parallel-rdp's copy path has no YUV case");

            Draw("copy a tile in the upper half of texture memory", copy, Loaded(0, 2, tile: 1, memory: 0x100), Rect(12, 10, 100, 90, tile: 1));
            Draw("copy a masked and mirrored tile", copy, Loaded(0, 2, maskS: 3, maskT: 2, mirrorS: true), wide);
            Draw("copy a shifted tile", copy, Loaded(0, 2, shiftS: 1, shiftT: 12), wide);
            Draw("copy a tile with a size and a load offset", copy, Loaded(0, 2, column: 3, row: 2).Append(TileSizeCommand(0, 9, 13, 37, 29)).ToArray(), wide);

            Draw("copy a textured triangle", copy, Loaded(0, 2), Shaded(0x0A, p, q, r));
            Draw("copy a perspective triangle", Modes(cycle: 2, perspective: true), Loaded(0, 2), Shaded(0x0A, p, q, r));
            Draw("copy a mipmapped triangle", Modes(cycle: 2, perspective: true, lod: true), Mipmaps(), Shaded(0x0A | (3 << 19), p, q, r));
            Draw("copy a detail texture", Modes(cycle: 2, perspective: true, lod: true, detail: true), Mipmaps(), Shaded(0x0A | (3 << 19), p, q, r),
                dispute: "angrylion picks the copy mode's tile by the level of detail, and parallel-rdp copies from the primitive's tile");
            Draw("copy a mipmapped rectangle", Modes(cycle: 2, lod: true), Mipmaps(), wide);

            var inside = new Vertex(4.0, 2.0, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var insideB = new Vertex(26.0, 8.0, 10, 240, 90, 20, 0x30000, S: 40, T: 6, W: 1.0);
            var insideC = new Vertex(8.0, 28.0, 40, 70, 230, 140, 0x1C000, S: 5, T: 44, W: 1.0);
            Draw("copy a triangle inside the scissor", copy, Loaded(0, 2), Shaded(0x0A, inside, insideB, insideC));
            Draw("copy a triangle clipped on the scissor's left", copy, Loaded(0, 2), Shaded(0x0A, inside with { X = -6.0 }, insideB, insideC));
            Draw("copy a triangle clipped on the scissor's right", copy, Loaded(0, 2), Shaded(0x0A, inside, insideB with { X = 46.0 }, insideC));
            Draw("copy a rectangle reaching past the scissor", copy, Loaded(0, 2), Rect(12, 10, 200, 90));
            Draw("copy a rectangle whose step is not whole texels", copy, Loaded(0, 2), Rect(12, 10, 100, 90, 0x0A00));
            Draw("copy a rectangle and a triangle in one primitive list", copy, Loaded(0, 2), With(Rect(12, 10, 60, 50), Shaded(0x0A, inside, insideB, insideC)));
            Draw("copy a rectangle stepping backwards", copy, Loaded(0, 2), Rect(12, 10, 100, 90, unchecked((short)0xF800), unchecked((short)0xFC00)));
            Draw("copy a flipped rectangle with an odd step", copy, Loaded(0, 2), Rect(12, 10, 100, 90, 0x0A00, 0x1000, flip: true));
            Draw("copy a rectangle from a mipmap chain's third tile", copy, Mipmaps(), Rect(12, 10, 100, 90, tile: 2));
            Draw("copy a triangle over a mipmap chain", copy, Mipmaps(), Shaded(0x0A, inside, insideB, insideC));
            Draw("copy a perspective triangle whose w varies widely", Modes(cycle: 2, perspective: true), Loaded(0, 2),
                Shaded(0x0A, inside with { W = 0.2 }, insideB with { W = 1.0 }, insideC with { W = 0.5 }));
            Draw("copy a thirty-two-bit texture into an eight-bit image", copy, Loaded(0, 3), wide, size: Bits8);
            Draw("copy a four-bit intensity texture into a sixteen-bit image", copy, Loaded(4, 0), wide);
            Draw("copy a triangle with negative texture coordinates", copy, Loaded(0, 2),
                Shaded(0x0A, inside with { S = -30, T = -12 }, insideB, insideC));
            Draw("copy a tile masked past its size", copy, Loaded(0, 2, maskS: 2, maskT: 2), Rect(12, 10, 100, 90, 0x2000));
            Draw("copy a triangle whose major edge is on the right", copy, Loaded(0, 2),
                Shaded(0x0A, inside with { X = 26.0, Y = 2.0 }, insideB with { X = 4.0, Y = 8.0 }, insideC with { X = 22.0, Y = 28.0 }),
                dispute: Torn);
            Draw("copy a right-major triangle with no texture step", copy, Loaded(0, 2),
                Shaded(0x0A, inside with { X = 26.0, Y = 2.0, S = 8, T = 4 }, insideB with { X = 4.0, Y = 8.0, S = 8, T = 4 },
                    insideC with { X = 22.0, Y = 28.0, S = 8, T = 4 }),
                dispute: Torn);
            Draw("copy a left-major triangle with no texture step", copy, Loaded(0, 2),
                Shaded(0x0A, inside with { S = 8, T = 4 }, insideB with { S = 8, T = 4 }, insideC with { S = 8, T = 4 }));
            Draw("copy a clipped triangle whose major edge is on the right", copy, Loaded(0, 2),
                Shaded(0x0A, inside with { X = 26.0, Y = 2.0 }, insideB with { X = -6.0, Y = 8.0 }, insideC with { X = 22.0, Y = 28.0 }),
                dispute: Torn);

            ulong[] Background(int size, bool tested = false) => new[] { ColorImage(Bits16, 32, Framebuffer), FillCycle, FillColor(0x1234_5678),
                Rectangle(0, 0, 124, 124), ColorImage(size, 32, Framebuffer), Modes(cycle: 2, alphaCompare: tested) };

            Draw("copy into a four-bit image over a filled background", copy, Loaded(0, 2), With(Background(Bits4), Rect(12, 10, 100, 90)), size: Bits4, crossChecked: false);
            Draw("copy into a four-bit image with alpha compare", copy, Loaded(0, 2), With(Background(Bits4, tested: true), Rect(12, 10, 100, 90)), size: Bits4, crossChecked: false);
            Draw("copy into an eight-bit image over a filled background", copy, Loaded(0, 1), With(Background(Bits8), Rect(12, 10, 100, 90)), size: Bits8);
            Draw("copy a tile whose line and row pass nine bits", copy, Loaded(0, 2, width: 32, height: 64), Rect(12, 10, 100, 120, 0x1000, 0x1000));
            Draw("copy an eight-bit texture from an odd column", copy, Loaded(0, 1), TextureRectangle(false, 0, 12, 10, 100, 90, 0x20, 0, 0x1000, 0x0400));
            Draw("copy coordinates past the divider's clamp", Modes(cycle: 2, perspective: true), Loaded(0, 2),
                Shaded(0x0A, inside with { S = 3000, W = 0.05 }, insideB with { S = -2400, W = 1.0 }, insideC with { T = 2600, W = 0.4 }));
            Draw("copy from a tile at the top of texture memory", copy, Loaded(0, 2, tile: 3, memory: 0x1F0), Rect(12, 10, 100, 90, tile: 3));
            Draw("copy a thirty-two-bit texture from texture memory's upper half", copy, Loaded(0, 3, tile: 1, memory: 0x100), Rect(12, 10, 100, 90, tile: 1));

            // Random cases keep away from the disputed textures and images: no YUV, no four-bit palette or intensity-alpha tile, and alpha compare only in a sixteen-bit image.
            var random = new Random(0x434F_5059);
            for (int n = 0; n < 120; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                int format = Pick(0, 0, 2, 3, 4), size = format == 0 ? Pick(1, 2, 2, 3) : random.Next(3);
                int image = Pick(Bits16, Bits16, Bits8);
                bool tlut = format == 2 && Coin();

                // An intensity-alpha or colour-indexed texel narrower than sixteen bits is one of §5.2's disputes, unless a palette reads it.
                if (format == 3 && size != 2) size = 2;
                if (format == 2 && size == 0 && !tlut) size = 1;
                ulong[] setup = Mipmaps(0, random.Next(1, 5), (uint)Pick(8, 16, 32), format, size, Coin());
                if (tlut)
                {
                    setup = setup.Concat(new[] { TextureImageCommand(0, 2, 16, TextureImage + 0x800), TileCommand(7, 0, 0, 0, 0x100),
                        LoadCommand(0x30, 7, 0, 0, 255 << 2, 0) }).ToArray();
                }

                ulong modes = Modes(cycle: 2, alphaCompare: image == Bits16 && random.Next(3) == 0, palette: tlut, paletteIa: tlut && Coin(),
                    perspective: Coin());

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 3); t < count; t++)
                {
                    if (random.Next(3) == 0)
                    {
                        uint left = (uint)random.Next(0, 100), top = (uint)random.Next(0, 100);
                        shapes.AddRange(TextureRectangle(Coin(), random.Next(4), left, top, left + (uint)random.Next(4, 60), top + (uint)random.Next(4, 60),
                            (short)random.Next(-0x400, 0x800), (short)random.Next(-0x400, 0x800), (short)random.Next(-0x2000, 0x6000), (short)random.Next(-0x1800, 0x1800)));
                    }
                    else
                    {
                        int reach = Pick(32, 128, 512);
                        Vertex Point() => new(random.Next(-8, 136) / 4.0, random.Next(-8, 136) / 4.0, random.Next(256), random.Next(256), random.Next(256), random.Next(256),
                            random.Next(0x40000), random.Next(-reach, reach), random.Next(-reach, reach), 0.2 + 0.8 * random.NextDouble());

                        // A right-major span is §5.2's torn dispute, so a triangle that would walk right to left is mirrored.
                        Vertex[] points = { Point(), Point(), Point() };
                        Vertex[] byRow = points.OrderBy(v => v.Y).ToArray();
                        double slope = byRow[2].Y > byRow[0].Y ? (byRow[2].X - byRow[0].X) / (byRow[2].Y - byRow[0].Y) : 0;
                        if (byRow[1].X <= byRow[0].X + slope * (byRow[1].Y - byRow[0].Y))
                        {
                            for (int v = 0; v < 3; v++) points[v] = points[v] with { X = 32.0 - points[v].X };
                        }

                        shapes.AddRange(Shaded(Pick(0x0A, 0x0B, 0x0E, 0x0F) | (random.Next(4) << 19), points[0], points[1], points[2]));
                    }
                }

                Draw($"random copy case {n}", modes, setup, shapes.ToArray(), size: image);
            }

            return cases;
        }

        private static ulong KeyRed(int width, int centre, int scale) =>
            (0x2BUL << 56) | ((ulong)(uint)(width & 0xFFF) << 16) | ((ulong)(uint)(centre & 0xFF) << 8) | (uint)(scale & 0xFF);

        private static ulong KeyGreenBlue(int widthG, int centreG, int scaleG, int widthB, int centreB, int scaleB) =>
            (0x2AUL << 56) | ((ulong)(uint)(widthG & 0xFFF) << 44) | ((ulong)(uint)(widthB & 0xFFF) << 32)
            | ((ulong)(uint)(centreG & 0xFF) << 24) | ((ulong)(uint)(scaleG & 0xFF) << 16) | ((ulong)(uint)(centreB & 0xFF) << 8) | (uint)(scaleB & 0xFF);

        // Chroma key: the key alpha the last combiner cycle measures, and the first input it passes through - see Mars_RdpChromaKey.md §3.
        private static IEnumerable<Case> ChromaKeyCases()
        {
            var cases = new List<Case>();
            const string Missing = "parallel-rdp does not implement chroma keying at all, which its own README lists as a missing feature";

            void Draw(string name, ulong modes, ulong combine, ulong[] setup, ulong[] shapes, int size = Bits16) =>
                cases.Add(Textured(name, modes, combine, setup, shapes, size, dispute: null, crossChecked: false));

            ulong[] With(ulong[] first, params ulong[][] rest) => rest.Aggregate(first, (all, next) => all.Concat(next).ToArray());

            var p = new Vertex(2.25, 1.5, 250, 20, 60, 255, 0x08000, S: 0, T: 0, W: 1.0);
            var q = new Vertex(29.75, 6.75, 10, 240, 90, 20, 0x30000, S: 40, T: 6, W: 0.55);
            var r = new Vertex(6.5, 29.25, 40, 70, 230, 140, 0x1C000, S: 5, T: 44, W: 0.8);
            ulong[] triangle = Shaded(0x0F, p, q, r);

            // The key's distance is visible as a weight against the memory colour, and its bypass as the colour that weight is applied to.
            ulong keyed = Modes(keyEnabled: true, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0, forceBlend: true, imageRead: true);
            ulong[] key = { KeyRed(0x60, 0x80, 0x40), KeyGreenBlue(0x80, 0x60, 0x30, 0x40, 0xA0, 0x50) };

            // (texel - key centre) x key scale, which is what content keys with.
            ulong distance = Combine(subA: 1, subB: 6, mul: 6, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1);
            ulong shadeDistance = Combine(subA: 4, subB: 6, mul: 6, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 4);

            Draw("chroma key on a texel's distance from the key", keyed, distance, With(key, Loaded(0, 2)), triangle);
            Draw("chroma key on shade's distance from the key", keyed, shadeDistance, key, Shaded(0x0C, p, q, r));
            Draw("chroma key passing its first input through", keyed, Combine(subA: 1, subB: 6, mul: 6, add: 3, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 1),
                With(key, Loaded(0, 2)), triangle);
            Draw("chroma key with no width", keyed, distance, With(new[] { KeyRed(0, 0x80, 0x40), KeyGreenBlue(0, 0x60, 0x30, 0, 0xA0, 0x50) }, Loaded(0, 2)), triangle);
            Draw("chroma key with every width at its widest", keyed, distance,
                With(new[] { KeyRed(0xFFF, 0x80, 0x40), KeyGreenBlue(0xFFF, 0x60, 0x30, 0xFFF, 0xA0, 0x50) }, Loaded(0, 2)), triangle);
            Draw("chroma key on one channel alone", keyed, distance,
                With(new[] { KeyRed(0x40, 0x80, 0x40), KeyGreenBlue(0xFFF, 0x60, 0, 0xFFF, 0xA0, 0) }, Loaded(0, 2)), triangle);
            Draw("chroma key with no scale, so every distance is the centre's", keyed, distance,
                With(new[] { KeyRed(0x60, 0x80, 0), KeyGreenBlue(0x80, 0x60, 0, 0x40, 0xA0, 0) }, Loaded(0, 2)), triangle);
            Draw("chroma key where a distance's low nibble is eight", keyed, distance,
                With(new[] { KeyRed(0x58, 0x20, 8), KeyGreenBlue(0x58, 0x20, 8, 0x58, 0x20, 8) }, Loaded(0, 2)), triangle);
            Draw("chroma key measuring the primitive colour", keyed, Combine(subA: 3, subB: 6, mul: 6, add: 7, alphaSubA: 7, alphaSubB: 7, alphaMul: 7, alphaAdd: 3),
                key, Shaded(0x08, p, q, r));

            Draw("chroma key with alpha from coverage", Modes(keyEnabled: true, biLerp0: true, alphaCvgSelect: true, antialias: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0,
                forceBlend: true, imageRead: true), distance, With(key, Loaded(0, 2)), triangle);
            Draw("chroma key with coverage times alpha", Modes(keyEnabled: true, biLerp0: true, cvgTimesAlpha: true, antialias: true, imageRead: true, cvgDest: 0,
                m1a: 0, m1b: 0, m2a: 1, m2b: 0, forceBlend: true), distance, With(key, Loaded(0, 2)), triangle);
            Draw("chroma key with a dithered alpha", Modes(keyEnabled: true, biLerp0: true, alphaDither: 0, m1a: 0, m1b: 0, m2a: 1, m2b: 0, forceBlend: true,
                imageRead: true), distance, With(key, Loaded(0, 2)), triangle);
            Draw("chroma key tested by alpha compare", Modes(keyEnabled: true, biLerp0: true, alphaCompare: true), distance, With(key, Loaded(0, 2)), triangle);

            Draw("chroma key in the two-cycle mode", Modes(cycle: 1, keyEnabled: true, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0, forceBlend: true, imageRead: true),
                CombineCycles(new[] { 1, 8, 4, 7, 1, 7, 4, 7 }, new[] { 0, 6, 6, 7, 7, 7, 7, 0 }), With(key, Loaded(0, 2)), triangle);
            Draw("chroma key on the two-cycle mode's first cycle alone", Modes(cycle: 1, keyEnabled: true, biLerp0: true, m1a: 0, m1b: 0, m2a: 1, m2b: 0,
                forceBlend: true, imageRead: true), CombineCycles(new[] { 1, 6, 6, 7, 1, 7, 7, 7 }, new[] { 0, 8, 16, 0, 7, 7, 7, 0 }), With(key, Loaded(0, 2)), triangle);

            var random = new Random(0x4B45_5901);
            for (int n = 0; n < 60; n++)
            {
                int Pick(params int[] choices) => choices[random.Next(choices.Length)];
                bool Coin() => random.Next(2) == 1;

                ulong[] keys = { KeyRed(random.Next(0x1000), random.Next(256), random.Next(256)),
                    KeyGreenBlue(random.Next(0x1000), random.Next(256), random.Next(256), random.Next(0x1000), random.Next(256), random.Next(256)) };

                int[] Cycle() => new[]
                {
                    Pick(1, 2, 3, 4, 5, 6, 8), Pick(6, 6, 1, 2, 3, 4, 5, 8), Pick(6, 6, 1, 2, 3, 4, 10, 11, 16),
                    Pick(7, 7, 0, 1, 2, 3, 4, 6), Pick(1, 2, 3, 4, 6, 7), Pick(1, 2, 3, 4, 7), Pick(0, 1, 2, 3, 4, 6, 7), Pick(1, 2, 3, 4, 6, 7),
                };

                int cycle = Coin() ? 1 : 0;
                ulong modes = Modes(cycle: cycle, keyEnabled: true, biLerp0: true, rgbDither: Pick(0, 1, 3), alphaDither: Pick(0, 1, 3),
                    m1a: 0, m1b: 0, m2a: Pick(1, 2, 3), m2b: 0, m1a1: 0, m1b1: 0, m2a1: Pick(1, 2, 3), m2b1: 0, forceBlend: true, imageRead: true,
                    alphaCvgSelect: random.Next(4) == 0, cvgTimesAlpha: random.Next(4) == 0, antialias: Coin(), alphaCompare: random.Next(4) == 0,
                    perspective: Coin(), sampleFour: random.Next(3) == 0);

                var shapes = new List<ulong>();
                for (int t = 0, count = random.Next(1, 3); t < count; t++)
                {
                    Vertex Point() => new(random.Next(-8, 136) / 4.0, random.Next(-8, 136) / 4.0, random.Next(256), random.Next(256), random.Next(256),
                        random.Next(256), random.Next(0x40000), random.Next(-64, 64), random.Next(-64, 64), 0.3 + 0.7 * random.NextDouble());
                    shapes.AddRange(Shaded(Pick(0x0A, 0x0C, 0x0E, 0x0F), Point(), Point(), Point()));
                }

                Draw($"random chroma key case {n}", modes, CombineCycles(Cycle(), Cycle()), With(keys, Loaded(Pick(0, 0, 3, 4), Pick(1, 2, 2))), shapes.ToArray());
            }

            Draw("chroma key in the fill cycle", Modes(keyEnabled: true), distance, With(key, Loaded(0, 2)), new[] { FillCycle, FillColor(0x3C64_A21F), Rectangle(8, 12, 100, 88) });
            Draw("chroma key in the copy mode", Modes(cycle: 2, keyEnabled: true), distance, With(key, Loaded(0, 2)),
                TextureRectangle(false, 0, 12, 10, 100, 90, 0, 0, 0x1000, 0x0400));

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
            words[0] |= (ulong)((id >> 16) & 7) << 48 | (ulong)((id >> 19) & 7) << 51;
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
