using System;
using System.IO;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // A game an engine refuses is a fault in the engine's words on the API, never "no engine" - see EmuSen_Backend.md §17.5 (C1).
    [Collection(TestCollections.ProcessGlobals)]
    public class GameRefusalTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenGameRefusal_" + Guid.NewGuid().ToString("N"));

        public GameRefusalTests()
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

        // MBC7 ($22) is a board MercuryRT does not build; the Game Boy has an engine, so the answer is the engine's refusal.
        [Fact]
        public void A_cartridge_type_mercuryrt_does_not_build_is_a_fault_naming_the_type()
        {
            Assert.True(MercuryEngines.Available, MercuryEngines.Report);
            string rom = Path.Combine(_root, "Mbc7.gb");
            File.WriteAllBytes(rom, SyntheticGbRom.Build(cartridgeType: 0x22));
            using var runtime = new CrystalRuntime();
            var refused = Assert.Throws<CrystalException>(() => runtime.Open(new CrystalOpen { Path = rom, NoBattery = true }));
            Assert.Equal(CrystalSessionAbi.Fault, refused.Status);
            Assert.Contains("MercuryRT", refused.Message);
            Assert.Contains("$22", refused.Message);
        }
    }
}
