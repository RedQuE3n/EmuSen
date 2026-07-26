using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Shell;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Shell
{
    // IDebugTarget.DecodeTilePixels (SnesDebugTarget's planar bitplane
    // decode) and the `tile` shell command built on top of it - see
    // IDebugTarget.cs's own comment for why this exists: `tile` used to
    // hardcode this exact SNES bitplane layout directly in the
    // "core-agnostic" shell layer.
    public class TilePixelDecodeTests
    {
        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        [Fact]
        public void Bpp2_both_planes_set_decodes_every_pixel_to_three()
        {
            var target = BuildTarget();
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            wram.Write(0x1000, 0xFF); // plane 0, row 0
            wram.Write(0x1001, 0xFF); // plane 1, row 0

            byte[] pixels = target.DecodeTilePixels(wram, 0x1000, 2);

            Assert.All(pixels.Take(8), p => Assert.Equal(3, p));
        }

        [Fact]
        public void Bpp2_single_bit_decodes_to_expected_row()
        {
            var target = BuildTarget();
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            wram.Write(0x1002, 0x80); // plane 0, row 1: bit 7 only
            wram.Write(0x1003, 0x00); // plane 1, row 1: nothing

            byte[] pixels = target.DecodeTilePixels(wram, 0x1000, 2);

            Assert.Equal(1, pixels[8]); // row 1, col 0
            Assert.All(pixels.Skip(9).Take(7), p => Assert.Equal(0, p));
        }

        [Theory]
        [InlineData(1)]
        [InlineData(3)]
        [InlineData(16)]
        public void Unsupported_bpp_throws(int bpp)
        {
            var target = BuildTarget();
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            Assert.Throws<ArgumentException>(() => target.DecodeTilePixels(wram, 0x1000, bpp));
        }

        [Fact]
        public void Tile_command_renders_bpp2_row_as_ascii()
        {
            var target = BuildTarget();
            var wram = target.GetMemorySpaces().First(s => s.Name == "WRAM");
            wram.Write(0x1000, 0xFF);
            wram.Write(0x1001, 0xFF);

            var shell = ShellInterpreter.CreateDefault(target);
            var result = shell.Submit("tile WRAM 1000 2");

            Assert.Contains("3 3 3 3 3 3 3 3", result.Output);
        }
    }
}
