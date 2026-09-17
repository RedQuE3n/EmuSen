using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The TLB's own registers and what an entry keeps of them, in the corpus's vectors - see Mars_Tlb.md §7.
    public class MarsTlbRegisterTests
    {
        private const ulong Kx = Cpu.StatusKernelExtendedAddressing;

        [Theory]
        [InlineData(Cpu.EntryLo0Register, 0x0000_0000_FFFF_0002UL, 0x0000_0000_3FFF_0002UL)]
        [InlineData(Cpu.EntryLo0Register, 0x1234_5678_FFFF_FFFFUL, 0x0000_0000_3FFF_FFFFUL)]
        [InlineData(Cpu.EntryLo1Register, 0xFFFF_FFFF_FFFF_FFFFUL, 0x0000_0000_3FFF_FFFFUL)]
        [InlineData(Cpu.PageMaskRegister, 0x0000_0000_FFFF_FFFFUL, 0x0000_0000_01FF_E000UL)]
        [InlineData(Cpu.PageMaskRegister, 0x0000_0000_0000_0001UL, 0x0000_0000_0000_0000UL)]
        [InlineData(Cpu.PageMaskRegister, 0x0000_0000_017F_C000UL, 0x0000_0000_017F_C000UL)]
        [InlineData(Cpu.EntryHiRegister, 0x1234_5678_FFFF_FFFFUL, 0x0000_0078_FFFF_E0FFUL)]
        [InlineData(Cpu.EntryHiRegister, 0xFFFF_FFFF_FFFF_FFFFUL, 0xC000_00FF_FFFF_E0FFUL)]
        public void A_tlb_register_keeps_only_the_bits_it_owns(int register, ulong written, ulong expected)
        {
            var cpu = Program(a => a.Dmtc0(1, register), c => c.Gpr[1] = written);

            Assert.Equal(expected, cpu.Cop0[register]);
        }

        // The register keeps a raw mask; an entry keeps it in pairs, each decided by its higher bit - see §7.2.
        [Theory]
        [InlineData(0b0000000011u << 13, 0b0000000011u << 13)]
        [InlineData(0b1111111111u << 13, 0b1111111111u << 13)]
        [InlineData(0b00000000001u << 13, 0b00000000000u << 13)]
        [InlineData(0b00000000111u << 13, 0b00000000011u << 13)]
        [InlineData(0b00000000010u << 13, 0b00000000011u << 13)]
        [InlineData(0b00000000100u << 13, 0b00000000000u << 13)]
        [InlineData(0b00000100000u << 13, 0b00000110000u << 13)]
        [InlineData(0b00_11_00_01_10_10_01u << 13, 0b00_11_00_00_11_11_00u << 13)]
        [InlineData(0x017F_C000u, 0x01FF_E000u)]
        public void An_entry_keeps_its_page_mask_in_whole_pairs(uint written, uint expected)
        {
            var cpu = RoundTrip(written, 0, 0, 0);

            Assert.Equal((ulong)expected, cpu.Cop0[Cpu.PageMaskRegister]);
        }

        // Twenty frame bits and four flags survive, and the global flag is one bit for the pair - see §7.3.
        [Theory]
        [InlineData(0x3FFF_FFFFu, 0x3FFF_FFFFu, 0x03FF_FFFFu, 0x03FF_FFFFu)]
        [InlineData(0x3FFF_FFFFu, 0x0000_0000u, 0x03FF_FFFEu, 0x0000_0000u)]
        [InlineData(0x0000_0000u, 0x3FFF_FFFFu, 0x0000_0000u, 0x03FF_FFFEu)]
        public void An_entry_keeps_fewer_frame_bits_than_the_register_and_one_global_flag(
            uint lo0, uint lo1, uint expected0, uint expected1)
        {
            var cpu = RoundTrip(0, lo0, lo1, 0);

            Assert.Equal((ulong)expected0, cpu.Cop0[Cpu.EntryLo0Register]);
            Assert.Equal((ulong)expected1, cpu.Cop0[Cpu.EntryLo1Register]);
        }

        // The page number an entry keeps loses whatever bits its own page mask covers.
        [Theory]
        [InlineData(0u, 0x0000_0000_FFFF_E0FFUL, 0x0000_0000_FFFF_E0FFUL)]
        [InlineData(0u, 0xC000_00FF_FFFF_E0FFUL, 0xC000_00FF_FFFF_E0FFUL)]
        [InlineData(0b00_00_00_00_11u << 13, 0x0000_0000_FFFF_E0FFUL, 0x0000_0000_FFFF_80FFUL)]
        [InlineData(0b11_11_11_11_11u << 13, 0xC000_00FF_FFFF_E0FFUL, 0xC000_00FF_FF80_00FFUL)]
        [InlineData(0b11_11_11_10_01u << 13, 0x0000_0000_FFFF_E0FFUL, 0x0000_0000_FF80_60FFUL)]
        public void An_entry_keeps_only_the_page_number_its_mask_leaves(uint pageMask, ulong entryHi, ulong expected)
        {
            var cpu = RoundTrip(pageMask, 0, 0, entryHi);

            Assert.Equal(expected, cpu.Cop0[Cpu.EntryHiRegister]);
        }

        // Random counts down on its own, so a write to it has nowhere to go.
        [Fact]
        public void A_write_to_random_is_ignored()
        {
            var cpu = Program(a => a.Dmtc0(1, Cpu.RandomRegister).Dmfc0(2, Cpu.RandomRegister), c => c.Gpr[1] = 100);

            Assert.True(cpu.Gpr[2] < 64);
            Assert.NotEqual(100UL, cpu.Gpr[2]);
        }

        // Every value from Wired up to 31 comes round, and nothing below Wired ever does - see §7.4.
        [Theory]
        [InlineData(0)]
        [InlineData(20)]
        [InlineData(31)]
        public void Random_stays_between_wired_and_the_last_entry(int wired)
        {
            var seen = SampleRandom(wired);

            Assert.All(seen, value => Assert.InRange(value, (ulong)wired, 31UL));
            Assert.Equal(32 - wired, seen.Count);
        }

        // Above 31 the counter has nowhere to stop and runs the whole six-bit range.
        [Fact]
        public void Random_with_wired_above_the_last_entry_runs_the_whole_six_bit_range()
        {
            var seen = SampleRandom(40);

            Assert.Contains(seen, value => value < 10);
            Assert.Contains(seen, value => value > 54);
            Assert.All(seen, value => Assert.InRange(value, 0UL, 63UL));
        }

        // Bits 63:62 and the whole forty-bit page number take part in a match, not only the low word.
        [Theory]
        [InlineData(0x0000_0003_F000_0000UL, 0x0000_0001_F000_0000UL)]
        [InlineData(0x4000_0000_DEA0_0000UL, 0x0000_0000_DEA0_0000UL)]
        [InlineData(0xC000_00FF_2000_0000UL, 0x4000_00FF_2000_0000UL)]
        public void A_match_reads_the_region_bits_and_the_whole_page_number(ulong mapped, ulong missed)
        {
            var hit = Mapped(mapped, a => a.Lw(2, 1, 0), c => c.Gpr[1] = mapped);
            var miss = Mapped(mapped, a => a.Lw(2, 1, 0), c => c.Gpr[1] = missed);

            Assert.Null(hit.LastException);
            Assert.Equal(ExceptionCode.TlbLoad, miss.LastException!.Code);
        }

        [Fact]
        public void A_probe_reads_the_region_bits_too()
        {
            var cpu = Mapped(0x4000_0000_DEA0_0000UL, a => a.Dmtc0(1, Cpu.EntryHiRegister).Tlbp(),
                c => c.Gpr[1] = 0x0000_0000_DEA0_0000UL);

            Assert.Equal(0x8000_0000UL, cpu.Cop0[Cpu.IndexRegister]);
        }

        // A miss in 64-bit addressing has a refill door of its own, chosen by the mode that missed - see §7.5.
        [Fact]
        public void A_refill_under_sixty_four_bit_addressing_takes_the_extended_vector()
        {
            var cpu = Program(a => a.Lw(2, 1, 0), c =>
            {
                c.Cop0[Cpu.StatusRegister] = Kx;
                c.Gpr[1] = 0x0000_0000_8000_1000UL;
            });

            Assert.Equal(ExceptionCode.TlbLoad, cpu.LastException!.Code);
            Assert.Equal(0xFFFF_FFFF_8000_0080UL, cpu.Pc);
            Assert.Equal(0x0000_0000_8000_0000UL, cpu.Cop0[Cpu.EntryHiRegister]);
        }

        // Page zero is avoided: an empty entry matches it, which is an invalid entry and not a refill.
        [Fact]
        public void The_same_refill_without_it_takes_the_ordinary_one()
        {
            var cpu = Program(a => a.Lw(2, 1, 0), c => c.Gpr[1] = 0x0000_0000_0020_1000UL);

            Assert.Equal(0xFFFF_FFFF_8000_0000UL, cpu.Pc);
        }

        // The bit that counts is the one belonging to the mode that missed, read before the fault enters kernel.
        [Theory]
        [InlineData(1UL << 5, 0xFFFF_FFFF_8000_0080UL)]
        [InlineData(Kx, 0xFFFF_FFFF_8000_0000UL)]
        public void A_user_mode_refill_reads_the_user_bit_and_not_the_kernel_one(ulong status, ulong vector)
        {
            var cpu = PrivilegeFixture.Run(2, wide: false, 0x0000_0000_0020_1000UL, a => a.Lw(2, 1, 0), status: status);

            Assert.Equal(ExceptionCode.TlbLoad, cpu.LastException!.Code);
            Assert.Equal(vector, cpu.Pc);
        }

        // An address error fills in EntryHi exactly as a miss does, from the address it could not use - see §7.6.
        [Theory]
        [InlineData(0x0000_0100_0020_FFF4UL, 0x0000_0000_0020_E000UL)]
        [InlineData(0x4000_0100_0020_FFF4UL, 0x4000_0000_0020_E000UL)]
        [InlineData(0xC000_00FF_8000_0000UL, 0xC000_00FF_8000_0000UL)]
        public void An_address_error_fills_in_entry_hi_as_a_miss_would(ulong address, ulong expected)
        {
            var cpu = Program(a => a.Lw(2, 1, 0), c =>
            {
                c.Cop0[Cpu.StatusRegister] = Kx;
                c.Gpr[1] = address;
            });

            Assert.Equal(ExceptionCode.AddressErrorLoad, cpu.LastException!.Code);
            Assert.Equal(expected, cpu.Cop0[Cpu.EntryHiRegister]);
        }

        // Written through the register, then read back through a second write that TLBR has to overwrite.
        private static Cpu RoundTrip(uint pageMask, uint lo0, uint lo1, ulong entryHi) =>
            Program(a => a
                .Dmtc0(1, Cpu.PageMaskRegister)
                .Dmtc0(2, Cpu.EntryLo0Register)
                .Dmtc0(3, Cpu.EntryLo1Register)
                .Dmtc0(4, Cpu.EntryHiRegister)
                .Dmtc0(0, Cpu.IndexRegister)
                .Tlbwi()
                .Dmtc0(5, Cpu.EntryLo0Register)
                .Dmtc0(5, Cpu.EntryLo1Register)
                .Dmtc0(5, Cpu.EntryHiRegister)
                .Tlbr(), c =>
            {
                c.Gpr[1] = pageMask;
                c.Gpr[2] = lo0;
                c.Gpr[3] = lo1;
                c.Gpr[4] = entryHi;
                c.Gpr[5] = 0xFFFF_FFFF_FFFF_FFFF;
            });

        // One 16K pair at the given page, valid and global, reached with 64-bit kernel addressing.
        private static Cpu Mapped(ulong page, Func<MipsAssembler, MipsAssembler> program, Action<Cpu> before) =>
            Program(program, c =>
            {
                c.Cop0[Cpu.StatusRegister] = Kx;
                c.Tlb.Entries[3] = new TlbEntry
                {
                    EntryHi = page,
                    PageMask = 0x6000,
                    EntryLo0 = ((0x40000UL >> 12) << 6) | Tlb.EntryLoValid | Tlb.EntryLoGlobal,
                    EntryLo1 = Tlb.EntryLoGlobal,
                };
                before(c);
            });

        // The instruction after a write to Wired, then each one after that, until the counter comes round twice.
        private static HashSet<ulong> SampleRandom(int wired)
        {
            var seen = new HashSet<ulong>();

            for (int delay = 0; delay < 200; delay++)
            {
                var cpu = Program(a =>
                {
                    a.Dmtc0(1, Cpu.WiredRegister);
                    for (int i = 0; i < delay; i++) a.Nop();
                    return a.Dmfc0(2, Cpu.RandomRegister);
                }, c => c.Gpr[1] = (ulong)wired);

                seen.Add(cpu.Gpr[2]);
            }

            return seen;
        }

        private static Cpu Program(Func<MipsAssembler, MipsAssembler> program, Action<Cpu> before)
        {
            var assembler = program(new MipsAssembler());
            var cpu = assembler.Build();

            before(cpu);
            cpu.Run(assembler.ToArray().Length);

            return cpu;
        }
    }
}
