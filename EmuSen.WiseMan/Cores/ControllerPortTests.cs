using EmuSen.Crystal.Systems.N64;
using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Every core's controller ports: how many, that a second, third and fourth player's buttons reach the game's own reads, and that nothing past the last folds onto it - see EmuSen_Input.md §8.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class ControllerPortTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenControllerPorts_" + Guid.NewGuid().ToString("N"));

        public ControllerPortTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Rom(byte[] image, string name)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, image);
            return path;
        }

        // Strobes the pads, then shifts controller 2's eight bits from $4017 into $0300-$0307, forever.
        private static readonly byte[] ReadSecondNesPad =
        {
            0xA9, 0x01, 0x8D, 0x16, 0x40, // lda #1; sta $4016
            0xA9, 0x00, 0x8D, 0x16, 0x40, // lda #0; sta $4016
            0xA2, 0x00,                   // ldx #0
            0xAD, 0x17, 0x40,             // lda $4017
            0x29, 0x01,                   // and #1
            0x9D, 0x00, 0x03,             // sta $0300,x
            0xE8, 0xE0, 0x08, 0xD0, 0xF3, // inx; cpx #8; bne
            0x4C, 0x00, 0x80,             // jmp $8000
        };

        // Copies the auto-read's pad 2 low and high bytes, $421C (pad 3's with a multitap) and pad 1's low byte to $0000-$0003, forever.
        private static readonly byte[] ReadSnesPads =
        {
            0xAD, 0x1A, 0x42, 0x8D, 0x00, 0x00,
            0xAD, 0x1B, 0x42, 0x8D, 0x01, 0x00,
            0xAD, 0x1C, 0x42, 0x8D, 0x02, 0x00,
            0xAD, 0x18, 0x42, 0x8D, 0x03, 0x00,
            0x4C, 0x05, 0x80,
        };

        [Fact]
        public void Each_core_says_how_many_controllers_its_console_reads()
        {
            string nes = Rom(SyntheticNesRom.Build(patches: (0, ReadSecondNesPad)), "pads.nes");
            string snes = Rom(SyntheticRom.Build((5, ReadSnesPads)), "pads.sfc");
            string gb = Rom(SyntheticGbRom.Build(), "pads.gb");

            Assert.Equal(2, ControllerPorts.Of(CoreFactory.Load(nes, options: TestCores.NoBattery).Core));
            Assert.Equal(2, ControllerPorts.Of(CoreFactory.Load(snes, options: TestCores.NoBattery).Core));
            Assert.Equal(1, ControllerPorts.Of(CoreFactory.Load(gb, options: TestCores.NoBattery).Core));
            Assert.Equal((2, 2, 1, 4, 1), (ControllerPorts.ForConsole("NES"), ControllerPorts.ForConsole("SNES"), ControllerPorts.ForConsole("GB"), ControllerPorts.ForConsole("N64"), ControllerPorts.ForConsole("Lynx")));
            Assert.Equal(0, ControllerPorts.Of(null));

            if (MercuryRtCore.Available) Assert.Equal(1, ControllerPorts.Of(CoreFactory.Load(gb, engine: CoreCatalog.MercuryRtEngine, options: TestCores.NoBattery).Core));
            if (MarsRt.Available) Assert.Equal(4, ControllerPorts.Of(new MarsRtTestCore() { Options = TestCores.NoBattery }));
        }

        // Port 1 is controller 2; ports 2 and 3 used to land on controller 2 as well.
        [Fact]
        public void The_nes_second_pad_reads_port_1_alone()
        {
            Assert.True(MoonEngines.Available, MoonEngines.Report);
            string nes = Rom(SyntheticNesRom.Build(patches: (0, ReadSecondNesPad)), "pads.nes");
            var core = (EmuSen.Cores.Native.CoreEngine)CoreFactory.Load(nes, options: TestCores.NoBattery).Core;
            Func<int, byte> ram = a => core.ReadSpace("RAM", a);
            byte[] Bits() { core.RunFrame(); core.RunFrame(); return Enumerable.Range(0x300, 8).Select(ram).ToArray(); }

            core.SetButton(2, PadButton.A, true);
            core.SetButton(3, PadButton.Start, true);
            Assert.Equal(new byte[8], Bits());

            core.SetButton(1, PadButton.A, true);
            core.SetButton(1, PadButton.Start, true);
            core.SetButton(0, PadButton.B, true);
            Assert.Equal(new byte[] { 1, 0, 0, 1, 0, 0, 0, 0 }, Bits());
            (core as IDisposable)?.Dispose();
        }

        // Pad 2 is the auto-read's second word; ports 2 and 3, which reached it through the console's 1-based index, now reach nothing.
        [Fact]
        public void The_snes_second_pad_reads_port_1_alone()
        {
            string snes = Rom(SyntheticRom.Build((5, ReadSnesPads)), "pads.sfc");
            ICore core = CoreFactory.Load(snes, options: TestCores.NoBattery).Core;
            Func<int, byte> wram = a => ((CoreEngine)core).ReadSpace("WRAM", a);
            byte[] Pads() { for (int i = 0; i < 3; i++) core.RunFrame(); return Enumerable.Range(0, 4).Select(wram).ToArray(); }

            // Pad 3's register is not a pad without a multitap, so its byte is whatever the bus gives, before and after.
            byte[] idle = Pads();
            Assert.Equal(new byte[] { 0, 0, 0 }, idle.Where((_, i) => i != 2));
            core.SetButton(2, PadButton.B, true);
            core.SetButton(3, PadButton.A, true);
            Assert.Equal(idle, Pads());

            core.SetButton(1, PadButton.B, true);
            core.SetButton(1, PadButton.A, true);
            core.SetButton(0, PadButton.X, true);
            Assert.Equal(new byte[] { 0x80, 0x80, idle[2], 0x40 }, Pads());
            (core as IDisposable)?.Dispose();
        }

        [Fact]
        public void The_game_boys_one_port_is_all_its_engine_reads()
        {
            Assert.True(MercuryRtCore.Available, MercuryNative.Report);
            using var rt = new MercuryRtCore() { Options = TestCores.NoBattery };
            Assert.Equal(1, ((ICore)rt).ControllerPorts);
        }
    }
}
