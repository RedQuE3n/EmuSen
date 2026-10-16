using EmuSen.Crystal.Systems.Nes;
using EmuSen.Pharaoh;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic iNES image - never real game data. The NES counterpart of SyntheticRom.
    public static class SyntheticNesRom
    {
        public const int PrgBankSize = NesHeader.PrgBankSize;
        public const int ChrBankSize = NesHeader.ChrBankSize;

        // NOP fill, not zero: zero decodes as BRK, which vectors and misaligns a linear disassembly.
        public const byte Filler = 0xEA;

        // <patches> are (offset within PRG, bytes) pairs; PRG offset 0 is CPU $8000 on a 16K NROM.
        public static byte[] Build(
            int prgBanks = 1,
            int chrBanks = 1,
            int mapper = 0,
            NesMirroring mirroring = NesMirroring.Horizontal,
            bool battery = false,
            params (int Offset, byte[] Bytes)[] patches)
        {
            int prgLength = prgBanks * PrgBankSize;
            int chrLength = chrBanks * ChrBankSize;

            var image = new byte[NesHeader.Size + prgLength + chrLength];

            image[0] = (byte)'N';
            image[1] = (byte)'E';
            image[2] = (byte)'S';
            image[3] = 0x1A;
            image[4] = (byte)prgBanks;
            image[5] = (byte)chrBanks;

            byte flags6 = (byte)((mapper & 0x0F) << 4);
            if (mirroring == NesMirroring.Vertical) flags6 |= 0x01;
            if (mirroring == NesMirroring.FourScreen) flags6 |= 0x08;
            if (battery) flags6 |= 0x02;
            image[6] = flags6;
            image[7] = (byte)(mapper & 0xF0);

            var prg = new System.Span<byte>(image, NesHeader.Size, prgLength);
            prg.Fill(Filler);

            foreach (var (offset, bytes) in patches)
            {
                bytes.CopyTo(prg[offset..]);
            }

            // Reset lands at $8000, which is PRG offset 0 of the bank the board maps there.
            WriteVector(prg, 0xFFFC, 0x8000);
            WriteVector(prg, 0xFFFA, 0x8000);
            WriteVector(prg, 0xFFFE, 0x8000);

            return image;
        }

        // $C000-$FFFF is the last bank on every board here, and a 16K one mirrors its only bank there.
        private static void WriteVector(System.Span<byte> prg, int cpuAddress, int target)
        {
            int offset = prg.Length - PrgBankSize + (cpuAddress - 0xC000);
            prg[offset] = (byte)target;
            prg[offset + 1] = (byte)(target >> 8);
        }

        // Speaks the blargg $6000 protocol, so the test-ROM runner is provable with no third-party data.
        public static byte[] BuildProtocolRom(byte resultCode, string message = "OK")
        {
            var code = new Assembler();

            code.Store(NesTestRomProtocol.SignatureAddress, NesTestRomProtocol.Signature[0]);
            code.Store(NesTestRomProtocol.SignatureAddress + 1, NesTestRomProtocol.Signature[1]);
            code.Store(NesTestRomProtocol.SignatureAddress + 2, NesTestRomProtocol.Signature[2]);
            code.Store(NesTestRomProtocol.StatusAddress, NesTestRomProtocol.StatusRunning);

            for (int i = 0; i < message.Length; i++)
            {
                code.Store(NesTestRomProtocol.TextAddress + i, (byte)message[i]);
            }
            code.Store(NesTestRomProtocol.TextAddress + message.Length, 0x00);

            code.Delay();
            code.Store(NesTestRomProtocol.StatusAddress, resultCode);
            code.Spin();

            return Build(patches: (0, code.Bytes));
        }

        // Asks for a reset the first time through, then passes - PRG RAM $6100 is what remembers.
        public static byte[] BuildResetProtocolRom()
        {
            const int MarkerAddress = 0x6100;
            var code = new Assembler();

            code.LoadAbsolute(MarkerAddress);
            int branch = code.BranchIfNotZero();

            code.Store(NesTestRomProtocol.SignatureAddress, NesTestRomProtocol.Signature[0]);
            code.Store(NesTestRomProtocol.SignatureAddress + 1, NesTestRomProtocol.Signature[1]);
            code.Store(NesTestRomProtocol.SignatureAddress + 2, NesTestRomProtocol.Signature[2]);

            // Real ROMs hold the running status before asking, and the runner requires it - see §2.2.
            code.Store(NesTestRomProtocol.StatusAddress, NesTestRomProtocol.StatusRunning);
            code.Delay();
            code.Store(MarkerAddress, 0x01);
            code.Store(NesTestRomProtocol.StatusAddress, NesTestRomProtocol.StatusNeedsReset);
            code.Spin();

            // The reset leaves $81 in PRG RAM, so this half must republish $80 before its verdict.
            code.PatchBranch(branch);
            code.Store(NesTestRomProtocol.StatusAddress, NesTestRomProtocol.StatusRunning);
            code.Delay();
            code.Store(NesTestRomProtocol.StatusAddress, 0x00);
            code.Spin();

            return Build(patches: (0, code.Bytes));
        }

        // Just enough 6502 to write the protocol block; PRG offset 0 is CPU $8000 - see Moon_TestRoms.md §4.
        private sealed class Assembler
        {
            private readonly List<byte> _code = new();

            public byte[] Bytes => _code.ToArray();

            public void LoadImmediate(byte value) { _code.Add(0xA9); _code.Add(value); }

            public void LoadAbsolute(int address) { _code.Add(0xAD); AddAddress(address); }

            public void StoreAbsolute(int address) { _code.Add(0x8D); AddAddress(address); }

            public void Store(int address, byte value) { LoadImmediate(value); StoreAbsolute(address); }

            // Returns the operand index, for PatchBranch to fill in once the target is known.
            public int BranchIfNotZero()
            {
                _code.Add(0xD0);
                _code.Add(0x00);
                return _code.Count - 1;
            }

            public void PatchBranch(int operandIndex) =>
                _code[operandIndex] = (byte)(_code.Count - (operandIndex + 1));

            // Two nested 256-step counters, so the running status stays observable for about eleven frames.
            public void Delay()
            {
                _code.Add(0xA2); _code.Add(0x00);
                _code.Add(0xA0); _code.Add(0x00);

                int loop = _code.Count;
                _code.Add(0x88);
                BranchTo(loop);
                _code.Add(0xCA);
                BranchTo(loop);
            }

            public void Spin()
            {
                int here = _code.Count;
                _code.Add(0x4C);
                AddAddress(0x8000 + here);
            }

            private void BranchTo(int target)
            {
                _code.Add(0xD0);
                _code.Add((byte)(target - (_code.Count + 1)));
            }

            private void AddAddress(int address)
            {
                _code.Add((byte)address);
                _code.Add((byte)(address >> 8));
            }
        }

        // A core loads a ROM from a path, so a test that needs a whole core needs a real file.
        public static string WriteTemp(byte[] image)
        {
            string path = Path.Combine(Path.GetTempPath(), $"moon-{Guid.NewGuid():N}.nes");
            File.WriteAllBytes(path, image);
            return path;
        }
    }
}
