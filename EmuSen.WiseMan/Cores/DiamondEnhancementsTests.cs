using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Diamond's enhancements as a host meets them: the schema's four in the PlayStation's Enhancements group with their costs, and turned on and off between frames they change the picture alone - see Diamond_Enhancements.md §24.2 and §42.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondEnhancementsTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DiamondEnhancementsTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondEnhancements_" + Guid.NewGuid().ToString("N"));

        public DiamondEnhancementsTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = _root;
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static uint I(uint op, int rs, int rt, int imm) => op << 26 | (uint)rs << 21 | (uint)rt << 16 | (ushort)imm;

        private static uint Cop(uint top, int rt, int rd) => top | (uint)rt << 16 | (uint)rd << 11;

        // A program that each frame clears the screen, projects a triangle by RTPT, its third corner half a pixel down, and draws it shaded from the GTE's words by SWC2 straight to GP0, then waits for the vertical blank.
        internal static byte[] Triangle()
        {
            const int zero = 0, t0 = 8, a0 = 4, t9 = 25;
            uint Lui(int rt, int imm) => I(0x0F, 0, rt, imm);
            uint Ori(int rt, int rs, int imm) => I(0x0D, rs, rt, imm);
            uint Sw(int rt, int imm) => I(0x2B, a0, rt, imm);
            uint Ctc2(int rt, int rd) => Cop(0x48C0_0000, rt, rd);
            uint Mtc2(int rt, int rd) => Cop(0x4880_0000, rt, rd);
            uint Swc2(int rt) => I(0x3A, a0, rt, 0x1810);
            var w = new List<uint> { Lui(t9, 0x4000), Cop(0x4080_0000, t9, 12), Lui(a0, 0x1F80), Sw(zero, 0x1814), Lui(t0, 0x0300), Sw(t0, 0x1814) };
            w.AddRange(new[] { Ori(t0, zero, 0x1000), Ctc2(t0, 0), Ctc2(t0, 2), Ctc2(t0, 4), Lui(t0, 160), Ctc2(t0, 24), Lui(t0, 120), Ctc2(t0, 25), Ori(t0, zero, 0x100), Ctc2(t0, 26) });
            w.AddRange(new[] { Lui(t0, 0xE300), Sw(t0, 0x1810), Lui(t0, 0xE407), Ori(t0, t0, 0xFFFF), Sw(t0, 0x1810), Lui(t0, 0xE500), Sw(t0, 0x1810) });
            int loop = w.Count;
            w.AddRange(new[] { Lui(t0, 0x0200), Sw(t0, 0x1810), Sw(zero, 0x1810), Lui(t0, 0x00F0), Ori(t0, t0, 0x0140), Sw(t0, 0x1810) });
            foreach (var (xy, z, n) in new[] { (0x0040_0080u, 0x400, 0), (0xFFC0_0040u, 0x800, 1), (0x0005_FF90u, 0x200, 2) })
                w.AddRange(new[] { Lui(t0, (int)(xy >> 16)), Ori(t0, t0, (int)(xy & 0xFFFF)), Mtc2(t0, 2 * n), Ori(t0, zero, z), Mtc2(t0, 2 * n + 1) });
            w.AddRange(new[] { 0u, 0x4A28_0030u });
            w.AddRange(new[] { Lui(t0, 0x3000), Ori(t0, t0, 0x00FF), Sw(t0, 0x1810), Swc2(12), Ori(t0, zero, 0xFF00), Sw(t0, 0x1810), Swc2(13), Lui(t0, 0x00FF), Sw(t0, 0x1810), Swc2(14) });
            int wait = w.Count;
            w.AddRange(new[] { I(0x23, a0, t0, 0x1070), 0u, I(0x0C, t0, t0, 1) });
            w.Add(I(0x04, t0, zero, wait - (w.Count + 1)));
            w.AddRange(new[] { 0u, Lui(t0, 0xFFFF), Ori(t0, t0, 0xFFFE), Sw(t0, 0x1070), 0x0800_0000u | ((SyntheticPsExe.Origin + 4 * (uint)loop) >> 2 & 0x03FF_FFFF), 0u });
            return SyntheticPsExe.Build(SyntheticPsExe.Words(w.ToArray()));
        }

        private static byte[] State(CoreEngine core)
        {
            using var s = new MemoryStream();
            core.SaveState(s);
            return s.ToArray();
        }

        private static readonly (string Key, string Label, string Kind, string Default)[] Enhancements =
        {
            ("precision", "Precise geometry", "switch", "false"), ("resolution", "Internal resolution", "choice", "1"), ("colour", "True colour", "switch", "false"), ("filtering", "Texture filtering", "choice", "off"),
        };

        [Fact]
        public void The_playstations_four_enhancements_are_offered_at_the_hardwares_values_with_their_costs()
        {
            var library = CoreLibrary.Open(DiamondTests.LibraryPath);
            Assert.True(library.Has(CoreInterface.CapSettings) && library.Has(CoreInterface.CapSettingNotes), library.Report);
            var offered = library.Settings.Where(x => x.Effect == "enhancement").ToList();
            Assert.Equal(Enhancements, offered.Select(s => (s.Key, s.Label, s.Kind, s.Default)));
            Assert.All(offered, s => Assert.False(s.CreateScope));
            Assert.All(offered, s => Assert.Equal("Enhancements", s.Category));
            Assert.All(offered, s => Assert.Contains("Vulkan", s.Cost));
            var rows = CoreCatalog.SettingsFor("PlayStation");
            foreach (var s in offered)
            {
                var row = Assert.Single(rows, x => x.Key == s.Key);
                Assert.Equal(CoreSettingDescriptor.EnhancementsGroup, row.Group);
                Assert.EndsWith(s.Cost!, row.Hint);
            }
            Assert.Equal(new[] { "1", "2", "3", "4" }, rows.Single(x => x.Key == "resolution").Choices);
            Assert.All(rows.Where(x => offered.All(s => s.Key != x.Key)), x => Assert.Null(x.Group));
        }

        [Fact]
        public Task The_playstations_tab_shows_the_enhancements_under_their_heading() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), _ => { }, "PlayStation");
            window.Show();
            window.CaptureRenderedFrame();
            var panel = window.GetVisualDescendants().OfType<StackPanel>().First(p => p.Children.OfType<SectionHeader>().Any(h => h.Name == "PlayStation.Group.Enhancements"));
            var children = panel.Children.ToList();
            int heading = children.FindIndex(c => c.Name == "PlayStation.Group.Enhancements");
            for (int i = 0; i < Enhancements.Length; i++)
            {
                var row = Assert.IsType<FieldRow>(children[heading + 1 + i]);
                Assert.Equal(Enhancements[i].Label, row.Label);
                Assert.Contains("Vulkan", row.Hint);
                string name = "PlayStation." + Enhancements[i].Key;
                Assert.True(Enhancements[i].Kind == "switch"
                    ? row.GetVisualDescendants().OfType<LunaSwitch>().Any(t => t.Name == name)
                    : row.GetVisualDescendants().OfType<Control>().Any(c => c.Name == name), name);
            }
            Assert.Single(children.OfType<SectionHeader>());
            window.Close();
        }, default);

        // Through the host, between frames: the sound and the machine's state are those of a run that never had the setting, and the picture differs only while it is on; where no Vulkan device is usable the row's note says so and nothing changes.
        [Theory]
        [InlineData("precision=true")]
        [InlineData("resolution=4")]
        [InlineData("colour=true")]
        [InlineData("filtering=3d")]
        [InlineData("precision=true resolution=2 colour=true filtering=all")]
        public void Turning_enhancements_on_and_off_between_frames_changes_only_the_picture(string chosen)
        {
            string path = Path.Combine(_root, "triangle.psexe");
            File.WriteAllBytes(path, Triangle());
            using var plain = (CoreEngine)CoreFactory.Load(path).Core;
            using var core = (CoreEngine)CoreFactory.Load(path).Core;
            var pairs = chosen.Split(' ').Select(p => p.Split('=')).Select(p => (Key: p[0], Value: p[1])).ToList();
            Assert.Equal("diamond", core.Info.Id);
            int differed = 0;
            bool drawn = true;
            for (int f = 0; f < 12; f++)
            {
                if (f == 3)
                {
                    foreach (var (key, value) in pairs) core.Set(key, value);
                    string? note = core.Settings.Single(s => s.Key == pairs[0].Key).Note?.Invoke(_ => "");
                    _output.WriteLine($"the note: {note ?? "none"}");
                    drawn = note is null || !note.StartsWith("No usable");
                }
                if (f == 9)
                {
                    foreach (var (key, _) in pairs) core.Set(key, core.Settings.Single(s => s.Key == key).Default);
                    Assert.All(pairs, p => Assert.Null(core.Settings.Single(s => s.Key == p.Key).Note?.Invoke(_ => "")));
                }
                plain.RunFrame();
                core.RunFrame();
                Assert.Equal(plain.DequeueAudioSamples(4096), core.DequeueAudioSamples(4096));
                byte[] a = State(plain), b = State(core);
                Assert.True(b.Length >= a.Length && a.AsSpan().SequenceEqual(b.AsSpan(0, a.Length)), $"frame {f}: the machine's state differs");
                if (!plain.GetFrameBufferRgba().AsSpan().SequenceEqual(core.GetFrameBufferRgba()))
                {
                    differed++;
                    Assert.InRange(f, 3, 9);
                }
            }
            // The triangle is untextured, which filtering alone leaves as the console draws it.
            if (drawn && chosen != "filtering=3d") Assert.True(differed > 0, "the enhanced picture never differed");
            if (!drawn) Assert.Equal(0, differed);
        }
    }
}
