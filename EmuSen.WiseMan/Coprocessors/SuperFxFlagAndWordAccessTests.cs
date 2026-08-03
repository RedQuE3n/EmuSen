using Gsu = EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx.SuperFx;

namespace EmuSen.WiseMan.Coprocessors
{
    // Flag effects and word-RAM alignment, all four found by the GSU trace differ - see Venus_SuperFX.md §4.4 and §5.1a.
    public class SuperFxFlagAndWordAccessTests
    {
        private const ushort Clsr = 0x3039, R15Low = 0x301E, R15High = 0x301F;

        private const ushort FlagZ = 0x0002;
        private const ushort FlagS = 0x0008;

        private static Gsu Run(byte[] program, byte[]? rom = null, byte[]? ram = null)
        {
            byte[] image = new byte[0x10000];
            if (rom != null) rom.CopyTo(image, 0);
            program.CopyTo(image, 0);
            var gsu = new Gsu(image, ram ?? new byte[0x8000]);
            gsu.WriteRegister(Clsr, 0x01);
            gsu.WriteRegister(R15Low, 0x00);
            gsu.WriteRegister(R15High, 0x00);
            gsu.Run(4000);
            return gsu;
        }

        // GETB/GETBH/GETBL/GETBS write the destination and nothing else.
        [Fact]
        public void Getb_leaves_the_flags_alone()
        {
            var gsu = Run(new byte[]
            {
                0xF1, 0x00, 0x80, // IWT R1,#$8000
                0x51,             // ADD R1      -> R0 = $8000, S set, Z clear
                0xFE, 0x40, 0x00, // IWT R14,#$0040 - starts the ROM fetch
                0xEF,             // GETB        -> R0 = $00
                0x00,             // STOP
            });

            Assert.Equal(0x0000, gsu.R[0]);
            Assert.Equal(FlagS, gsu.DebugSfr & (FlagS | FlagZ));
        }

        // LDW/LDB likewise - this one cost the intro camera its whole investigation.
        [Fact]
        public void Ldw_leaves_the_flags_alone()
        {
            var gsu = Run(new byte[]
            {
                0xF1, 0x00, 0x80, // IWT R1,#$8000
                0x51,             // ADD R1      -> R0 = $8000, S set, Z clear
                0xF1, 0x04, 0x00, // IWT R1,#$0004
                0x41,             // LDW (R1)    -> R0 = $0000 from cleared RAM
                0x00,             // STOP
            });

            Assert.Equal(0x0000, gsu.R[0]);
            Assert.Equal(FlagS, gsu.DebugSfr & (FlagS | FlagZ));
        }

        // A 16-bit access pairs address with address ^ 1 - see Venus_SuperFX.md §5.1a.
        [Fact]
        public void A_word_load_from_an_odd_address_pairs_with_the_byte_below_it()
        {
            byte[] ram = new byte[0x8000];
            ram[0x0010] = 0x11;
            ram[0x0011] = 0x22;
            ram[0x0012] = 0x33;

            var gsu = Run(new byte[]
            {
                0xF1, 0x11, 0x00, // IWT R1,#$0011 - odd
                0x41,             // LDW (R1)
                0x00,             // STOP
            }, ram: ram);

            Assert.Equal(0x1122, gsu.R[0]);
        }

        [Fact]
        public void A_word_store_to_an_odd_address_pairs_the_same_way()
        {
            byte[] ram = new byte[0x8000];

            var gsu = Run(new byte[]
            {
                0xF0, 0x34, 0x12, // IWT R0,#$1234
                0xF1, 0x11, 0x00, // IWT R1,#$0011 - odd
                0x31,             // STW (R1)
                0x00,             // STOP
            }, ram: ram);

            Assert.Equal(0x34, gsu.DebugRam[0x0011]);
            Assert.Equal(0x12, gsu.DebugRam[0x0010]);
            Assert.Equal(0x00, gsu.DebugRam[0x0012]);
        }

        // An even address is the case that never told the two apart.
        [Fact]
        public void A_word_load_from_an_even_address_is_unchanged()
        {
            byte[] ram = new byte[0x8000];
            ram[0x0010] = 0x11;
            ram[0x0011] = 0x22;

            var gsu = Run(new byte[]
            {
                0xF1, 0x10, 0x00, // IWT R1,#$0010
                0x41,             // LDW (R1)
                0x00,             // STOP
            }, ram: ram);

            Assert.Equal(0x2211, gsu.R[0]);
        }

        // With WITH still standing the TO would be a MOVE and R2 would read $0005.
        [Fact]
        public void An_alt_prefix_clears_a_pending_with()
        {
            var gsu = Run(new byte[]
            {
                0xF1, 0x05, 0x00, // IWT R1,#$0005
                0x21,             // WITH R1
                0x3D,             // ALT1
                0x12,             // TO R2
                0x01,             // NOP - runs with dreg = R2, writes nothing
                0x00,             // STOP
            });

            Assert.Equal(0x0000, gsu.R[2]);
        }
    }
}
