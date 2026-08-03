using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;

namespace EmuSen.WiseMan.Cores
{
    // The SPC700 and NEC DSP decoders - see Venus_APU.md §8 and Venus_NecDSP.md §8.
    public class Spc700DisassemblerTests
    {
        private static (string Mnemonic, string Operands, int Length) One(params byte[] bytes)
        {
            var result = Spc700Disassembler.Disassemble(a => a < bytes.Length ? bytes[a] : (byte)0, 0, 1);
            var instr = result[0];
            return (instr.Mnemonic, instr.OperandText, instr.Length);
        }

        [Theory]
        // The IPL boot sequence, which every SNES executes from power-on.
        [InlineData(new byte[] { 0xCD, 0xEF }, "MOV", "X, #$EF", 2)]
        [InlineData(new byte[] { 0xBD }, "MOV", "SP, X", 1)]
        [InlineData(new byte[] { 0xE8, 0x00 }, "MOV", "A, #$00", 2)]
        [InlineData(new byte[] { 0xC6 }, "MOV", "(X), A", 1)]
        [InlineData(new byte[] { 0x1D }, "DEC", "X", 1)]
        // Addressing modes that differ only in one nibble.
        [InlineData(new byte[] { 0xE4, 0x20 }, "MOV", "A, $20", 2)]
        [InlineData(new byte[] { 0xF4, 0x20 }, "MOV", "A, $20+X", 2)]
        [InlineData(new byte[] { 0xE5, 0x34, 0x12 }, "MOV", "A, $1234", 3)]
        [InlineData(new byte[] { 0xF5, 0x34, 0x12 }, "MOV", "A, $1234+X", 3)]
        [InlineData(new byte[] { 0xF6, 0x34, 0x12 }, "MOV", "A, $1234+Y", 3)]
        [InlineData(new byte[] { 0xE7, 0x20 }, "MOV", "A, [$20+X]", 2)]
        [InlineData(new byte[] { 0xF7, 0x20 }, "MOV", "A, [$20]+Y", 2)]
        // ALU rows.
        [InlineData(new byte[] { 0x04, 0x10 }, "OR", "A, $10", 2)]
        [InlineData(new byte[] { 0x24, 0x10 }, "AND", "A, $10", 2)]
        [InlineData(new byte[] { 0x44, 0x10 }, "EOR", "A, $10", 2)]
        [InlineData(new byte[] { 0x64, 0x10 }, "CMP", "A, $10", 2)]
        [InlineData(new byte[] { 0x84, 0x10 }, "ADC", "A, $10", 2)]
        [InlineData(new byte[] { 0xA4, 0x10 }, "SBC", "A, $10", 2)]
        [InlineData(new byte[] { 0x19 }, "OR", "(X), (Y)", 1)]
        // 16-bit and multiply/divide.
        [InlineData(new byte[] { 0xBA, 0x40 }, "MOVW", "YA, $40", 2)]
        [InlineData(new byte[] { 0xDA, 0x40 }, "MOVW", "$40, YA", 2)]
        [InlineData(new byte[] { 0x7A, 0x40 }, "ADDW", "YA, $40", 2)]
        [InlineData(new byte[] { 0x9A, 0x40 }, "SUBW", "YA, $40", 2)]
        [InlineData(new byte[] { 0x5A, 0x40 }, "CMPW", "YA, $40", 2)]
        [InlineData(new byte[] { 0xCF }, "MUL", "YA", 1)]
        [InlineData(new byte[] { 0x9E }, "DIV", "YA, X", 1)]
        // Control flow.
        [InlineData(new byte[] { 0x3F, 0x00, 0x08 }, "CALL", "$0800", 3)]
        [InlineData(new byte[] { 0x6F }, "RET", "", 1)]
        [InlineData(new byte[] { 0x7F }, "RETI", "", 1)]
        [InlineData(new byte[] { 0x5F, 0xC0, 0xFF }, "JMP", "$FFC0", 3)]
        [InlineData(new byte[] { 0x1F, 0x00, 0x20 }, "JMP", "[$2000+X]", 3)]
        [InlineData(new byte[] { 0x4F, 0x30 }, "PCALL", "$FF30", 2)]
        [InlineData(new byte[] { 0xEF }, "SLEEP", "", 1)]
        [InlineData(new byte[] { 0xFF }, "STOP", "", 1)]
        public void An_instruction_decodes_to_its_documented_form(byte[] bytes, string mnemonic, string operands, int length)
        {
            var (m, o, l) = One(bytes);

            Assert.Equal(mnemonic, m);
            Assert.Equal(operands, o);
            Assert.Equal(length, l);
        }

        // Relative operands are targets, not raw displacements.
        [Theory]
        [InlineData(0x2F, "BRA")]
        [InlineData(0xF0, "BEQ")]
        [InlineData(0xD0, "BNE")]
        [InlineData(0x10, "BPL")]
        [InlineData(0x30, "BMI")]
        [InlineData(0x90, "BCC")]
        [InlineData(0xB0, "BCS")]
        [InlineData(0x50, "BVC")]
        [InlineData(0x70, "BVS")]
        public void A_branch_resolves_its_target_from_the_end_of_the_instruction(byte opcode, string mnemonic)
        {
            var forward = Spc700Disassembler.Disassemble(a => a == 0 ? opcode : (byte)0x10, 0, 1)[0];
            var backward = Spc700Disassembler.Disassemble(a => a == 0 ? opcode : (byte)0xFE, 0, 1)[0];

            Assert.Equal(mnemonic, forward.Mnemonic);
            Assert.Equal("$0012", forward.OperandText);   // 0 + 2 + 0x10
            Assert.Equal("$0000", backward.OperandText);  // 0 + 2 - 2
        }

        // Bytes are stored source-then-destination, but read destination-first.
        [Fact]
        public void A_two_operand_direct_page_move_reads_destination_first()
        {
            var (m, o, l) = One(0xFA, 0x11, 0x22);

            Assert.Equal("MOV", m);
            Assert.Equal("$22, $11", o);
            Assert.Equal(3, l);
        }

        [Fact]
        public void An_immediate_to_direct_page_move_reads_destination_first()
        {
            var (_, o, _) = One(0x8F, 0xEE, 0x40);

            Assert.Equal("$40, #$EE", o);
        }

        // The 13-bit address packs the bit number into its top three bits.
        [Theory]
        [InlineData(0xAA, "MOV1", "C, $1234.0", 0x1234)]
        [InlineData(0xCA, "MOV1", "$1234.0, C", 0x1234)]
        [InlineData(0xEA, "NOT1", "$1234.0", 0x1234)]
        public void A_membit_operand_splits_address_from_bit_number(byte opcode, string mnemonic, string operands, int address)
        {
            var (m, o, l) = One(opcode, (byte)address, (byte)(address >> 8));

            Assert.Equal(mnemonic, m);
            Assert.Equal(operands, o);
            Assert.Equal(3, l);
        }

        [Fact]
        public void The_bit_number_comes_from_the_top_three_bits()
        {
            // $E234 = bit 7, address $0234.
            var (_, o, _) = One(0xAA, 0x34, 0xE2);

            Assert.Equal("C, $0234.7", o);
        }

        [Theory]
        [InlineData(0x02, "SET1", "$40.0")]
        [InlineData(0xE2, "SET1", "$40.7")]
        [InlineData(0x12, "CLR1", "$40.0")]
        [InlineData(0xF2, "CLR1", "$40.7")]
        public void The_bit_ladders_map_to_their_nibble_pattern(byte opcode, string mnemonic, string operands)
        {
            var (m, o, _) = One(opcode, 0x40);

            Assert.Equal(mnemonic, m);
            Assert.Equal(operands, o);
        }

        [Fact]
        public void A_bit_branch_carries_both_a_direct_page_byte_and_a_target()
        {
            var (m, o, l) = One(0x03, 0x40, 0x10);

            Assert.Equal("BBS", m);
            Assert.Equal("$40.0, $0013", o);   // 0 + 3 + 0x10
            Assert.Equal(3, l);
        }

        [Theory]
        [InlineData(0x2E, "CBNE", "$40, $0013")]
        [InlineData(0x6E, "DBNZ", "$40, $0013")]
        public void A_compare_and_branch_carries_both_operands(byte opcode, string mnemonic, string operands)
        {
            var (m, o, _) = One(opcode, 0x40, 0x10);

            Assert.Equal(mnemonic, m);
            Assert.Equal(operands, o);
        }

        // TCALL n is a whole nibble column, so an off-by-one would be silent.
        [Theory]
        [InlineData(0x01, "0")]
        [InlineData(0x31, "3")]
        [InlineData(0xF1, "15")]
        public void Every_tcall_slot_decodes_to_its_own_number(byte opcode, string slot)
        {
            var (m, o, l) = One(opcode);

            Assert.Equal("TCALL", m);
            Assert.Equal(slot, o);
            Assert.Equal(1, l);
        }

        // Nothing may fall through to the raw-byte placeholder.
        [Fact]
        public void Every_opcode_decodes_to_a_real_instruction()
        {
            for (int op = 0; op < 256; op++)
            {
                byte opcode = (byte)op;
                var instr = Spc700Disassembler.Disassemble(a => a == 0 ? opcode : (byte)0, 0, 1)[0];
                Assert.False(instr.Mnemonic == ".db", $"opcode 0x{op:X2} has no entry");
                Assert.InRange(instr.Length, 1, 3);
            }
        }

        // Walking forward must respect each instruction's real length.
        [Fact]
        public void Consecutive_instructions_advance_by_their_own_lengths()
        {
            byte[] program = { 0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0x5F, 0xC0, 0xFF };

            var result = Spc700Disassembler.Disassemble(a => program[a], 0, 4);

            Assert.Equal(new[] { 0, 2, 3, 5 }, result.Select(i => i.Address));
            Assert.Equal(new[] { "MOV", "MOV", "MOV", "JMP" }, result.Select(i => i.Mnemonic));
        }

        [Fact]
        public void Disassembly_wraps_at_the_end_of_the_64k_address_space()
        {
            var result = Spc700Disassembler.Disassemble(_ => 0xBD, 0xFFFF, 2);

            Assert.Equal(0xFFFF, result[0].Address);
            Assert.Equal(0x0000, result[1].Address);
        }
    }

    public class NecDspDisassemblerTests
    {
        private static (string Mnemonic, string Operands) One(int word, int pc = 0)
        {
            var instr = NecDspDisassembler.Disassemble(_ => word, pc, 1)[0];
            return (instr.Mnemonic, instr.OperandText);
        }

        // Bits 23-22 pick the form: 00 OP, 01 RT, 10 JP, 11 LD.
        [Fact]
        public void An_ld_loads_a_sixteen_bit_immediate_into_a_named_destination()
        {
            var (m, o) = One((0x3 << 22) | (0x1234 << 6) | 0x01);

            Assert.Equal("LDI", m);
            Assert.Equal("A, #$1234", o);
        }

        [Fact]
        public void An_op_with_no_work_at_all_reads_as_a_nop()
        {
            var (m, o) = One(0x000000);

            Assert.Equal("OP", m);
            Assert.Equal("NOP", o);
        }

        [Fact]
        public void The_return_form_is_distinguished_from_a_plain_op()
        {
            Assert.Equal("OP", One(0x000000).Mnemonic);
            Assert.Equal("RT", One(0x1 << 22).Mnemonic);
        }

        // ALU field is bits 19-16, accumulator select is bit 15.
        [Theory]
        [InlineData(0x01, "OR")]
        [InlineData(0x02, "AND")]
        [InlineData(0x05, "ADD")]
        [InlineData(0x07, "ADC")]
        public void A_binary_alu_operation_names_its_second_operand(int alu, string name)
        {
            var (_, o) = One(alu << 16);

            Assert.Equal($"{name} RAM, A", o);
        }

        // Operations 8-15 act on the accumulator alone, so they take no operand.
        [Theory]
        [InlineData(0x08, "DEC")]
        [InlineData(0x09, "INC")]
        [InlineData(0x0A, "CMP")]
        [InlineData(0x0B, "SHR1")]
        [InlineData(0x0F, "XCHG")]
        public void A_unary_alu_operation_takes_no_second_operand(int alu, string name)
        {
            var (_, o) = One(alu << 16);

            Assert.Equal($"{name} A", o);
        }

        [Fact]
        public void The_accumulator_select_bit_picks_B()
        {
            var (_, o) = One((0x05 << 16) | (1 << 15));

            Assert.Equal("ADD RAM, B", o);
        }

        // Bits 21-20 pick the second ALU operand.
        [Theory]
        [InlineData(0, "RAM")]
        [InlineData(2, "M")]
        [InlineData(3, "N")]
        public void The_second_operand_selector_is_decoded(int selector, string expected)
        {
            var (_, o) = One((selector << 20) | (0x05 << 16));

            Assert.Equal($"ADD {expected}, A", o);
        }

        [Fact]
        public void One_word_can_carry_both_an_alu_operation_and_a_move()
        {
            // ADD with source A (index 1) into destination B (index 2).
            var (_, o) = One((0x05 << 16) | (1 << 4) | 0x02);

            Assert.Contains("ADD", o);
            Assert.Contains("MOV A, B", o);
            Assert.Contains("|", o);
        }

        [Fact]
        public void An_rp_decrement_is_reported_alongside_the_rest()
        {
            var (_, o) = One(1 << 8);

            Assert.Equal("RP--", o);
        }

        // Only the unconditional forms choose their own 8KB half.
        [Fact]
        public void An_unconditional_jump_ignores_the_current_bank()
        {
            var (m, o) = One((0x2 << 22) | (0x100 << 13) | (0x123 << 2), pc: 0x2000);

            Assert.Equal("JMP", m);
            Assert.Equal("$0123", o);
        }

        [Fact]
        public void The_high_bank_jump_selects_the_upper_half()
        {
            var (m, o) = One((0x2 << 22) | (0x101 << 13) | (0x123 << 2), pc: 0x0000);

            Assert.Equal("JMP", m);
            Assert.Equal("$2123", o);
        }

        [Fact]
        public void A_call_is_named_separately_from_a_jump()
        {
            Assert.Equal("CALL", One((0x2 << 22) | (0x140 << 13)).Mnemonic);
            Assert.Equal("CALL", One((0x2 << 22) | (0x141 << 13)).Mnemonic);
        }

        // A conditional branch stays in the half it started in.
        [Fact]
        public void A_conditional_branch_inherits_the_current_bank()
        {
            var (m, o) = One((0x2 << 22) | (0x0BC << 13) | (0x123 << 2), pc: 0x2000);

            Assert.Equal("JNRQM", m);
            Assert.Equal("$2123", o);
        }

        [Theory]
        [InlineData(0x080, "JNCA")]
        [InlineData(0x08A, "JZA")]
        [InlineData(0x0BC, "JNRQM")]
        [InlineData(0x0BE, "JRQM")]
        public void Each_branch_condition_has_its_own_name(int jumpType, string mnemonic)
        {
            Assert.Equal(mnemonic, One((0x2 << 22) | (jumpType << 13)).Mnemonic);
        }

        [Fact]
        public void Every_word_is_three_bytes_long()
        {
            var result = NecDspDisassembler.Disassemble(_ => 0x123456, 0, 4);

            Assert.All(result, i => Assert.Equal(3, i.Length));
            Assert.Equal(new[] { 0, 1, 2, 3 }, result.Select(i => i.Address));
        }
    }
}
