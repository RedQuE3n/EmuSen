using System;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Core ABI 1.15's routed system beside an image, through the host on the test core - see EmuSen_CoreAPI.md §6.40.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreAbi115Tests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreAbi115_" + Guid.NewGuid().ToString("N"));

        public CoreAbi115Tests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [Fact]
        public void The_interface_is_1_15_or_later()
        {
            Assert.True(CoreInterface.Major == 1 && CoreInterface.Minor >= 15);
            Assert.True(CoreAdapterTests.Full.AbiVersion >= 0x1000Fu);
        }

        // §6.40: a listed system beside an image is made, and one core info does not list is refused with BAD_IMAGE and words before the core sees it.
        [Fact]
        public void A_system_named_beside_an_image_is_made_or_refused_when_unlisted()
        {
            byte[] image = TestCoreImage.Of(1, 2, 3);
            using (var named = new CoreMachine(CoreAdapterTests.Full, image, "", Array.Empty<(uint, byte[])>(), system: "test"))
                Assert.Equal("test", named.Info.System);
            var refused = Assert.Throws<CoreRefusedException>(() => new CoreMachine(CoreAdapterTests.Plain, image, "", Array.Empty<(uint, byte[])>(), system: "nes"));
            Assert.Equal(-264, refused.Status);
            Assert.Contains("the image was routed to nes, which is not a system of this core", refused.Message);
        }

        // §6.40: the engine names the system it routed a file to, the test core's own by its extension, and the game runs as before.
        [Fact]
        public void The_engine_names_the_routed_system_and_the_game_runs_as_before()
        {
            string path = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(path, TestCoreImage.Of(1, 2, 3));
            using var engine = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            engine.LoadRom(path);
            engine.RunFrame();
            Assert.Equal(("test", 1L), (engine.Machine.Info.System, engine.TotalFrames));
        }
    }
}
