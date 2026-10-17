using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Galaxia.Input;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Sprite 0 hits on the dot its pixel is output, so a split timed from the hit lands where the game meant, on MoonRT - see Moon_Native.md §3.14.
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

        private static MoonRtCore Load(byte[] image)
        {
            string path = SyntheticNesRom.WriteTemp(image);
            try
            {
                var core = new MoonRtCore() { Options = TestCores.NoBattery };
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
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
            using var core = Load(Image(SplitProgram()));
            for (int f = 0; f < 10; f++) core.RunFrame();
            byte[] frame = core.GetFrameBufferRgba();
            Assert.False(Row(frame, 31, 33), "the stripes never appeared below the split");
            Assert.True(Row(frame, 32, 33), "line 32 was drawn with the status bar's coarse X");
        }

        // blargg's 2005 sprite 0 hit set, from the corpus; each leaves its result at $F8, 1 for a pass. 9 and 10 time the hit.
        public static IEnumerable<object[]> SpriteHit2005() =>
            Enumerable.Range(1, 11).Select(i => new object[] { i });

        [Theory]
        [MemberData(nameof(SpriteHit2005))]
        public void Blarggs_2005_sprite_hit_tests_pass(int number)
        {
            string? dir = MoonGoldenTests.CorpusRoot is { } corpus ? Path.Combine(corpus, "sprite_hit_tests_2005.10.05") : null;
            string? path = dir is not null && Directory.Exists(dir) ? Directory.GetFiles(dir, $"{number:D2}.*.nes").SingleOrDefault() : null;
            if (path is null)
            {
                _output.WriteLine($"{MoonGoldenTests.CorpusVariable} unset, not run");
                return;
            }
            Assert.True(MoonRtCore.Available, "MoonRT is not built");
            using var native = new MoonRtCore() { Options = TestCores.NoBattery };
            native.LoadRom(path);
            for (int f = 0; f < 120; f++) native.RunFrame();
            Assert.Equal(1, native.ReadSpace(NesSystem.Ram, 0xF8));
        }

        // Super Mario Bros. into 1-1, walking right and jumping; a black pixel on line 32 must sit over the cloud it outlines, not over sky.
        [Fact]
        public void Super_mario_bros_draws_no_slivers_below_its_split()
        {
            if (MoonGoldenTests.Game("F61548FDF1670CFF") is not { } image)
            {
                _output.WriteLine($"{MoonGoldenTests.GamesVariable} unset or without the game, not run");
                return;
            }
            using var core = Load(image);
            int slivers = 0, checkedFrames = 0;
            for (int f = 0; f < 1700; f++)
            {
                core.SetButton(0, PadButton.Start, f is >= 40 and < 46);
                core.SetButton(0, PadButton.Right, f >= 220);
                core.SetButton(0, PadButton.A, f >= 220 && f % 50 < 18);
                core.RunFrame();
                core.DequeueAudioSamples(int.MaxValue);
                int backdrop = core.ReadSpace(NesSystem.Palette, 0) & 0x3F;
                // Between lives the backdrop is black, and there is no level to check.
                if (f < 220 || backdrop == 0x0F) continue;
                checkedFrames++;
                byte[] frame = core.GetFrameBufferRgba();
                byte[] sky = NesDebugViews.MasterPalette.AsSpan(backdrop * 3, 3).ToArray();
                for (int x = 0; x < 256; x++)
                {
                    int p = ((32 * 256) + x) * 4;
                    if (frame[p] != 0 || frame[p + 1] != 0 || frame[p + 2] != 0) continue;
                    bool over = false;
                    for (int dx = -1; dx <= 1 && !over; dx++)
                    {
                        int q = ((33 * 256) + Math.Clamp(x + dx, 0, 255)) * 4;
                        over = !frame.AsSpan(q, 3).SequenceEqual(sky);
                    }
                    if (!over) { slivers++; break; }
                }
            }
            _output.WriteLine($"{checkedFrames} frames checked, {slivers} with a sliver on line 32");
            Assert.True(checkedFrames > 0, "no frame of the level was checked");
            Assert.Equal(0, slivers);
        }
    }
}
