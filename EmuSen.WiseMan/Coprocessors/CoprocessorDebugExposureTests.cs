using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Coprocessors
{
    // The whole DianaOS toolchain used to be blind to cartridge
    // coprocessors - see IDebugTarget.CoprocessorRegisters. These pin that
    // the chips are visible, that a plain cartridge stays quiet, and above
    // all that reading them is side-effect-free, which is the property that
    // makes a debugger safe to point at a running chip.
    public class CoprocessorDebugExposureTests
    {
        private static SnesDebugTarget TargetFor(byte[] rom)
        {
            var core = SyntheticRom.LoadCore(rom);
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void A_plain_cartridge_reports_no_coprocessor_registers()
        {
            var target = TargetFor(SyntheticRom.BuildBlank());
            Assert.Empty(target.CoprocessorRegisters.Current);
        }

        [Fact]
        public void A_plain_cartridge_exposes_no_coprocessor_memory_space()
        {
            var target = TargetFor(SyntheticRom.BuildBlank());
            var names = target.GetMemorySpaces().Select(s => s.Name).ToList();

            Assert.DoesNotContain("GSURAM", names);
            Assert.DoesNotContain("SA1IRAM", names);
        }

        [Fact]
        public void A_superfx_cartridge_reports_the_gsu_register_file()
        {
            var target = TargetFor(SyntheticRom.BuildSuperFx());
            var names = target.CoprocessorRegisters.Current.Select(r => r.Name).ToList();

            Assert.Contains("SFR", names);
            Assert.Contains("PBR", names);
            Assert.Contains("SCBR", names);
            // R15 is the program counter, so the file has to be there in full.
            Assert.Contains("R0", names);
            Assert.Contains("R15", names);
        }

        [Fact]
        public void A_superfx_cartridge_exposes_game_pak_ram()
        {
            var target = TargetFor(SyntheticRom.BuildSuperFx());
            var gsuRam = target.GetMemorySpaces().Single(s => s.Name == "GSURAM");

            Assert.True(gsuRam.Size > 0);
            Assert.False(gsuRam.HasSideEffects);
        }

        [Fact]
        public void An_sa1_cartridge_reports_its_control_registers_and_iram()
        {
            var target = TargetFor(SyntheticRom.BuildSa1());
            var names = target.CoprocessorRegisters.Current.Select(r => r.Name).ToList();

            Assert.Contains("CCNT", names);
            Assert.Contains("Halted", names);
            Assert.Contains("SA1IRAM", target.GetMemorySpaces().Select(s => s.Name));
        }

        // The reason CoprocessorRegisters reads dedicated Debug* views
        // instead of the chip's own ReadRegister: $3031 acknowledges the
        // GSU's interrupt, so a debugger routed through it would clear a
        // flag simply by looking. Reading the provider must never do that.
        [Fact]
        public void Reading_the_provider_does_not_acknowledge_the_gsu_interrupt()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildSuperFx());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var gsu = core.Bus!.Cart.SuperFx!;

            // Run a bare STOP, which sets the IRQ flag - see Venus_SuperFX.md §3.3.
            gsu.WriteRegister(0x3039, 0x01);
            gsu.WriteRegister(0x301E, 0x00);
            gsu.WriteRegister(0x301F, 0x00);
            gsu.Run(2000);
            Assert.True(gsu.ScpuIrqPending);

            for (int i = 0; i < 5; i++) target.RefreshProviders();

            Assert.True(gsu.ScpuIrqPending);
        }

        [Fact]
        public void The_coprocessor_history_retains_successive_refreshes()
        {
            var target = TargetFor(SyntheticRom.BuildSuperFx());
            for (int i = 0; i < 3; i++) target.RefreshProviders();

            var history = target.CoprocessorHistory.GetHistory();
            Assert.Equal(4, history.Count); // the initial snapshot plus three
            Assert.All(history, snapshot => Assert.NotEmpty(snapshot));
        }

        // A stopped chip publishes an identical snapshot every frame, which
        // is exactly the "when did it stop" signal - see HistoryProvider.
        [Fact]
        public void An_idle_coprocessor_reports_refreshes_since_change()
        {
            var target = TargetFor(SyntheticRom.BuildSuperFx());
            for (int i = 0; i < 4; i++) target.RefreshProviders();

            Assert.True(target.CoprocessorHistory.RefreshesSinceChange > 0);
        }
    }
}
