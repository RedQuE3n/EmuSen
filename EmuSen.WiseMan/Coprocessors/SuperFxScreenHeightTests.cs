using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.Coprocessors
{
    // SCMR's split height field is the column stride of the framebuffer, so
    // reading its two halves the wrong way round shears the picture - see
    // Venus_SuperFX.md §6.1a for the Yoshi's Island title screen it was found on.
    public class SuperFxScreenHeightTests
    {
        private const ushort Scmr = 0x303A, Scbr = 0x3038, Clsr = 0x3039;
        private const ushort R15Low = 0x301E, R15High = 0x301F;

        // Plots colour 1 at (x, 0) under <scmr> and returns the Game Pak RAM
        // offset that ended up holding it, or -1 if nothing was written.
        private static int PlotOneAndFindIt(byte scmr, int x)
        {
            byte[] rom = new byte[0x10000];
            byte[] program =
            [
                0xF1, (byte)x, (byte)(x >> 8),   // IWT R1, #x
                0xF2, 0x00, 0x00,                // IWT R2, #0
                0xF0, 0x01, 0x00,                // IWT R0, #1
                0x4E,                            // COLOR
                0x4C,                            // PLOT
                0x00,                            // STOP
            ];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(Scmr, scmr);
            gsu.WriteRegister(Scbr, 0x00);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);

            for (int offset = 0; offset < 0x8000; offset++)
            {
                if (gsu.ReadRam(offset) != 0x00) return offset;
            }
            return -1;
        }

        // Bit 2 is HT0 and bit 5 is HT1, so $05 is 160 lines and $21 is 192 -
        // reading them the other way round swaps exactly these two rows.
        // Column 1 at 4bpp lands on tile <columnHeightInTiles>, 32 bytes each.
        [Theory]
        [InlineData(0x01, 16 * 32)]   // 128 lines, 16-tile columns
        [InlineData(0x05, 20 * 32)]   // 160 lines, 20-tile columns
        [InlineData(0x21, 24 * 32)]   // 192 lines, 24-tile columns
        [InlineData(0x25, 1 * 32)]    // OBJ mode - column 1 is simply tile 1
        public void The_height_field_sets_the_column_stride(int scmr, int expectedOffset)
        {
            Assert.Equal(expectedOffset, PlotOneAndFindIt((byte)scmr, x: 8));
        }

        // The value Yoshi's Island's title screen plots with. It is the only
        // scene here that sets one half of the field without the other, which
        // is why the bit order could not be settled before it.
        [Fact]
        public void Yoshis_island_title_screen_scmr_is_a_160_line_framebuffer()
        {
            Assert.Equal(20 * 32, PlotOneAndFindIt(0x1D, x: 8));
        }

        // A 160-line buffer is 20 tiles tall, so y = 159 is the last row of
        // the column and must stay inside it rather than spilling into the next.
        [Fact]
        public void The_bottom_row_of_a_160_line_column_stays_in_its_column()
        {
            byte[] rom = new byte[0x10000];
            byte[] program =
            [
                0xF1, 0x00, 0x00,   // IWT R1, #0
                0xF2, 0x9F, 0x00,   // IWT R2, #159
                0xF0, 0x01, 0x00,   // IWT R0, #1
                0x4E, 0x4C, 0x00,   // COLOR, PLOT, STOP
            ];
            program.CopyTo(rom, 0);

            var gsu = new Gsu(rom, new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(Scmr, 0x05);
            gsu.WriteRegister(Scbr, 0x00);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(20000);

            // Tile 19 (the 20th of column 0), row 7 of that tile.
            Assert.Equal(0x80, gsu.ReadRam(19 * 32 + 7 * 2));
        }
    }
}
