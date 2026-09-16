using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Almost none of coprocessor zero is a plain word of storage - see Mars_Cop0.md §2.
    public class MarsCop0RegisterTests
    {
        // Each pair is a hardware measurement: write all ones, read back what the register keeps.
        [Theory]
        [InlineData(Cpu.IndexRegister, 0x8000_003FUL)]
        [InlineData(Cpu.WiredRegister, 0x0000_003FUL)]
        [InlineData(Cpu.ParityErrorRegister, 0x0000_00FFUL)]
        [InlineData(Cpu.CacheErrorRegister, 0x0000_0000UL)]
        [InlineData(Cpu.TagHiRegister, 0x0000_0000UL)]
        [InlineData(Cpu.TagLoRegister, 0xFFFF_FFFFUL)]
        [InlineData(Cpu.LinkedAddressRegister, 0xFFFF_FFFFUL)]
        public void A_register_written_with_all_ones_keeps_only_the_bits_it_owns(int register, ulong expected)
        {
            var cpu = Written(register, 0xFFFF_FFFF_FFFF_FFFF);

            Assert.Equal(expected, cpu.Cop0[register]);
        }

        [Fact]
        public void The_index_register_keeps_its_top_bit_mutable()
        {
            var cpu = Written(Cpu.IndexRegister, 0);

            Assert.Equal(0UL, cpu.Cop0[Cpu.IndexRegister]);
        }

        // Hardware owns the low bits of these two and software owns the rest, so a write is a merge.
        [Fact]
        public void A_write_to_context_leaves_the_field_the_hardware_fills_in()
        {
            var cpu = Prepared(c => c.Cop0[Cpu.ContextRegister] = 0x0000_0000_0055_5555,
                a => a.Dmtc0(1, Cpu.ContextRegister), 0xFFFF_FFFF_FFFF_FFFF);

            Assert.Equal(0xFFFF_FFFF_FFD5_5555UL, cpu.Cop0[Cpu.ContextRegister]);
        }

        [Fact]
        public void A_write_to_xcontext_leaves_the_field_the_hardware_fills_in()
        {
            var cpu = Prepared(c => c.Cop0[Cpu.XContextRegister] = 0x0000_0001_5555_5555,
                a => a.Dmtc0(1, Cpu.XContextRegister), 0xFFFF_FFFF_FFFF_FFFF);

            Assert.Equal(0xFFFF_FFFF_5555_5555UL, cpu.Cop0[Cpu.XContextRegister]);
        }

        [Fact]
        public void The_faulting_address_register_refuses_every_write()
        {
            var cpu = Prepared(c => c.Cop0[Cpu.BadVirtualAddressRegister] = 0x1234,
                a => a.Dmtc0(1, Cpu.BadVirtualAddressRegister), 0xFFFF_FFFF_FFFF_FFFF);

            Assert.Equal(0x1234UL, cpu.Cop0[Cpu.BadVirtualAddressRegister]);
        }

        [Fact]
        public void The_processor_identifier_is_constant()
        {
            var cpu = Written(Cpu.ProcessorIdRegister, 0xFFFF_FFFF);

            Assert.Equal((ulong)Cpu.ProcessorId, cpu.Cop0[Cpu.ProcessorIdRegister]);
        }

        // Half constant and half writable, which is why a read-modify-write of it needs a reset value.
        [Fact]
        public void The_configuration_register_keeps_its_writable_half_and_forces_the_rest()
        {
            var cpu = Written(Cpu.ConfigRegister, 0xFFFF_FFFF);

            Assert.Equal(Cpu.ConfigWritable | Cpu.ConfigConstant, cpu.Cop0[Cpu.ConfigRegister]);
        }

        [Fact]
        public void The_status_register_refuses_bit_nineteen_and_the_whole_upper_word()
        {
            var cpu = Written(Cpu.StatusRegister, 0xFFFF_FFFF_FFFF_FFFF);

            Assert.Equal(0xFFF7_FFFFUL, cpu.Cop0[Cpu.StatusRegister]);
        }

        [Theory]
        [InlineData(Cpu.ExceptionPcRegister)]
        [InlineData(Cpu.ErrorExceptionPcRegister)]
        public void The_return_address_registers_are_not_masked_at_all(int register)
        {
            var cpu = Written(register, 0xF765_4321_89AB_CDEF);

            Assert.Equal(0xF765_4321_89AB_CDEFUL, cpu.Cop0[register]);
        }

        // A 32-bit move carries all sixty-four bits of its source; the destination decides - see §2.1.
        [Fact]
        public void A_thirty_two_bit_move_still_delivers_the_whole_source_register()
        {
            var cpu = Prepared(_ => { }, a => a.Mtc0(1, Cpu.ExceptionPcRegister), 0xF765_4321_89AB_CDEF);

            Assert.Equal(0xF765_4321_89AB_CDEFUL, cpu.Cop0[Cpu.ExceptionPcRegister]);
        }

        // Not storage: they answer with whatever the last COP0 write put on the bus - see §5.
        [Theory]
        [InlineData(7)]
        [InlineData(21)]
        [InlineData(22)]
        [InlineData(23)]
        [InlineData(24)]
        [InlineData(25)]
        [InlineData(31)]
        public void An_unused_register_reads_back_the_last_write_to_any_register(int register)
        {
            var cpu = Prepared(_ => { }, a => a
                .Dmtc0(1, register)
                .Lui(2, 0x8BAD).Ori(2, 2, 0xF00D)
                .Dmtc0(2, Cpu.CompareRegister)
                .Dmfc0(3, register), 0x1317_1A1E, steps: 5);

            Assert.Equal(0xFFFF_FFFF_8BAD_F00DUL, cpu.Gpr[3]);
        }

        [Fact]
        public void An_unused_register_written_and_read_straight_back_returns_what_was_written()
        {
            var cpu = Prepared(_ => { }, a => a.Dmtc0(1, 7).Nop().Dmfc0(3, 7), 0x1317_1A1E, steps: 3);

            Assert.Equal(0x1317_1A1EUL, cpu.Gpr[3]);
        }

        private static Cpu Written(int register, ulong value) =>
            Prepared(_ => { }, a => a.Dmtc0(1, register), value);

        private static Cpu Prepared(
            Action<Cpu> before, Func<MipsAssembler, MipsAssembler> program, ulong source, int steps = 1)
        {
            var cpu = program(new MipsAssembler()).Build();

            cpu.Gpr[1] = source;
            before(cpu);
            cpu.Run(steps);

            return cpu;
        }
    }
}
