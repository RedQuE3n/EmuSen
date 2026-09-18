using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The registry a frontend hands the factory has to be the one the core applies - see EmuSen_Cheats.md §6.
    public class CoreFactoryCheatWiringTests : IDisposable
    {
        private const byte Original = 0xCE;
        private readonly string _gb;
        private readonly string _nes;
        private readonly string _snes;
        private readonly string _n64;

        // Status.IE set with nothing unmasked, then a spin, so the frame boundary applies cheats as it does in a game - see Mars_Cheats.md §5.1.
        private static readonly byte[] RunningSpin =
        {
            0x3C, 0x08, 0x34, 0x00, 0x35, 0x08, 0x00, 0x01, 0x40, 0x88, 0x60, 0x00,
            0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        };
        private const int N64RomOffset = 0x1040;

        public CoreFactoryCheatWiringTests()
        {
            _gb = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: (0, new byte[] { Original })));
            _nes = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: (0, new byte[] { Original })));
            _snes = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.sfc");
            File.WriteAllBytes(_snes, SyntheticRom.BuildBlank());
            _n64 = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(
                length: 0x2000, patches: new[] { (0, RunningSpin), (N64RomOffset - 0x40, new byte[] { Original }) }));
        }

        public void Dispose()
        {
            foreach (string path in new[] { _gb, _nes, _snes, _n64 })
            {
                try { File.Delete(path); } catch { }
            }
        }

        [Fact]
        public void A_game_boy_core_patches_rom_through_the_registry_it_was_handed()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_gb, headless: true, cheats);
            var core = (MercuryCore)bundle.Core;

            Assert.Equal(Original, core.ReadSpace(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint));

            cheats.AddRomPatch(SyntheticGbRom.EntryPoint, 0xAD, null, "patch");

            Assert.Equal(0xAD, core.ReadSpace(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint));
        }

        [Fact]
        public void A_game_boy_core_pokes_ram_through_the_registry_it_was_handed()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_gb, headless: true, cheats);
            var core = (MercuryCore)bundle.Core;

            cheats.AddRamPoke(MercuryCore.SpaceCpuBus, 0xC000, 0x42, "poke");
            bundle.Core.RunFrame();

            Assert.Equal(0x42, core.ReadSpace(MercuryCore.SpaceCpuBus, 0xC000));
        }

        [Fact]
        public void An_nes_core_patches_rom_through_the_registry_it_was_handed()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_nes, headless: true, cheats);
            var core = (MoonCore)bundle.Core;

            Assert.Equal(Original, core.Bus!.Read(0x8000));

            cheats.AddRomPatch(0x8000, 0xAD, null, "patch");

            Assert.Equal(0xAD, core.Bus!.Read(0x8000));
        }

        // Mistress's Apply button, pressed while the emulation thread is parked - see EmuSen_Cheats.md §6.
        [Fact]
        public void A_game_boy_debug_target_pokes_without_waiting_for_a_frame()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_gb, headless: true, cheats);
            var core = (MercuryCore)bundle.Core;

            cheats.AddRamPoke(MercuryCore.SpaceCpuBus, 0xC000, 0x42, "poke");
            bundle.DebugTarget.ApplyCheats();

            Assert.Equal(0x42, core.ReadSpace(MercuryCore.SpaceCpuBus, 0xC000));
        }

        [Fact]
        public void An_nes_debug_target_pokes_without_waiting_for_a_frame()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_nes, headless: true, cheats);
            var core = (MoonCore)bundle.Core;

            cheats.AddRamPoke(MoonCore.SpaceRam, 0x0000, 0x42, "poke");
            bundle.DebugTarget.ApplyCheats();

            Assert.Equal(0x42, core.ReadSpace(MoonCore.SpaceRam, 0x0000));
        }

        // The path Mistress actually takes: the window's registry, through the session - see EmuSen_Cheats.md §6.
        [Fact]
        public void A_session_hands_its_registry_to_the_core_it_loads()
        {
            var cheats = new CheatRegistry();
            var session = new EmulatorSession { Cheats = cheats };
            session.LoadRom(_gb);

            cheats.AddRamPoke(MercuryCore.SpaceCpuBus, 0xC000, 0x42, "poke");
            session.RunFrame();

            Assert.Equal(0x42, ((MercuryCore)session.Core!).ReadSpace(MercuryCore.SpaceCpuBus, 0xC000));
        }

        // Mars reads its cartridge through the PI, so a patch is asked for by ROM offset - see Mars_Cheats.md §6.
        [Fact]
        public void A_mars_core_patches_rom_through_the_registry_it_was_handed()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_n64, headless: true, cheats);
            var core = (MarsCore)bundle.Core;
            uint physical = MemoryMap.CartDomain1Address2 + N64RomOffset;

            Assert.Equal(Original, core.Bus!.Read8(physical));

            cheats.AddRomPatch(N64RomOffset, 0xAD, null, "patch");

            Assert.Equal(0xAD, core.Bus.Read8(physical));
        }

        [Fact]
        public void A_mars_core_pokes_ram_through_the_registry_it_was_handed()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_n64, headless: true, cheats);
            var core = (MarsCore)bundle.Core;

            cheats.AddRamPoke(MarsCore.SpaceRdram, 0x100, 0x42, "poke");
            bundle.Core.RunFrame();

            Assert.Equal(0x42, core.Bus!.Rdram[0x100]);
        }

        [Fact]
        public void A_mars_debug_target_pokes_without_waiting_for_a_frame()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_n64, headless: true, cheats);
            var core = (MarsCore)bundle.Core;

            cheats.AddRamPoke(MarsCore.SpaceRdram, 0x100, 0x42, "poke");
            bundle.DebugTarget.ApplyCheats();

            Assert.Equal(0x42, core.Bus!.Rdram[0x100]);
            Assert.Equal(0, core.TotalFrames);
        }

        // The bundle's own codec, decoding a two-line code the way Mistress's N64 tab does - see Mars_Cheats.md §5.
        [Fact]
        public void A_mars_bundle_decodes_and_applies_a_gameshark_code()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_n64, headless: true, cheats);
            var core = (MarsCore)bundle.Core;
            core.Bus!.Rdram[0x200] = 0x05;

            cheats.AddCheat(CheatKind.RamPoke, bundle.CheatAutoDetectCodec!.DecodeWrites("D0000200 0005+81000100 1234")!, null, "guarded");
            bundle.DebugTarget.ApplyCheats();

            Assert.Equal(new byte[] { 0x12, 0x34 }, core.Bus.Rdram[0x100..0x102]);
        }

        [Fact]
        public void A_session_hands_its_registry_to_a_mars_core()
        {
            var cheats = new CheatRegistry();
            var session = new EmulatorSession { Cheats = cheats };
            session.LoadRom(_n64);

            cheats.AddRamPoke(MarsCore.SpaceRdram, 0x100, 0x42, "poke");
            session.RunFrame();

            Assert.Equal(0x42, ((MarsCore)session.Core!).Bus!.Rdram[0x100]);
        }

        // The core the seam was written for, and the one that already routes it - see EmuSen_Cheats.md §4.
        [Fact]
        public void The_snes_core_already_pokes_through_the_registry_it_was_handed()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(_snes, headless: true, cheats);

            cheats.AddRamPoke("WRAM", 0x0000, 0x42, "poke");
            bundle.Core.RunFrame();

            var space = Assert.Single(bundle.DebugTarget.GetMemorySpaces(),
                s => string.Equals(s.Name, "WRAM", StringComparison.OrdinalIgnoreCase));

            Assert.Equal(0x42, space.Read(0x0000));
        }
    }
}
