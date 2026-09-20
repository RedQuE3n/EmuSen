using System;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;

namespace EmuSen.WiseMan.Cores
{
    // The four video settings Mars offers a frontend: their round trip, the clamp on a count, and what is refused - see Mars_Core.md §10.
    public class MarsCoreSettingsTests
    {
        [Fact]
        public void The_catalogue_offers_the_same_four_settings_the_core_answers()
        {
            ICoreSettings core = new MarsCore();
            Assert.Equal(new[] { "ThreadedRdp", "RdpWorkers", "DeferredPresentation", "SkipRepeatedScans" }, core.Settings.Select(s => s.Key));
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
