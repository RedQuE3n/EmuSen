using System.IO;
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
    // `cheat import`/`cheat export` driven through the real shell against a
    // real core, with the real SNES codecs wired up - see `man cheat`.
    public class CheatChtCommandTests
    {
        private static DianaOSInterpreter NewShell(out SnesDebugTarget target)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return DianaOSInterpreter.CreateDefault(target, null,
                new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec(),
                new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec());
        }

        // Written inside the sandbox, since that is all `cheat import` will resolve.
        private static string WriteChtInSandbox(string fileName, string contents)
        {
            DianaOSInterpreter.CreateDefault(null); // ensures the skeleton exists
            string path = Path.Combine(DianaOSSandbox.RootDirectory, "tmp", fileName);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, contents);
            return path;
        }

        [Fact]
        public void A_libretro_database_file_imports_disabled_and_can_then_be_enabled()
        {
            string path = WriteChtInSandbox("smw.cht", """
                cheats = 2

                cheat0_desc = "Infinite Maximum Coins"
                cheat0_code = "7E0DBF63"
                cheat0_enable = false

                cheat1_desc = "Lives and coins together"
                cheat1_code = "7E0DBF63+7E001900"
                cheat1_enable = false
                """);

            DianaOSInterpreter shell = NewShell(out SnesDebugTarget target);

            string imported = shell.Submit($"cheat import {path}").Output;
            Assert.Contains("Imported 2 cheat(s)", imported);
            Assert.Contains("all disabled", imported);

            var cheats = target.Cheats.GetCheats();
            Assert.Equal(2, cheats.Count);
            Assert.All(cheats, c => Assert.False(c.Enabled));

            // The '+' entry is ONE cheat with two writes, not two cheats.
            CheatInfo joined = cheats.Single(c => c.Description == "Lives and coins together");
            Assert.Equal(2, joined.Writes.Count);

            // And the list shows both writes under the one ID.
            string listed = shell.Submit("cheat list").Output;
            Assert.Contains("2 writes", listed);

            Assert.Contains("enabled", shell.Submit($"cheat enable {joined.Id}").Output);
            Assert.True(target.Cheats.GetCheats().Single(c => c.Id == joined.Id).Enabled);
        }

        [Fact]
        public void An_imported_cheat_actually_pokes_memory_once_enabled()
        {
            // CpuBus $7E0019 is WRAM - what a real Action Replay would target.
            string path = WriteChtInSandbox("poke.cht", """
                cheats = 1
                cheat0_desc = "99 lives"
                cheat0_code = "7E001963"
                cheat0_enable = false
                """);

            DianaOSInterpreter shell = NewShell(out SnesDebugTarget target);
            shell.Submit($"cheat import {path}");

            var space = target.GetMemorySpaces().Single(s => s.Name == "CpuBus");

            // Disabled on import, so a frame changes nothing.
            target.OnFrame(1);
            Assert.NotEqual(0x63, space.Read(0x7E0019));

            shell.Submit($"cheat enable {target.Cheats.GetCheats().Single().Id}");
            target.OnFrame(2);
            Assert.Equal(0x63, space.Read(0x7E0019));
        }

        [Fact]
        public void Export_then_import_returns_the_same_cheats()
        {
            DianaOSInterpreter shell = NewShell(out SnesDebugTarget target);
            shell.Submit("cheat poke CpuBus 7E0019 63 99 lives");
            shell.Submit("cheat poke CpuBus 7E0DBF 04 coins");

            string path = Path.Combine(DianaOSSandbox.RootDirectory, "tmp", "exported.cht");
            string exported = shell.Submit($"cheat export {path}").Output;
            Assert.Contains("Exported 2 cheat(s)", exported);
            Assert.True(File.Exists(path));

            shell.Submit("cheat clear");
            Assert.Empty(target.Cheats.GetCheats());

            shell.Submit($"cheat import {path}");

            var reloaded = target.Cheats.GetCheats();
            Assert.Equal(2, reloaded.Count);
            Assert.Contains(reloaded, c => c.Description == "99 lives" && c.Writes[0].Address == 0x7E0019);
            Assert.Contains(reloaded, c => c.Description == "coins" && c.Writes[0].Value == 0x04);
        }

        // RetroArch's model has no ROM-read substitution to write them into.
        [Fact]
        public void Export_says_so_when_it_has_to_skip_rom_patches()
        {
            DianaOSInterpreter shell = NewShell(out _);
            shell.Submit("cheat poke CpuBus 7E0019 63 a poke");
            shell.Submit("cheat rompatch 00C05F EA - a patch");

            string path = Path.Combine(DianaOSSandbox.RootDirectory, "tmp", "mixed.cht");
            string output = shell.Submit($"cheat export {path}").Output;

            Assert.Contains("Exported 1 cheat(s)", output);
            Assert.Contains("1 ROM patch(es) skipped", output);
        }

        [Fact]
        public void Importing_a_missing_file_says_so_rather_than_throwing()
        {
            DianaOSInterpreter shell = NewShell(out _);

            string output = shell.Submit("cheat import /tmp/definitely-not-here.cht").Output;

            Assert.Contains("no file at", output);
        }

        [Fact]
        public void A_path_outside_the_sandbox_is_refused()
        {
            DianaOSInterpreter shell = NewShell(out _);

            string output = shell.Submit("cheat import /../../etc/passwd.cht").Output;

            Assert.Contains("outside the sandbox", output);
        }
    }
}
