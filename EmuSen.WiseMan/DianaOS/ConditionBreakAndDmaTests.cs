using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // `bp when` and `dma`, the fifth Mesen pass - see `man bp`, `man dma`.
    public class ConditionBreakAndDmaTests
    {
        private static (SnesDebugTarget Target, EmuSen.Cores.Nintendo.Venus.VenusCore Core) Build(params (int Offset, byte[] Bytes)[] patches)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.Build(patches));
            return (new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!), core);
        }

        private static SnesDebugTarget BuildTarget() => Build().Target;

        private static DianaOSInterpreter Shell(SnesDebugTarget target) => DianaOSInterpreter.CreateDefault(target);

        // --- bp when ---

        [Fact]
        public void Bp_when_lists_the_conditions_this_core_detects()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp when").Output;

            Assert.Contains("stp", output);
            Assert.Contains("ppuaccess", output);
            Assert.Contains("autojoy", output);
            Assert.Contains("[off]", output);
        }

        [Fact]
        public void Bp_when_refuses_a_condition_the_core_does_not_detect()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("bp when wibble").Output;

            Assert.Contains("not a condition this core detects", output);
            Assert.Empty(target.Breakpoints.GetConditions());
        }

        [Fact]
        public void Arming_a_condition_reports_it_and_shows_up_in_the_listing()
        {
            var target = BuildTarget();
            var shell = Shell(target);

            Assert.Contains("Halting on stp", shell.Submit("bp when stp").Output);
            Assert.Contains("when stp: halting", shell.Submit("bp list").Output);
            Assert.True(target.Breakpoints.IsConditionArmed("stp"));
            Assert.True(target.Breakpoints.AnyConditionArmed);
        }

        [Fact]
        public void A_condition_can_be_disarmed_again()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp when stp");

            Assert.Contains("No longer watching", shell.Submit("bp when stp off").Output);
            Assert.False(target.Breakpoints.AnyConditionArmed);
        }

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

        [Fact]
        public void Executing_stp_halts_when_the_condition_is_armed()
        {
            // $DB is STP, at $008000 where reset lands after the boot stub.
            var (target, core) = Build((5, new byte[] { 0xDB }));
            Shell(target).Submit("bp when stp");

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Contains("STP executed", target.Breakpoints.LastBreakReason);
        }

        [Fact]
        public void Executing_wdm_halts_when_the_condition_is_armed()
        {
            // $42 is WDM, which takes one operand byte.
            var (target, core) = Build((5, new byte[] { 0x42, 0x00 }));
            Shell(target).Submit("bp when wdm");

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Contains("WDM executed", target.Breakpoints.LastBreakReason);
        }

        [Fact]
        public void A_log_only_condition_records_instead_of_halting()
        {
            var (target, core) = Build((5, new byte[] { 0x42, 0x00 }));
            var shell = Shell(target);
            Assert.Contains("without halting", shell.Submit("bp when wdm log").Output);

            core.RunFrame();

            Assert.False(core.IsHaltedAtBreakpoint);
            Assert.True(target.Breakpoints.LogEntriesRecorded > 0);
            Assert.Contains("WDM executed", shell.Submit("bp log").Output);
        }

        [Fact]
        public void A_logged_condition_prints_as_when_rather_than_a_breakpoint_id()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp when stp log");
            target.Breakpoints.NoteCondition("stp", 0x808000, "STP executed, at $808000");

            string logged = shell.Submit("bp log").Output;

            Assert.Contains("when", logged);
            Assert.DoesNotContain("#0", logged);
        }

        [Fact]
        public void Bp_log_knows_a_log_only_condition_counts_as_a_logpoint()
        {
            var target = BuildTarget();
            var shell = Shell(target);
            shell.Submit("bp when stp log");

            Assert.Contains("Nothing logged yet", shell.Submit("bp log").Output);
        }

        [Fact]
        public void A_ppu_data_port_written_while_rendering_is_reported()
        {
            var (target, core) = Build();
            Shell(target).Submit("bp when ppuaccess log");

            // Not vblank and not forced blank: exactly when hardware drops the write.
            core.Bus!.Interrupts.InVBlank = false;
            core.Bus!.Ppu.Inidisp = 0x0F;
            core.Bus!.CurrentScanline = 100;
            core.Bus!.Write8(0x002118, 0x42);

            Assert.Equal(1, target.Breakpoints.LogEntriesRecorded);
        }

        [Fact]
        public void The_same_write_during_forced_blank_is_not_reported()
        {
            var (target, core) = Build();
            Shell(target).Submit("bp when ppuaccess log");

            core.Bus!.Interrupts.InVBlank = false;
            core.Bus!.Ppu.Inidisp = 0x80; // forced blank, so the write really does land
            core.Bus!.CurrentScanline = 100;
            core.Bus!.Write8(0x002118, 0x42);

            Assert.Equal(0, target.Breakpoints.LogEntriesRecorded);
        }

        [Fact]
        public void An_address_port_write_while_rendering_is_not_reported()
        {
            var (target, core) = Build();
            Shell(target).Submit("bp when ppuaccess log");

            core.Bus!.Interrupts.InVBlank = false;
            core.Bus!.Ppu.Inidisp = 0x0F;
            core.Bus!.CurrentScanline = 100;
            core.Bus!.Write8(0x002116, 0x00); // VMADDL latches fine mid-frame

            Assert.Equal(0, target.Breakpoints.LogEntriesRecorded);
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
            var (target, core) = Build();
            core.Bus!.Write8(0x004301, 0x18); // channel 0 destination = VMDATAL

            string output = Shell(target).Submit("dma").Output;

            Assert.Contains("VMDATAL $2118", output);
        }

        [Fact]
        public void Dma_log_is_off_until_it_is_armed()
        {
            var target = BuildTarget();
            var shell = Shell(target);

            Assert.Contains("logging is off", shell.Submit("dma log").Output);
            Assert.Contains("logging on", shell.Submit("dma log on").Output);
            Assert.Contains("Nothing logged yet", shell.Submit("dma log").Output);
        }

        [Fact]
        public void A_general_transfer_is_recorded_with_its_destination_and_length()
        {
            var (target, core) = Build();
            var shell = Shell(target);
            shell.Submit("dma log on");

            RunOneVramTransfer(core, length: 0x20);

            string logged = shell.Submit("dma log").Output;
            Assert.Contains("VMDATAL $2118", logged);
            Assert.Contains("gen", logged);
            Assert.Equal(1, target.DmaLog!.TotalTransfers);
            Assert.Equal(0x20, target.DmaLog.TotalBytes);
        }

        [Fact]
        public void Stats_attributes_the_bytes_to_the_destination_register()
        {
            var (target, core) = Build();
            var shell = Shell(target);
            shell.Submit("dma log on");

            RunOneVramTransfer(core, length: 0x40);

            string stats = shell.Submit("dma stats").Output;
            Assert.Contains("VMDATAL $2118", stats);
            Assert.Contains("64", stats);
            var destination = Assert.Single(target.DmaLog!.DestinationStats());
            Assert.Equal(0x18, destination.DestinationRegister);
            Assert.Equal(0x40, destination.Bytes);
        }

        [Fact]
        public void Clearing_the_log_forgets_the_totals_but_leaves_it_armed()
        {
            var (target, core) = Build();
            var shell = Shell(target);
            shell.Submit("dma log on");
            RunOneVramTransfer(core, length: 0x10);

            Assert.Contains("cleared", shell.Submit("dma log clear").Output);

            Assert.Equal(0, target.DmaLog!.TotalTransfers);
            Assert.True(target.DmaLog.IsArmed);
            Assert.Contains("Nothing logged yet", shell.Submit("dma log").Output);
        }

        [Fact]
        public void A_disarmed_log_records_nothing()
        {
            var (target, core) = Build();

            RunOneVramTransfer(core, length: 0x10);

            Assert.Equal(0, target.DmaLog!.TotalTransfers);
        }

        [Fact]
        public void An_unknown_dma_subcommand_names_the_real_ones()
        {
            var target = BuildTarget();

            string output = Shell(target).Submit("dma wibble").Output;

            Assert.Contains("log", output);
            Assert.Contains("stats", output);
        }

        // Channel 0, WRAM -> VMDATAL, then the $420B kick that runs it.
        private static void RunOneVramTransfer(EmuSen.Cores.Nintendo.Venus.VenusCore core, int length)
        {
            var bus = core.Bus!;
            bus.Write8(0x004300, 0x01);                  // mode 1, A->B
            bus.Write8(0x004301, 0x18);                  // destination VMDATAL
            bus.Write8(0x004302, 0x00);
            bus.Write8(0x004303, 0x00);
            bus.Write8(0x004304, 0x7E);                  // source $7E0000
            bus.Write8(0x004305, (byte)(length & 0xFF));
            bus.Write8(0x004306, (byte)(length >> 8));
            bus.Write8(0x00420B, 0x01);
        }
    }
}
