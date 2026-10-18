using System;
using System.IO;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // A state's compatibility on the API: machine info's loads_from and the core's own refusal, never a frontend's comparison of versions - see EmuSen_Backend.md §14.12 and Moon_PAL.md §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class StateRefusalTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenStateRefusal_" + Guid.NewGuid().ToString("N"));

        public StateRefusalTests()
        {
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Rom(string name, int? timing)
        {
            byte[] image = SyntheticNesRom.Build();
            if (timing is { } t) (image[7], image[12]) = ((byte)((image[7] & 0xF3) | 0x08), (byte)t);
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        private static void Frames(CrystalSession s, int count)
        {
            var info = new CrystalFrameInfo();
            for (int i = 0; i < count; i++) s.Frame(ref info);
        }

        // A PAL NES state (version 7, inside the NTSC machine's loads_from) offered to an NTSC session is refused by MoonRT itself, its words carried, the machine unchanged.
        [Fact]
        public void A_pal_state_in_an_ntsc_session_is_refused_in_the_core_s_words()
        {
            Assert.True(Fixtures.MoonEngines.Available, Fixtures.MoonEngines.Report);
            using var runtime = new CrystalRuntime();
            string state = Path.Combine(_root, "Pal.state");
            using (CrystalSession pal = runtime.Open(new CrystalOpen { Path = Rom("Pal.nes", 1), NoBattery = true }))
            {
                Frames(pal, 3);
                Assert.Equal(7, JsonDocument.Parse(pal.Info()).RootElement.GetProperty("state").GetProperty("version").GetInt32());
                pal.SaveState(state);
            }
            Assert.Equal(7, JsonDocument.Parse(runtime.StateRecord(state)!).RootElement.GetProperty("state_version").GetInt32());
            using CrystalSession ntsc = runtime.Open(new CrystalOpen { Path = Rom("Ntsc.nes", null), NoBattery = true });
            Frames(ntsc, 3);
            JsonElement info = JsonDocument.Parse(ntsc.Info()).RootElement.GetProperty("state");
            Assert.Equal(6, info.GetProperty("version").GetInt32());
            Assert.Contains(info.GetProperty("loads_from").EnumerateArray(), v => v.GetInt32() == 7);
            var refused = Assert.Throws<CrystalException>(() => ntsc.LoadState(state));
            Assert.Equal(CrystalAbi.StateRefused, refused.Status);
            Assert.Contains("a state saved on another console", refused.Message);
            Assert.DoesNotContain("newer build", refused.Message);
        }

        // A reset a core does not claim is a status, NOT_SUPPORTED with its words, never an exception across the API; the session runs on.
        [Fact]
        public void A_reset_the_core_does_not_claim_is_not_supported()
        {
            Assert.True(EmuSen.Cores.Nintendo.MercuryRT.MercuryRtCore.Available, EmuSen.Cores.Nintendo.MercuryRT.MercuryNative.Report);
            using var runtime = new CrystalRuntime();
            string rom = Path.Combine(_root, "Mono.gb");
            File.WriteAllBytes(rom, SyntheticGbRom.Build());
            using CrystalSession s = runtime.Open(new CrystalOpen { Path = rom, NoBattery = true });
            Frames(s, 2);
            var refused = Assert.Throws<CrystalException>(() => s.Reset());
            Assert.Equal(CrystalSessionAbi.NotSupported, refused.Status);
            Assert.Contains("reset", refused.Message);
            Frames(s, 2);
        }
    }
}
