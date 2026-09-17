using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The merging loads and stores, whose expectations are derived from the architecture - see Mars_Cpu.md §7.
    public class MarsCpuUnalignedTests
    {
        private const uint Data = 0x800;

        // 0xAAAAAAAA in the register beforehand, so every byte the instruction leaves alone is visible.
        private static MipsAssembler Preloaded() =>
            new MipsAssembler().Lui(1, 0x8000).Ori(1, 1, (ushort)Data).Lui(2, 0xAAAA).Ori(2, 2, 0xAAAA);

        private static MarsBus WithWord()
        {
            var bus = new MarsBus();
            bus.Write32(Data, 0x01020304);
            return bus;
        }

        private static MarsBus WithDoubleWord()
        {
            var bus = new MarsBus();
            bus.Write64(Data, 0x0102030405060708UL);
            return bus;
        }

        // A doubleword test needs the pattern in both halves, which lui cannot put there - see §7.2.
        private static MipsAssembler PreloadedWide(MarsBus bus)
        {
            bus.Write64(Data + 0x10, 0xAAAAAAAAAAAAAAAAUL);
            return new MipsAssembler().Lui(1, 0x8000).Ori(1, 1, (ushort)Data).Ld(2, 1, 0x10);
        }

        [Theory]
        [InlineData(0, 0x0000_0000_0102_0304UL)]
        [InlineData(1, 0x0000_0000_0203_04AAUL)]
        [InlineData(2, 0x0000_0000_0304_AAAAUL)]
        [InlineData(3, 0x0000_0000_04AA_AAAAUL)]
        public void A_left_load_fills_from_the_addressed_byte_upwards(short offset, ulong expected)
        {
            var cpu = Preloaded().Lwl(2, 1, offset).Run(5, WithWord());

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        [Theory]
        [InlineData(3, 0x0000_0000_0102_0304UL)]
        [InlineData(2, 0xFFFF_FFFF_AA01_0203UL)]
        [InlineData(1, 0xFFFF_FFFF_AAAA_0102UL)]
        [InlineData(0, 0xFFFF_FFFF_AAAA_AA01UL)]
        public void A_right_load_fills_from_the_addressed_byte_downwards(short offset, ulong expected)
        {
            var cpu = Preloaded().Lwr(2, 1, offset).Run(5, WithWord());

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        // The rows above cannot tell preservation from sign extension; these corpus vectors can - see §7.3.
        [Theory]
        [InlineData(0, 0x0000_0000_0123_4567UL)]
        [InlineData(1, 0x0000_0000_2345_6710UL)]
        [InlineData(2, 0x0000_0000_4567_3210UL)]
        [InlineData(3, 0x0000_0000_6754_3210UL)]
        [InlineData(4, 0xFFFF_FFFF_89AB_CDEFUL)]
        [InlineData(5, 0xFFFF_FFFF_ABCD_EF10UL)]
        [InlineData(6, 0xFFFF_FFFF_CDEF_3210UL)]
        [InlineData(7, 0xFFFF_FFFF_EF54_3210UL)]
        public void A_left_load_sign_extends_however_little_of_the_word_it_took(short offset, ulong expected)
        {
            var cpu = Measured(a => a.Lwl(2, 1, offset), out _);

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        [Theory]
        [InlineData(0, 0xFEDC_BA98_7654_3201UL)]
        [InlineData(1, 0xFEDC_BA98_7654_0123UL)]
        [InlineData(2, 0xFEDC_BA98_7601_2345UL)]
        [InlineData(3, 0x0000_0000_0123_4567UL)]
        [InlineData(4, 0xFEDC_BA98_7654_3289UL)]
        [InlineData(5, 0xFEDC_BA98_7654_89ABUL)]
        [InlineData(6, 0xFEDC_BA98_7689_ABCDUL)]
        [InlineData(7, 0xFFFF_FFFF_89AB_CDEFUL)]
        public void A_right_load_sign_extends_only_when_it_took_the_whole_word(short offset, ulong expected)
        {
            var cpu = Measured(a => a.Lwr(2, 1, offset), out _);

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        // A register whose upper half is neither zero nor all ones, which is what makes the pair separable.
        private static Cpu Measured(Func<MipsAssembler, MipsAssembler> load, out MarsBus bus)
        {
            bus = new MarsBus();
            bus.Write64(Data, 0x0123_4567_89AB_CDEFUL);
            bus.Write64(Data + 0x10, 0xFEDC_BA98_7654_3210UL);

            var program = new MipsAssembler().Lui(1, 0x8000).Ori(1, 1, (ushort)Data).Ld(2, 1, 0x10);
            return load(program).Run(4, bus);
        }

        // The idiom the pair exists for: one unaligned word out of two aligned accesses.
        [Fact]
        public void The_pair_together_reads_a_word_that_straddles_the_boundary()
        {
            var bus = WithWord();
            bus.Write32(Data + 4, 0x05060708);

            var cpu = Preloaded().Lwl(2, 1, 1).Lwr(2, 1, 4).Run(6, bus);

            Assert.Equal(0x0000_0000_0203_0405UL, cpu.Gpr[2]);
        }

        [Fact]
        public void A_left_double_load_merges_with_the_register_it_finds()
        {
            var bus = WithDoubleWord();
            var cpu = PreloadedWide(bus).Ldl(2, 1, 2).Run(4, bus);

            Assert.Equal(0x0304_0506_0708_AAAAUL, cpu.Gpr[2]);
        }

        [Fact]
        public void A_right_double_load_merges_from_the_other_end()
        {
            var bus = WithDoubleWord();
            var cpu = PreloadedWide(bus).Ldr(2, 1, 5).Run(4, bus);

            Assert.Equal(0xAAAA_0102_0304_0506UL, cpu.Gpr[2]);
        }

        [Fact]
        public void The_double_pair_reads_a_straddling_doubleword()
        {
            var bus = WithDoubleWord();
            bus.Write64(Data + 8, 0x090A0B0C0D0E0F10UL);

            var cpu = Preloaded().Ldl(2, 1, 1).Ldr(2, 1, 8).Run(6, bus);

            Assert.Equal(0x0203_0405_0607_0809UL, cpu.Gpr[2]);
        }

        [Fact]
        public void A_left_store_writes_the_high_bytes_and_leaves_the_rest()
        {
            var bus = new MarsBus();
            bus.Write32(Data, 0x11223344);

            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, (ushort)Data)
                .Lui(2, 0x5566).Ori(2, 2, 0x7788)
                .Swl(2, 1, 1)
                .Run(5, bus);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x11556677u, bus.Read32(Data));
        }

        [Fact]
        public void A_right_store_writes_the_low_bytes_and_leaves_the_rest()
        {
            var bus = new MarsBus();
            bus.Write32(Data, 0x11223344);

            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, (ushort)Data)
                .Lui(2, 0x5566).Ori(2, 2, 0x7788)
                .Swr(2, 1, 2)
                .Run(5, bus);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x66778844u, bus.Read32(Data));
        }

        [Fact]
        public void The_store_pair_writes_a_word_across_the_boundary()
        {
            var bus = new MarsBus();
            bus.Write32(Data, 0x11223344);
            bus.Write32(Data + 4, 0x99AABBCC);

            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, (ushort)Data)
                .Lui(2, 0x5566).Ori(2, 2, 0x7788)
                .Swl(2, 1, 1).Swr(2, 1, 4)
                .Run(6, bus);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x11556677u, bus.Read32(Data));
            Assert.Equal(0x88AABBCCu, bus.Read32(Data + 4));
        }

        [Fact]
        public void A_double_store_merges_at_both_ends()
        {
            var bus = new MarsBus();
            bus.Write64(Data, 0x1122334455667788UL);

            var cpu = new MipsAssembler()
                .Lui(1, 0x8000).Ori(1, 1, (ushort)Data)
                .Addiu(2, 0, -1)
                .Sdl(2, 1, 6)
                .Run(4, bus);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x112233445566FFFFUL, bus.Read64(Data));
        }

        // The whole point of the family: these addresses are legal where an ordinary load would fault.
        [Fact]
        public void None_of_them_fault_on_an_address_an_ordinary_load_would_refuse()
        {
            var cpu = Preloaded().Lwl(2, 1, 1).Lwr(3, 1, 2).Ldl(4, 1, 3).Swl(2, 1, 1).Run(8, WithWord());

            Assert.Null(cpu.LastException);
        }
    }
}
