using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Theme;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Moonrise over big picture's start: once, skipped by any press, still under reduced motion, held only by start-up work - see EmuSen_Interface.md §22.
    [Collection(TestCollections.ProcessGlobals)]
    public class StartupSplashTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(StartupSplashTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private static LunaSplash? Splash(ThemedSession s) => (LunaSplash?)typeof(MainWindow).GetProperty("SplashShown", Hidden)!.GetValue(s.Window);

        // Whether the splash still takes the pad and the keys: false while it fades.
        private static bool Taking(ThemedSession s) => (bool)typeof(MainWindow).GetProperty("SplashOpen", Hidden)!.GetValue(s.Window)!;

        private static string Bells => System.IO.Path.GetFileNameWithoutExtension(StartupSplash.BellsKey);

        private static int LengthMs => (int)Math.Round(SplashTimeline.Moonrise.Length * 1000);

        private static ThemedSession Open(Task? work = null, Action<AppSettings>? more = null, double width = 1280, double height = 800) =>
            new(width, height, settings: a => { a.StartupSplash = true; more?.Invoke(a); }, startupWork: work ?? Task.CompletedTask);

        // It covers the library from the first frame, which is drawn beneath it, and is gone at its length with start-up done.
        [Fact]
        public Task It_plays_once_at_the_start_and_never_on_returning_from_a_game() => Session.Dispatch(() =>
        {
            using var s = Open();
            LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
            Assert.True(s.Shown, "big picture is drawn behind it at once");
            s.Run(LengthMs - 16);
            Assert.NotNull(Splash(s));
            Assert.Equal(LengthMs / 1000.0 - 0.016, splash.Time, 3);
            s.Run(192);
            Assert.Null(Splash(s));
            Assert.Equal(1, s.Sounds.Count(n => n == Bells));

            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.A();
            Assert.NotNull(LaunchScreenTests.Running(s.Window));
            LaunchScreenTests.Stop(s.Window);
            typeof(MainWindow).GetMethod("ShowLibrary", Hidden)!.Invoke(s.Window, null);
            s.Settle();
            s.Run(500);
            Assert.Null(Splash(s));
            Assert.Equal(1, s.Sounds.Count(n => n == Bells));
        }, default);

        // It fades into big picture over 150 ms at its end and on a skip, taking no press while it fades (decided 2026-11-04).
        [Fact]
        public Task It_fades_out_over_150_ms_at_its_end_and_on_a_skip() => Session.Dispatch(() =>
        {
            using (var s = Open())
            {
                s.Run(LengthMs + 8);
                LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
                Assert.False(Taking(s));
                s.Run(64);
                Assert.InRange(splash.Opacity, 0.4, 0.7);
                s.Run(96);
                Assert.Null(Splash(s));
            }
            using (var s = Open())
            {
                s.Run(500);
                s.Pad.A();
                LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
                Assert.True(splash.AtEnd);
                s.Run(80);
                Assert.InRange(splash.Opacity, 0.3, 0.8);
                s.Run(80);
                Assert.Null(Splash(s));
            }
        }, default);

        [Fact]
        public Task A_pad_button_skips_to_the_end_and_does_nothing_more() => Session.Dispatch(() =>
        {
            using var s = Open();
            s.Run(800);
            string? system = s.System;
            s.Pad.A();
            Assert.False(Taking(s), "a skipped splash fades and takes no press");
            Assert.Equal("system", s.View);
            Assert.Equal(system, s.System);
            s.Run(3000);
            Assert.DoesNotContain(Bells, s.Sounds);
        }, default);

        [Fact]
        public Task A_key_skips_to_the_end() => Session.Dispatch(() =>
        {
            using var s = Open();
            s.Run(800);
            s.Window.KeyPress(Key.Space, RawInputModifiers.None, PhysicalKey.None, null);
            s.Settle();
            Assert.False(Taking(s));
            Assert.Equal("system", s.View);
        }, default);

        // Skipped while start-up still works, it shows the finished frame and goes the moment the work is done.
        [Fact]
        public Task A_skip_before_start_up_is_done_holds_the_finished_frame_until_it_is() => Session.Dispatch(() =>
        {
            var work = new TaskCompletionSource();
            using var s = Open(work.Task);
            s.Run(500);
            s.Window.KeyPress(Key.Enter, RawInputModifiers.None, PhysicalKey.None, null);
            s.Settle();
            LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
            Assert.True(splash.AtEnd);
            work.SetResult();
            s.Settle();
            Assert.False(Taking(s));
        }, default);

        [Fact]
        public Task Reduced_motion_shows_the_still_frame_for_the_splashs_length() => Session.Dispatch(() =>
        {
            bool before = LunaMotion.Reduced;
            try
            {
                using var s = Open(more: a => a.ReducedMotion = AppSettings.ReducedMotionOn);
                LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
                Assert.True(splash.AtEnd);
                Assert.Equal(splash.Timeline.Length, splash.Time);
                s.Run(LengthMs - 100);
                Assert.Same(splash, Splash(s));
                Assert.Equal(splash.Timeline.Length, splash.Time);
                s.Run(112);
                Assert.Null(Splash(s));
            }
            finally
            {
                LunaMotion.Reduced = before;
            }
        }, default);

        // The bells follow Navigation sounds: off, or at 0 %, they are never played.
        [Theory]
        [InlineData(false, 60)]
        [InlineData(true, 0)]
        public Task The_bells_never_play_with_sounds_off(bool sounds, int volume) => Session.Dispatch(() =>
        {
            using var s = Open(more: a => { a.NavigationSounds = sounds; a.BigPictureInterface.NavigationVolume = volume; });
            s.Run(LengthMs + 200);
            Assert.Null(Splash(s));
            Assert.DoesNotContain(Bells, s.Sounds);
        }, default);

        [Fact]
        public Task The_setting_off_shows_no_splash() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: a => a.StartupSplash = false, startupWork: Task.CompletedTask, splashes: true);
            Assert.Null(Splash(s));
            Assert.True(new AppSettings().StartupSplash, "On until the player turns it off");
        }, default);

        [Fact]
        public Task There_is_a_planet_for_each_system_with_an_engine_in_its_own_colour() => Session.Dispatch(() =>
        {
            using var s = Open();
            LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
            SystemFacts[] engines = RuntimeCatalog.Systems.Where(x => x.Engines.Count > 0).ToArray();
            Assert.NotEmpty(engines);
            Assert.Equal(engines.Length, splash.Bodies.Count);
            Assert.Equal($"{engines.Length} CONSOLES", splash.Caption);
            Assert.Equal(engines.Select(e => StartupSplash.HueOf(e.Id)), splash.Bodies.Select(b => b.Hue));
        }, default);

        // Each console drawn by its maker: the five Nintendo planets in the scouts' colours, Sega's shards, Sony's gem, and the plain planet for the rest (EmuSen_Interface.md §24.6).
        [Fact]
        public void Each_console_is_drawn_by_its_maker_and_any_other_as_the_plain_planet()
        {
            Assert.Equal([330.0, 38, 352, 212, 190], new[] { "nes", "snes", "n64", "gb", "gbc" }.Select(StartupSplash.HueOf));
            static (WorldShape, string?, double, double, double, WorldRing, int) Facts(WorldBody b) =>
                (b.Shape, b.Palette is { } p ? string.Join(" ", p) : null, b.Craters, b.Bands, b.Cap, b.Ring, b.Moons);
            Assert.Equal((WorldShape.Planet, "#fffff8fb #fff3cfe3 #ffa9789f", 0.28, 0, 0, WorldRing.None, 0), Facts(StartupSplash.BodyOf("nes", "Nintendo")));
            Assert.Equal((WorldShape.Planet, "#fffff3cf #ffffbe5c #ffb8652a", 0, 0.5, 0, WorldRing.None, 0), Facts(StartupSplash.BodyOf("snes", "Nintendo")));
            Assert.Equal((WorldShape.Planet, "#ffffd3d8 #ffee4e66 #ff7e1a38", 0, 0, 0.9, WorldRing.None, 2), Facts(StartupSplash.BodyOf("n64", "Nintendo")));
            Assert.Equal((WorldShape.Planet, "#ffdcf1ff #ff4e9ceb #ff1f3a7a", 0, 0, 0, WorldRing.None, 0), Facts(StartupSplash.BodyOf("gb", "Nintendo")));
            Assert.Equal((WorldShape.Planet, "#ffe0fcff #ff62d2e6 #ff226a88", 0, 0, 0, WorldRing.None, 0), Facts(StartupSplash.BodyOf("gbc", "Nintendo")));
            foreach (string sega in new[] { "sms", "md", "mcd", "32x", "sg1000", "saturn", "dreamcast" }) Assert.Equal(new WorldBody(WorldShape.Shards), StartupSplash.BodyOf(sega, "Sega"));
            foreach (string sony in new[] { "psx", "ps2" }) Assert.Equal(new WorldBody(WorldShape.Gem), StartupSplash.BodyOf(sony, "Sony"));
            Assert.Same(WorldBody.Plain, StartupSplash.BodyOf("virtualboy", "Nintendo"));
            Assert.Same(WorldBody.Plain, StartupSplash.BodyOf("zx81", "Sinclair"));
            Assert.Same(WorldBody.Plain, StartupSplash.BodyOf("unknown", ""));
            Assert.Same(WorldBody.Plain, StartupSplash.BodyOf(null, null));
        }

        // Moonrise draws every console of the table as the same body, palette and feature as its tile on Choose a console (EmuSen_Interface.md §24.6).
        [Fact]
        public void Moonrise_and_Choose_a_console_resolve_the_same_body_for_every_console()
        {
            var systems = RuntimeCatalog.Systems.Where(s => s.Engines.Count > 0).ToList();
            var bodies = StartupSplash.Bodies(systems);
            Assert.Equal(systems.Count, bodies.Count);
            for (int i = 0; i < systems.Count; i++)
            {
                WorldBody picker = StartupSplash.BodyOf(systems[i].Id, systems[i].Manufacturer);
                Assert.Same(picker, bodies[i].Body);
                Assert.Equal(systems[i].Manufacturer switch { "Sega" => WorldShape.Shards, "Sony" => WorldShape.Gem, _ => WorldShape.Planet }, bodies[i].Body!.Shape);
            }
            Assert.Contains(bodies, b => b.Body!.Palette is not null);
        }

        // The count follows the table: a made-up table of three systems, one without an engine, has two planets.
        [Fact]
        public void The_count_and_colours_come_from_the_table_given()
        {
            static SystemFacts System(string id, params string[] engines) =>
                new(id, id, id.ToUpperInvariant(), "", 0, "", null, null, false, engines, [], [], null, new PadFacts([], [], 1, []));
            var bodies = StartupSplash.Bodies([System("nes", "Moon"), System("psx"), System("zx81", "Spectre")]);
            Assert.Equal(["NES", "ZX81"], bodies.Select(b => b.Name));
            Assert.Equal(330, bodies[0].Hue);
            Assert.Equal(StartupSplash.HueOf("zx81"), bodies[1].Hue);
            Assert.InRange(bodies[1].Hue, 0, 359);
            Assert.Equal(["2 CONSOLES", "1 CONSOLE"], new[] { StartupSplash.Caption(2), StartupSplash.Caption(1) });
        }

        // The finished frame inside the window at each size, the size set after the first layout as a window going full screen at 4K is.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1080)]
        [InlineData(1280, 720)]
        [InlineData(3840, 2160)]
        public Task The_finished_frame_fits_after_a_late_resize(int width, int height) => Session.Dispatch(() =>
        {
            using var s = Open(work: new TaskCompletionSource().Task);
            LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
            s.Window.Width = width;
            s.Window.Height = height;
            s.Settle();
            s.Run(LengthMs + 100);
            Assert.True(splash.AtEnd);
            Assert.Equal(s.Window.ClientSize, splash.Bounds.Size);
            var box = new Rect(splash.Bounds.Size);
            Assert.All(splash.Parts().Where(p => p.Name != "halo"), p => Assert.True(box.Contains(p.Bounds), $"{p.Name} at {p.Bounds} outside {box}"));
            Assert.Contains(splash.Parts(), p => p.Name == "caption");
        }, default);

        // Loading slower than the splash: the finished frame holds, and the splash goes on the frame the work ends, not a length later.
        [Fact]
        public Task Start_up_slower_than_the_splash_is_not_delayed_by_it() => Session.Dispatch(() =>
        {
            var work = new TaskCompletionSource();
            using var s = Open(work.Task);
            s.Run(LengthMs + 3000);
            LunaSplash splash = Assert.IsType<LunaSplash>(Splash(s));
            Assert.True(splash.AtEnd);
            TimeSpan done = s.Now;
            work.SetResult();
            s.Settle();
            Assert.False(Taking(s));
            Assert.Equal(done, s.Now);
            Assert.True(s.Shown);
        }, default);
    }
}
