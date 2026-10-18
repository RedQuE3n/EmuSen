using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT's PAL NES and Dendy through the host: the console from the header or the region setting, its standard, rate and state version, and a state of another console refused - see Moon_PAL.md §3, §7.
    public class MoonRtPalTests : IDisposable
    {
        private readonly List<string> _files = new();

        public void Dispose()
        {
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        // The synthetic image with an NES 2.0 header naming the console by byte 12, or the iNES 1.0 header it is built with.
        private string Rom(int? timing)
        {
            byte[] image = SyntheticNesRom.Build();
            if (timing is { } t) (image[7], image[12]) = ((byte)((image[7] & 0xF3) | 0x08), (byte)t);
            string path = SyntheticNesRom.WriteTemp(image);
            _files.Add(path);
            return path;
        }

        private static MoonRtCore Loaded(string rom, string? region = null)
        {
            var core = new MoonRtCore() { Options = TestCores.NoBattery };
            if (region is not null) core.Set("region", region);
            core.LoadRom(rom);
            for (int i = 0; i < 3; i++) core.RunFrame();
            return core;
        }

        private static byte[] State(ICore core)
        {
            using var ms = new MemoryStream();
            core.SaveState(ms);
            return ms.ToArray();
        }

        [Theory]
        [InlineData(null, null, "ntsc", 60.0985, 6)]
        [InlineData(1, null, "pal", 50.0070, 7)]
        [InlineData(3, null, "pal", 50.0070, 7)]
        [InlineData(null, "nes.pal", "pal", 50.0070, 7)]
        [InlineData(1, "nes.ntsc", "ntsc", 60.0985, 6)]
        public void The_console_comes_from_the_header_or_the_setting_and_the_host_reads_its_standard_rate_and_version(int? timing, string? region, string standard, double hz, int version)
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            using var core = Loaded(Rom(timing), region);
            Assert.Equal(standard, core.TelevisionStandard);
            Assert.InRange(core.FrameRateHz, hz - 0.0005, hz + 0.0005);
            Assert.Equal(version, ((IStateFormat)core).StateVersion);
            Assert.Equal(version, BitConverter.ToInt32(State(core), 4));
        }

        [Fact]
        public void A_state_loads_into_its_own_console_and_is_refused_by_another_with_the_cores_words()
        {
            Assert.True(MoonRtCore.Available, MoonNative.Report);
            string rom = Rom(null);
            using var pal = Loaded(rom, "nes.pal");
            using var ntsc = Loaded(rom);
            byte[] palState = State(pal), ntscState = State(ntsc);

            using var again = Loaded(rom, "nes.pal");
            again.LoadState(new MemoryStream(palState, writable: false));
            Assert.Equal(palState, State(again));

            foreach (var (into, state) in new[] { (ntsc, palState), (pal, ntscState) })
            {
                byte[] before = State(into);
                var refused = Assert.Throws<InvalidDataException>(() => into.LoadState(new MemoryStream(state, writable: false)));
                Assert.Contains("a state saved on another console", refused.Message);
                Assert.Equal(before, State(into));
            }
        }
    }
}
