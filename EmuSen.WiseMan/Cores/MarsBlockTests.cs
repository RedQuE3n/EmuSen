using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Cores
{
    // Compiled blocks against the interpreter, whole state for whole state - see Mars_Recompiler.md §6.
    public class MarsBlockTests
    {
        private const int Zero = 0, T0 = 8, T1 = 9, T2 = 10, T3 = 11, T4 = 12, T5 = 13, T6 = 14, T7 = 15, A0 = 4, A1 = 5, K0 = 26, K1 = 27, Ra = 31;

        [Fact]
        public void A_hot_loop_is_compiled_and_leaves_the_machine_the_interpreter_leaves()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 5000)
                .Addiu(T0, T0, 1)
                .Bne(T0, T1, -2)
                .Addu(T2, T2, T0)
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 20_000);

            Assert.True(compiled.BlocksCompiled >= 1);
            Assert.Equal(5000UL, compiled.Gpr[T0]);
            Assert.Equal(5000UL * 5001 / 2, compiled.Gpr[T2]);
        }

        [Fact]
        public void A_block_is_compiled_only_once_it_has_run_as_often_as_the_threshold_asks()
        {
            foreach ((int turns, bool expected) in new[] { (5, false), (63, false), (64, true) })
            {
                MipsAssembler program = new MipsAssembler()
                    .Addiu(T1, Zero, (short)turns)
                    .Beq(Zero, Zero, 1)
                    .Nop()
                    .Addiu(T0, T0, 1)
                    .Bne(T0, T1, -2)
                    .Nop();
                for (int i = 0; i < 400; i++) program.Nop();

                Cpu cpu = program.Build();
                cpu.CompileInBackground = false;
                cpu.RunBlocks(3 * turns + 300);

                Assert.Equal(expected, cpu.BlocksCompiled >= 1);
            }
        }

        // The VI's and AI's events fall between a block's instructions and end it on the same one - see Mars_Recompiler.md §3.2.
        [Fact]
        public void An_event_inside_a_block_fires_on_the_same_instruction()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0), ProgramDevices);

            AssertSame(interpreted, compiled, 3_500_000);

            Assert.True(compiled.Bus.Vi.Fields >= 2);
            Assert.True(compiled.Bus.Ai.SamplesPlayed > 0);
        }

        [Fact]
        public void The_timer_interrupts_a_block_on_the_instruction_the_interpreter_takes_it_on()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0), (bus, cpu) =>
                {
                    Handler(bus, a => a
                        .Mfc0(K0, Cpu.CompareRegister).Addiu(K0, K0, 3000).Mtc0(K0, Cpu.CompareRegister)
                        .Lui(K1, 0x8000).Lw(T3, K1, 0x800).Addiu(T3, T3, 1).Sw(T3, K1, 0x800)
                        .Eret());
                    cpu.Cop0[Cpu.CompareRegister] = 3000;
                    cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | (1UL << 15);
                    cpu.Cop0Written();
                });

            AssertSame(interpreted, compiled, 100_000);

            Assert.True(compiled.Bus.Read32(0x800) > 5);
        }

        [Fact]
        public void A_fault_inside_a_block_reports_the_instruction_that_raised_it()
        {
            var (interpreted, compiled) = Pair(a => a
                .Lui(A0, 0x8000).Ori(A0, A0, 0x0801).Addiu(T1, Zero, 300)
                .Addiu(T0, T0, 1)
                .Addu(T2, T2, T0)
                .Lw(T3, A0, 0)
                .Addiu(T2, T2, 5)
                .Bne(T0, T1, -5)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop(), (bus, _) => Handler(bus, a => a
                    .Mfc0(K0, Cpu.ExceptionPcRegister).Addiu(K0, K0, 4).Mtc0(K0, Cpu.ExceptionPcRegister)
                    .Lui(K1, 0x8000).Lw(T4, K1, 0x800).Addiu(T4, T4, 1).Sw(T4, K1, 0x800)
                    .Eret()));

            AssertSame(interpreted, compiled, 8_000);

            Assert.Equal(300u, compiled.Bus.Read32(0x800));
            Assert.True(compiled.BlocksCompiled >= 1);
        }

        // The block's own store rewrites a word it has already compiled, and the next pass sees the new one - see Mars_Recompiler.md §4.
        [Fact]
        public void A_store_into_the_running_block_is_seen_by_the_next_pass()
        {
            var (interpreted, compiled) = Pair(a => a
                .Lui(A0, 0x8000).Addiu(T1, Zero, 2000).Lui(T4, 0x240A).Ori(T4, T4, 7).Addiu(T5, Zero, 6)
                .Addiu(T0, T0, 1)
                .Addiu(T2, Zero, 1)
                .Addu(T3, T3, T2)
                .Xor(T4, T4, T5)
                .Sw(T4, A0, 24)
                .Bne(T0, T1, -6)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 20_000);

            Assert.True(compiled.BlocksDiscarded >= 1);
            Assert.True(compiled.Gpr[T3] > 2000);
        }

        [Fact]
        public void Code_replaced_under_a_compiled_block_is_compiled_again_from_the_new_words()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0));

            AssertSame(interpreted, compiled, 300);
            Assert.Equal(1, compiled.BlocksCompiled);

            foreach (Cpu cpu in new[] { interpreted, compiled }) cpu.Bus.Write32(0, 0x2508_0002);

            AssertSame(interpreted, compiled, 300);
            Assert.Equal(1, compiled.BlocksDiscarded);
            Assert.Equal(2, compiled.BlocksCompiled);
        }

        [Fact]
        public void A_likely_branch_not_taken_skips_its_slot_inside_a_block()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 10)
                .Addiu(T0, T0, 1)
                .Bnel(T0, T1, -2)
                .Addiu(T2, T2, 100)
                .Addiu(T3, Zero, 1)
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 300);

            Assert.Equal(900UL, compiled.Gpr[T2]);
            Assert.Equal(1UL, compiled.Gpr[T3]);
            Assert.True(compiled.BlocksCompiled >= 1);
        }

        [Fact]
        public void Calls_and_returns_inside_blocks_reach_the_observers()
        {
            int callsA = 0, returnsA = 0, callsB = 0, returnsB = 0;

            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 200)
                .Jal(0x20)
                .Nop()
                .Addiu(T0, T0, 1)
                .Bne(T0, T1, -4)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop()
                .Addiu(T2, T2, 1)
                .Jr(Ra)
                .Nop());

            interpreted.CallObserver = (_, _) => callsA++;
            interpreted.ReturnObserver = () => returnsA++;
            compiled.CallObserver = (_, _) => callsB++;
            compiled.ReturnObserver = () => returnsB++;

            AssertSame(interpreted, compiled, 3_000);

            Assert.Equal(200, callsA);
            Assert.Equal(200, returnsA);
            Assert.Equal(callsA, callsB);
            Assert.Equal(returnsA, returnsB);
        }

        [Fact]
        public void A_branch_in_a_delay_slot_is_left_to_the_interpreter()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, 1)
                .Beq(Zero, Zero, -3)
                .Addiu(T1, T1, 1)
                .Addiu(T2, T2, 1));

            AssertSame(interpreted, compiled, 3_000);
        }

        [Fact]
        public void Multiply_and_divide_stalls_are_charged_inside_a_block()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 7).Addiu(T2, Zero, 3)
                .Mult(T1, T2)
                .Mflo(T3)
                .Div(T1, T2)
                .Mfhi(T4)
                .Dmult(T1, T2)
                .Ddivu(T1, T2)
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -8)
                .Addu(T5, T5, T3));

            AssertSame(interpreted, compiled, 2_000);

            Assert.True(compiled.Bus.Cycles > 25_000);
        }

        // A store to a device may change what is due when; the block ends on it so the next one is scheduled afresh - see Mars_Recompiler.md §4.
        [Fact]
        public void A_store_to_a_device_ends_the_block_and_the_new_schedule_is_kept()
        {
            var (interpreted, compiled) = Pair(a => a
                .Lui(A0, 0xA440).Addiu(T1, Zero, 525).Addiu(T2, Zero, 625)
                .Addiu(T0, T0, 1)
                .Sw(T1, A0, 0x18)
                .Addiu(T3, T3, 1)
                .Sw(T2, A0, 0x18)
                .Beq(Zero, Zero, -5)
                .Nop(), ProgramDevices);

            AssertSame(interpreted, compiled, 2_000_000);

            Assert.True(compiled.Bus.Vi.Fields >= 1);
        }

        [Fact]
        public void The_signal_processor_breaking_beside_a_block_interrupts_it_on_the_same_instruction()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0), (bus, cpu) =>
                {
                    for (int i = 0; i < 200; i++) bus.Write32(MemoryMap.SpImemBase + (uint)i * 4, 0);
                    bus.Write32(MemoryMap.SpImemBase + 200 * 4, 0x0000_000D);

                    Handler(bus, a => a
                        .Lui(K0, 0xA404).Ori(K0, K0, 0x0010).Addiu(K1, Zero, 0x10D).Sw(K1, K0, 0)
                        .Lui(K1, 0x8000).Lw(T3, K1, 0x800).Addiu(T3, T3, 1).Sw(T3, K1, 0x800)
                        .Eret());

                    bus.Mi.Mask = MiInterrupt.SignalProcessor;
                    cpu.Cop0[Cpu.StatusRegister] = Cpu.StatusInterruptEnable | (1UL << 10);
                    cpu.Cop0Written();
                    bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x101);
                });

            AssertSame(interpreted, compiled, 200_000);

            Assert.True(compiled.Bus.Read32(0x800) > 20);
        }

        // The compiler's thread publishes the block while the processor keeps interpreting it; the result cannot depend on when - see Mars_Recompiler.md §2.4.
        [Fact]
        public void A_block_compiled_in_the_background_arrives_and_leaves_the_same_machine()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0));
            compiled.CompileInBackground = true;

            AssertSame(interpreted, compiled, 3_000);

            var waited = System.Diagnostics.Stopwatch.StartNew();
            while (compiled.BlocksCompiled == 0 && waited.ElapsedMilliseconds < 10_000) compiled.RunBlocks(300);
            Assert.True(compiled.BlocksCompiled >= 1, "the compiler's thread never published the block");

            long end = compiled.Instructions + 3_000;
            while (interpreted.Instructions < end) interpreted.Step();
            compiled.RunBlocks(end - compiled.Instructions);

            Assert.Equal(State(interpreted), State(compiled));
        }

        // A store the MI repeats lands sixteen bytes from eight before the block; the bus path reports beyond memory so the block ends - see Mars_Recompiler.md §4.
        [Fact]
        public void A_repeated_store_reaching_the_block_from_before_it_is_seen_by_the_next_pass()
        {
            var (interpreted, compiled) = Pair(a => a
                .Lui(A0, 0x8000).Addiu(T1, Zero, 3000).Lui(T4, 0x240A).Ori(T4, T4, 7)
                .Beq(Zero, Zero, 2)
                .Nop()
                .Nop()
                .Nop()
                .Addiu(T2, Zero, 1)
                .Nop()
                .Addu(T3, T3, T2)
                .Addiu(T0, T0, 1)
                .Sw(T4, A0, 24)
                .Bne(T0, T1, -6)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 600);
            Assert.True(compiled.BlocksCompiled >= 1);

            foreach (Cpu cpu in new[] { interpreted, compiled })
            {
                cpu.Bus.Mi.Repeating = true;
                cpu.Bus.Mi.RepeatCount = 15;
            }

            AssertSame(interpreted, compiled, 20_000);

            Assert.Equal(0x240A_0007u, compiled.Bus.Read32(0x20));
            Assert.True(compiled.BlocksDiscarded >= 1);
        }

        // The signal processor's DMA lands new code under a running block; the RSP step that wrote ends it - see Mars_Recompiler.md §4.
        [Fact]
        public void Code_the_signal_processor_writes_under_a_running_block_is_seen_by_the_next_pass()
        {
            const uint Patched = 0x240A_0007, Kept = 0x016A_5821;

            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 4000)
                .Addiu(T0, T0, 1)
                .Addiu(T2, Zero, 1)
                .Addu(T3, T3, T2)
                .Bne(T0, T1, -4)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop(), (bus, _) =>
                {
                    bus.Write32(MemoryMap.SpDmemBase + 0x100, Patched);
                    bus.Write32(MemoryMap.SpDmemBase + 0x104, Kept);

                    uint[] rsp = new MipsAssembler()
                        .Nop().Nop().Nop().Nop().Nop().Nop().Nop().Nop().Nop().Nop().Nop().Nop()
                        .Addiu(T0, Zero, 0x100).Word(RspMtc0(T0, 0))
                        .Addiu(T0, Zero, 0x08).Word(RspMtc0(T0, 1))
                        .Addiu(T0, Zero, 0).Word(RspMtc0(T0, 3))
                        .Break().ToArray();
                    for (int i = 0; i < rsp.Length; i++) bus.Write32(MemoryMap.SpImemBase + (uint)i * 4, rsp[i]);
                });

            AssertSame(interpreted, compiled, 2_000);
            Assert.True(compiled.BlocksCompiled >= 1);

            foreach (Cpu cpu in new[] { interpreted, compiled }) cpu.Bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x01);

            AssertSame(interpreted, compiled, 20_000);

            Assert.Equal(Patched, compiled.Bus.Read32(0x08));
            Assert.True(compiled.BlocksDiscarded >= 1);
            Assert.True(compiled.Gpr[T3] > 4000);
        }

        // Outside kernel mode a kseg0 fetch faults in the interpreter; a block there would have run it - see Mars_Recompiler.md §1.
        [Fact]
        public void A_hot_loop_outside_kernel_mode_is_left_to_the_interpreter()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0), (bus, _) => Handler(bus, a => a.Eret()));

            AssertSame(interpreted, compiled, 600);
            Assert.True(compiled.BlocksCompiled >= 1);

            foreach (Cpu cpu in new[] { interpreted, compiled })
            {
                cpu.Cop0[Cpu.StatusRegister] = 0x10;
                cpu.Cop0Written();
            }

            AssertSame(interpreted, compiled, 200);

            Assert.Equal(200UL, compiled.Gpr[T0]);
            Assert.Equal(MipsAssembler.EntryPoint, compiled.Cop0[Cpu.ExceptionPcRegister]);
        }

        // The inlined arithmetic on the values that tell it from the interpreter's - see Mars_Recompiler.md §3.3.
        [Fact]
        public void The_inlined_arithmetic_matches_the_interpreter_where_the_widths_matter()
        {
            const int S0 = 16, S1 = 17, S2 = 18, T8 = 24, T9 = 25;

            var (interpreted, compiled) = Pair(a => a
                .Lui(T8, 0x7FFF).Ori(T8, T8, 0xFFFF)
                .Lui(T5, 0x8000).Dsll32(T5, T5, 0).Dsrl32(T5, T5, 0)
                .Addiu(T1, Zero, 300)
                .Addiu(T0, T0, 1)
                .Sra(T6, T5, 1)
                .Addiu(T7, T8, 1)
                .Sltiu(T9, T0, -1)
                .Addiu(Zero, Zero, 5)
                .Addu(S0, S0, T6)
                .Addu(S1, S1, T7)
                .Addu(S2, S2, T9)
                .Bne(T0, T1, -8)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 4_000);

            Assert.True(compiled.BlocksCompiled >= 1);
            Assert.Equal(0x4000_0000UL, compiled.Gpr[T6]);
            Assert.Equal(0xFFFF_FFFF_8000_0000UL, compiled.Gpr[T7]);
            Assert.Equal(1UL, compiled.Gpr[T9]);
            Assert.Equal(0UL, compiled.Gpr[Zero]);
        }

        // The count the timer is scheduled from is what the last completed instruction left, even when a fault ended the block - see Mars_Recompiler.md §3.2.
        [Fact]
        public void A_fault_that_ends_a_block_leaves_the_count_the_interpreter_leaves()
        {
            var (interpreted, compiled) = Pair(a => a
                .Lui(A0, 0x8000).Ori(A0, A0, 0x0801).Addiu(T1, Zero, 3000)
                .Addiu(T0, T0, 1)
                .Addu(T2, T2, T0)
                .Lw(T3, A0, 0)
                .Addiu(T2, T2, 5)
                .Bne(T0, T1, -5)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop(), (bus, _) => Handler(bus, a => a
                    .Mfc0(K0, Cpu.ExceptionPcRegister).Addiu(K0, K0, 4).Mtc0(K0, Cpu.ExceptionPcRegister)
                    .Eret()));

            AssertSame(interpreted, compiled, 3 + 9 * 600);
            Assert.Equal(MipsAssembler.EntryPoint + 12, compiled.Pc);
            Assert.True(compiled.BlocksCompiled >= 1);

            interpreted.Step();
            interpreted.Step();
            interpreted.Step();
            compiled.StepBlock(long.MaxValue, compiled.Bus.Vi.Fields);

            Assert.Equal(interpreted.Instructions, compiled.Instructions);
            Assert.Equal(Cpu.VectorBase | (ulong)Cpu.VectorOffsetGeneral, compiled.Pc);
            Assert.Equal(State(interpreted), State(compiled));
        }

        // A field so short that frames end inside a block's cold runs, which the interpreter walks no further than the frame - see Mars_Recompiler.md §3.1.
        [Fact]
        public void A_frame_ending_inside_a_cold_run_ends_on_the_same_instruction()
        {
            MipsAssembler program = new MipsAssembler()
                .Lui(A0, 0xA440).Addiu(T1, Zero, 2).Sw(T1, A0, 0x18).Addiu(T1, Zero, 0x40).Sw(T1, A0, 0x1C);
            for (int i = 0; i < 30; i++) program.Addiu(T2, T2, (short)(i + 1)).Xor(T3, T3, T2);
            program.Beq(Zero, Zero, -61).Nop();

            (MarsCore interpreted, MarsCore compiled) = AssertSameFrames(program.ToArray(), 120);

            Assert.True(compiled.Bus!.Vi.Fields >= 120);
            Assert.Equal(interpreted.Bus!.Cycles, compiled.Bus.Cycles);
        }

        // A loop with a branch inside it, stopped once on the slot of a branch that fell through, where the slot flag is still up - see Mars_Recompiler.md §10.
        [Fact]
        public void A_branch_inside_a_loop_leaves_the_machine_the_interpreter_leaves_including_on_its_slot()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 3000)
                .Addiu(T0, T0, 1)
                .Andi(T5, T0, 1)
                .Beq(T5, Zero, 2)
                .Addiu(T6, T6, 1)
                .Addiu(T7, T7, 1)
                .Addu(T2, T2, T0)
                .Bne(T0, T1, -7)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 1 + 100 * 15 + 4);
            Assert.True(compiled.InDelaySlot);
            Assert.Equal(MipsAssembler.EntryPoint + 20, compiled.Pc);

            AssertSame(interpreted, compiled, 30_000 - 1505);

            Assert.Equal(3000UL, compiled.Gpr[T6]);
            Assert.Equal(1500UL, compiled.Gpr[T7]);
            Assert.True(compiled.BlocksCompiled >= 1);
        }

        // A likely branch inside a loop, nullifying its slot every other iteration - see Mars_Recompiler.md §10.
        [Fact]
        public void A_likely_branch_inside_a_loop_leaves_the_machine_the_interpreter_leaves()
        {
            var (interpreted, compiled) = Pair(a => a
                .Addiu(T1, Zero, 3000)
                .Addiu(T0, T0, 1)
                .Andi(T5, T0, 1)
                .Bnel(T5, Zero, 2)
                .Addiu(T6, T6, 1)
                .Addiu(T7, T7, 1)
                .Addu(T2, T2, T0)
                .Bne(T0, T1, -7)
                .Nop()
                .Beq(Zero, Zero, -1)
                .Nop());

            AssertSame(interpreted, compiled, 30_000);

            Assert.Equal(1500UL, compiled.Gpr[T6]);
            Assert.Equal(1500UL, compiled.Gpr[T7]);
        }

        [Fact]
        public void A_synthetic_rom_runs_the_same_frames_with_blocks_as_without()
        {
            uint[] program = new MipsAssembler()
                .Lui(A0, 0xA440).Addiu(T1, Zero, 525).Sw(T1, A0, 0x18).Addiu(T1, Zero, 3093).Sw(T1, A0, 0x1C)
                .Lui(A1, 0x8000)
                .Lw(T0, A1, 0x1000).Addiu(T0, T0, 1).Sw(T0, A1, 0x1000)
                .Addiu(T2, T2, 3).Xor(T3, T3, T2)
                .Beq(Zero, Zero, -6)
                .Nop()
                .ToArray();

            AssertSameFrames(program, 5);
        }

        [Fact]
        public void The_cycle_cap_ends_a_frame_on_the_same_cycle_with_blocks_as_without()
        {
            uint[] program = new MipsAssembler()
                .Addiu(T0, T0, 1)
                .Beq(Zero, Zero, -2)
                .Addu(T1, T1, T0)
                .ToArray();

            (MarsCore interpreted, MarsCore compiled) = AssertSameFrames(program, 3);

            Assert.Equal(0, compiled.Bus!.Vi.Fields);
            Assert.InRange(compiled.Bus.Cycles, 3 * MarsCore.CycleCap, 3 * MarsCore.CycleCap + 24);
            Assert.Equal(interpreted.Bus!.Cycles, compiled.Bus.Cycles);
        }

        private static (Cpu Interpreted, Cpu Compiled) Pair(Func<MipsAssembler, MipsAssembler> program, Action<MemoryBus, Cpu>? setup = null)
        {
            Cpu interpreted = program(new MipsAssembler()).Build();
            Cpu compiled = program(new MipsAssembler()).Build();
            compiled.CompileInBackground = false;

            setup?.Invoke(interpreted.Bus, interpreted);
            setup?.Invoke(compiled.Bus, compiled);

            return (interpreted, compiled);
        }

        // The same count of instructions that completed, since neither side counts one that faulted - see Mars_Cpu.md §4.
        private static void AssertSame(Cpu interpreted, Cpu compiled, int steps)
        {
            long end = interpreted.Instructions + steps;
            while (interpreted.Instructions < end) interpreted.Step();
            compiled.RunBlocks(steps);

            Assert.Equal(interpreted.Instructions, compiled.Instructions);
            Assert.Equal(interpreted.Pc, compiled.Pc);
            Assert.Equal(interpreted.Bus.Cycles, compiled.Bus.Cycles);
            Assert.Equal(State(interpreted), State(compiled));
        }

        // The boot runs a synthetic image from the signal processor's memory; a stub carries the program into RDRAM, where blocks apply.
        private static uint[] IntoRdram(uint[] program)
        {
            const int Base = 24, Word = 25;
            var stub = new MipsAssembler().Lui(Base, 0x8000).Ori(Base, Base, 0x0400);

            for (int i = 0; i < program.Length; i++)
            {
                stub.Lui(Word, (ushort)(program[i] >> 16)).Ori(Word, Word, (ushort)program[i]).Sw(Word, Base, (short)(i * 4));
            }

            return stub.Jr(Base).Nop().ToArray();
        }

        private static (MarsCore Interpreted, MarsCore Compiled) AssertSameFrames(uint[] program, int frames)
        {
            uint[] words = IntoRdram(program);
            var bytes = new byte[words.Length * 4];
            for (int i = 0; i < words.Length; i++) System.Buffers.Binary.BinaryPrimitives.WriteUInt32BigEndian(bytes.AsSpan(i * 4), words[i]);

            string path = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, bytes)));

            var interpreted = new MarsCore { UseBlocks = false };
            var compiled = new MarsCore();
            interpreted.LoadRom(path);
            compiled.LoadRom(path);
            compiled.Cpu!.CompileInBackground = false;

            for (int i = 0; i < frames; i++)
            {
                interpreted.RunFrame();
                compiled.RunFrame();
                Assert.Equal(State(interpreted), State(compiled));
            }

            Assert.True(compiled.Cpu!.BlocksCompiled >= 1);
            return (interpreted, compiled);
        }

        private static byte[] State(Cpu cpu)
        {
            using var stream = new MemoryStream();
            using var w = new BinaryWriter(stream);
            StateSerializer.Write(w, cpu);
            cpu.Bus.WriteState(w);
            return stream.ToArray();
        }

        private static byte[] State(MarsCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }

        // The signal processor's mtc0, whose registers are the interface's own - see Mars_Rsp.md §5.
        private static uint RspMtc0(int rt, int rd) => (0x10u << 26) | (4u << 21) | ((uint)rt << 16) | ((uint)rd << 11);

        // An interrupt or fault lands at the general vector, where this program waits - see Mars_Cpu.md §11.
        private static void Handler(MemoryBus bus, Func<MipsAssembler, MipsAssembler> program)
        {
            uint[] words = program(new MipsAssembler()).ToArray();
            for (int i = 0; i < words.Length; i++) bus.Write32((uint)Cpu.VectorOffsetGeneral + (uint)i * 4, words[i]);
        }

        // The VI and the AI as tickbench programs them: an NTSC field and a fast sample clock.
        private static void ProgramDevices(MemoryBus bus, Cpu cpu)
        {
            bus.Write32(MemoryMap.ViBase + 0x18, 525);
            bus.Write32(MemoryMap.ViBase + 0x1C, 3093);
            bus.Write32(MemoryMap.AiBase + 0x10, 1520);
            bus.Write32(MemoryMap.AiBase + 0x08, 1);
            bus.Write32(MemoryMap.AiBase + 0x00, 0x100000);
            bus.Write32(MemoryMap.AiBase + 0x04, 0x3FFF8);
            bus.Write32(MemoryMap.AiBase + 0x04, 0x3FFF8);
        }
    }
}
