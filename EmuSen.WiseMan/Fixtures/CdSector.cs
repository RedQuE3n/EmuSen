namespace EmuSen.WiseMan.Fixtures
{
    // A raw 2,352-byte CD-ROM sector as ECMA-130 §14 and Annex A write it, with psx-spx's mode 2 forms - see Diamond_Native.md §5.2.
    public static class CdSector
    {
        public const int Raw = 2352, User = 2048, Form2User = 2324;

        // Track 1's data starts after the two-second pregap, so a sector's address is its block number plus 150.
        public const int Pregap = 150;

        private static readonly byte[] Sync = { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 };
        private static readonly uint[] EdcTable = new uint[256];
        private static readonly byte[] Exp = new byte[512], Log = new byte[256];

        static CdSector()
        {
            for (uint i = 0; i < 256; i++)
            {
                uint x = i;
                for (int j = 0; j < 8; j++) x = (x & 1) != 0 ? (x >> 1) ^ 0xD801_8001 : x >> 1;
                EdcTable[i] = x;
            }
            int a = 1;
            for (int i = 0; i < 255; i++)
            {
                Exp[i] = Exp[i + 255] = (byte)a;
                Log[a] = (byte)i;
                a <<= 1;
                if ((a & 0x100) != 0) a ^= 0x11D;
            }
        }

        public static byte[] Mode1(int block, ReadOnlySpan<byte> data)
        {
            var s = Header(block, 1);
            data[..Math.Min(User, data.Length)].CopyTo(s.AsSpan(16));
            PutEdc(s, 0, 2064);
            Ecc(s);
            return s;
        }

        public static byte[] Mode2Form1(int block, ReadOnlySpan<byte> data, byte submode = 0x08, byte file = 0, byte channel = 0)
        {
            var s = Header(block, 2);
            SubHeader(s, file, channel, (byte)(submode & ~0x20));
            data[..Math.Min(User, data.Length)].CopyTo(s.AsSpan(24));
            PutEdc(s, 16, 2072);
            // The header takes no part in a mode 2 sector's ECC (psx-spx "encode_sector").
            var header = s[12..16];
            Array.Clear(s, 12, 4);
            Ecc(s);
            header.CopyTo(s, 12);
            return s;
        }

        public static byte[] Mode2Form2(int block, ReadOnlySpan<byte> data, byte submode = 0x20, byte file = 0, byte channel = 0)
        {
            var s = Header(block, 2);
            SubHeader(s, file, channel, (byte)(submode | 0x20));
            data[..Math.Min(Form2User, data.Length)].CopyTo(s.AsSpan(24));
            PutEdc(s, 16, 2348);
            return s;
        }

        // The block number's address as BCD minutes, seconds and frames.
        public static (byte M, byte S, byte F) Msf(int block)
        {
            int a = block + Pregap;
            return (Bcd(a / 75 / 60), Bcd(a / 75 % 60), Bcd(a % 75));
        }

        public static uint Edc(ReadOnlySpan<byte> bytes)
        {
            uint crc = 0;
            foreach (byte b in bytes) crc = (crc >> 8) ^ EdcTable[(crc ^ b) & 0xFF];
            return crc;
        }

        // Annex A's P- and Q-parity over bytes 12 to 2,247 of a sector whose EDC is in place.
        public static void Ecc(byte[] s)
        {
            for (int plane = 0; plane < 2; plane++)
            {
                for (int np = 0; np < 43; np++)
                    Parity(s, plane, 26, m => 43 * m + np);
                for (int nq = 0; nq < 26; nq++)
                    Parity(s, plane, 45, m => m < 43 ? (44 * m + 43 * nq) % 1118 : 1118 + 26 * (m - 43) + nq);
            }
        }

        // Each P- and Q-vector's two checks of Annex A, the sum and the sum weighted by alpha's powers, which hold for an intact sector.
        public static int Syndromes(byte[] s)
        {
            int bad = 0;
            for (int plane = 0; plane < 2; plane++)
            {
                for (int np = 0; np < 43; np++)
                    bad += Check(s, plane, 26, m => 43 * m + np) ? 0 : 1;
                for (int nq = 0; nq < 26; nq++)
                    bad += Check(s, plane, 45, m => m < 43 ? (44 * m + 43 * nq) % 1118 : 1118 + 26 * (m - 43) + nq) ? 0 : 1;
            }
            return bad;
        }

        private static byte[] Header(int block, byte mode)
        {
            var s = new byte[Raw];
            Sync.CopyTo(s, 0);
            (s[12], s[13], s[14]) = Msf(block);
            s[15] = mode;
            return s;
        }

        private static void SubHeader(byte[] s, byte file, byte channel, byte submode)
        {
            s[16] = s[20] = file;
            s[17] = s[21] = channel;
            s[18] = s[22] = submode;
        }

        private static void PutEdc(byte[] s, int from, int to) => BitConverter.TryWriteBytes(s.AsSpan(to, 4), Edc(s.AsSpan(from, to - from)));

        private static byte Bcd(int v) => (byte)(v / 10 * 16 + v % 10);

        private static int At(int plane, int word) => 12 + 2 * word + plane;

        private static byte Mul(byte a, int power) => a == 0 ? (byte)0 : Exp[(Log[a] + power) % 255];

        // A vector's last two bytes from H = [1 ... 1; a^(n-1) ... a 1]: p0 + p1 = S0 and a*p0 + p1 = S1.
        private static void Parity(byte[] s, int plane, int n, Func<int, int> word)
        {
            byte s0 = 0, s1 = 0;
            for (int m = 0; m < n - 2; m++)
            {
                byte v = s[At(plane, word(m))];
                s0 ^= v;
                s1 ^= Mul(v, n - 1 - m);
            }
            byte d = (byte)(s0 ^ s1);
            byte p0 = d == 0 ? (byte)0 : Exp[(Log[d] + 255 - Log[3]) % 255];
            s[At(plane, word(n - 2))] = p0;
            s[At(plane, word(n - 1))] = (byte)(s0 ^ p0);
        }

        private static bool Check(byte[] s, int plane, int n, Func<int, int> word)
        {
            byte s0 = 0, s1 = 0;
            for (int m = 0; m < n; m++)
            {
                byte v = s[At(plane, word(m))];
                s0 ^= v;
                s1 ^= Mul(v, n - 1 - m);
            }
            return s0 == 0 && s1 == 0;
        }
    }
}
