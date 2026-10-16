using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MarsRT;

namespace EmuSen.WiseMan.Cores
{
    // The video settings the N64 offers a frontend: their round trip, the clamp on a count, and what is refused - see Mars_Core.md §10.
    public class MarsCoreSettingsTests
    {
        private static MarsRtCore Core()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            return new MarsRtCore();
        }

        [Fact]
        public void The_catalogue_offers_the_console_s_eight_settings_and_the_core_answers_them_and_its_own()
        {
            var keys = new[] { "ThreadedRdp", "RdpWorkers", "DeferredPresentation", "SkipRepeatedScans", "RenderScale", "Antialiasing", "Gpu", "ExpansionPak" };
            Assert.Equal(keys, CoreCatalog.SettingsFor("N64").Select(s => s.Key));
            Assert.Same(N64Console.VideoSettings, CoreCatalog.SettingsFor("N64"));
            Assert.Equal(keys.Append("Recompiler"), MarsRtCore.VideoSettings.Select(s => s.Key));
            Assert.Empty(CoreCatalog.SettingsFor("SNES"));
        }

        // The Antialiasing row's note is the core's own rule: silent exactly where the level chosen is the level drawn - see Mars_Performance.md §42.5.
        [Fact]
        public void The_antialiasing_note_speaks_exactly_where_the_core_draws_less_than_was_chosen()
        {
            using MarsRtCore core = Core();
            foreach (string scale in new[] { "1", "2", "3", "4" })
                foreach (string level in new[] { "Off", "2x", "3x", "4x" })
                {
                    ((ICoreSettings)core).Set("RenderScale", scale);
                    ((ICoreSettings)core).Set("Antialiasing", level);
                    string? note = N64Console.AntialiasingNote(key => key == "RenderScale" ? scale : level);
                    Assert.Equal(core.EffectiveAntialiasing < core.Antialiasing, note is not null);
                    if (note is not null) Assert.StartsWith(core.EffectiveAntialiasing == 1 ? $"Off at {scale}x" : $"{core.EffectiveAntialiasing}x at {scale}x", note);
                }
            Assert.NotNull(MarsRtCore.VideoSettings.Single(s => s.Key == "Antialiasing").Note);
        }

        [Fact]
        public void A_setting_set_is_the_setting_read_back_and_the_property_it_names()
        {
            using MarsRtCore core = Core();
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
            using MarsRtCore core = Core();
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
            using MarsRtCore core = Core();
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
            using MarsRtCore core = Core();
            ICoreSettings settings = core;
            Assert.Equal(new[] { "1", "2", "3", "4" }, settings.Settings.Single(s => s.Key == "RenderScale").Choices);

            settings.Set("RenderScale", "3");
            Assert.Equal(3, core.RenderScale);
            Assert.Equal("3", settings.Get("RenderScale"));
            Assert.Throws<ArgumentException>(() => settings.Set("RenderScale", "9"));
        }

        [Fact]
        public void A_count_outside_its_range_is_clamped_and_text_that_is_neither_is_refused()
        {
            using MarsRtCore core = Core();
            ICoreSettings settings = core;

            settings.Set("RdpWorkers", "40");
            Assert.Equal(8, core.RdpWorkers);
            settings.Set("RdpWorkers", "0");
            Assert.Equal(1, core.RdpWorkers);

            Assert.Throws<ArgumentException>(() => settings.Set("RdpWorkers", "many"));
            Assert.Throws<ArgumentException>(() => settings.Set("ThreadedRdp", "yes"));
            Assert.Throws<ArgumentException>(() => settings.Set("Brightness", "1"));
            Assert.Throws<ArgumentException>(() => settings.Get("Brightness"));
        }
    }
}
