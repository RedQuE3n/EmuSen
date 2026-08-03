using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.WiseMan.Fixtures;
using MoonPpu = EmuSen.Cores.Nintendo.Moon.Video.Ppu;
using NesButton = EmuSen.Cores.Nintendo.Moon.Input.NesButton;

namespace EmuSen.WiseMan.Cores
{
    // The ICore half of the second core - see Moon_Core.md.
    public class MoonCoreTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private MoonCore Load(byte[]? image = null)
        {
            string path = SyntheticNesRom.WriteTemp(image ?? SyntheticNesRom.Build());
            _temporaryFiles.Add(path);

            var core = new MoonCore();
            core.LoadRom(path);
            return core;
        }

        [Fact]
        public void An_ines_header_survives_the_round_trip_into_a_cartridge()
        {
            var cart = SyntheticNesRom.LoadCartridge();

            Assert.Equal(0, cart.MapperNumber);
            Assert.Equal(1, cart.PrgBanks);
            Assert.Equal(1, cart.ChrBanks);
            Assert.False(cart.ChrIsRam);
            Assert.Equal("NROM", cart.Mapper.Name);
        }

        // A header claiming more PRG than the file holds is corruption, not something to guess through.
        [Fact]
        public void A_truncated_image_is_rejected_rather_than_silently_padded()
        {
            byte[] image = SyntheticNesRom.Build();
            Array.Resize(ref image, Cartridge.HeaderSize + 1024);

            var error = Assert.Throws<InvalidDataException>(() => Cartridge.FromImage(image));
            Assert.Contains("PRG", error.Message);
        }

        [Fact]
        public void A_non_ines_image_is_rejected()
        {
            Assert.Throws<InvalidDataException>(() => Cartridge.FromImage(new byte[64]));
        }

        [Fact]
        public void The_core_reports_the_nes_screen_and_refresh_rate()
        {
            var core = Load();

            Assert.Equal("NES", core.CoreName);
            Assert.Equal(256, core.ScreenWidth);
            Assert.Equal(240, core.ScreenHeight);
            Assert.InRange(core.FrameRateHz, 60.0, 60.2);
            Assert.True(core.IsRomLoaded);
        }

        [Fact]
        public void Running_a_frame_advances_the_counter_and_fills_a_full_framebuffer()
        {
            var core = Load();

            core.RunFrame();

            Assert.Equal(1, core.TotalFrames);
            Assert.Equal(256 * 240 * 4, core.GetFrameBufferRgba().Length);
        }

        // A 16K NROM has to mirror its one bank into both slots or the reset vector is unreachable.
        [Fact]
        public void Reset_enters_at_the_vector_the_last_bank_carries()
        {
            var core = Load();

            Assert.Equal(0x8000, core.Cpu!.PC);
        }

        [Fact]
        public void The_cpu_keeps_pace_with_the_master_clock_across_a_frame()
        {
            var core = Load();

            core.RunFrame();

            // 262 lines x 1364 master clocks / 12 per cycle ~= 29780 CPU cycles a frame.
            Assert.InRange(core.Cpu!.Cycles, 29700, 29850);
        }

        [Fact]
        public void The_scheduler_lands_exactly_on_a_frame_boundary()
        {
            var core = Load();

            core.RunFrame();

            long expected = (long)MoonPpu.TotalScanlines * MoonCore.MasterClocksPerScanline;
            Assert.Equal(expected, core.MasterClock);
        }

        // Nothing else in the frame loop matters if the game never gets its vblank interrupt.
        [Fact]
        public void Vblank_raises_the_ppu_nmi_line_once_enabled()
        {
            var core = Load();
            core.Ppu!.Control = 0x80;

            bool sawNmi = false;
            for (int line = 0; line < MoonPpu.TotalScanlines; line++)
            {
                core.Ppu.EndScanline(line);
                if (core.Ppu.NmiOutput) sawNmi = true;
            }

            Assert.True(sawNmi);
        }

        [Fact]
        public void The_prerender_line_clears_vblank_and_the_sprite_flags()
        {
            var core = Load();
            var ppu = core.Ppu!;

            ppu.EndScanline(MoonPpu.VBlankScanline);
            Assert.True(ppu.VBlankFlag);

            ppu.Sprite0Hit = true;
            ppu.SpriteOverflow = true;
            ppu.EndScanline(MoonPpu.PreRenderScanline);

            Assert.False(ppu.VBlankFlag);
            Assert.False(ppu.Sprite0Hit);
            Assert.False(ppu.SpriteOverflow);
        }

        // Reading $2002 is how every game acknowledges vblank, and it also resets the address latch.
        [Fact]
        public void Reading_the_status_register_clears_vblank_and_the_write_toggle()
        {
            var core = Load();
            var ppu = core.Ppu!;

            ppu.VBlankFlag = true;
            ppu.WriteRegister(6, 0x21);
            Assert.True(ppu.WriteToggle);

            byte status = ppu.ReadRegister(2);

            Assert.Equal(0x80, status & 0x80);
            Assert.False(ppu.VBlankFlag);
            Assert.False(ppu.WriteToggle);
        }

        [Fact]
        public void Two_writes_to_the_address_register_load_the_live_vram_pointer()
        {
            var core = Load();
            var ppu = core.Ppu!;

            ppu.WriteRegister(6, 0x21);
            ppu.WriteRegister(6, 0x08);

            Assert.Equal(0x2108, ppu.V);
        }

        // $2007 below the palette hands back the previous byte, so a game reads one address late.
        [Fact]
        public void Vram_reads_lag_one_access_behind_outside_the_palette()
        {
            var core = Load();
            var ppu = core.Ppu!;

            ppu.WriteRegister(6, 0x20);
            ppu.WriteRegister(6, 0x00);
            ppu.WriteRegister(7, 0x42);

            ppu.WriteRegister(6, 0x20);
            ppu.WriteRegister(6, 0x00);

            Assert.NotEqual(0x42, ppu.ReadRegister(7));
            Assert.Equal(0x42, ppu.ReadRegister(7));
        }

        [Theory]
        [InlineData(Mirroring.Horizontal, 0x2000, 0x2400, true)]
        [InlineData(Mirroring.Horizontal, 0x2000, 0x2800, false)]
        [InlineData(Mirroring.Vertical, 0x2000, 0x2800, true)]
        [InlineData(Mirroring.Vertical, 0x2000, 0x2400, false)]
        public void Nametable_mirroring_follows_the_board(Mirroring mirroring, int first, int second, bool shouldAlias)
        {
            var core = Load(SyntheticNesRom.Build(mirroring: mirroring));
            var ppu = core.Ppu!;

            Assert.Equal(shouldAlias, ppu.NametableOffset((ushort)first) == ppu.NametableOffset((ushort)second));
        }

        // The four sprite backdrop entries are holes that read the background's - see Moon_PPU.md §2.4.
        [Fact]
        public void The_sprite_backdrop_palette_entries_mirror_the_background_ones()
        {
            Assert.Equal(MoonPpu.PaletteOffset(0x3F00), MoonPpu.PaletteOffset(0x3F10));
            Assert.Equal(MoonPpu.PaletteOffset(0x3F04), MoonPpu.PaletteOffset(0x3F14));
            Assert.NotEqual(MoonPpu.PaletteOffset(0x3F01), MoonPpu.PaletteOffset(0x3F11));
        }

        [Fact]
        public void Work_ram_mirrors_every_two_kilobytes()
        {
            var core = Load();
            var bus = core.Bus!;

            bus.Write(0x0005, 0x5A);

            Assert.Equal(0x5A, bus.Read(0x0805));
            Assert.Equal(0x5A, bus.Read(0x1005));
            Assert.Equal(0x5A, bus.Read(0x1805));
        }

        [Fact]
        public void Ppu_registers_mirror_every_eight_bytes_to_the_end_of_the_range()
        {
            var core = Load();

            // $3FF8 is the last mirror of $2000 in the range; $3FFE would be $2006 instead.
            core.Bus!.Write(0x3FF8, 0x21);

            Assert.Equal(0x21, core.Ppu!.Control);
        }

        [Fact]
        public void Oam_dma_copies_a_page_and_bills_the_cpu_for_it()
        {
            var core = Load();
            var bus = core.Bus!;

            for (int i = 0; i < 256; i++) bus.Write((ushort)(0x0300 + i), (byte)i);
            bus.Write(0x4014, 0x03);

            Assert.Equal(0x00, core.Ppu!.Oam[0]);
            Assert.Equal(0x7F, core.Ppu.Oam[0x7F]);
            Assert.Equal(MemoryBus.OamDmaCycles, bus.TakePendingDmaCycles());
            Assert.Equal(0, bus.TakePendingDmaCycles());
        }

        [Fact]
        public void A_controller_clocks_its_buttons_out_one_read_at_a_time()
        {
            var core = Load();

            core.SetButton(0, NesButton.A, true);
            core.SetButton(0, NesButton.Start, true);

            core.Bus!.Write(0x4016, 1);
            core.Bus.Write(0x4016, 0);

            Assert.Equal(1, core.Bus.Read(0x4016) & 1); // A
            Assert.Equal(0, core.Bus.Read(0x4016) & 1); // B
            Assert.Equal(0, core.Bus.Read(0x4016) & 1); // Select
            Assert.Equal(1, core.Bus.Read(0x4016) & 1); // Start
        }

        [Fact]
        public void A_save_state_round_trips_through_a_stream()
        {
            var core = Load();
            for (int i = 0; i < 3; i++) core.RunFrame();

            core.Bus!.Write(0x0010, 0xC3);

            using var stream = new MemoryStream();
            core.SaveState(stream);

            long frames = core.TotalFrames;
            long clock = core.MasterClock;

            core.Bus.Write(0x0010, 0x00);
            core.RunFrame();

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(frames, core.TotalFrames);
            Assert.Equal(clock, core.MasterClock);
            Assert.Equal(0xC3, core.Bus.Read(0x0010));
        }

        [Fact]
        public void A_state_from_another_core_is_refused()
        {
            var core = Load();

            using var stream = new MemoryStream();
            using (var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true))
            {
                w.Write(0x53454E53u); // Venus's magic
                w.Write(1);
            }

            stream.Position = 0;
            Assert.Throws<InvalidDataException>(() => core.LoadState(stream));
        }

        [Fact]
        public void The_catalog_reaches_the_nes_core_by_console_name_and_codename()
        {
            Assert.True(CoreCatalog.Registry.ContainsKey("nes"));
            Assert.True(CoreCatalog.Registry.ContainsKey("moon"));
            Assert.Same(CoreCatalog.Registry["nes"], CoreCatalog.Registry["moon"]);
            Assert.Contains(".nes", CoreCatalog.Registry["nes"].Extensions);
        }

        [Fact]
        public void An_unimplemented_mapper_says_so_rather_than_loading_wrong()
        {
            var error = Assert.Throws<NotSupportedException>(
                () => Cartridge.FromImage(SyntheticNesRom.Build(mapper: 5)));

            Assert.Contains("mapper 5", error.Message);
        }
    }
}
