using System;
using EmuSen.Cores.Nintendo.Moon.Memory;
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

        [Theory]
        [InlineData(4)]
        [InlineData(11)]
        [InlineData(66)]
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
