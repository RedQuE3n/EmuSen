using EmuSen.Cores.Nintendo.Mars.Cpu.Core;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The mapped segments, and the three different ways a lookup can fail - see Mars_Tlb.md.
    public class MarsTlbTests
    {
        private const ulong MappedPage = 0x0000_0000_0010_0000;
        private const uint Frame = 0x2000;

        private static Cpu Machine(MarsBus? bus = null)
        {
            var cpu = new MipsAssembler().Nop().Run(0, bus ?? new MarsBus());
            cpu.Pc = MipsAssembler.EntryPoint;
            cpu.NextPc = cpu.Pc + 4;
            return cpu;
        }

        // One 4K pair mapping the test page, valid and writable on both halves.
        private static void MapTestPage(Cpu cpu, bool valid = true, bool writable = true, bool global = true, ulong asid = 0)
        {
            ulong flags = (valid ? Tlb.EntryLoValid : 0) | (writable ? Tlb.EntryLoDirty : 0) | (global ? Tlb.EntryLoGlobal : 0);

            cpu.Tlb.Entries[4] = new TlbEntry
            {
                EntryHi = MappedPage | asid,
                PageMask = 0,
                EntryLo0 = ((ulong)(Frame >> 12) << 6) | flags,
                EntryLo1 = ((ulong)((Frame + 0x1000) >> 12) << 6) | flags,
            };
        }

        // A pair of 4K pages, whose halves are chosen by bit 12 and may name any two frames - see Mars_Tlb.md §1.1.
        [Fact]
        public void The_two_halves_of_a_pair_are_four_kilobytes_apart_and_independent()
        {
            var bus = new MarsBus();
            bus.Write32(0x5000 + 0x20, 0x0DD0_0DD0);
            bus.Write32(0x9000 + 0x20, 0xBADF00D5);

            var cpu = Machine(bus);
            MapPair(cpu, 0, MappedPage, even: 0x9000, odd: 0x5000);

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0x1020).LoadInto(bus);
            cpu.Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x0000_0000_0DD0_0DD0UL, cpu.Gpr[2]);
        }

        // Each pair owns eight kilobytes and no more, so the next pair up is a different entry.
        [Fact]
        public void A_pair_does_not_reach_into_the_pair_above_it()
        {
            var bus = new MarsBus();
            bus.Write32(0x7000 + 0x20, 0xFEEDFACE);

            var cpu = Machine(bus);
            MapPair(cpu, 0, MappedPage, even: 0x3000, odd: 0x4000);
            MapPair(cpu, 1, MappedPage + 0x2000, even: 0x7000, odd: 0x8000);

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0x2020).LoadInto(bus);
            cpu.Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(0xFFFF_FFFF_FEED_FACEUL, cpu.Gpr[2]);
        }

        private static void MapPair(Cpu cpu, int index, ulong page, uint even, uint odd)
        {
            const ulong Usable = Tlb.EntryLoValid | Tlb.EntryLoDirty | Tlb.EntryLoGlobal;

            cpu.Tlb.Entries[index] = new TlbEntry
            {
                EntryHi = page,
                PageMask = 0,
                EntryLo0 = ((ulong)(even >> 12) << 6) | Usable,
                EntryLo1 = ((ulong)(odd >> 12) << 6) | Usable,
            };
        }

        [Fact]
        public void A_mapped_page_translates_and_the_load_reaches_memory()
        {
            var bus = new MarsBus();
            bus.Write32(Frame + 0x20, 0xCAFEBABE);

            var cpu = Machine(bus);
            MapTestPage(cpu);

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0x20).LoadInto(bus);
            cpu.Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(0xFFFF_FFFF_CAFE_BABEUL, cpu.Gpr[2]);
        }

        // The odd half of the pair is a second page, chosen by the bit above the page offset.
        [Fact]
        public void The_second_half_of_the_pair_is_a_page_of_its_own()
        {
            var bus = new MarsBus();
            bus.Write32(Frame + 0x1000 + 0x20, 0x12345678);

            var cpu = Machine(bus);
            MapTestPage(cpu);

            new MipsAssembler().Lui(1, 0x0010).Ori(1, 1, 0x1000).Lw(2, 1, 0x20).LoadInto(bus);
            cpu.Run(3);

            Assert.Equal(0x1234_5678UL, cpu.Gpr[2]);
        }

        [Fact]
        public void An_address_with_no_entry_at_all_takes_the_refill_vector()
        {
            var cpu = Machine();
            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0).LoadInto(cpu.Bus);
            cpu.Run(2);

            Assert.Equal(ExceptionCode.TlbLoad, cpu.LastException!.Code);
            Assert.True(cpu.LastException.Refill);
            Assert.Equal(Cpu.VectorBase + Cpu.VectorOffsetTlbRefill, cpu.Pc);
        }

        // The distinction this slice added: an entry that exists but is unusable is not a miss.
        [Fact]
        public void An_entry_that_exists_but_is_invalid_takes_the_general_vector_instead()
        {
            var cpu = Machine();
            MapTestPage(cpu, valid: false);

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0).LoadInto(cpu.Bus);
            cpu.Run(2);

            Assert.Equal(ExceptionCode.TlbLoad, cpu.LastException!.Code);
            Assert.False(cpu.LastException.Refill);
            Assert.Equal(Cpu.VectorBase + Cpu.VectorOffsetGeneral, cpu.Pc);
        }

        [Fact]
        public void A_store_to_a_page_that_is_not_writable_is_its_own_exception()
        {
            var cpu = Machine();
            MapTestPage(cpu, writable: false);

            new MipsAssembler().Lui(1, 0x0010).Addiu(2, 0, 5).Sw(2, 1, 0).LoadInto(cpu.Bus);
            cpu.Run(3);

            Assert.Equal(ExceptionCode.TlbModification, cpu.LastException!.Code);
            Assert.False(cpu.LastException.Refill);
        }

        [Fact]
        public void A_load_from_the_same_page_is_still_allowed()
        {
            var bus = new MarsBus();
            bus.Write32(Frame, 0x99);

            var cpu = Machine(bus);
            MapTestPage(cpu, writable: false);

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0).LoadInto(bus);
            cpu.Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x99UL, cpu.Gpr[2]);
        }

        [Fact]
        public void A_non_global_entry_only_matches_its_own_address_space()
        {
            var cpu = Machine();
            MapTestPage(cpu, global: false, asid: 0x11);
            cpu.Cop0[Cpu.EntryHiRegister] = 0x22;

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0).LoadInto(cpu.Bus);
            cpu.Run(2);

            Assert.True(cpu.LastException!.Refill);
        }

        [Fact]
        public void The_same_entry_matches_once_the_address_space_agrees()
        {
            var bus = new MarsBus();
            bus.Write32(Frame, 0x77);

            var cpu = Machine(bus);
            MapTestPage(cpu, global: false, asid: 0x11);
            cpu.Cop0[Cpu.EntryHiRegister] = 0x11;

            new MipsAssembler().Lui(1, 0x0010).Lw(2, 1, 0).LoadInto(bus);
            cpu.Run(2);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x77UL, cpu.Gpr[2]);
        }

        // What a refill handler reads instead of recomputing the faulting address itself.
        [Fact]
        public void A_miss_leaves_the_faulting_page_where_a_handler_expects_it()
        {
            var cpu = Machine();
            new MipsAssembler().Lui(1, 0x0010).Ori(1, 1, 0x0040).Lw(2, 1, 0).LoadInto(cpu.Bus);
            cpu.Run(3);

            Assert.Equal(MappedPage, cpu.Cop0[Cpu.EntryHiRegister] & 0xFFFF_E000UL);
            Assert.Equal((MappedPage >> 9) & 0x007F_FFF0UL, cpu.Cop0[Cpu.ContextRegister] & 0x007F_FFF0UL);
        }

        [Fact]
        public void An_entry_written_by_a_program_is_the_one_the_lookup_uses()
        {
            var bus = new MarsBus();
            bus.Write32(Frame, 0x4242);

            var cpu = Machine(bus);
            cpu.Cop0[Cpu.IndexRegister] = 7;
            cpu.Cop0[Cpu.EntryHiRegister] = MappedPage;
            cpu.Cop0[Cpu.PageMaskRegister] = 0;
            cpu.Cop0[Cpu.EntryLo0Register] = ((ulong)(Frame >> 12) << 6) | Tlb.EntryLoValid | Tlb.EntryLoGlobal;
            cpu.Cop0[Cpu.EntryLo1Register] = Tlb.EntryLoGlobal;

            new MipsAssembler().Tlbwi().Lui(1, 0x0010).Lw(2, 1, 0).LoadInto(bus);
            cpu.Run(3);

            Assert.Null(cpu.LastException);
            Assert.Equal(0x4242UL, cpu.Gpr[2]);
        }

        [Fact]
        public void A_probe_finds_a_written_entry_and_says_so_when_it_cannot()
        {
            var cpu = Machine();
            MapTestPage(cpu);

            cpu.Cop0[Cpu.EntryHiRegister] = MappedPage;
            new MipsAssembler().Tlbp().LoadInto(cpu.Bus);
            cpu.Step();

            Assert.Equal(4UL, cpu.Cop0[Cpu.IndexRegister]);

            var second = Machine();
            second.Cop0[Cpu.EntryHiRegister] = MappedPage;
            new MipsAssembler().Tlbp().LoadInto(second.Bus);
            second.Step();

            Assert.Equal(0x8000_0000UL, second.Cop0[Cpu.IndexRegister]);
        }

        [Fact]
        public void An_entry_can_be_read_back_out_again()
        {
            var cpu = Machine();
            MapTestPage(cpu);
            cpu.Cop0[Cpu.IndexRegister] = 4;

            new MipsAssembler().Tlbr().LoadInto(cpu.Bus);
            cpu.Step();

            Assert.Equal(MappedPage, cpu.Cop0[Cpu.EntryHiRegister]);
            Assert.Equal(Tlb.EntryLoValid | Tlb.EntryLoDirty | Tlb.EntryLoGlobal, cpu.Cop0[Cpu.EntryLo0Register] & 0x7);
        }

        [Fact]
        public void A_random_write_never_lands_on_a_wired_entry()
        {
            var cpu = Machine();
            cpu.Cop0[Cpu.WiredRegister] = 30;
            cpu.Cop0[Cpu.EntryHiRegister] = MappedPage;
            cpu.Cop0[Cpu.EntryLo0Register] = Tlb.EntryLoValid | Tlb.EntryLoGlobal;

            var program = new MipsAssembler();
            for (int i = 0; i < 8; i++) program.Tlbwr();
            program.LoadInto(cpu.Bus);
            cpu.Run(8);

            for (int i = 0; i < 30; i++) Assert.Equal(0UL, cpu.Tlb.Entries[i].EntryHi);
            Assert.True(cpu.Tlb.Entries[30].EntryHi == MappedPage || cpu.Tlb.Entries[31].EntryHi == MappedPage);
        }
    }
}
