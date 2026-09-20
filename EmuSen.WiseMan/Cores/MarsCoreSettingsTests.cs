using System;
using System.IO;
using System.Linq;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;

namespace EmuSen.WiseMan.Cores
{
    // The four video settings Mars offers a frontend: their round trip, the clamp on a count, and what is refused - see Mars_Core.md §10.
    public class MarsCoreSettingsTests
    {
        [Fact]
        public void The_catalogue_offers_the_same_seven_settings_the_core_answers()
        {
            ICoreSettings core = new MarsCore();
            Assert.Equal(new[] { "ThreadedRdp", "RdpWorkers", "DeferredPresentation", "SkipRepeatedScans", "RenderScale", "Antialiasing", "ExpansionPak" }, core.Settings.Select(s => s.Key));
            Assert.Same(core.Settings, CoreCatalog.SettingsFor("N64"));
            Assert.Empty(CoreCatalog.SettingsFor("SNES"));
        }

        [Fact]
        public void A_setting_set_is_the_setting_read_back_and_the_property_it_names()
        {
            var core = new MarsCore();
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
            var core = new MarsCore();
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
        [InlineData(1, "Off", 1, 1)]
        [InlineData(1, "4x", 4, 4)]
        [InlineData(2, "2x", 4, 2)]
        [InlineData(2, "4x", 4, 2)]
        [InlineData(3, "2x", 3, 1)]
        [InlineData(4, "3x", 4, 1)]
        public void Antialiasing_multiplies_the_drawing_and_is_held_to_four_with_the_resolution(int scale, string level, int drawn, int averaged)
        {
            var core = new MarsCore(batteryRamDisabled: true);
            core.LoadRom(WriteRom());
            ICoreSettings settings = core;
            settings.Set("RenderScale", scale.ToString());
            settings.Set("Antialiasing", level);

            Assert.Equal(level, settings.Get("Antialiasing"));
            Assert.Equal(scale, core.RenderScale);
            Assert.Equal(drawn, core.Bus!.Dp.Scale);
            Assert.Equal(averaged, core.Bus.Vi.Average);
            Assert.Throws<ArgumentException>(() => settings.Set("Antialiasing", "8x"));
        }

        // A square of the multiple's raster becomes one pixel, its rounded mean in every channel - see Mars_Video.md §2.10.
        [Fact]
        public void The_average_of_a_square_is_its_rounded_mean_channel_by_channel()
        {
            byte[] source =
            {
                0, 10, 255, 255,   1, 20, 255, 255,   100, 0, 0, 255,   100, 0, 0, 255,
                1, 30, 255, 255,   1, 40, 255, 255,   100, 0, 0, 255,   104, 0, 0, 255,
            };
            var into = new byte[8];
            EmuSen.Cores.Nintendo.Mars.Vi.Vi.BoxAverage(source, 4, 2, into);
            Assert.Equal(new byte[] { 1, 25, 255, 255, 101, 0, 0, 255 }, into);
        }

        private static string WriteRom()
        {
            string path = Path.Combine(Path.GetTempPath(), "mars_settings_" + Guid.NewGuid().ToString("N") + ".z64");
            File.WriteAllBytes(path, SyntheticN64Rom.Build());
            return path;
        }

        [Fact]
        public void The_multiple_is_a_choice_of_one_to_four_and_reaches_the_display_processor()
        {
            var core = new MarsCore();
            ICoreSettings settings = core;
            Assert.Equal(new[] { "1", "2", "3", "4" }, settings.Settings.Single(s => s.Key == "RenderScale").Choices);

            settings.Set("RenderScale", "3");
            Assert.Equal(3, core.RenderScale);
            Assert.Equal("3", settings.Get("RenderScale"));
            settings.Set("RenderScale", "9");
            Assert.Equal(4, core.RenderScale);
        }

        [Fact]
        public void A_count_outside_its_range_is_clamped_and_text_that_is_neither_is_refused()
        {
            var core = new MarsCore();
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
