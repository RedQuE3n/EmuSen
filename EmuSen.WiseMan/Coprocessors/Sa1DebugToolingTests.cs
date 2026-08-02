using System.Collections.Generic;
using System.Linq;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Coprocessors
{
    // The debug toolchain's coverage of the SA-1 specifically: the memory
    // spaces the chip's own address space needs, watch coverage of writes
    // neither MemoryBus nor the PPU ever sees, breakpoints on the second
    // CPU, and the clock counters that say whether it is running at rate.
    // See Venus_SA1.md §11.
    public class Sa1DebugToolingTests
    {
        private static (VenusCore Core, SnesDebugTarget Target) Sa1Target()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildSa1());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return (core, target);
        }

        // CCNT powers on at $20 (RESB held), so a fresh chip executes nothing
        // until the S-CPU drops reset - see Venus_SA1.md §4.1. Anything
        // asserting on execution has to do this first.
        private static void ReleaseReset(EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1.Sa1 sa1)
            => sa1.WriteRegister(0x2200, 0x00);

        // --- Memory spaces (§11.2, §11.3) ---

        [Fact]
        public void An_sa1_cartridge_exposes_bw_ram_as_its_own_space()
        {
            var (core, target) = Sa1Target();
            var bwRam = target.GetMemorySpaces().Single(s => s.Name == "BWRAM");

            Assert.Equal(core.Cart!.Sa1!.BwRamSize, bwRam.Size);
            Assert.False(bwRam.HasSideEffects);
        }

        // The regression this space exists for: SRAM is anchored at bank $70,
        // which an SA-1 cart does not map, so it reads as zeroes no matter
        // what BW-RAM actually holds - see Venus_SA1.md §11.2.
        [Fact]
        public void Bw_ram_contents_are_invisible_through_the_sram_space()
        {
            var (core, target) = Sa1Target();
            core.Cart!.Sa1!.DebugBwRam[0x20] = 0xA5;

            var sram = target.GetMemorySpaces().Single(s => s.Name == "SRAM");
            var bwRam = target.GetMemorySpaces().Single(s => s.Name == "BWRAM");

            Assert.Equal(0x00, sram.Read(0x20));
            Assert.Equal(0xA5, bwRam.Read(0x20));
        }

        [Fact]
        public void The_sa1_bus_space_reads_through_the_chips_own_map()
        {
            var (core, target) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;
            sa1.IRam[0x40] = 0x7C;

            var sa1Bus = target.GetMemorySpaces().Single(s => s.Name == "SA1BUS");

            // I-RAM is mirrored low on the SA-1's side and at $3000 on both -
            // the S-CPU sees WRAM at $0040, the SA-1 sees I-RAM. See §3.
            Assert.Equal(0x7C, sa1Bus.Read(0x000040));
            Assert.Equal(0x7C, sa1Bus.Read(0x003040));
        }

        [Fact]
        public void The_sa1_bus_space_reports_itself_side_effect_free()
        {
            var (_, target) = Sa1Target();
            Assert.False(target.GetMemorySpaces().Single(s => s.Name == "SA1BUS").HasSideEffects);
        }

        // $2302 latches the timer counters and $230D advances the bitstream
        // cursor; a debugger reading either must not do that - see §11.1.
        [Fact]
        public void Peeking_the_timer_latch_register_does_not_relatch()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;

            sa1.WriteRegister(0x2210, 0x00); // HV mode
            byte firstPeek = sa1.DebugPeekRegister(0x2302);
            sa1.Run(4000);                   // counters advance
            byte secondPeek = sa1.DebugPeekRegister(0x2302);

            // A real read would have re-latched and shown the newer count.
            Assert.Equal(firstPeek, secondPeek);
        }

        [Fact]
        public void Peeking_the_bitstream_high_byte_does_not_advance_the_cursor()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;

            byte first = sa1.DebugPeekRegister(0x230D);
            byte second = sa1.DebugPeekRegister(0x230D);

            Assert.Equal(first, second);
        }

        [Fact]
        public void A_plain_cartridge_exposes_no_sa1_spaces()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var names = target.GetMemorySpaces().Select(s => s.Name).ToList();

            Assert.DoesNotContain("BWRAM", names);
            Assert.DoesNotContain("SA1BUS", names);
        }

        // --- Write observation (§11.4) ---

        private sealed class RecordingObserver : IWriteObserver
        {
            public readonly List<(string Space, int Address, byte Value, bool FromCoprocessor)> Events = new();
            public void OnWrite(string space, int address, byte value) => Events.Add((space, address, value, false));
            public void OnCoprocessorWrite(string space, int address, byte value) => Events.Add((space, address, value, true));
        }

        [Fact]
        public void Writes_the_sa1_makes_to_its_own_i_ram_are_observed()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;
            var observer = new RecordingObserver();
            sa1.WriteObserver = observer;

            sa1.WriteSa1(0x003040, 0x99);

            var evt = Assert.Single(observer.Events);
            Assert.Equal("SA1IRAM", evt.Space);
            Assert.Equal(0x40, evt.Address);
            Assert.Equal(0x99, evt.Value);
            Assert.True(evt.FromCoprocessor);
        }

        [Fact]
        public void Writes_the_sa1_makes_to_bw_ram_are_observed()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;
            var observer = new RecordingObserver();
            sa1.WriteObserver = observer;

            sa1.WriteSa1(0x400010, 0x5A); // banks $40-$4F are BW-RAM on both sides

            var evt = Assert.Single(observer.Events);
            Assert.Equal("BWRAM", evt.Space);
            Assert.Equal(0x10, evt.Address);
            Assert.True(evt.FromCoprocessor);
        }

        // The S-CPU reaches the same two memories through Cartridge.Write8,
        // which MemoryBus routes to directly - so it needs its own hook.
        [Fact]
        public void Writes_the_s_cpu_makes_to_sa1_memory_are_observed_as_not_from_the_chip()
        {
            var (core, _) = Sa1Target();
            var observer = new RecordingObserver();
            core.Cart!.WriteObserver = observer;

            core.Bus!.Write8(0x003040, 0x11); // I-RAM window, S-CPU side
            core.Bus!.Write8(0x400010, 0x22); // BW-RAM

            Assert.Equal(2, observer.Events.Count);
            Assert.Equal("SA1IRAM", observer.Events[0].Space);
            Assert.False(observer.Events[0].FromCoprocessor);
            Assert.Equal("BWRAM", observer.Events[1].Space);
            Assert.False(observer.Events[1].FromCoprocessor);
        }

        [Fact]
        public void A_plain_cartridge_reports_its_save_ram_as_sram_not_bwram()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.Build());
            var observer = new RecordingObserver();
            core.Cart!.WriteObserver = observer;

            core.Bus!.Write8(0x700000, 0x33);

            var evt = Assert.Single(observer.Events);
            Assert.Equal("SRAM", evt.Space);
            Assert.False(evt.FromCoprocessor);
        }

        // --- Breakpoints on the second CPU (§11.5) ---

        [Fact]
        public void An_sa1_cartridge_gets_its_own_breakpoint_registry()
        {
            var (_, target) = Sa1Target();
            Assert.NotNull(target.CoprocessorBreakpoints);
        }

        [Fact]
        public void A_plain_cartridge_has_no_coprocessor_breakpoint_registry()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            Assert.Null(target.CoprocessorBreakpoints);
        }

        [Fact]
        public void A_breakpoint_on_the_sa1_halts_run_and_keeps_its_unspent_clocks()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;
            ReleaseReset(sa1);

            // Break on whatever the chip is about to execute, so the very
            // first checked instruction trips it.
            int pc24 = (sa1.Cpu.PB << 16) | sa1.Cpu.PC;
            sa1.BreakpointChecker = pc => pc == pc24;

            sa1.Run(1000);

            Assert.True(sa1.HaltedAtBreakpoint);
            Assert.Equal(pc24, sa1.HaltedAddress);
            // Nothing executed, so the chip has not moved.
            Assert.Equal(0, sa1.ExecutedMasterClocks);
            Assert.Equal(1000, sa1.OfferedMasterClocks);
        }

        [Fact]
        public void Resuming_past_an_sa1_breakpoint_executes_the_instruction_it_sat_on()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;
            ReleaseReset(sa1);

            int pc24 = (sa1.Cpu.PB << 16) | sa1.Cpu.PC;
            sa1.BreakpointChecker = pc => pc == pc24;
            sa1.Run(1000);
            Assert.True(sa1.HaltedAtBreakpoint);

            sa1.ResumeFromBreakpoint();
            sa1.Run(0); // no new clocks - the carried budget alone must drive it

            Assert.False(sa1.HaltedAtBreakpoint);
            Assert.True(sa1.ExecutedMasterClocks > 0);
        }

        // --- Clock accounting (§2.3) ---

        [Fact]
        public void A_running_sa1_consumes_every_clock_it_is_offered()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;
            ReleaseReset(sa1);

            for (int i = 0; i < 50; i++) sa1.Run(1000);

            Assert.Equal(50_000, sa1.OfferedMasterClocks);

            // Run() executes any instruction the budget can still partly
            // afford and carries the overshoot as a debt into the next call,
            // so the two totals track each other to within one instruction in
            // EITHER direction - never drifting apart - see Venus_SA1.md §2.2.
            Assert.InRange(sa1.ExecutedMasterClocks - sa1.OfferedMasterClocks, -24, 24);
        }

        [Fact]
        public void A_halted_sa1_is_offered_clocks_but_runs_none()
        {
            var (core, _) = Sa1Target();
            var sa1 = core.Cart!.Sa1!;

            sa1.WriteRegister(0x2200, 0x60); // RESB | RDYB - see §4.1
            sa1.Run(5000);

            Assert.Equal(5000, sa1.OfferedMasterClocks);
            Assert.Equal(0, sa1.ExecutedMasterClocks);
        }

        [Fact]
        public void The_master_clocks_a_frame_is_worth_matches_the_frame_rate()
        {
            var (core, _) = Sa1Target();
            Assert.Equal(262 * VenusCore.CyclesPerScanline, core.MasterClocksPerFrame);
        }
    }
}
