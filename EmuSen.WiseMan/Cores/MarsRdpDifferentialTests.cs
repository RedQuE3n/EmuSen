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
        private sealed record Case(string Name, uint Image, int Size, int Width, ulong[] Commands, string? Artefact = null, string? Dispute = null, bool CrossChecked = true);

        private sealed record Drawn(byte[] Rdram, byte[] Hidden);

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
                dump.Reset();
                foreach (ulong[] command in Split(c.Commands)) dump.Command(command);
                dump.Sync();
            }

            File.WriteAllBytes(path, dump.Finish());
            return path;
        }

        // One display processor for every case, as in the references, with memory cleared where they flush.
        private static IReadOnlyList<Drawn> ReplayMars()
        {
            var bus = new MarsBus(expansionPak: true);
            var drawn = new List<Drawn>();

            foreach (Case c in Cases)
            {
                Array.Clear(bus.Rdram);
                Array.Clear(bus.RdramHidden);
                foreach (ulong word in c.Commands) bus.Dp.Processor.Accept(word);
                drawn.Add(new Drawn((byte[])bus.Rdram.Clone(), (byte[])bus.RdramHidden.Clone()));
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

            Compare(reference.Pages, mars.Rdram, 1, "");
            Compare(reference.HiddenPages, mars.Hidden, 2, "hidden ");

            if (count > 12) report.AppendLine($"  ...{count} differing bytes in all");
            return report.ToString();

            void Compare(IReadOnlyDictionary<uint, byte[]> expectedPages, byte[] actual, int scale, string label)
            {
                var pages = new SortedSet<uint>(expectedPages.Keys);
                for (uint page = 0; page < actual.Length; page += RdpReference.PageSize)
                {
                    if (actual.AsSpan((int)page, RdpReference.PageSize).IndexOfAnyExcept((byte)0) >= 0) pages.Add(page);
                }

                foreach (uint page in pages)
                {
                    byte[] expected = expectedPages.TryGetValue(page, out byte[]? p) ? p : new byte[RdpReference.PageSize];

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
            bool zSource = false, bool alphaCompare = false)
        {
            ulong blender = (ulong)(uint)((m1a << 30) | (m1a << 28) | (m1b << 26) | (m1b << 24) | (m2a << 22) | (m2a << 20) | (m2b << 18) | (m2b << 16));
            return (0x2FUL << 56) | ((ulong)rgbDither << 38) | ((ulong)alphaDither << 36) | blender
                | (forceBlend ? 1UL << 14 : 0) | (alphaCvgSelect ? 1UL << 13 : 0) | (cvgTimesAlpha ? 1UL << 12 : 0) | ((ulong)cvgDest << 8)
                | (colorOnCvg ? 1UL << 7 : 0) | (imageRead ? 1UL << 6 : 0) | (antialias ? 1UL << 3 : 0) | (zSource ? 1UL << 2 : 0) | (alphaCompare ? 1UL : 0);
        }

        // Both cycles given the same inputs, since the one-cycle mode reads the second.
        private static ulong Combine(int subA, int subB, int mul, int add, int alphaSubA, int alphaSubB, int alphaMul, int alphaAdd)
        {
            ulong high = (ulong)((subA << 20) | (mul << 15) | (alphaSubA << 12) | (alphaMul << 9) | (subA << 5) | mul);
            ulong low = (ulong)(uint)((subB << 28) | (subB << 24) | (alphaSubA << 21) | (alphaMul << 18) | (add << 15) | (alphaSubB << 12) | (alphaAdd << 9) | (add << 6) | (alphaSubB << 3) | alphaAdd);
            return (0x3CUL << 56) | (high << 32) | low;
        }

        private static ulong Color(uint id, uint rgba, uint extra = 0) => ((ulong)id << 56) | ((ulong)extra << 32) | rgba;

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
