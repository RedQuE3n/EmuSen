using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The video settings the N64 offers a frontend, from MarsRT's schema on the generic adapter: their round trip, what is refused, and the note the core gives - see Mars_Core.md §10, EmuSen_Backend.md §18.10.
    public class MarsCoreSettingsTests
    {
        private static MarsRtTestCore Core()
        {
            Assert.True(MarsRt.Available, MarsRt.Report);
            return new MarsRtTestCore { Options = TestCores.NoBattery };
        }

        private static readonly string[] Keys = { "ThreadedRdp", "RdpWorkers", "DeferredPresentation", "SkipRepeatedScans", "RenderScale", "Antialiasing", "Gpu", "ExpansionPak", "Recompiler" };

        [Fact]
        public void The_catalogue_offers_the_schema_s_settings_with_no_game_loaded()
        {
            Assert.True(MarsRt.Available, MarsRt.Report);
            Assert.Equal(Keys, CoreCatalog.SettingsFor("N64").Select(s => s.Key));
            Assert.Equal(Keys, MarsRt.VideoSettings.Select(s => s.Key));
            using MarsRtTestCore core = Core();
            Assert.Equal(Keys, ((ICoreSettings)core).Settings.Select(s => s.Key));
        }

        // The Antialiasing row's note is the core's own, read from the running machine: silent exactly where the level chosen is the level drawn - see Mars_Performance.md §42.5.
        [Fact]
        public void The_antialiasing_note_speaks_exactly_where_the_core_draws_less_than_was_chosen()
        {
            using MarsRtTestCore core = Core();
            core.LoadRom(SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build()));
            ICoreSettings settings = core;
            foreach (string scale in new[] { "1", "2", "3", "4" })
                foreach (string level in new[] { "Off", "2x", "3x", "4x" })
                {
                    settings.Set("RenderScale", scale);
                    settings.Set("Antialiasing", level);
                    string? note = settings.Settings.Single(s => s.Key == "Antialiasing").Note?.Invoke(settings.Get);
                    Assert.Equal(core.EffectiveAntialiasing < core.Antialiasing, note is not null);
                    if (note is not null) Assert.StartsWith(core.EffectiveAntialiasing == 1 ? $"Off at {scale}x" : $"{core.EffectiveAntialiasing}x at {scale}x", note);
                }
        }

        // A game opened with a resolution that holds the antialiasing back shows the note from its first frame, not only after a setting changes - see EmuSen_Backend.md §18.10.
        [Fact]
        public void The_note_is_read_when_the_game_is_loaded()
        {
            using MarsRtTestCore core = Core();
            ICoreSettings settings = core;
            settings.Set("RenderScale", "2");
            settings.Set("Antialiasing", "4x");
            core.LoadRom(SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build()));
            Assert.Equal("2x at 2x resolution: the two together are held to four.", settings.Settings.Single(s => s.Key == "Antialiasing").Note?.Invoke(settings.Get));
        }

        [Fact]
        public void A_setting_set_is_the_setting_read_back_and_the_property_it_names()
        {
            using MarsRtTestCore core = Core();
            ICoreSettings settings = core;

            settings.Set("RdpWorkers", "3");
            settings.Set("SkipRepeatedScans", "false");
            settings.Set("DeferredPresentation", "true");
            settings.Set("ThreadedRdp", "true");

            Assert.Equal("3", settings.Get("RdpWorkers"));
            Assert.Equal(3, core.RdpWorkers);
            Assert.False(core.SkipRepeatedScans);
            Assert.Equal("false", settings.Get("SkipRepeatedScans"));
            Assert.True(core.DeferredPresentation);
            Assert.True(core.ThreadedRdp);
        }

        // Every default applied is the machine the frontend used to set by hand - see EmuSen_Settings_Reference.md §4.21b.
        [Fact]
        public void The_defaults_are_what_the_frontend_set_by_hand()
        {
            using MarsRtTestCore core = Core();
            ICoreSettings settings = core;
            foreach (CoreSetting setting in settings.Settings) settings.Set(setting.Key, setting.Default);

            Assert.True(core.ThreadedRdp);
            Assert.True(core.DeferredPresentation);
            Assert.True(core.SkipRepeatedScans);
            Assert.Equal(Math.Clamp(Environment.ProcessorCount / 3, 1, 4), core.RdpWorkers);
            Assert.Equal(1, core.RenderScale);
            Assert.True(core.ExpansionPak);
            Assert.Equal(1, core.Antialiasing);
        }

        // The drawing is the resolution times the averaging and never past four, the averaging giving way - see Mars_Video.md §2.10.
        [Theory]
        [InlineData(1, "Off", 1)]
        [InlineData(1, "4x", 4)]
        [InlineData(2, "2x", 2)]
        [InlineData(2, "4x", 2)]
        [InlineData(3, "2x", 1)]
        [InlineData(4, "3x", 1)]
        public void Antialiasing_is_held_to_four_with_the_resolution(int scale, string level, int averaged)
        {
            using MarsRtTestCore core = Core();
            ICoreSettings settings = core;
            settings.Set("RenderScale", scale.ToString());
            settings.Set("Antialiasing", level);

            Assert.Equal(level, settings.Get("Antialiasing"));
            Assert.Equal(scale, core.RenderScale);
            Assert.Equal(averaged, core.EffectiveAntialiasing);
            Assert.Throws<ArgumentException>(() => settings.Set("Antialiasing", "8x"));
        }

        [Fact]
        public void The_multiple_is_a_choice_of_one_to_four()
        {
            using MarsRtTestCore core = Core();
            ICoreSettings settings = core;
            Assert.Equal(new[] { "1", "2", "3", "4" }, settings.Settings.Single(s => s.Key == "RenderScale").Choices);

            settings.Set("RenderScale", "3");
            Assert.Equal(3, core.RenderScale);
            Assert.Equal("3", settings.Get("RenderScale"));
            Assert.Throws<ArgumentException>(() => settings.Set("RenderScale", "9"));
        }

        // The adapter's rule for every v1 core: a value outside the schema's domain is refused, a count past its range among them, and an unknown key reads as nothing - see EmuSen_Backend.md §18.2's B4.
        [Fact]
        public void A_value_outside_the_schema_is_refused_and_an_unknown_key_reads_as_nothing()
        {
            using MarsRtTestCore core = Core();
            ICoreSettings settings = core;

            Assert.Throws<ArgumentException>(() => settings.Set("RdpWorkers", "40"));
            Assert.Throws<ArgumentException>(() => settings.Set("RdpWorkers", "0"));
            Assert.Throws<ArgumentException>(() => settings.Set("RdpWorkers", "many"));
            Assert.Throws<ArgumentException>(() => settings.Set("ThreadedRdp", "yes"));
            Assert.Throws<ArgumentException>(() => settings.Set("Brightness", "1"));
            Assert.Equal("", settings.Get("Brightness"));
        }
    }
}
