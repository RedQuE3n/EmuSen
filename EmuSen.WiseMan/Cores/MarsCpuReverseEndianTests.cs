using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Status.RE, which mirrors every user-mode access inside its doubleword - see Mars_ReverseEndian.md.
    public class MarsCpuReverseEndianTests
    {
        private const int Kernel = 0;
        private const int User = 2;

        private const ulong ReverseEndian = 1UL << 25;

        // The corpus's own sixteen fixture bytes, whose values name their own offsets backwards.
        private static readonly byte[] Fixture =
        {
            0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01,
            0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09,
        };

        // The corpus chose these bytes so that under the mirror a byte load returns its own offset plus one.
        [Theory]
        [InlineData("lb", 0, 0x0000_0000_0000_0001UL)]
        [InlineData("lb", 3, 0x0000_0000_0000_0004UL)]
        [InlineData("lb", 7, 0x0000_0000_0000_0008UL)]
        [InlineData("lb", 9, 0x0000_0000_0000_000AUL)]
        [InlineData("lb", 15, 0x0000_0000_0000_0010UL)]
        [InlineData("lbu", 0, 0x0000_0000_0000_0001UL)]
        [InlineData("lh", 0, 0x0000_0000_0000_0201UL)]
        [InlineData("lh", 2, 0x0000_0000_0000_0403UL)]
        [InlineData("lh", 6, 0x0000_0000_0000_0807UL)]
        [InlineData("lh", 14, 0x0000_0000_0000_100FUL)]
        [InlineData("lhu", 0, 0x0000_0000_0000_0201UL)]
        [InlineData("lw", 0, 0x0000_0000_0403_0201UL)]
        [InlineData("lw", 4, 0x0000_0000_0807_0605UL)]
        [InlineData("lw", 12, 0x0000_0000_100F_0E0DUL)]
        [InlineData("lwu", 0, 0x0000_0000_0403_0201UL)]
        [InlineData("ld", 0, 0x0807_0605_0403_0201UL)]
        [InlineData("ld", 8, 0x100F_0E0D_0C0B_0A09UL)]
        public void An_aligned_load_reads_its_mirror_within_the_doubleword(
            string name, short offset, ulong expected)
        {
            var cpu = Access(User, ReverseEndian, a => Emit(a, name, offset));

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, cpu.Gpr[2]);
        }

        // The same loads with the bit clear, which is the control the mirror is measured against.
        [Theory]
        [InlineData("lb", 0, 0x0000_0000_0000_0008UL)]
        [InlineData("lh", 0, 0x0000_0000_0000_0807UL)]
        [InlineData("lw", 0, 0x0000_0000_0807_0605UL)]
        [InlineData("ld", 0, 0x0807_0605_0403_0201UL)]
        public void The_same_load_without_the_bit_reads_what_it_asked_for(
            string name, short offset, ulong expected)
        {
            var cpu = Access(User, 0, a => Emit(a, name, offset));

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        // The bit belongs to user mode alone, so the kernel reads straight through it - see §1.
        [Theory]
        [InlineData("lb", 0x0000_0000_0000_0008UL)]
        [InlineData("lw", 0x0000_0000_0807_0605UL)]
        public void Kernel_mode_ignores_the_bit_entirely(string name, ulong expected)
        {
            var cpu = Access(Kernel, ReverseEndian, a => Emit(a, name, 0));

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        [Theory]
        [InlineData("sb", 0, 0xAAAA_AAAA_AAAA_AA22UL)]
        [InlineData("sb", 4, 0xAAAA_AA22_AAAA_AAAAUL)]
        [InlineData("sh", 0, 0xAAAA_AAAA_AAAA_1122UL)]
        [InlineData("sh", 4, 0xAAAA_1122_AAAA_AAAAUL)]
        [InlineData("sw", 0, 0xAAAA_AAAA_EEFF_1122UL)]
        [InlineData("sw", 4, 0xEEFF_1122_AAAA_AAAAUL)]
        [InlineData("sd", 0, 0xAABB_CCDD_EEFF_1122UL)]
        public void An_aligned_store_writes_to_its_mirror_within_the_doubleword(
            string name, short offset, ulong expected)
        {
            var bus = Cleared();
            var cpu = Access(User, ReverseEndian, a => Emit(a, name, offset), bus,
                c => c.Gpr[2] = 0xAABB_CCDD_EEFF_1122, StoreArea);

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, bus.Read64(StoreFrame));
        }

        // The corpus's own twenty-four rows: every partial access mirrors as a byte, not as its width.
        [Theory]
        [InlineData("lwl", 0, 0x0000_0000_01EF_0102UL)]
        [InlineData("lwl", 1, 0x0000_0000_0201_0102UL)]
        [InlineData("lwl", 2, 0x0000_0000_0302_0102UL)]
        [InlineData("lwl", 3, 0x0000_0000_0403_0201UL)]
        [InlineData("lwr", 0, 0x0000_0000_0403_0201UL)]
        [InlineData("lwr", 1, 0x0000_0000_0004_0302UL)]
        [InlineData("lwr", 2, 0x0000_0000_00EF_0403UL)]
        [InlineData("lwr", 3, 0x0000_0000_00EF_0104UL)]
        public void A_partial_word_load_mirrors_as_a_byte(string name, short offset, ulong expected)
        {
            var cpu = Access(User, ReverseEndian, a => Emit(a, name, offset), null,
                c => c.Gpr[2] = 0x0000_0000_00EF_0102);

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        [Theory]
        [InlineData("ldl", 0, 0x0122_3344_5566_7788UL)]
        [InlineData("ldl", 1, 0x0201_3344_5566_7788UL)]
        [InlineData("ldl", 2, 0x0302_0144_5566_7788UL)]
        [InlineData("ldl", 3, 0x0403_0201_5566_7788UL)]
        [InlineData("ldl", 4, 0x0504_0302_0166_7788UL)]
        [InlineData("ldl", 5, 0x0605_0403_0201_7788UL)]
        [InlineData("ldl", 6, 0x0706_0504_0302_0188UL)]
        [InlineData("ldl", 7, 0x0807_0605_0403_0201UL)]
        [InlineData("ldr", 0, 0x0807_0605_0403_0201UL)]
        [InlineData("ldr", 1, 0x1108_0706_0504_0302UL)]
        [InlineData("ldr", 2, 0x1122_0807_0605_0403UL)]
        [InlineData("ldr", 3, 0x1122_3308_0706_0504UL)]
        [InlineData("ldr", 4, 0x1122_3344_0807_0605UL)]
        [InlineData("ldr", 5, 0x1122_3344_5508_0706UL)]
        [InlineData("ldr", 6, 0x1122_3344_5566_0807UL)]
        [InlineData("ldr", 7, 0x1122_3344_5566_7708UL)]
        public void A_partial_doubleword_load_mirrors_as_a_byte(string name, short offset, ulong expected)
        {
            var cpu = Access(User, ReverseEndian, a => Emit(a, name, offset), null,
                c => c.Gpr[2] = 0x1122_3344_5566_7788);

            Assert.Equal(expected, cpu.Gpr[2]);
        }

        // The corpus's store tables, read back as the doubleword they land in.
        [Theory]
        [InlineData("swl", 0, 0xAAAA_AAAA_AAAA_AA50UL)]
        [InlineData("swl", 1, 0xAAAA_AAAA_AAAA_5060UL)]
        [InlineData("swl", 2, 0xAAAA_AAAA_AA50_6070UL)]
        [InlineData("swl", 3, 0xAAAA_AAAA_5060_7080UL)]
        [InlineData("swr", 0, 0xAAAA_AAAA_5060_7080UL)]
        [InlineData("swr", 1, 0xAAAA_AAAA_6070_80AAUL)]
        [InlineData("swr", 2, 0xAAAA_AAAA_7080_AAAAUL)]
        [InlineData("swr", 3, 0xAAAA_AAAA_80AA_AAAAUL)]
        public void A_partial_word_store_mirrors_as_a_byte(string name, short offset, ulong expected)
        {
            var bus = Cleared();
            var cpu = Access(User, ReverseEndian, a => Emit(a, name, offset), bus,
                c => c.Gpr[2] = 0x0000_0000_5060_7080, StoreArea);

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, bus.Read64(StoreFrame));
        }

        [Theory]
        [InlineData("sdl", 0, 0xAAAA_AAAA_AAAA_AA10UL)]
        [InlineData("sdl", 3, 0xAAAA_AAAA_1020_3040UL)]
        [InlineData("sdl", 7, 0x1020_3040_5060_7080UL)]
        [InlineData("sdr", 0, 0x1020_3040_5060_7080UL)]
        [InlineData("sdr", 4, 0x5060_7080_AAAA_AAAAUL)]
        [InlineData("sdr", 7, 0x80AA_AAAA_AAAA_AAAAUL)]
        public void A_partial_doubleword_store_mirrors_as_a_byte(string name, short offset, ulong expected)
        {
            var bus = Cleared();
            var cpu = Access(User, ReverseEndian, a => Emit(a, name, offset), bus,
                c => c.Gpr[2] = 0x1020_3040_5060_7080, StoreArea);

            Assert.Null(cpu.LastException);
            Assert.Equal(expected, bus.Read64(StoreFrame));
        }

        // Instruction fetch is mirrored too, which is why the corpus writes its programs in swapped pairs.
        [Fact]
        public void An_instruction_fetch_is_mirrored_like_any_other_word()
        {
            var bus = new MemoryBus();
            var cpu = PrivilegeFixture.Run(User, wide: false, 0,
                a => a.Addiu(2, 0, 0x1111).Addiu(2, 0, 0x2222), bus, ReverseEndian);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x2222UL, cpu.Gpr[2]);
        }

        // The one place the mirror does not reach: the address a fault reports is the one asked for.
        [Fact]
        public void A_fault_reports_the_address_before_the_mirror()
        {
            var cpu = Access(User, ReverseEndian, a => Emit(a, "lb", 0), address: 0x0000_0000_0002_0000);

            Assert.Equal(ExceptionCode.TlbLoad, cpu.LastException!.Code);
            Assert.Equal(0x0000_0000_0002_0000UL, cpu.Cop0[Cpu.BadVirtualAddressRegister]);
        }

        // Alignment is judged on the address the program named, not on the mirrored one - see §3.
        [Fact]
        public void Alignment_is_judged_before_the_mirror()
        {
            var aligned = Access(User, ReverseEndian, a => Emit(a, "lw", 4));
            var misaligned = Access(User, ReverseEndian, a => Emit(a, "lw", 2));

            Assert.Null(aligned.LastException);
            Assert.Equal(ExceptionCode.AddressErrorLoad, misaligned.LastException!.Code);
        }

        // A store area away from the load fixture, so a store never has to be read back through it.
        private const ulong StoreArea = PrivilegeFixture.DataPage + 0x100;

        private const uint StoreFrame = PrivilegeFixture.DataFrame + 0x100;

        private static MemoryBus Cleared()
        {
            var bus = new MemoryBus();

            bus.Write64(StoreFrame, 0xAAAA_AAAA_AAAA_AAAA);
            bus.Write64(StoreFrame + 8, 0xAAAA_AAAA_AAAA_AAAA);
            return bus;
        }

        // The instruction is written into both halves of the fetch pair, so the mirror finds it either way.
        private static Cpu Access(
            int mode, ulong status, System.Func<MipsAssembler, MipsAssembler> program,
            MemoryBus? bus = null, System.Action<Cpu>? before = null, ulong address = 0)
        {
            bus ??= new MemoryBus();
            for (int i = 0; i < Fixture.Length; i++) bus.Write8(PrivilegeFixture.DataFrame + (uint)i, Fixture[i]);

            return PrivilegeFixture.Run(
                mode, wide: true, address == 0 ? PrivilegeFixture.DataPage : address,
                a => program(program(a)), bus, status, before);
        }

        private static MipsAssembler Emit(MipsAssembler a, string name, short offset) => name switch
        {
            "lb" => a.Lb(2, 1, offset),
            "lbu" => a.Lbu(2, 1, offset),
            "lh" => a.Lh(2, 1, offset),
            "lhu" => a.Lhu(2, 1, offset),
            "lw" => a.Lw(2, 1, offset),
            "lwu" => a.Lwu(2, 1, offset),
            "ld" => a.Ld(2, 1, offset),
            "sb" => a.Sb(2, 1, offset),
            "sh" => a.Sh(2, 1, offset),
            "sw" => a.Sw(2, 1, offset),
            "sd" => a.Sd(2, 1, offset),
            "lwl" => a.Lwl(2, 1, offset),
            "lwr" => a.Lwr(2, 1, offset),
            "ldl" => a.Ldl(2, 1, offset),
            "ldr" => a.Ldr(2, 1, offset),
            "swl" => a.Swl(2, 1, offset),
            "swr" => a.Swr(2, 1, offset),
            "sdl" => a.Sdl(2, 1, offset),
            _ => a.Sdr(2, 1, offset),
        };
    }
}
