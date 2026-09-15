using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;
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

        public CoreFactoryCheatWiringTests()
        {
            _gb = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: (0, new byte[] { Original })));
            _nes = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: (0, new byte[] { Original })));
            _snes = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.sfc");
            File.WriteAllBytes(_snes, SyntheticRom.BuildBlank());
        }

        public void Dispose()
        {
            foreach (string path in new[] { _gb, _nes, _snes })
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
