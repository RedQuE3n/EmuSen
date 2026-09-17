using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Where coprocessor zero refuses an encoding and where it ignores one - see Mars_Cop0.md §10.
    public class MarsCop0DecodeTests
    {
        private static readonly int[] Defined = { 0x01, 0x02, 0x06, 0x08, 0x18 };

        // The corpus's sweep of the whole CO=1 space, which is inert everywhere it is not defined.
        public static TheoryData<uint> ReservedFunctions()
        {
            var data = new TheoryData<uint>();
            for (uint funct = 0; funct < 0x40; funct++)
            {
                if (funct == 0x10 || Array.IndexOf(Defined, (int)funct) >= 0) continue;
                data.Add(funct);
            }

            return data;
        }

        [Theory]
        [MemberData(nameof(ReservedFunctions))]
        public void A_reserved_coprocessor_zero_function_does_nothing_at_all(uint funct)
        {
            var cpu = new MipsAssembler().Cop0Function(funct).Addiu(1, 0, 7).Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(7UL, cpu.Gpr[1]);
            Assert.Equal(0UL, cpu.Cop0[Cpu.EntryHiRegister]);
            Assert.Equal(0UL, cpu.Cop0[Cpu.IndexRegister]);
        }

        // The single exception to that rule: the R3000's return-from-exception slot, dropped in the R4000.
        [Fact]
        public void The_function_code_the_R3000_used_for_its_own_return_still_refuses()
        {
            var cpu = new MipsAssembler().Cop0Function(0x10).Run(1);

            Assert.Equal(ExceptionCode.ReservedInstruction, cpu.LastException!.Code);
        }

        // Nothing between the sub-opcode and the function field is decoded, so no answer moves.
        [Theory]
        [InlineData(0x1FFu)]
        [InlineData(0x2AAu)]
        public void The_operand_bits_of_a_coprocessor_zero_function_change_nothing(uint operands)
        {
            var reserved = new MipsAssembler().Cop0Function(0x1F, operands).Run(1);
            var refused = new MipsAssembler().Cop0Function(0x10, operands).Run(1);

            Assert.Null(reserved.LastException);
            Assert.Equal(ExceptionCode.ReservedInstruction, refused.LastException!.Code);

            // A defined function has to execute through the same bits, so this one has to miss and say so.
            var probe = new MipsAssembler().Cop0Function(0x08, operands).Build();

            probe.Cop0[Cpu.EntryHiRegister] = 0x4000_0000;
            probe.Run(1);

            Assert.Equal(0x8000_0000UL, probe.Cop0[Cpu.IndexRegister]);
        }

        // Decoded but idle: the two control moves have no control registers, and the branch never goes.
        [Theory]
        [InlineData(0x02)]
        [InlineData(0x06)]
        [InlineData(0x08)]
        public void The_recognised_coprocessor_zero_sub_opcodes_are_inert_rather_than_refused(int rs)
        {
            var cpu = new MipsAssembler().Cop0SubOpcode(rs).Addiu(1, 0, 7).Nop().Run(3);

            Assert.Null(cpu.LastException);
            Assert.Equal(7UL, cpu.Gpr[1]);
        }

        // The boundary the pair above makes precise: the sub-opcodes around them have no meaning at all.
        [Theory]
        [InlineData(0x03)]
        [InlineData(0x07)]
        [InlineData(0x09)]
        [InlineData(0x0A)]
        [InlineData(0x0B)]
        [InlineData(0x0C)]
        [InlineData(0x0D)]
        [InlineData(0x0E)]
        [InlineData(0x0F)]
        public void A_reserved_coprocessor_zero_sub_opcode_refuses(int rs)
        {
            var cpu = new MipsAssembler().Cop0SubOpcode(rs).Run(1);

            Assert.Equal(ExceptionCode.ReservedInstruction, cpu.LastException!.Code);
        }
    }
}
