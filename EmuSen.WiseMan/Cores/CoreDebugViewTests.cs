using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems;

namespace EmuSen.WiseMan.Cores
{
    // What DianaSh's golden corpus recorded wrong in the cores' debug views, each shown here - see EmuSen_CoreAPI.md §30.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreDebugViewTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDebugView_" + Guid.NewGuid().ToString("N"));

        public CoreDebugViewTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Image(string name, byte[] bytes)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // Built, run and attached as the corpus and a frontend do: the target is made after the frames.
        private static (ICore Core, IDebugTarget Target) Run(string path, string? engine, int frames = 2)
        {
            ICore core = CoreFactory.Create(path, headless: true, engine, options: TestCores.NoBattery);
            core.LoadRom(path);
            for (int i = 0; i < frames; i++) core.RunFrame();
            return (core, CoreFactory.Bundle(core).DebugTarget);
        }

        private static string Regs(IDebugTarget target) => DianaShInterpreter.CreateDefault(target).Submit("regs").Output;

        // MoonRT on the generic target shows the frame, the board and the registers, and names its engine.
        [Fact]
        public void MoonRT_shows_the_frame_and_registers_and_names_itself()
        {
            var (rust, rustView) = Run(Image("game.nes", SyntheticNesRom.Build()), null);
            using (rust as IDisposable)
            {
                Assert.StartsWith("MoonRT (Rust) ", rustView.GetSummaryText());
                Assert.Contains(", nes, frame 2\n", rustView.GetSummaryText());
                Assert.Contains("Mapper 0 (NROM)", rustView.GetSummaryText());
                Assert.Contains("PC", Regs(rustView));
            }
        }

        // A v1 core's registers, read when its target is made, as every C# target reads its own.
        [Fact]
        public void A_v1_target_made_after_two_frames_shows_their_registers()
        {
            var (core, target) = Run(Image("game.md", SyntheticMdRom.Cartridge()), null);
            using (core as IDisposable)
            {
                var first = target.CpuRegisters.Current.Select(r => (r.Name, r.Value)).ToArray();
                Assert.Contains(first, r => r.Name == "PC");
                target.RefreshProviders();
                Assert.Equal(target.CpuRegisters.Current.Select(r => (r.Name, r.Value)), first);
                Assert.Contains("PC", Regs(target));
            }
        }

        // `cpu` is the first processor machine info lists, whatever the core calls it - see `man cpus`.
        [Fact]
        public void On_Nephrite_the_word_cpu_names_the_68000()
        {
            var (core, target) = Run(Image("game.md", SyntheticMdRom.Cartridge()), null);
            using (core as IDisposable)
            {
                var shell = DianaShInterpreter.CreateDefault(target);
                string regs = shell.Submit("regs cpu").Output;
                Assert.StartsWith("M68K registers:", regs);
                Assert.Contains("SSP", regs);
                Assert.Equal(regs, shell.Submit("regs m68k").Output);

                var disasm = shell.Submit("disasm cpu 200 2");
                Assert.Equal("0", shell.LastExitStatus);
                Assert.Equal(shell.Submit("disasm M68KBUS 200 2").Output, disasm.Output);

                Assert.StartsWith("Breakpoint #", shell.Submit("bp m68k add 302").Output);
                Assert.StartsWith("Z80 breakpoint #", shell.Submit("bp cop add 10").Output);
            }
        }

        // Endou on the generic target: its summary, the Z80's registers, the VDP's and the PSG's units, a tile and the sprites, through the shell - see Endou_Native.md §15.
        [Fact]
        public void Endou_shows_its_board_registers_units_and_views()
        {
            // A Japanese cartridge starts on the Japanese Master System, so its open boot program has handed over by frame 60 - see Endou_Native.md §16.4.
            var (core, target) = Run(Image("game.sms", SyntheticSmsRom.Cartridge(region: 0x3)), null, frames: 60);
            using (core as IDisposable)
            {
                string summary = target.GetSummaryText();
                Assert.StartsWith("Endou ", summary);
                Assert.Contains(", sms, frame 60\nBoard Plain, 32 KiB ROM\nZ80  AF=", summary);
                Assert.Contains("\nIO   $3E=AB", summary);
                var shell = DianaShInterpreter.CreateDefault(target);
                string regs = shell.Submit("regs cpu").Output;
                Assert.StartsWith("Z80 registers:", regs);
                Assert.Contains("IFF2", regs);
                string all = Regs(target);
                Assert.Contains("R1", all);
                Assert.Contains("Volume3", all);
                Assert.Contains("Board:Plain", all);
                shell.Submit("tile VRAM 0 4");
                Assert.Equal("0", shell.LastExitStatus);
                Assert.Equal(64, target.MaxSprites);
                PhysicalAddress place = target.ResolvePhysical(0xE123)!.Value;
                Assert.Equal(("RAM", 0x123, true), (place.Space, place.Offset, place.IsAddressable));
            }
        }

        [Fact]
        public void Tile_on_a_core_that_decodes_none_says_so()
        {
            var (core, target) = Run(Image("game.md", SyntheticMdRom.Cartridge()), null);
            using (core as IDisposable)
            {
                var shell = DianaShInterpreter.CreateDefault(target);
                string output = shell.Submit("tile VRAM 0 2").Output;
                Assert.Equal("1", shell.LastExitStatus);
                Assert.Contains("decodes no tiles", output);
                Assert.DoesNotContain("Index was outside", output);
            }
        }

        // The Genesis is in `core`'s registry as every console is: by its own name and its engine's.
        [Fact]
        public void Core_takes_the_Genesis_by_its_name_and_its_engines()
        {
            Assert.Contains("Genesis", CoreCatalog.DiscoveredConsoles);
            CoreDescriptor genesis = CoreCatalog.Cores.Single(c => c.Console == "Genesis");
            Assert.Same(genesis, CoreCatalog.Registry["genesis"]);
            Assert.Same(genesis, CoreCatalog.Registry["nephrite"]);

            Directory.CreateDirectory(DianaShSandbox.ScratchDirectory);
            string rom = Path.Combine(DianaShSandbox.ScratchDirectory, $"wiseman_genesis_{Guid.NewGuid():N}.md");
            File.WriteAllBytes(rom, SyntheticMdRom.Cartridge());
            try
            {
                DianaShResult result = new CoreCommand(EmuSen.WiseMan.DianaSh.CoreChoices.Of(CoreCatalog.Registry)).Execute(null, new[] { "core", "genesis", rom }, null);
                Assert.True(result.Action is HostAction.LoadCore, result.Output);
            }
            finally
            {
                File.Delete(rom);
            }
        }
    }
}
