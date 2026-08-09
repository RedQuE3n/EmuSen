using System;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.Cores.Nintendo.Moon.Memory.Mappers;
using EmuSen.Cores.Nintendo.Moon.Video;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The boards added for real-game coverage - see Moon_Memory.md §4.6/§4.7.
    public class MoonMapperTests
    {
        // Distinct byte per 8K PRG page, so a read proves which bank is mapped.
        private static Cartridge Mmc3Cart(int prgBanks = 8, int chrBanks = 8)
        {
            byte[] image = SyntheticNesRom.Build(prgBanks, chrBanks, mapper: 4);
            var cart = Cartridge.FromImage(image);

            for (int page = 0; page < cart.PrgRom.Length / 0x2000; page++) cart.PrgRom[page * 0x2000] = (byte)(0xB0 + page);
            for (int page = 0; page < cart.Chr.Length / 0x0400; page++) cart.Chr[page * 0x0400] = (byte)(0xC0 + page);

            return cart;
        }

        // $8000 carries the mode bits too, so a bank-select write must keep them.
        private static void Select(IMapper m, int register, int value, int modeBits = 0)
        {
            m.WritePrg(0x8000, (byte)(modeBits | register));
            m.WritePrg(0x8001, (byte)value);
        }

        [Fact]
        public void Mmc3_maps_the_last_two_prg_banks_fixed_in_mode_0()
        {
            Cartridge cart = Mmc3Cart();
            int last = (cart.PrgRom.Length / 0x2000) - 1;

            cart.Mapper.WritePrg(0x8000, 0x00); // PRG mode 0
            Select(cart.Mapper, 6, 3);
            Select(cart.Mapper, 7, 5);

            Assert.Equal(0xB0 + 3, cart.Mapper.ReadPrg(0x8000));
            Assert.Equal(0xB0 + 5, cart.Mapper.ReadPrg(0xA000));
            Assert.Equal(0xB0 + last - 1, cart.Mapper.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, cart.Mapper.ReadPrg(0xE000));
        }

        // Mode 1 swaps which end R6 lives at, and $C000 becomes the switchable one.
        [Fact]
        public void Mmc3_prg_mode_1_moves_the_switchable_bank_to_C000()
        {
            Cartridge cart = Mmc3Cart();
            int last = (cart.PrgRom.Length / 0x2000) - 1;

            Select(cart.Mapper, 6, 3, modeBits: 0x40); // PRG mode 1

            Assert.Equal(0xB0 + last - 1, cart.Mapper.ReadPrg(0x8000));
            Assert.Equal(0xB0 + 3, cart.Mapper.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, cart.Mapper.ReadPrg(0xE000));
        }

        // R0/R1 address 2K pairs, so writing an odd number must not move the pair.
        [Fact]
        public void Mmc3_ignores_bit_zero_of_the_two_kilobyte_chr_registers()
        {
            Cartridge cart = Mmc3Cart();

            cart.Mapper.WritePrg(0x8000, 0x00);
            Select(cart.Mapper, 0, 5); // odd, so it addresses the pair starting at 4

            Assert.Equal(0xC0 + 4, cart.Mapper.ReadChr(0x0000));
            Assert.Equal(0xC0 + 5, cart.Mapper.ReadChr(0x0400));
        }

        [Fact]
        public void Mmc3_chr_mode_swaps_the_two_halves()
        {
            Cartridge cart = Mmc3Cart();

            cart.Mapper.WritePrg(0x8000, 0x00);
            Select(cart.Mapper, 0, 2); // 2K pair at pages 2,3
            Select(cart.Mapper, 2, 7); // 1K page 7

            Assert.Equal(0xC0 + 2, cart.Mapper.ReadChr(0x0000));
            Assert.Equal(0xC0 + 7, cart.Mapper.ReadChr(0x1000));

            cart.Mapper.WritePrg(0x8000, 0x80); // inversion on

            Assert.Equal(0xC0 + 7, cart.Mapper.ReadChr(0x0000));
            Assert.Equal(0xC0 + 2, cart.Mapper.ReadChr(0x1000));
        }

        [Fact]
        public void Mmc3_mirroring_follows_A000_unless_the_header_says_four_screen()
        {
            Cartridge cart = Mmc3Cart();

            cart.Mapper.WritePrg(0xA000, 0x00);
            Assert.Equal(Mirroring.Vertical, cart.Mapper.Mirroring);

            cart.Mapper.WritePrg(0xA000, 0x01);
            Assert.Equal(Mirroring.Horizontal, cart.Mapper.Mirroring);
        }

        // The counter reloads, counts down, and fires exactly on the latched line.
        [Fact]
        public void Mmc3_irq_fires_after_the_latched_number_of_scanlines()
        {
            Cartridge cart = Mmc3Cart();
            IMapper m = cart.Mapper;

            m.WritePrg(0xC000, 4);    // latch
            m.WritePrg(0xC001, 0);    // reload on next clock
            m.WritePrg(0xE001, 0);    // enable

            long clock = 0;
            // The gap is in PPU dots and has to clear the board's filter, which is
            // three CPU cycles - nine dots, not three. See Moon_Memory.md §4.6a.
            void A12Rise()
            {
                m.OnPpuAddress(0x0000, clock);
                clock += 12;
                m.OnPpuAddress(0x1000, clock);
                clock += 12;
            }

            A12Rise();                // reloads to 4
            Assert.False(m.IrqPending);

            // Four more rises to walk 4 down to 0.
            for (int i = 0; i < 3; i++)
            {
                A12Rise();
                Assert.False(m.IrqPending);
            }

            A12Rise();
            Assert.True(m.IrqPending);
        }

        [Fact]
        public void Mmc3_irq_stays_off_while_disabled_and_E000_acknowledges()
        {
            Cartridge cart = Mmc3Cart();
            IMapper m = cart.Mapper;

            long clock = 0;
            // The gap is in PPU dots and has to clear the board's filter, which is
            // three CPU cycles - nine dots, not three. See Moon_Memory.md §4.6a.
            void A12Rise()
            {
                m.OnPpuAddress(0x0000, clock);
                clock += 12;
                m.OnPpuAddress(0x1000, clock);
                clock += 12;
            }

            m.WritePrg(0xC000, 1);
            m.WritePrg(0xC001, 0);
            for (int i = 0; i < 4; i++) A12Rise();
            Assert.False(m.IrqPending); // never enabled

            m.WritePrg(0xE001, 0);
            A12Rise();
            A12Rise();
            Assert.True(m.IrqPending);

            m.WritePrg(0xE000, 0);
            Assert.False(m.IrqPending);
        }

        // The filter is the whole reason the sprite-fetch phase does not clock the
        // counter eight times a line. It is specified in CPU cycles, and this
        // counter is in PPU dots; measuring three *dots* instead of three cycles
        // made SMB3's title split fire six scanlines early - see Moon_Memory.md §4.6a.
        [Fact]
        public void Mmc3_ignores_an_a12_rise_that_was_not_low_for_three_cpu_cycles()
        {
            Cartridge cart = Mmc3Cart();
            IMapper m = cart.Mapper;

            m.WritePrg(0xC000, 1);
            m.WritePrg(0xC001, 0);
            m.WritePrg(0xE001, 0);

            long clock = 0;
            void Toggle(int lowDots)
            {
                m.OnPpuAddress(0x0000, clock);
                clock += lowDots;
                m.OnPpuAddress(0x1000, clock);
                clock += 2;
            }

            // Eight dots is the gap between two sprite pattern fetches; it must not count.
            for (int i = 0; i < 20; i++) Toggle(8);
            Assert.False(m.IrqPending);

            // Nine is three CPU cycles, which does.
            Toggle(9);
            Toggle(9);
            Assert.True(m.IrqPending);
        }

        // $A001 bit 7 gates the 8K work RAM MMC3 carries at $6000.
        [Fact]
        public void Mmc3_work_ram_can_be_disabled_and_write_protected()
        {
            Cartridge cart = Mmc3Cart();
            IMapper m = cart.Mapper;

            m.WritePrg(0xA001, 0x80); // enabled, writable
            m.WritePrg(0x6000, 0x42);
            Assert.Equal(0x42, m.ReadPrg(0x6000));

            m.WritePrg(0xA001, 0xC0); // enabled, write-protected
            m.WritePrg(0x6000, 0x99);
            Assert.Equal(0x42, m.ReadPrg(0x6000));

            m.WritePrg(0xA001, 0x00); // disabled
            Assert.Equal(0x00, m.ReadPrg(0x6000));
        }

        [Fact]
        public void GxRom_switches_a_32k_prg_bank_and_an_8k_chr_bank_from_one_write()
        {
            var cart = Cartridge.FromImage(SyntheticNesRom.Build(prgBanks: 8, chrBanks: 4, mapper: 66));
            for (int b = 0; b < 4; b++) cart.PrgRom[b * 0x8000] = (byte)(0xA0 + b);
            for (int b = 0; b < 4; b++) cart.Chr[b * 0x2000] = (byte)(0xD0 + b);

            cart.Mapper.WritePrg(0x8000, 0x21); // PRG bank 2, CHR bank 1

            Assert.Equal(0xA0 + 2, cart.Mapper.ReadPrg(0x8000));
            Assert.Equal(0xD0 + 1, cart.Mapper.ReadChr(0x0000));
        }

        // Color Dreams is GxROM's register with the fields the other way round.
        [Fact]
        public void ColorDreams_reads_the_bank_fields_the_opposite_way_to_GxRom()
        {
            var cart = Cartridge.FromImage(SyntheticNesRom.Build(prgBanks: 8, chrBanks: 4, mapper: 11));
            for (int b = 0; b < 4; b++) cart.PrgRom[b * 0x8000] = (byte)(0xA0 + b);
            for (int b = 0; b < 4; b++) cart.Chr[b * 0x2000] = (byte)(0xD0 + b);

            cart.Mapper.WritePrg(0x8000, 0x12); // PRG bank 2, CHR bank 1

            Assert.Equal(0xA0 + 2, cart.Mapper.ReadPrg(0x8000));
            Assert.Equal(0xD0 + 1, cart.Mapper.ReadChr(0x0000));
        }

        // Camerica banks from $C000 and leaves $C000-$FFFF fixed to the last bank.
        [Fact]
        public void Camerica_switches_only_from_C000_and_fixes_the_last_bank()
        {
            var cart = Cartridge.FromImage(SyntheticNesRom.Build(prgBanks: 4, chrBanks: 1, mapper: 71));
            for (int b = 0; b < 4; b++) cart.PrgRom[b * 0x4000] = (byte)(0xE0 + b);

            cart.Mapper.WritePrg(0x8000, 0x02); // ignored - not the bank register
            Assert.Equal(0xE0 + 0, cart.Mapper.ReadPrg(0x8000));

            cart.Mapper.WritePrg(0xC000, 0x02);
            Assert.Equal(0xE0 + 2, cart.Mapper.ReadPrg(0x8000));
            Assert.Equal(0xE0 + 3, cart.Mapper.ReadPrg(0xC000));
        }

        // NINA's register lives below the cartridge window entirely.
        [Fact]
        public void Nina003_takes_its_register_at_4100()
        {
            var cart = Cartridge.FromImage(SyntheticNesRom.Build(prgBanks: 4, chrBanks: 8, mapper: 79));
            for (int b = 0; b < 2; b++) cart.PrgRom[b * 0x8000] = (byte)(0xF0 + b);
            for (int b = 0; b < 8; b++) cart.Chr[b * 0x2000] = (byte)(0x90 + b);

            cart.Mapper.WritePrg(0x4100, 0x0B); // PRG bank 1, CHR bank 3

            Assert.Equal(0xF0 + 1, cart.Mapper.ReadPrg(0x8000));
            Assert.Equal(0x90 + 3, cart.Mapper.ReadChr(0x0000));
        }

        // Distinct byte per 8K PRG page and per 1K CHR page, so a read names the bank.
        private static Cartridge Board(int mapper, int prgBanks = 8, int chrBanks = 8)
        {
            var cart = Cartridge.FromImage(SyntheticNesRom.Build(prgBanks, chrBanks, mapper: mapper));

            for (int page = 0; page < cart.PrgRom.Length / 0x2000; page++) cart.PrgRom[page * 0x2000] = (byte)(0xB0 + page);
            for (int page = 0; page < cart.Chr.Length / 0x0400; page++) cart.Chr[page * 0x0400] = (byte)(0xC0 + page);

            return cart;
        }

        [Fact]
        public void Mmc2_switches_only_the_first_prg_window()
        {
            Cartridge cart = Board(9);
            IMapper m = cart.Mapper;
            int last = (cart.PrgRom.Length / 0x2000) - 1;

            m.WritePrg(0xA000, 0x05);

            Assert.Equal(0xB0 + 5, m.ReadPrg(0x8000));
            Assert.Equal(0xB0 + last - 2, m.ReadPrg(0xA000));
            Assert.Equal(0xB0 + last - 1, m.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, m.ReadPrg(0xE000));
        }

        // The fetch that trips the latch is still served by the bank on its way out.
        [Fact]
        public void Mmc2_latch_takes_effect_on_the_fetch_after_the_one_that_set_it()
        {
            Cartridge cart = Board(9, chrBanks: 4);
            IMapper m = cart.Mapper;

            // Markers inside the trigger tiles themselves, so the read that flips a latch is identifiable.
            for (int bank = 0; bank < 8; bank++)
            {
                cart.Chr[(bank * 0x1000) + 0x0FD8] = (byte)(0x50 + bank);
                cart.Chr[(bank * 0x1000) + 0x0FE8] = (byte)(0x60 + bank);
            }

            m.WritePrg(0xB000, 3); // $0000 window, $FD bank
            m.WritePrg(0xC000, 5); // $0000 window, $FE bank

            // Power-on latch is $FE, so bank 5 is live.
            Assert.Equal(0xC0 + (5 * 4), m.ReadChr(0x0000));

            Assert.Equal(0x50 + 5, m.ReadChr(0x0FD8));
            Assert.Equal(0xC0 + (3 * 4), m.ReadChr(0x0000));

            Assert.Equal(0x60 + 3, m.ReadChr(0x0FE8));
            Assert.Equal(0xC0 + (5 * 4), m.ReadChr(0x0000));
        }

        // The right window watches eight-byte runs where the left one watches two exact addresses.
        [Fact]
        public void Mmc2_right_window_latches_across_a_range()
        {
            Cartridge cart = Board(9, chrBanks: 4);
            IMapper m = cart.Mapper;

            m.WritePrg(0xD000, 2); // $1000 window, $FD bank
            m.WritePrg(0xE000, 6); // $1000 window, $FE bank

            Assert.Equal(0xC0 + (6 * 4), m.ReadChr(0x1000));

            m.ReadChr(0x1FDF);
            Assert.Equal(0xC0 + (2 * 4), m.ReadChr(0x1000));

            m.ReadChr(0x1FEF);
            Assert.Equal(0xC0 + (6 * 4), m.ReadChr(0x1000));

            // One past the run is an ordinary fetch and must leave the latch alone.
            m.ReadChr(0x1FE0);
            Assert.Equal(0xC0 + (6 * 4), m.ReadChr(0x1000));
        }

        [Fact]
        public void Rambo1_bit_6_moves_the_fixed_bank_and_keeps_three_switchable()
        {
            Cartridge cart = Board(64);
            IMapper m = cart.Mapper;
            int last = (cart.PrgRom.Length / 0x2000) - 1;

            Select(m, 6, 1);
            Select(m, 7, 2);
            Select(m, 15, 3);

            Assert.Equal(0xB0 + 1, m.ReadPrg(0x8000));
            Assert.Equal(0xB0 + 2, m.ReadPrg(0xA000));
            Assert.Equal(0xB0 + 3, m.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, m.ReadPrg(0xE000));

            m.WritePrg(0x8000, 0x40);

            Assert.Equal(0xB0 + 3, m.ReadPrg(0x8000));
            Assert.Equal(0xB0 + 1, m.ReadPrg(0xA000));
            Assert.Equal(0xB0 + 2, m.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, m.ReadPrg(0xE000));
        }

        // Bit 5 splits the two 2K pairs, and unlike MMC3 the pair is R0/R0+1, not R0 with bit 0 masked.
        [Fact]
        public void Rambo1_bit_5_gives_the_first_two_pairs_their_own_registers()
        {
            Cartridge cart = Board(64);
            IMapper m = cart.Mapper;

            Select(m, 0, 9);
            Select(m, 8, 20);

            Assert.Equal(0xC0 + 9, m.ReadChr(0x0000));
            Assert.Equal(0xC0 + 10, m.ReadChr(0x0400));

            m.WritePrg(0x8000, 0x20);
            Assert.Equal(0xC0 + 9, m.ReadChr(0x0000));
            Assert.Equal(0xC0 + 20, m.ReadChr(0x0400));
        }

        // The board's A12 filter is far wider than MMC3's, so an MMC3-legal gap must not clock it.
        [Fact]
        public void Rambo1_ignores_an_a12_rise_that_would_have_clocked_an_mmc3()
        {
            Cartridge cart = Board(64);
            IMapper m = cart.Mapper;

            m.WritePrg(0xC000, 1);
            m.WritePrg(0xC001, 0);
            m.WritePrg(0xE001, 0);

            long clock = 0;
            void Toggle(int lowDots)
            {
                m.OnPpuAddress(0x0000, clock);
                clock += lowDots;
                m.OnPpuAddress(0x1000, clock);
                clock += 2;
            }

            for (int i = 0; i < 20; i++) Toggle(Mmc3.A12MinimumLowDots);
            Assert.False(m.IrqPending);

            // Two clocks empty a latch of 1, but the line only falls two CPU cycles later.
            for (int i = 0; i < 2; i++) Toggle(Rambo1.A12MinimumLowDots);
            Assert.False(m.IrqPending);

            m.OnCpuCycle();
            Assert.False(m.IrqPending);

            m.OnCpuCycle();
            Assert.True(m.IrqPending);
        }

        // Cycle mode disconnects A12 entirely and clocks once every four CPU cycles instead.
        [Fact]
        public void Rambo1_cycle_mode_clocks_the_counter_every_fourth_cpu_cycle()
        {
            Cartridge cart = Board(64);
            IMapper m = cart.Mapper;

            m.WritePrg(0xC000, 2);
            m.WritePrg(0xE001, 0);
            m.WritePrg(0xC001, 1); // cycle mode

            // A reload lands two above the latch, so four clocks of four cycles each are owed.
            for (int i = 0; i < 16; i++) m.OnCpuCycle();
            Assert.False(m.IrqPending);

            m.OnCpuCycle();
            Assert.True(m.IrqPending);

            // A12 must do nothing at all while the counter runs off the CPU.
            m.WritePrg(0xE000, 0);
            for (int i = 0; i < 40; i++)
            {
                m.OnPpuAddress(0x0000, i * 100);
                m.OnPpuAddress(0x1000, (i * 100) + 60);
            }
            Assert.False(m.IrqPending);
        }

        [Fact]
        public void IremH3001_maps_three_windows_and_fixes_the_last()
        {
            Cartridge cart = Board(65);
            IMapper m = cart.Mapper;
            int last = (cart.PrgRom.Length / 0x2000) - 1;

            m.WritePrg(0x8000, 4);
            m.WritePrg(0xA000, 5);
            m.WritePrg(0xC000, 6);

            Assert.Equal(0xB0 + 4, m.ReadPrg(0x8000));
            Assert.Equal(0xB0 + 5, m.ReadPrg(0xA000));
            Assert.Equal(0xB0 + 6, m.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, m.ReadPrg(0xE000));

            m.WritePrg(0xB003, 12);
            Assert.Equal(0xC0 + 12, m.ReadChr(0x0C00));
        }

        // The latch is written high byte first, and only $9004 moves it into the counter.
        [Fact]
        public void IremH3001_counts_cpu_cycles_and_disarms_when_it_fires()
        {
            IMapper m = Board(65).Mapper;

            m.WritePrg(0x9005, 0x00); // latch high
            m.WritePrg(0x9006, 0x10); // latch low
            m.WritePrg(0x9003, 0x80); // enable
            m.WritePrg(0x9004, 0x00); // reload

            for (int i = 0; i < 15; i++) m.OnCpuCycle();
            Assert.False(m.IrqPending);

            m.OnCpuCycle();
            Assert.True(m.IrqPending);

            // Firing disarms the counter, so nothing happens until the next reload.
            m.WritePrg(0x9003, 0x80);
            Assert.False(m.IrqPending);
            for (int i = 0; i < 64; i++) m.OnCpuCycle();
            Assert.False(m.IrqPending);
        }

        [Fact]
        public void Sunsoft3_takes_its_registers_from_the_upper_half_of_each_4k_block()
        {
            Cartridge cart = Board(67);
            IMapper m = cart.Mapper;
            int last = (cart.PrgRom.Length / 0x4000) - 1;

            m.WritePrg(0xF800, 2);
            Assert.Equal(0xB0 + 4, m.ReadPrg(0x8000));
            Assert.Equal(0xB0 + (last * 2), m.ReadPrg(0xC000));

            m.WritePrg(0x8800, 3);
            m.WritePrg(0xB800, 7);
            Assert.Equal(0xC0 + 6, m.ReadChr(0x0000));
            Assert.Equal(0xC0 + 14, m.ReadChr(0x1800));

            m.WritePrg(0xE800, 2);
            Assert.Equal(Mirroring.SingleScreenLower, m.Mirroring);
        }

        // One address takes both halves of the counter, high first, and $D800 resets which is next.
        [Fact]
        public void Sunsoft3_builds_its_counter_from_two_writes_to_one_address()
        {
            IMapper m = Board(67).Mapper;

            m.WritePrg(0xD800, 0x00); // disable, so the next $C800 is the high half
            m.WritePrg(0xC800, 0x00);
            m.WritePrg(0xC800, 0x08);
            m.WritePrg(0xD800, 0x10); // enable

            for (int i = 0; i < 8; i++) m.OnCpuCycle();
            Assert.False(m.IrqPending);

            m.OnCpuCycle();
            Assert.True(m.IrqPending);
        }

        [Fact]
        public void Sunsoft4_maps_four_2k_chr_windows_and_gates_its_work_ram()
        {
            Cartridge cart = Board(68);
            IMapper m = cart.Mapper;

            m.WritePrg(0x8000, 3);
            m.WritePrg(0xB000, 9);
            Assert.Equal(0xC0 + 6, m.ReadChr(0x0000));
            Assert.Equal(0xC0 + 18, m.ReadChr(0x1800));

            // $F000 bit 4 is the only thing that opens $6000.
            m.WritePrg(0x6000, 0x42);
            Assert.Equal(0x00, m.ReadPrg(0x6000));

            m.WritePrg(0xF000, 0x12);
            m.WritePrg(0x6000, 0x42);
            Assert.Equal(0x42, m.ReadPrg(0x6000));
            Assert.Equal(0xB0 + 4, m.ReadPrg(0x8000));
        }

        // $E000 bit 4 hands the nametables to CHR ROM, which the PPU must honour on a real read.
        [Fact]
        public void Sunsoft4_can_answer_the_ppu_nametable_fetches_from_chr()
        {
            // The nametable pages sit above $80, so the board needs enough CHR to hold them.
            Cartridge cart = Board(68, chrBanks: 32);
            var ppu = new EmuSen.Cores.Nintendo.Moon.Video.Ppu(cart);

            cart.Chr[0x80 * 0x400] = 0x77;
            cart.Chr[0x81 * 0x400] = 0x88;
            ppu.Ciram[0] = 0x11;

            Assert.Equal(0x11, ppu.ReadVram(0x2000));

            cart.Mapper.WritePrg(0xC000, 0x00); // nametable page 0 -> CHR 1K bank $80
            cart.Mapper.WritePrg(0xD000, 0x01); // nametable page 1 -> CHR 1K bank $81
            cart.Mapper.WritePrg(0xE000, 0x11); // horizontal, CHR nametables on

            Assert.Equal(0x77, ppu.ReadVram(0x2000));
            Assert.Equal(0x88, ppu.ReadVram(0x2800));

            // CHR ROM cannot be written through the nametable window.
            ppu.WriteVram(0x2000, 0x99);
            Assert.Equal(0x77, ppu.ReadVram(0x2000));
            Assert.Equal(0x11, ppu.Ciram[0]);
        }

        [Fact]
        public void SunsoftFme7_drives_everything_through_one_command_parameter_pair()
        {
            Cartridge cart = Board(69);
            IMapper m = cart.Mapper;
            int last = (cart.PrgRom.Length / 0x2000) - 1;

            void Command(int command, int value)
            {
                m.WritePrg(0x8000, (byte)command);
                m.WritePrg(0xA000, (byte)value);
            }

            Command(0x09, 4);
            Command(0x0A, 5);
            Command(0x0B, 6);
            Assert.Equal(0xB0 + 4, m.ReadPrg(0x8000));
            Assert.Equal(0xB0 + 5, m.ReadPrg(0xA000));
            Assert.Equal(0xB0 + 6, m.ReadPrg(0xC000));
            Assert.Equal(0xB0 + last, m.ReadPrg(0xE000));

            Command(0x03, 11);
            Assert.Equal(0xC0 + 11, m.ReadChr(0x0C00));

            Command(0x0C, 3);
            Assert.Equal(Mirroring.SingleScreenUpper, m.Mirroring);
        }

        // $6000 is a PRG ROM window until bit 6 says otherwise, which is not how most boards behave.
        [Fact]
        public void SunsoftFme7_serves_6000_from_rom_until_it_is_told_to_use_ram()
        {
            Cartridge cart = Board(69);
            IMapper m = cart.Mapper;

            Assert.Equal(0xB0 + 0, m.ReadPrg(0x6000));

            m.WritePrg(0x8000, 0x08);
            m.WritePrg(0xA000, 0x05); // ROM bank 5
            Assert.Equal(0xB0 + 5, m.ReadPrg(0x6000));

            m.WritePrg(0xA000, 0xC0); // RAM, enabled
            m.WritePrg(0x6000, 0x42);
            Assert.Equal(0x42, m.ReadPrg(0x6000));

            m.WritePrg(0xA000, 0x40); // RAM, disabled
            Assert.Equal(0x00, m.ReadPrg(0x6000));
        }

        // Unlike the Irem and Sunsoft-3 counters, this one wraps and keeps going.
        [Fact]
        public void SunsoftFme7_counter_wraps_instead_of_disarming()
        {
            IMapper m = Board(69).Mapper;

            m.WritePrg(0x8000, 0x0E);
            m.WritePrg(0xA000, 0x04); // counter low
            m.WritePrg(0x8000, 0x0F);
            m.WritePrg(0xA000, 0x00); // counter high
            m.WritePrg(0x8000, 0x0D);
            m.WritePrg(0xA000, 0x81); // counting, IRQ enabled

            for (int i = 0; i < 4; i++) m.OnCpuCycle();
            Assert.False(m.IrqPending);

            m.OnCpuCycle();
            Assert.True(m.IrqPending);

            // Acknowledging leaves it running, so it comes back a full period later.
            m.WritePrg(0x8000, 0x0D);
            m.WritePrg(0xA000, 0x81);
            Assert.False(m.IrqPending);

            for (int i = 0; i < 0x10000; i++) m.OnCpuCycle();
            Assert.True(m.IrqPending);
        }

        // A copier's signature in byte 7 is not a mapper number - see Moon_Memory.md §2.2.
        [Theory]
        [InlineData(0x00, "", 4)]
        [InlineData(0x40, "", 68)]
        [InlineData(0x44, "DiskDude!", 4)]
        [InlineData(0x40, "DiskDude!", 4)]
        public void An_archaic_header_does_not_contribute_a_high_mapper_nibble(int byte7, string tail, int expected)
        {
            byte[] image = SyntheticNesRom.Build(prgBanks: 8, chrBanks: 8, mapper: 4);
            image[7] = (byte)byte7;

            // "DiskDude!" starts at byte 7, so byte 7 itself is supplied by the case above.
            for (int i = 0; i < tail.Length - 1 && 8 + i < 16; i++) image[8 + i] = (byte)tail[i + 1];

            Assert.Equal(expected, Cartridge.FromImage(image).MapperNumber);
        }

        [Theory]
        [InlineData(4)]
        [InlineData(9)]
        [InlineData(11)]
        [InlineData(64)]
        [InlineData(65)]
        [InlineData(66)]
        [InlineData(67)]
        [InlineData(68)]
        [InlineData(69)]
        [InlineData(71)]
        [InlineData(79)]
        public void Every_new_mapper_number_builds_a_board(int mapper)
        {
            var cart = Cartridge.FromImage(SyntheticNesRom.Build(prgBanks: 8, chrBanks: 8, mapper: mapper));

            Assert.NotNull(cart.Mapper);
            Assert.False(string.IsNullOrWhiteSpace(cart.Mapper.Name));
        }
    }
}
