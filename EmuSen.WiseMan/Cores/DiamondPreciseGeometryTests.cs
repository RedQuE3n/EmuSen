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
    // Diamond's "Precise geometry" as a host meets it: the schema's enhancement in the PlayStation's Enhancements group with its cost, and turned on and off between frames it changes the picture alone - see Diamond_Enhancements.md §24.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondPreciseGeometryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DiamondPreciseGeometryTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondPrecise_" + Guid.NewGuid().ToString("N"));

        public DiamondPreciseGeometryTests(ITestOutputHelper output)
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

        [Fact]
        public void Precise_geometry_is_the_playstations_enhancement_off_by_default_with_its_cost()
        {
            var library = CoreLibrary.Open(DiamondTests.LibraryPath);
            Assert.True(library.Has(CoreInterface.CapSettings) && library.Has(CoreInterface.CapSettingNotes), library.Report);
            var s = Assert.Single(library.Settings, x => x.Effect == "enhancement");
            Assert.Equal(("precision", "Precise geometry", "switch", "false", false, "Enhancements"), (s.Key, s.Label, s.Kind, s.Default, s.CreateScope, s.Category));
            Assert.Contains("Vulkan", s.Cost);
            var row = Assert.Single(CoreCatalog.SettingsFor("PlayStation"), x => x.Key == "precision");
            Assert.Equal((CoreSettingDescriptor.EnhancementsGroup, CoreSettingKind.Switch), (row.Group, row.Kind));
            Assert.EndsWith(s.Cost!, row.Hint);
            Assert.All(CoreCatalog.SettingsFor("PlayStation").Where(x => x.Key != "precision"), x => Assert.Null(x.Group));
        }

        [Fact]
        public Task The_playstations_tab_shows_precise_geometry_under_its_enhancements_heading() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), _ => { }, "PlayStation");
            window.Show();
            window.CaptureRenderedFrame();
            var panel = window.GetVisualDescendants().OfType<StackPanel>().First(p => p.Children.OfType<SectionHeader>().Any(h => h.Name == "PlayStation.Group.Enhancements"));
            var children = panel.Children.ToList();
            int heading = children.FindIndex(c => c.Name == "PlayStation.Group.Enhancements");
            var next = Assert.IsType<FieldRow>(children[heading + 1]);
            Assert.Equal("Precise geometry", next.Label);
            Assert.Contains("Vulkan", next.Hint);
            Assert.NotNull(next.GetVisualDescendants().OfType<LunaSwitch>().FirstOrDefault(t => t.Name == "PlayStation.precision"));
            Assert.Single(children.OfType<SectionHeader>());
            window.Close();
        }, default);

        // Through the host, between frames: the sound and the machine's state are those of a run that never had the setting, and the picture differs only while it is on; where no Vulkan device is usable the row's note says so and nothing changes.
        [Fact]
        public void Turning_precise_geometry_on_and_off_between_frames_changes_only_the_picture()
        {
            string path = Path.Combine(_root, "triangle.psexe");
            File.WriteAllBytes(path, Triangle());
            using var plain = (CoreEngine)CoreFactory.Load(path).Core;
            using var core = (CoreEngine)CoreFactory.Load(path).Core;
            Assert.Equal(("diamond", "false"), (core.Info.Id, core.Get("precision")));
            int differed = 0;
            bool drawn = true;
            for (int f = 0; f < 12; f++)
            {
                if (f == 3)
                {
                    core.Set("precision", "true");
                    string? note = core.Settings.Single(s => s.Key == "precision").Note?.Invoke(_ => "");
                    _output.WriteLine($"the note: {note ?? "none"}");
                    drawn = note is null || !note.StartsWith("No usable");
                }
                if (f == 9)
                {
                    core.Set("precision", "false");
                    Assert.Null(core.Settings.Single(s => s.Key == "precision").Note?.Invoke(_ => ""));
                }
                plain.RunFrame();
                core.RunFrame();
                Assert.Equal(plain.DequeueAudioSamples(4096), core.DequeueAudioSamples(4096));
                byte[] a = State(plain), b = State(core);
                Assert.True(b.Length >= a.Length && a.AsSpan().SequenceEqual(b.AsSpan(0, a.Length)), $"frame {f}: the machine's state differs");
                if (!plain.GetFrameBufferRgba().AsSpan().SequenceEqual(core.GetFrameBufferRgba()))
                {
                    differed++;
                    Assert.InRange(f, 3, 8);
                }
            }
            if (drawn) Assert.True(differed > 0, "the precise triangle drew no pixel otherwise");
            else Assert.Equal(0, differed);
        }
    }
}
