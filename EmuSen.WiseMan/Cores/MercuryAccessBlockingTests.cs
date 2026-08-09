using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Cores.Nintendo.Mercury.Video;
using GbPpu = EmuSen.Cores.Nintendo.Mercury.Video.Ppu;

namespace EmuSen.WiseMan.Cores
{
    // What the CPU can and cannot reach while the PPU is using it - see Mercury_Ppu.md §7.
    public class MercuryAccessBlockingTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private MercuryCore Load(byte cgbFlag = 0x00)
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(cgbFlag: cgbFlag));
            _temporaryFiles.Add(path);

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        // Advances to a chosen dot of the current scanline; the LCD is on from reset.
        private static void AdvanceToDot(MercuryCore core, int dot)
        {
            while (core.Bus!.Ppu.Dot != dot) core.Bus.Tick(1);
        }

        [Fact]
        public void Vram_is_open_in_hblank_and_closed_while_drawing()
        {
            var core = Load();
            var bus = core.Bus!;

            AdvanceToDot(core, GbPpu.OamScanCycles + GbPpu.BaseDrawingCycles + 8);
            Assert.Equal(PpuMode.HBlank, bus.Ppu.Mode);

            bus.Write(0x8000, 0x3C);
            Assert.Equal(0x3C, bus.Read(0x8000));

            AdvanceToDot(core, GbPpu.OamScanCycles + 8);
            Assert.Equal(PpuMode.Drawing, bus.Ppu.Mode);

            Assert.Equal(0xFF, bus.Read(0x8000));

            // The write is dropped, not queued: the byte underneath is unchanged when drawing ends.
            bus.Write(0x8000, 0x99);
            AdvanceToDot(core, GbPpu.OamScanCycles + GbPpu.BaseDrawingCycles + 8);
            Assert.Equal(0x3C, bus.Read(0x8000));
        }

        [Fact]
        public void Oam_is_closed_for_the_scan_as_well_as_the_draw()
        {
            var core = Load();
            var bus = core.Bus!;

            AdvanceToDot(core, GbPpu.OamScanCycles + GbPpu.BaseDrawingCycles + 8);
            bus.Write(0xFE00, 0x2A);
            Assert.Equal(0x2A, bus.Read(0xFE00));

            AdvanceToDot(core, 8);
            Assert.Equal(PpuMode.OamScan, bus.Ppu.Mode);
            Assert.Equal(0xFF, bus.Read(0xFE00));

            AdvanceToDot(core, GbPpu.OamScanCycles + 8);
            Assert.Equal(PpuMode.Drawing, bus.Ppu.Mode);
            Assert.Equal(0xFF, bus.Read(0xFE00));
        }

        // Blocking is the PPU using the bus, so switching the LCD off returns both to the CPU.
        [Fact]
        public void A_disabled_lcd_blocks_nothing()
        {
            var core = Load();
            var bus = core.Bus!;

            bus.Write(0xFF40, 0x00);

            bus.Write(0x8000, 0x3C);
            bus.Write(0xFE00, 0x2A);

            for (int i = 0; i < GbPpu.CyclesPerScanline * 4; i++)
            {
                bus.Tick(1);
                Assert.Equal(0x3C, bus.Read(0x8000));
                Assert.Equal(0x2A, bus.Read(0xFE00));
            }
        }

        // The renderer reads VRAM directly rather than through the CPU decode, so it must be unaffected.
        [Fact]
        public void Blocking_does_not_starve_the_renderer()
        {
            var core = Load();
            var bus = core.Bus!;

            bus.Write(0xFF40, 0x00);

            // A tile of solid colour 3, mapped across the whole background.
            for (int i = 0; i < 16; i++) bus.Write((ushort)(0x8000 + i), 0xFF);
            for (int i = 0; i < 0x400; i++) bus.Write((ushort)(0x9800 + i), 0x00);

            bus.Write(0xFF47, 0xE4);
            bus.Write(0xFF40, 0x91);

            for (int i = 0; i < GbPpu.CyclesPerScanline * GbPpu.TotalScanlines; i++) bus.Tick(1);

            byte[] frame = core.GetFrameBufferRgba();
            Assert.Equal(0x00, frame[0]);
        }
    }
}
