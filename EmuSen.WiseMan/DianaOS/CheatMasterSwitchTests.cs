using System.Collections.Generic;
using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // The one switch over every cheat - see `man cheat`.
    public class CheatMasterSwitchTests
    {
        private static DianaOSInterpreter NewShell(out SnesDebugTarget target)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return DianaOSInterpreter.CreateDefault(target, null,
                new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec(),
                new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec());
        }

        // Collects what ApplyAll actually wrote, so "the poke stopped" is
        // observed rather than inferred from the flag.
        private static List<(string Space, int Address, byte Value)> Apply(CheatRegistry registry)
        {
            var written = new List<(string, int, byte)>();
            registry.ApplyAll((_, _) => 0, (space, address, value) => written.Add((space, address, value)));
            return written;
        }

        [Fact]
        public void A_registry_starts_with_its_master_switch_on()
        {
            Assert.True(new CheatRegistry().MasterEnabled);
        }

        [Fact]
        public void Master_off_stops_every_ram_poke_landing()
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives");

            Assert.Single(Apply(registry));

            registry.MasterEnabled = false;
            Assert.Empty(Apply(registry));

            registry.MasterEnabled = true;
            Assert.Single(Apply(registry));
        }

        [Fact]
        public void Master_off_stops_every_rom_patch_substituting()
        {
            var registry = new CheatRegistry();
            registry.AddRomPatch(0x008000, 0xEA, null, "nop it");

            Assert.True(registry.TryPatchRom(0x008000, 0x00, out byte patched));
            Assert.Equal(0xEA, patched);

            registry.MasterEnabled = false;
            Assert.False(registry.TryPatchRom(0x008000, 0x00, out _));

            registry.MasterEnabled = true;
            Assert.True(registry.TryPatchRom(0x008000, 0x00, out _));
        }

        // The switch is a separate axis: flipping it must not silently
        // rewrite what each cheat's own enable flag says.
        [Fact]
        public void Master_off_leaves_each_cheats_own_enabled_flag_alone()
        {
            var registry = new CheatRegistry();
            int on = registry.AddRamPoke("WRAM", 0x10, 0x01, "on one");
            int off = registry.AddRamPoke("WRAM", 0x11, 0x02, "off one");
            registry.SetEnabled(off, false);

            registry.MasterEnabled = false;
            registry.MasterEnabled = true;

            IReadOnlyList<CheatInfo> cheats = registry.GetCheats();
            Assert.True(cheats.Single(c => c.Id == on).Enabled);
            Assert.False(cheats.Single(c => c.Id == off).Enabled);
        }

        // A cheat added while the master switch is off must not start
        // applying the moment it is added.
        [Fact]
        public void A_cheat_added_while_master_is_off_stays_inert()
        {
            var registry = new CheatRegistry { MasterEnabled = false };
            registry.AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives");
            registry.AddRomPatch(0x008000, 0xEA, null, "nop it");

            Assert.Empty(Apply(registry));
            Assert.False(registry.TryPatchRom(0x008000, 0x00, out _));
        }

        [Fact]
        public void Cheat_master_reports_and_flips_the_switch()
        {
            var shell = NewShell(out SnesDebugTarget target);

            Assert.Contains("ON", shell.Execute("cheat master"));

            Assert.Contains("OFF", shell.Execute("cheat master off"));
            Assert.False(target.Cheats.MasterEnabled);

            Assert.Contains("ON", shell.Execute("cheat master on"));
            Assert.True(target.Cheats.MasterEnabled);
        }

        [Fact]
        public void Cheat_master_rejects_anything_that_is_not_on_or_off()
        {
            var shell = NewShell(out SnesDebugTarget target);

            Assert.Contains("Usage", shell.Execute("cheat master maybe"));
            Assert.True(target.Cheats.MasterEnabled);
        }

        // Or every [on] in the listing would be a lie.
        [Fact]
        public void Cheat_list_says_so_when_the_master_switch_is_off()
        {
            var shell = NewShell(out _);
            shell.Execute("cheat poke WRAM 9c 63 infinite lives");

            Assert.DoesNotContain("master switch", shell.Execute("cheat list"));

            shell.Execute("cheat master off");
            string listing = shell.Execute("cheat list");
            Assert.Contains("master switch", listing);
            Assert.Contains("infinite lives", listing);
        }

        [Fact]
        public void An_unknown_subcommand_suggests_master()
        {
            var shell = NewShell(out _);
            Assert.Contains("master", shell.Execute("cheat mastre"));
        }
    }
}
