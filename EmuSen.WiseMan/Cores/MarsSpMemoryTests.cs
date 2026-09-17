using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The main CPU reaching the signal processor's memories, in the corpus's own vectors - see Mars_Memory.md §2.4.
    public class MarsSpMemoryTests
    {
        private const uint Uncached = 0xA400;

        // A byte store latches the whole register shifted to the byte's lane, zeroes and higher bytes included - see §2.4.
        [Fact]
        public void A_byte_store_writes_the_whole_word_with_the_register_shifted_into_place()
        {
            var bus = new MemoryBus();

            new MipsAssembler()
                .Lui(2, (ushort)Uncached)
                .Lui(3, 0x1234).Ori(3, 3, 0x5678)
                .Sb(3, 2, 0).Sb(3, 2, 5).Sb(3, 2, 10).Sb(3, 2, 15)
                .Run(7, bus);

            Assert.Equal(0x7800_0000u, bus.Read32(MemoryMap.SpDmemBase + 0));
            Assert.Equal(0x5678_0000u, bus.Read32(MemoryMap.SpDmemBase + 4));
            Assert.Equal(0x3456_7800u, bus.Read32(MemoryMap.SpDmemBase + 8));
            Assert.Equal(0x1234_5678u, bus.Read32(MemoryMap.SpDmemBase + 12));
        }

        [Fact]
        public void A_half_store_writes_the_whole_word_and_clears_what_it_did_not_name()
        {
            var bus = new MemoryBus();
            bus.Write32(MemoryMap.SpDmemBase + 0, 0xDEAD_BEEF);
            bus.Write32(MemoryMap.SpDmemBase + 4, 0xBADD_ECAF);

            new MipsAssembler()
                .Lui(2, (ushort)Uncached)
                .Lui(3, 0x1234).Ori(3, 3, 0x5678)
                .Sh(3, 2, 0).Sh(3, 2, 6)
                .Run(5, bus);

            Assert.Equal(0x5678_0000u, bus.Read32(MemoryMap.SpDmemBase + 0));
            Assert.Equal(0x1234_5678u, bus.Read32(MemoryMap.SpDmemBase + 4));
        }

        // A doubleword store reaches one word only, with the register's upper half - see §2.4.
        [Fact]
        public void A_doubleword_store_writes_only_its_upper_half_to_one_word()
        {
            var bus = new MemoryBus();
            uint[] preset = { 0xDEAD_BEEF, 0xBADD_ECAF, 0xABAB_ABAB, 0xCDCD_CDCD, 0xDEDE_DEDE, 0xEFEF_EFEF };
            for (uint i = 0; i < preset.Length; i++) bus.Write32(MemoryMap.SpDmemBase + i * 4, preset[i]);

            new MipsAssembler()
                .Lui(2, (ushort)Uncached)
                .Lui(3, 0xABCD).Ori(3, 3, 0xEF98).Dsll32(3, 3, 0)
                .Lui(4, 0x7654).Ori(4, 4, 0x3210)
                .Or(3, 3, 4)
                .Sd(3, 2, 0)
                .Run(8, bus);

            Assert.Equal(0xABCD_EF98u, bus.Read32(MemoryMap.SpDmemBase + 0));
            for (uint i = 1; i < preset.Length; i++) Assert.Equal(preset[i], bus.Read32(MemoryMap.SpDmemBase + i * 4));
        }

        // The two memories repeat every eight kilobytes all the way to the interface registers - see §2.4.
        [Fact]
        public void Signal_processor_memory_repeats_up_to_its_registers()
        {
            var bus = new MemoryBus();

            bus.Write32(MemoryMap.SpDmemBase + 0x0000, 0x0123_4567);
            bus.Write32(MemoryMap.SpDmemBase + 0x1000, 0x89AB_CDEF);
            bus.Write32(MemoryMap.SpDmemBase + 0x3E000, 0x7654_3210);

            Assert.Equal(0x7654_3210u, bus.Read32(MemoryMap.SpDmemBase + 0x0000));
            Assert.Equal(0x89AB_CDEFu, bus.Read32(MemoryMap.SpDmemBase + 0x1000));
            Assert.Equal(0x7654_3210u, bus.Read32(MemoryMap.SpDmemBase + 0x3E000));
            Assert.Equal(0x89AB_CDEFu, bus.Read32(MemoryMap.SpDmemBase + 0x3F000));
        }

        // Loads were never the quirk: a byte or half load reads exactly the bytes it names - see §2.4.
        [Fact]
        public void Byte_and_half_loads_from_signal_processor_memory_read_only_their_bytes()
        {
            var bus = new MemoryBus();
            bus.Write32(MemoryMap.SpDmemBase + 0, 0x0123_4567);

            var cpu = new MipsAssembler()
                .Lui(2, (ushort)Uncached)
                .Lbu(3, 2, 1).Lbu(4, 2, 3).Lhu(5, 2, 2)
                .Run(4, bus);

            Assert.Equal(0x23UL, cpu.Gpr[3]);
            Assert.Equal(0x67UL, cpu.Gpr[4]);
            Assert.Equal(0x4567UL, cpu.Gpr[5]);
        }

        // The whole-word latch is the signal processor's; main memory still takes a byte as a byte.
        [Fact]
        public void A_byte_store_to_main_memory_still_touches_only_its_byte()
        {
            var bus = new MemoryBus();
            bus.Write32(0x900, 0xAABB_CCDD);

            new MipsAssembler()
                .Lui(2, 0x8000).Ori(2, 2, 0x0900)
                .Lui(3, 0x1234).Ori(3, 3, 0x5678)
                .Sb(3, 2, 1).Sh(3, 2, 2)
                .Run(6, bus);

            Assert.Equal(0xAA78_5678u, bus.Read32(0x900));
        }
    }
}
