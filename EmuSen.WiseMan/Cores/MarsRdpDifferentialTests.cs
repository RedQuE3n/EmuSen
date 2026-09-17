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

        private sealed record Replay(IReadOnlyList<RdpReferenceSync> Reference, IReadOnlyList<byte[]> Mars, int AgreementExit, string AgreementLog,
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
        private static IReadOnlyList<byte[]> ReplayMars()
        {
            var bus = new MarsBus(expansionPak: true);
            var drawn = new List<byte[]>();

            foreach (Case c in Cases)
            {
                Array.Clear(bus.Rdram);
                foreach (ulong word in c.Commands) bus.Dp.Processor.Accept(word);
                drawn.Add((byte[])bus.Rdram.Clone());
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

        // Every page either side touched, byte for byte, reported as pixels of the case's colour image.
        private static string Differences(Case c, RdpReferenceSync reference, byte[] mars)
        {
            var pages = new SortedSet<uint>(reference.Pages.Keys);
            for (uint page = 0; page < mars.Length; page += RdpReference.PageSize)
            {
                if (mars.AsSpan((int)page, RdpReference.PageSize).IndexOfAnyExcept((byte)0) >= 0) pages.Add(page);
            }

            var report = new StringBuilder();
            int count = 0;
            int bytes = Math.Max(1, (1 << c.Size) / 2);

            foreach (uint page in pages)
            {
                byte[] expected = reference.Pages.TryGetValue(page, out byte[]? p) ? p : new byte[RdpReference.PageSize];

                for (int i = 0; i < RdpReference.PageSize; i++)
                {
                    if (expected[i] == mars[page + i]) continue;

                    if (count++ < 12)
                    {
                        long pixel = ((long)page + i - (c.Image & ~(uint)(bytes - 1))) / bytes;
                        long x = ((pixel % c.Width) + c.Width) % c.Width, y = (long)Math.Floor((double)pixel / c.Width);
                        report.AppendLine($"  {page + i:X6} pixel ({x}, {y}): Mars {mars[page + i]:X2}, reference {expected[i]:X2}");
                    }
                }
            }

            if (count > 12) report.AppendLine($"  ...{count} differing bytes in all");
            return report.ToString();
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

            return cases.ToArray();
        }

        private static ulong ColorImage(int size, int width, uint address) =>
            (0x3FUL << 56) | ((ulong)size << 51) | ((ulong)(width - 1) << 32) | address;

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
