using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.DianaOS.DianaOS.Var;
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
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public ControllerPortTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
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

            Assert.Equal(2, ControllerPorts.Of(CoreFactory.Load(nes).Core));
            Assert.Equal(2, ControllerPorts.Of(CoreFactory.Load(snes).Core));
            Assert.Equal(1, ControllerPorts.Of(CoreFactory.Load(gb).Core));
            Assert.Equal(4, ControllerPorts.Of(new MarsCore()));
            Assert.Equal((2, 2, 1, 4, 1), (ControllerPorts.ForConsole("NES"), ControllerPorts.ForConsole("SNES"), ControllerPorts.ForConsole("GB"), ControllerPorts.ForConsole("N64"), ControllerPorts.ForConsole("Lynx")));
            Assert.Equal(0, ControllerPorts.Of(null));

            if (MoonRtCore.Available) Assert.Equal(2, ControllerPorts.Of(CoreFactory.Load(nes, engine: CoreCatalog.MoonRtEngine).Core));
            if (MercuryRtCore.Available) Assert.Equal(1, ControllerPorts.Of(CoreFactory.Load(gb, engine: CoreCatalog.MercuryRtEngine).Core));
            if (MarsRtCore.Available) Assert.Equal(4, ControllerPorts.Of(new MarsRtCore()));
            if (CoreDiscovery.Found.SingleOrDefault(c => c.Info.Id == "venusrt") is { } venusrt)
            {
                using var engine = Assert.IsType<CoreEngine>(CoreFactory.Load(snes, engine: venusrt.EngineName).Core);
                Assert.Equal(2, ControllerPorts.Of(engine));
            }
        }

        // Port 1 is controller 2; ports 2 and 3 used to land on controller 2 as well, in both engines.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void The_nes_second_pad_reads_port_1_alone(bool rust)
        {
            if (rust && !MoonRtCore.Available) return;
            string nes = Rom(SyntheticNesRom.Build(patches: (0, ReadSecondNesPad)), "pads.nes");
            ICore core = CoreFactory.Load(nes, engine: rust ? CoreCatalog.MoonRtEngine : null).Core;
            Func<int, byte> ram = a => core is MoonCore moon ? moon.ReadSpace(MoonCore.SpaceRam, a) : ((MoonRtCore)core).ReadSpace(MoonCore.SpaceRam, a);
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
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void The_snes_second_pad_reads_port_1_alone(bool rust)
        {
            string snes = Rom(SyntheticRom.Build((5, ReadSnesPads)), "pads.sfc");
            DiscoveredCore? venusrt = CoreDiscovery.Found.SingleOrDefault(c => c.Info.Id == "venusrt");
            if (rust && venusrt is null) return;
            ICore core = CoreFactory.Load(snes, engine: rust ? venusrt!.EngineName : null).Core;
            Func<int, byte> wram = a => core is VenusCore venus ? venus.Bus!.Ram[a] : ((CoreEngine)core).ReadSpace("WRAM", a);
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

        // The joybus answers a port only while a controller is in it; ports 2-4 start empty, so a one-player game's machine is as before.
        [Fact]
        public void The_n64s_other_ports_answer_once_a_controller_is_plugged_in()
        {
            var core = new MarsCore();
            core.LoadRom(Rom(SyntheticN64Rom.Build(patches: (0, new byte[] { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 })), "pads.z64"));
            var ports = core.Bus!.Si.Controllers;
            Assert.Equal(new[] { true, false, false, false }, ports.Select(p => p.Present));

            for (int port = 1; port < 4; port++)
            {
                core.SetControllerConnected(port, true);
                core.SetButton(port, PadButton.A, true);
                core.SetAxis(port, PadAxis.LeftX, 1);
            }
            core.SetButton(7, PadButton.B, true);
            core.SetControllerConnected(9, true);
            Assert.All(ports, p => Assert.True(p.Present));
            Assert.Equal(new ushort[] { 0, 0x8000, 0x8000, 0x8000 }, ports.Select(p => p.Buttons));
            Assert.Equal(new sbyte[] { 0, 127, 127, 127 }, ports.Select(p => p.StickX));

            core.SetControllerConnected(3, false);
            Assert.False(ports[3].Present);
        }

        // MarsRT's ports, by its state against the C# core's after the same plugging and presses on every port.
        [Fact]
        public void Marsrts_ports_take_a_controller_and_its_buttons_as_the_csharp_cores_do()
        {
            if (!MarsRtCore.Available) return;
            string rom = Rom(SyntheticN64Rom.Build(patches: (0, new byte[] { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 })), "pads.z64");
            MarsCore oracle = MarsRtTests.Oracle();
            using MarsRtCore twin = MarsRtTests.Twin();
            oracle.LoadRom(rom);
            twin.LoadRom(rom);
            byte[] before = twin.Save(false);
            foreach (ICore core in new ICore[] { oracle, twin })
            {
                core.SetControllerConnected(2, true);
                core.SetButton(2, PadButton.Start, true);
                core.SetAxis(2, PadAxis.LeftY, -1);
                core.SetControllerConnected(3, true);
                core.SetButton(3, PadButton.B, true);
            }

            using var stream = new MemoryStream();
            oracle.SaveState(stream);
            byte[] after = twin.Save(false);
            Assert.False(before.AsSpan().SequenceEqual(after), "MarsRT's state did not change");
            Assert.True(stream.ToArray().AsSpan().SequenceEqual(after), "MarsRT's ports differ from the C# core's");
        }

        [Fact]
        public void The_game_boys_one_port_is_all_either_engine_reads()
        {
            Assert.Equal(1, ((ICore)new MercuryCore()).ControllerPorts);
            if (MercuryRtCore.Available)
            {
                using var rt = new MercuryRtCore();
                Assert.Equal(1, ((ICore)rt).ControllerPorts);
            }
        }
    }
}
