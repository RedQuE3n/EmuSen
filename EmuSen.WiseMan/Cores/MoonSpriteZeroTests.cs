using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Sprite 0 hits on the dot its pixel is output, so a split timed from the hit lands where the game meant - see Moon_Native.md §3.14.
    public class MoonSpriteZeroTests
    {
        private readonly ITestOutputHelper _output;

        public MoonSpriteZeroTests(ITestOutputHelper output) => _output = output;

        // NROM with tile 1 solid in colour 1; everything else in CHR is transparent.
        private static byte[] Image(byte[] program)
        {
            byte[] image = SyntheticNesRom.Build(mapper: 0, patches: (0, program));
            int chr = 16 + SyntheticNesRom.PrgBankSize;
            for (int row = 0; row < 8; row++) image[chr + 16 + row] = 0xFF;
            return image;
        }

        private static MoonCore Load(byte[] image)
        {
            CoreOptions.BatteryRamDisabled = true;
            string path = SyntheticNesRom.WriteTemp(image);
            try
            {
                var core = new MoonCore();
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        // A solid sprite 0 over a solid background tile at x 88 on line 30, the PPU stepped dot by dot from the line's start.
        private static int HitDot(int enableDot = -1)
        {
            var core = Load(Image(Array.Empty<byte>()));
            var ppu = core.Ppu!;
            ppu.Ciram[(3 * 32) + 11] = 1;
            ppu.Oam[0] = 29;
            ppu.Oam[1] = 1;
            ppu.Oam[3] = 88;
            for (int i = 4; i < 256; i++) ppu.Oam[i] = 0xF0;
            ppu.Mask = 0x1E;
            ppu.Scanline = 30;
            ppu.Cycle = 0;
            ppu.RenderV = 3 << 5 | 6 << 12;
            if (enableDot >= 0) ppu.Mask = 0;
            for (int dot = 1; dot <= 256; dot++)
            {
                if (dot == enableDot) ppu.WriteRegister(1, 0x1E);
                ppu.Step(1);
                if (ppu.Sprite0Hit) return ppu.Cycle;
            }
            return -1;
        }

        [Fact]
        public void Sprite_zero_hits_on_the_dot_its_pixel_is_output()
        {
            Assert.Equal(89, HitDot());
        }

        // The background shifters stand still while rendering is off, so a tile fetched before the switch is not there to hit - see Moon_PPU.md §3.4.
        [Fact]
        public void A_hit_needs_the_tile_loaded_while_rendering_was_on()
        {
            Assert.Equal(-1, HitDot(enableDot: 85));
            Assert.Equal(89, HitDot(enableDot: 60));
        }

        // SMB's split in miniature: wait for the hit, wait a line, write the X scroll; line 32 must be the first scrolled line, as line 33 is.
        public static byte[] SplitProgram()
        {
            var a = new Asm(0x8000);
            a.Op(0x78).Op(0xD8).Ldx(0xFF).Op(0x9A);
            a.LdaImm(0x00).Sta(0x2000).Sta(0x2001);
            for (int w = 0; w < 2; w++) a.Bit(0x2002).Bpl(-3);
            a.LdaImm(0x3F).Sta(0x2006).LdaImm(0x00).Sta(0x2006).LdaImm(0x0F).Sta(0x2007).LdaImm(0x30).Sta(0x2007);
            a.LdaImm(0x3F).Sta(0x2006).LdaImm(0x11).Sta(0x2006).LdaImm(0x16).Sta(0x2007);
            a.LdaImm(0x20).Sta(0x2006).LdaImm(0x6B).Sta(0x2006).LdaImm(0x01).Sta(0x2007);
            a.LdaImm(0x20).Sta(0x2006).LdaImm(0x80).Sta(0x2006).Ldx(16);
            a.LdaImm(0x01).Sta(0x2007).LdaImm(0x00).Sta(0x2007).Op(0xCA).Bne(-11);
            a.LdaImm(0x00).Sta(0x2003).LdaImm(29).Sta(0x2004).LdaImm(0x01).Sta(0x2004).LdaImm(0x00).Sta(0x2004).LdaImm(88).Sta(0x2004);
            ushort main = a.Here;
            a.Bit(0x2002).Bpl(-3);
            a.LdaImm(0x00).Sta(0x2005).Sta(0x2005).Sta(0x2000).LdaImm(0x1E).Sta(0x2001);
            a.Bit(0x2002).Op(0x70, 0xFB);
            a.Bit(0x2002).Op(0x50, 0xFB);
            a.Ldx(24).Op(0xCA).Op(0xD0, 0xFD);
            a.LdaImm(0x08).Sta(0x2005).LdaImm(0x00).Sta(0x2005);
            a.Jmp(main);
            return a.Bytes;
        }

        private static bool Row(byte[] frame, int y, int other) =>
            frame.AsSpan(y * 1024, 1024).SequenceEqual(frame.AsSpan(other * 1024, 1024));

        [Fact]
        public void A_split_timed_from_the_hit_scrolls_the_line_after_next()
        {
            var core = Load(Image(SplitProgram()));
            for (int f = 0; f < 10; f++) core.RunFrame();
            byte[] frame = core.GetFrameBufferRgba();
            Assert.False(Row(frame, 31, 33), "the stripes never appeared below the split");
            Assert.True(Row(frame, 32, 33), "line 32 was drawn with the status bar's coarse X");
        }

        // Both engines, picture and state compared every frame by the pair.
        [Fact]
        public void A_split_timed_from_the_hit_scrolls_alike_in_both_engines()
        {
            var pair = new MoonRtPair(Image(SplitProgram()), skipRendering: false);
            pair.Run(10, null);
            Assert.True(Row(pair.Csharp.GetFrameBufferRgba(), 32, 33), "line 32 was drawn with the status bar's coarse X");
            _output.WriteLine(pair.Summary);
        }

        // blargg's 2005 sprite 0 hit set, from the corpus; each leaves its result at $F8, 1 for a pass. 9 and 10 time the hit.
        public static IEnumerable<object[]> SpriteHit2005() =>
            Enumerable.Range(1, 11).Select(i => new object[] { i });

        [Theory]
        [MemberData(nameof(SpriteHit2005))]
        public void Blarggs_2005_sprite_hit_tests_pass_on_both_engines(int number)
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtCorpusTests.CorpusVariable);
            string? dir = folder is null ? null : Path.Combine(folder, "sprite_hit_tests_2005.10.05");
            string? path = dir is not null && Directory.Exists(dir) ? Directory.GetFiles(dir, $"{number:D2}.*.nes").SingleOrDefault() : null;
            if (path is null)
            {
                _output.WriteLine($"{MoonRtCorpusTests.CorpusVariable} unset, not run");
                return;
            }
            CoreOptions.BatteryRamDisabled = true;

            var csharp = new MoonCore();
            csharp.LoadRom(path);
            for (int f = 0; f < 120; f++) csharp.RunFrame();
            Assert.Equal(1, csharp.ReadSpace(MoonCore.SpaceRam, 0xF8));

            Assert.True(MoonRtCore.Available, "MoonRT is not built");
            using var native = new MoonRtCore();
            native.LoadRom(path);
            for (int f = 0; f < 120; f++) native.RunFrame();
            Assert.Equal(1, native.ReadSpace(MoonCore.SpaceRam, 0xF8));
        }

        // Super Mario Bros. into 1-1, walking right and jumping; a black pixel on line 32 must sit over the cloud it outlines, not over sky.
        [Fact]
        public void Super_mario_bros_draws_no_slivers_below_its_split()
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtStateTests.RomsVariable);
            string? rom = folder is null ? null : Path.Combine(folder, "smb.nes");
            if (rom is null || !File.Exists(rom))
            {
                _output.WriteLine($"{MoonRtStateTests.RomsVariable} has no smb.nes, not run");
                return;
            }
            var pair = new MoonRtPair(File.ReadAllBytes(rom), skipRendering: false, stateEvery: 60);
            int slivers = 0, checkedFrames = 0;
            for (int f = 0; f < 1700; f++)
            {
                uint pad = (f is >= 40 and < 46 ? 1u << 3 : 0) | (f >= 220 ? 1u << 7 : 0) | (f >= 220 && f % 50 < 18 ? 1u : 0);
                pair.Frame(_ => (pad, 0u), f);
                // Between lives the backdrop is black, and there is no level to check.
                if (f < 220 || (pair.Csharp.Ppu!.PaletteRam[0] & 0x3F) == 0x0F) continue;
                checkedFrames++;
                byte[] frame = pair.Csharp.GetFrameBufferRgba();
                int sky = (pair.Csharp.Ppu.PaletteRam[0] & 0x3F) * 3;
                byte[] backdrop = EmuSen.Cores.Nintendo.Moon.Video.Ppu.NesPalette.AsSpan(sky, 3).ToArray();
                for (int x = 0; x < 256; x++)
                {
                    int p = ((32 * 256) + x) * 4;
                    if (frame[p] != 0 || frame[p + 1] != 0 || frame[p + 2] != 0) continue;
                    bool over = false;
                    for (int dx = -1; dx <= 1 && !over; dx++)
                    {
                        int q = ((33 * 256) + Math.Clamp(x + dx, 0, 255)) * 4;
                        over = !frame.AsSpan(q, 3).SequenceEqual(backdrop);
                    }
                    if (!over) { slivers++; break; }
                }
            }
            _output.WriteLine($"{checkedFrames} frames checked, {slivers} with a sliver on line 32; {pair.Summary}");
            Assert.Equal(0, slivers);
        }
    }
}
