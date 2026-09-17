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

            string graded = Write(Path.Combine(directory, "fill.rdp"), Cases);
            string agreed = Write(Path.Combine(directory, "fill-cross-checked.rdp"), Cases.Where(c => c.CrossChecked && c.Dispute is null));

            IReadOnlyList<RdpReferenceSync> reference = RdpReference.Replay(graded);
            (int exit, string log) = RdpReference.Agree(agreed);

            var disputes = new Dictionary<string, int>();
            foreach ((Case c, int i) in Cases.Select((c, i) => (c, i)).Where(p => p.c.Dispute is not null))
            {
                disputes[c.Name] = RdpReference.Agree(Write(Path.Combine(directory, $"fill-dispute-{i}.rdp"), new[] { c })).Exit;
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

            return cases.ToArray();
        }

        private static ulong ColorImage(int size, int width, uint address) =>
            (0x3FUL << 56) | ((ulong)size << 51) | ((ulong)(width - 1) << 32) | address;

        private static ulong Scissor(uint left, uint top, uint right, uint bottom, bool field = false, bool keepOdd = false) =>
            (0x2DUL << 56) | ((ulong)left << 44) | ((ulong)top << 32) | ((ulong)right << 12) | bottom
            | (field ? 1UL << 25 : 0) | (keepOdd ? 1UL << 24 : 0);

        private static ulong FillColor(uint color) => (0x37UL << 56) | color;

        private static ulong Rectangle(uint left, uint top, uint right, uint bottom) =>
            (0x36UL << 56) | ((ulong)right << 44) | ((ulong)bottom << 32) | ((ulong)left << 12) | top;
    }
}
