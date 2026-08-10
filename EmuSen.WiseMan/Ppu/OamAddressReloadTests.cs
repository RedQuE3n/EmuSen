using EmuSen.Cores.Nintendo.Venus;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Ppu
{
    // $2102/$2103 latch an OAM address that hardware reloads into the
    // running write pointer every vblank - see Venus_PPU.md §6.4. Donkey
    // Kong Country is the game that needs it: it never writes $2102/$2103
    // at all after setup, and DMAs all 544 OAM bytes to $2104 each frame,
    // trusting the reload to put the pointer back at 0.
    [Collection(TestCollections.ProcessGlobals)]
    public class OamAddressReloadTests
    {
        private const uint OamAddL = 0x2102;
        private const uint OamAddH = 0x2103;
        private const uint OamData = 0x2104;

        private static VenusCore NewCore()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            core.Bus!.Ppu.Inidisp = 0x0F; // not forced blank
            return core;
        }

        private static void Vblank(VenusCore core) => core.Bus!.Ppu.ReloadOamAddressForVBlank();

        [Fact]
        public void Writing_the_address_registers_moves_the_running_pointer_immediately()
        {
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamAddL, 0x02); // word 2 = byte 4
            ppu.WriteRegister(OamData, 0xAB);

            Assert.Equal(0xAB, ppu.Oam[4]);
        }

        [Fact]
        public void Oamdata_writes_advance_the_pointer_without_moving_the_latch()
        {
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamAddL, 0x00);
            ppu.WriteRegister(OamData, 0x11);
            ppu.WriteRegister(OamData, 0x22);

            Vblank(core);
            ppu.WriteRegister(OamData, 0x33);

            // Back at 0 because the latch never moved, so this overwrites 0x11.
            Assert.Equal(0x33, ppu.Oam[0]);
            Assert.Equal(0x22, ppu.Oam[1]);
        }

        [Fact]
        public void A_stray_write_does_not_shift_the_next_frames_upload()
        {
            // The exact DKC failure: two odd bytes reached $2104, and without
            // a vblank reload every subsequent full-OAM DMA landed two bytes
            // late, so every sprite read its neighbour's fields.
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamData, 0x00);
            ppu.WriteRegister(OamData, 0x00);

            Vblank(core);
            for (int i = 0; i < 544; i++) ppu.WriteRegister(OamData, (byte)(i & 0xFF));

            Assert.Equal(0, ppu.Oam[0]);
            Assert.Equal(1, ppu.Oam[1]);
            Assert.Equal(2, ppu.Oam[2]);
        }

        [Fact]
        public void The_reload_restores_a_non_zero_latched_address()
        {
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamAddL, 0x04); // word 4 = byte 8
            ppu.WriteRegister(OamData, 0x77);
            ppu.WriteRegister(OamData, 0x88);

            Vblank(core);
            ppu.WriteRegister(OamData, 0x99);

            Assert.Equal(0x99, ppu.Oam[8]);
        }

        [Fact]
        public void The_high_bit_of_OAMADDH_survives_the_reload()
        {
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamAddL, 0x00);
            ppu.WriteRegister(OamAddH, 0x01); // word 0x100 = byte 0x200, the high table
            ppu.WriteRegister(OamData, 0x5A);

            Vblank(core);
            ppu.WriteRegister(OamData, 0xA5);

            Assert.Equal(0xA5, ppu.Oam[0x200]);
        }

        [Fact]
        public void Forced_blank_suppresses_the_reload()
        {
            // Hardware only reloads when rendering was actually running, so a
            // game uploading OAM during forced blank keeps its pointer.
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamAddL, 0x00);
            ppu.WriteRegister(OamData, 0x11);
            ppu.Inidisp = 0x8F; // forced blank

            Vblank(core);
            ppu.WriteRegister(OamData, 0x22);

            Assert.Equal(0x11, ppu.Oam[0]);
            Assert.Equal(0x22, ppu.Oam[1]);
        }

        [Fact]
        public void A_running_core_reloads_the_pointer_once_per_frame()
        {
            // End to end through RunFrame rather than the helper, so the
            // VenusCore call site is covered too.
            var core = NewCore();
            var ppu = core.Bus!.Ppu;

            ppu.WriteRegister(OamAddL, 0x00);
            ppu.WriteRegister(OamData, 0xEE);
            core.RunFrame();
            ppu.WriteRegister(OamData, 0xDD);

            Assert.Equal(0xDD, ppu.Oam[0]);
        }
    }
}
