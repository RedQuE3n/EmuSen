using EmuSen.Cores.Nintendo.Moon.Processor;

namespace EmuSen.WiseMan.Cores
{
    // Self-contained 2A03 checks; the exhaustive vector run is Pharaoh's --singlestep - see Moon_CPU.md §7.4.
    public class Nes6502CpuTests
    {
        private sealed class FlatBus : ICpuBus
        {
            public readonly byte[] Memory = new byte[0x10000];
            public readonly List<(ushort Address, byte Value, bool Write)> Trace = new();

            public byte Read(ushort address)
            {
                Trace.Add((address, Memory[address], false));
                return Memory[address];
            }

            public void Write(ushort address, byte data)
            {
                Memory[address] = data;
                Trace.Add((address, data, true));
            }
        }

        private const ushort CodeBase = 0x8000;

        private static (Cpu Cpu, FlatBus Bus) Machine(params byte[] code)
        {
            var bus = new FlatBus();
            var cpu = new Cpu(bus);
            cpu.Reset();

            for (int i = 0; i < code.Length; i++) bus.Memory[CodeBase + i] = code[i];

            cpu.PC = CodeBase;
            bus.Trace.Clear();
            return (cpu, bus);
        }

        // Two cycles: the opcode fetch, then the discarded read of the byte after it.
        [Theory]
        [InlineData(0xEA)] // NOP
        [InlineData(0xE8)] // INX
        [InlineData(0x88)] // DEY
        [InlineData(0xAA)] // TAX
        [InlineData(0x0A)] // ASL A
        [InlineData(0x18)] // CLC
        public void Register_only_opcodes_cost_two_cycles(byte opcode)
        {
            var (cpu, _) = Machine(opcode);
            Assert.Equal(2, cpu.Step());
        }

        [Fact]
        public void Indexed_read_pays_an_extra_cycle_only_when_it_crosses_a_page()
        {
            var (same, _) = Machine(0xBD, 0x00, 0x20); // LDA $2000,X
            same.X = 0x10;
            Assert.Equal(4, same.Step());

            var (crossed, _) = Machine(0xBD, 0xF0, 0x20); // LDA $20F0,X
            crossed.X = 0x20;
            Assert.Equal(5, crossed.Step());
        }

        // A store cannot un-write a wrong address, so it always pays the fixup - see Moon_CPU.md §3.1.
        [Fact]
        public void Indexed_store_pays_the_fixup_cycle_even_without_a_page_cross()
        {
            var (cpu, bus) = Machine(0x9D, 0x00, 0x20); // STA $2000,X
            cpu.X = 0x10;
            cpu.A = 0x42;

            Assert.Equal(5, cpu.Step());
            Assert.Equal(0x42, bus.Memory[0x2010]);
        }

        // The unmodified value goes back on the bus before the modified one - see Moon_CPU.md §3.3.
        [Fact]
        public void Read_modify_write_writes_the_old_value_before_the_new_one()
        {
            var (cpu, bus) = Machine(0xEE, 0x00, 0x20); // INC $2000
            bus.Memory[0x2000] = 0x07;

            Assert.Equal(6, cpu.Step());

            var writes = bus.Trace.Where(t => t.Write).ToArray();
            Assert.Equal(2, writes.Length);
            Assert.Equal((ushort)0x2000, writes[0].Address);
            Assert.Equal(0x07, writes[0].Value);
            Assert.Equal(0x08, writes[1].Value);
        }

        [Fact]
        public void Jmp_indirect_takes_its_high_byte_from_the_start_of_the_same_page()
        {
            var (cpu, bus) = Machine(0x6C, 0xFF, 0x02); // JMP ($02FF)
            bus.Memory[0x02FF] = 0x34;
            bus.Memory[0x0300] = 0xAA; // what a carrying chip would read
            bus.Memory[0x0200] = 0x12; // what the real one reads

            Assert.Equal(5, cpu.Step());
            Assert.Equal(0x1234, cpu.PC);
        }

        [Fact]
        public void Zero_page_indexing_wraps_inside_the_zero_page()
        {
            var (cpu, bus) = Machine(0xB5, 0xFF); // LDA $FF,X
            cpu.X = 0x02;
            bus.Memory[0x0001] = 0x5A;
            bus.Memory[0x0101] = 0xFF; // where a carrying chip would look

            Assert.Equal(4, cpu.Step());
            Assert.Equal(0x5A, cpu.A);
        }

        [Fact]
        public void Indirect_pointer_high_byte_wraps_inside_the_zero_page()
        {
            var (cpu, bus) = Machine(0xB1, 0xFF); // LDA ($FF),Y
            cpu.Y = 0x00;
            bus.Memory[0x00FF] = 0x00;
            bus.Memory[0x0000] = 0x30; // pointer high byte, wrapped
            bus.Memory[0x0100] = 0xAA; // where a carrying chip would look
            bus.Memory[0x3000] = 0x77;

            Assert.Equal(5, cpu.Step());
            Assert.Equal(0x77, cpu.A);
        }

        [Fact]
        public void Branches_cost_two_three_or_four_cycles()
        {
            var (notTaken, _) = Machine(0xD0, 0x10); // BNE +16
            notTaken.SetFlag(CpuFlags.Z, true);
            Assert.Equal(2, notTaken.Step());

            var (taken, _) = Machine(0xD0, 0x10);
            taken.SetFlag(CpuFlags.Z, false);
            Assert.Equal(3, taken.Step());

            var (crossed, bus) = Machine();
            bus.Memory[0x80F0] = 0xD0;
            bus.Memory[0x80F1] = 0x40; // lands in the next page
            crossed.PC = 0x80F0;
            crossed.SetFlag(CpuFlags.Z, false);
            Assert.Equal(4, crossed.Step());
        }

        [Fact]
        public void Php_pushes_both_phantom_bits_and_plp_restores_neither()
        {
            var (push, bus) = Machine(0x08); // PHP
            push.P = 0xA4;
            push.S = 0xFF;
            push.Step();

            Assert.Equal(0xB4, bus.Memory[0x01FF]);
            Assert.Equal(0xA4, push.P);

            var (pull, pullBus) = Machine(0x28); // PLP
            pull.S = 0xFE;
            pullBus.Memory[0x01FF] = 0xFF;
            pull.Step();

            Assert.Equal(0xEF, pull.P);
        }

        [Fact]
        public void Brk_pushes_the_address_after_its_signature_byte_and_masks_irqs()
        {
            var (cpu, bus) = Machine(0x00, 0x00); // BRK
            bus.Memory[0xFFFE] = 0x00;
            bus.Memory[0xFFFF] = 0x90;
            cpu.S = 0xFF;
            cpu.P = (byte)CpuFlags.U;

            Assert.Equal(7, cpu.Step());
            Assert.Equal(0x9000, cpu.PC);
            Assert.Equal(0x80, bus.Memory[0x01FF]);       // PCH
            Assert.Equal(0x02, bus.Memory[0x01FE]);       // PCL - the signature byte is skipped
            Assert.Equal(0x30, bus.Memory[0x01FD]);       // P with both phantom bits set
            Assert.True(cpu.GetFlag(CpuFlags.I));
        }

        // Three cycles, PC left on the opcode, and nothing but Reset gets it back - Moon_CPU.md §6.4.
        [Fact]
        public void Jam_wedges_the_cpu_without_advancing_pc()
        {
            var (cpu, _) = Machine(0x02);

            Assert.Equal(3, cpu.Step());
            Assert.Equal(CodeBase, cpu.PC);
            Assert.True(cpu.Jammed);

            cpu.Step();
            Assert.Equal(CodeBase, cpu.PC);

            cpu.Reset();
            Assert.False(cpu.Jammed);
        }

        // Interrupts are this file's alone, and every case NOP-fills its path - see Moon_CPU.md §7.4.
        [Fact]
        public void Irq_is_ignored_while_the_disable_flag_is_set()
        {
            var (cpu, bus) = Machine(0xEA, 0xEA); // NOP ; NOP
            bus.Memory[0xFFFE] = 0x00;
            bus.Memory[0xFFFF] = 0x90;
            cpu.SetFlag(CpuFlags.I, true);
            cpu.SetIrqLine(true);

            cpu.Step();
            cpu.Step();

            Assert.Equal(CodeBase + 2, cpu.PC);
        }

        // The poll happens before SEI's flag write lands, so the IRQ it "blocks" still fires.
        [Fact]
        public void Sei_does_not_mask_an_irq_already_pending_when_it_runs()
        {
            var (cpu, bus) = Machine(0x78, 0xEA); // SEI ; NOP
            bus.Memory[0xFFFE] = 0x00;
            bus.Memory[0xFFFF] = 0x90;
            cpu.SetFlag(CpuFlags.I, false);
            cpu.SetIrqLine(true);

            cpu.Step();
            Assert.True(cpu.GetFlag(CpuFlags.I));

            cpu.Step();
            Assert.Equal(0x9000, cpu.PC);
        }

        // The mirror image: CLI's write also lands late, so one more instruction runs first.
        [Fact]
        public void Cli_lets_one_more_instruction_run_before_the_irq_is_taken()
        {
            var (cpu, bus) = Machine(0x58, 0xEA, 0xEA); // CLI ; NOP ; NOP
            bus.Memory[0xFFFE] = 0x00;
            bus.Memory[0xFFFF] = 0x90;
            cpu.SetFlag(CpuFlags.I, true);
            cpu.SetIrqLine(true);

            cpu.Step();
            Assert.False(cpu.GetFlag(CpuFlags.I));

            cpu.Step();
            Assert.Equal(CodeBase + 2, cpu.PC);

            cpu.Step();
            Assert.Equal(0x9000, cpu.PC);
        }

        // Edge-triggered: holding the line high must not queue a second entry - Moon_CPU.md §5.2.
        [Fact]
        public void Nmi_fires_once_per_edge_not_once_per_step_while_held()
        {
            var (cpu, bus) = Machine(0xEA, 0xEA, 0xEA);
            bus.Memory[0xFFFA] = 0x00;
            bus.Memory[0xFFFB] = 0x90;
            bus.Memory[0x9000] = 0xEA;

            bus.Memory[0x9000] = 0xEA;
            bus.Memory[0x9001] = 0xEA;

            cpu.SetNmiLine(true);
            cpu.Step();
            cpu.Step();
            Assert.Equal(0x9000, cpu.PC);

            // Still high, so there is no second edge: the handler must just run on.
            cpu.SetNmiLine(true);
            cpu.Step();
            cpu.Step();
            Assert.Equal(0x9002, cpu.PC);
        }

        [Fact]
        public void Nmi_entry_pushes_the_status_byte_with_the_break_bit_clear()
        {
            var (cpu, bus) = Machine(0xEA);
            bus.Memory[0xFFFA] = 0x00;
            bus.Memory[0xFFFB] = 0x90;
            cpu.S = 0xFF;
            cpu.P = (byte)CpuFlags.U;

            cpu.SetNmiLine(true);
            cpu.Step();
            cpu.Step();

            Assert.Equal(0x20, bus.Memory[0x01FD]);
            Assert.True(cpu.GetFlag(CpuFlags.I));
        }

        [Fact]
        public void Reset_leaves_the_stack_pointer_where_three_phantom_pushes_would()
        {
            var bus = new FlatBus();
            bus.Memory[0xFFFC] = 0x34;
            bus.Memory[0xFFFD] = 0x12;

            var cpu = new Cpu(bus);
            cpu.Reset();

            Assert.Equal(0x1234, cpu.PC);
            Assert.Equal(Cpu.ResetStackPointer, cpu.S);
            Assert.True(cpu.GetFlag(CpuFlags.I));
        }
    }
}
