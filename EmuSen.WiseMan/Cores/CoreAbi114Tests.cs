using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Core ABI 1.14's byte order of a space through the host, on the test core whose ROM states big - see EmuSen_CoreAPI.md §6.39.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreAbi114Tests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCoreAbi114_" + Guid.NewGuid().ToString("N"));

        public CoreAbi114Tests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private CoreEngine Engine(CoreLibrary library)
        {
            string path = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(path, CoreAdapterTests.Image(1, 2, 3));
            var engine = new CoreEngine(library) { Options = TestCores.NoBattery };
            engine.LoadRom(path);
            return engine;
        }

        [Fact]
        public void The_interface_is_1_14_or_later()
        {
            Assert.True(CoreInterface.Major == 1 && CoreInterface.Minor >= 14);
            Assert.True(CoreAdapterTests.Full.AbiVersion >= 0x1000Eu);
        }

        // §6.39: a stated order is read into the space's descriptor; a space that states none, and every space of the plain core, has none.
        [Fact]
        public void A_spaces_byte_order_is_read_where_the_core_states_one()
        {
            using (CoreEngine engine = Engine(CoreAdapterTests.Full))
                Assert.Equal(new[] { ("RAM", (string?)null), ("ROM", "big") }, engine.Machine.Info.Spaces.Select(s => (s.Name, s.ByteOrder)));
            using (CoreEngine plain = Engine(CoreAdapterTests.Plain))
                Assert.All(plain.Machine.Info.Spaces, s => Assert.Null(s.ByteOrder));
        }

        // §6.39: the frame log reads a space in its own order, and one that states none in the system's, little for a system the table does not hold.
        [Fact]
        public void The_frame_log_reads_a_space_in_its_own_order_before_the_systems()
        {
            using CoreEngine engine = Engine(CoreAdapterTests.Full);
            engine.WriteSpace("RAM", 0x10, 0x12);
            engine.WriteSpace("RAM", 0x11, 0x34);
            int ram = engine.FrameLog.AddEntry("RAM", 0x10, 2), rom = engine.FrameLog.AddEntry("ROM", 0x10, 2);
            engine.RunFrame();
            Assert.Equal((0x3412L, 0x1234L), (engine.FrameLog.GetSamples(ram).Single().Value, engine.FrameLog.GetSamples(rom).Single().Value));
        }
    }
}
