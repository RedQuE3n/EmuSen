using EmuSen.Cores;
using EmuSen.Crystal.Debugger;
using System.Linq;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaSh
{
    // `bp when` and `dma`, the fifth Mesen pass - see `man bp`, `man dma`.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConditionBreakAndDmaTests
    {
        private static (IDebugTarget Target, ICore Core) Build(params (int Offset, byte[] Bytes)[] patches)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.Build(patches));
            return (SyntheticRom.TargetOf(core), core);
        }

        private static IDebugTarget BuildTarget() => Build().Target;

        private static DianaShInterpreter Shell(IDebugTarget target) => DianaShInterpreter.CreateDefault(target);

        // --- bp when ---

        [Fact]
        public void An_unarmed_condition_costs_the_core_nothing()
        {
            var target = BuildTarget();

            // The gate every detection site tests before building a string.
            Assert.False(target.Breakpoints.AnyConditionArmed);
            target.Breakpoints.NoteCondition("stp", 0x808000, "ignored");

            Assert.Equal(0, target.Breakpoints.LogEntriesRecorded);
            Assert.False(target.Breakpoints.ShouldBreak(0x808001));
        }

        // --- dma ---

        [Fact]
        public void Dma_prints_a_row_per_channel()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("dma").Output;

            Assert.Equal(8, target.DmaChannels.Count);
            Assert.Contains("hdma table", output);
            foreach (int channel in Enumerable.Range(0, 8)) Assert.Contains($"  {channel}   ", output);
        }

        [Fact]
        public void The_channel_table_names_the_destination_register()
        {
            var (target, core) = Build((5, new byte[] { 0xA9, 0x18, 0x8D, 0x01, 0x43, 0x80, 0xFE })); // channel 0 destination = VMDATAL
            core.RunFrame();
            target.RefreshProviders();

            string output = Shell(target).Submit("dma").Output;

            Assert.Contains("VMDATAL $2118", output);
        }

        [Fact]
        public void An_unknown_dma_subcommand_names_the_real_ones()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("dma wibble").Output;

            Assert.Contains("log", output);
            Assert.Contains("stats", output);
        }
    }
}
