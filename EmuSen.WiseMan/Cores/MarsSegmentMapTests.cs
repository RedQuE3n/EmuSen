using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The whole virtual address map, as a function of the mode reading it - see Mars_Privilege.md §2.
    public class MarsSegmentMapTests
    {
        private const int Kernel = 0;
        private const int Supervisor = 1;
        private const int User = 2;

        // The corpus's own forty-five rows, its names kept: Sys means the access went through.
        [Theory]
        [InlineData("k32_kuseg_map", Kernel, false, 0x0000_0000_0000_1000UL, "TLBL")]
        [InlineData("k32_kseg0_c", Kernel, false, 0xFFFF_FFFF_8000_1000UL, "Sys")]
        [InlineData("k32_kseg1_d", Kernel, false, 0xFFFF_FFFF_A000_1000UL, "Sys")]
        [InlineData("k32_ksseg_map", Kernel, false, 0xFFFF_FFFF_C000_1000UL, "TLBL")]
        [InlineData("k32_kseg3_map", Kernel, false, 0xFFFF_FFFF_E000_1000UL, "TLBL")]
        [InlineData("s32_suseg_map", Supervisor, false, 0x0000_0000_0000_1000UL, "TLBL")]
        [InlineData("s32_low_unused", Supervisor, false, 0xFFFF_FFFF_9000_1000UL, "AdEL")]
        [InlineData("s32_sseg_map", Supervisor, false, 0xFFFF_FFFF_C000_1000UL, "TLBL")]
        [InlineData("s32_top_unused", Supervisor, false, 0xFFFF_FFFF_E000_1000UL, "AdEL")]
        [InlineData("u32_useg_map", User, false, 0x0000_0000_0000_1000UL, "TLBL")]
        [InlineData("u32_unused", User, false, 0xFFFF_FFFF_9000_1000UL, "AdEL")]
        [InlineData("k64_xkuseg_map", Kernel, true, 0x0000_0000_0000_1000UL, "TLBL")]
        [InlineData("k64_xkuseg_gap", Kernel, true, 0x0000_0100_0000_0000UL, "AdEL")]
        [InlineData("k64_xksseg_map", Kernel, true, 0x4000_0000_0000_1000UL, "TLBL")]
        [InlineData("k64_xksseg_gap", Kernel, true, 0x4000_0100_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys0_c32", Kernel, true, 0x8000_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys0_gap", Kernel, true, 0x8000_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys1_c32", Kernel, true, 0x8800_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys1_gap", Kernel, true, 0x8800_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys2_d32", Kernel, true, 0x9000_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys2_gap", Kernel, true, 0x9000_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys3_c32", Kernel, true, 0x9800_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys3_gap", Kernel, true, 0x9800_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys4_c32", Kernel, true, 0xA000_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys4_gap", Kernel, true, 0xA000_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys5_c32", Kernel, true, 0xA800_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys5_gap", Kernel, true, 0xA800_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys6_c32", Kernel, true, 0xB000_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys6_gap", Kernel, true, 0xB000_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkphys7_c32", Kernel, true, 0xB800_0000_0000_1000UL, "Sys")]
        [InlineData("k64_xkphys7_gap", Kernel, true, 0xB800_0001_0000_0000UL, "AdEL")]
        [InlineData("k64_xkseg_map", Kernel, true, 0xC000_0000_0000_1000UL, "TLBL")]
        [InlineData("k64_xkseg_gap", Kernel, true, 0xC000_0100_0000_0000UL, "AdEL")]
        [InlineData("k64_xkseg_top", Kernel, true, 0xC000_00FF_7FFF_FFFCUL, "TLBL")]
        [InlineData("k64_xkseg_short", Kernel, true, 0xC000_00FF_8000_0000UL, "AdEL")]
        [InlineData("k64_xkseg_short_high", Kernel, true, 0xC000_00FF_F000_0000UL, "AdEL")]
        [InlineData("k64_ckseg0_c", Kernel, true, 0xFFFF_FFFF_8000_1000UL, "Sys")]
        [InlineData("k64_ckseg1_d", Kernel, true, 0xFFFF_FFFF_A000_1000UL, "Sys")]
        [InlineData("k64_ckseg2_map", Kernel, true, 0xFFFF_FFFF_C000_1000UL, "TLBL")]
        [InlineData("k64_ckseg3_map", Kernel, true, 0xFFFF_FFFF_E000_1000UL, "TLBL")]
        [InlineData("s64_xsuseg_map", Supervisor, true, 0x0000_0000_0000_1000UL, "TLBL")]
        [InlineData("s64_xsuseg_gap", Supervisor, true, 0x0000_0100_0000_0000UL, "AdEL")]
        [InlineData("s64_xsseg_map", Supervisor, true, 0x4000_0000_0000_1000UL, "TLBL")]
        [InlineData("s64_xsseg_gap", Supervisor, true, 0x4000_0100_0000_0000UL, "AdEL")]
        [InlineData("s64_csseg_map", Supervisor, true, 0xFFFF_FFFF_C000_1000UL, "TLBL")]
        [InlineData("s64_top_unused", Supervisor, true, 0xFFFF_FFFF_E000_1000UL, "AdEL")]
        [InlineData("u64_xuseg_map", User, true, 0x0000_0000_0000_1000UL, "TLBL")]
        [InlineData("u64_xuseg_gap", User, true, 0x0000_0100_0000_0000UL, "AdEL")]
        public void A_load_lands_where_the_mode_says_it_can(
            string name, int mode, bool wide, ulong address, string outcome)
        {
            var cpu = PrivilegeFixture.Load(mode, wide, address);

            Assert.Equal($"{name}: {outcome}", $"{name}: {PrivilegeFixture.Describe(cpu)}");
        }

        // A store follows the same map and names itself the other way round when it is refused.
        [Theory]
        [InlineData(User, false, 0xFFFF_FFFF_9000_1000UL)]
        [InlineData(Supervisor, false, 0xFFFF_FFFF_E000_1000UL)]
        [InlineData(Kernel, true, 0x0000_0100_0000_0000UL)]
        public void A_store_outside_the_mode_is_an_address_error_of_its_own_kind(
            int mode, bool wide, ulong address)
        {
            var cpu = PrivilegeFixture.Store(mode, wide, address);

            Assert.Equal(ExceptionCode.AddressErrorStore, cpu.LastException!.Code);
        }

        // The direct segments still reach the same physical word from every name they have.
        [Theory]
        [InlineData(0xFFFF_FFFF_8000_1000UL)]
        [InlineData(0xFFFF_FFFF_A000_1000UL)]
        [InlineData(0x8000_0000_0000_1000UL)]
        [InlineData(0x9000_0000_0000_1000UL)]
        [InlineData(0xB800_0000_0000_1000UL)]
        public void Every_direct_segment_strips_to_the_same_physical_address(ulong address)
        {
            var bus = new MemoryBus();
            bus.Write32(0x1000, 0xC0FFEE00);

            var cpu = PrivilegeFixture.Load(Kernel, wide: true, address, bus);

            Assert.Null(cpu.LastException);
            Assert.Equal(0xFFFF_FFFF_C0FF_EE00UL, cpu.Gpr[2]);
        }
    }
}
