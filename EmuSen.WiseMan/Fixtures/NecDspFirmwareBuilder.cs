using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;

namespace EmuSen.WiseMan.Fixtures
{
    // Hand-assembles uPD7725 firmware for the DSP tests. Nintendo's real
    // firmware is copyrighted and undumped here, and the chip is useless
    // without *some* program, so the tests supply their own. See Venus_NecDSP.md §5.
    public static class NecDspFirmwareBuilder
    {
        // Destination / source operand encodings - see Venus_NecDSP.md §5.3.
        public const byte DestNone = 0x00;
        public const byte DestA = 0x01;
        public const byte DestB = 0x02;
        public const byte DestDp = 0x04;
        public const byte DestRp = 0x05;
        public const byte DestDr = 0x06;
        public const byte DestSr = 0x07;
        public const byte DestK = 0x0A;
        public const byte DestL = 0x0D;
        public const byte DestRam = 0x0F;

        public const byte SrcA = 0x01;
        public const byte SrcB = 0x02;
        public const byte SrcDataRom = 0x06;
        public const byte SrcDrNoFlag = 0x09;
        public const byte SrcSr = 0x0A;
        public const byte SrcK = 0x0D;
        public const byte SrcRam = 0x0F;

        // ALU operation codes - see Venus_NecDSP.md §5.2.
        public const byte AluOr = 0x01;
        public const byte AluAnd = 0x02;
        public const byte AluXor = 0x03;
        public const byte AluSub = 0x04;
        public const byte AluAdd = 0x05;
        public const byte AluShr = 0x0B;
        public const byte AluShl = 0x0C;
        public const byte AluSwap = 0x0F;

        // Which operand the ALU pairs with the accumulator.
        public const int PSelectRam = 0;
        public const int PSelectSource = 1;
        public const int PSelectM = 2;
        public const int PSelectN = 3;

        // LD: a 16-bit immediate straight into one destination.
        public static uint Ld(ushort value, byte dest) => 0xC00000u | ((uint)value << 6) | dest;

        // OP: optional ALU work, then source -> destination.
        public static uint Op(byte alu = 0, byte source = 0, byte dest = DestNone, int accumulator = 0, int pSelect = PSelectSource)
        {
            return ((uint)pSelect << 20) | ((uint)alu << 16) | ((uint)accumulator << 15) | ((uint)source << 4) | dest;
        }

        // RT: the same encoding as OP, but the chip also pops the stack after it.
        public static uint Rt(uint op) => 0x400000u | op;

        // JP: an unconditional or condition-coded branch inside one 8KB half.
        public static uint Jp(ushort jumpType, ushort target) => 0x800000u | ((uint)jumpType << 13) | ((uint)(target & 0x7FF) << 2) | (uint)((target >> 11) & 0x03);

        public const ushort JumpAlways = 0x100;
        public const ushort JumpCall = 0x140;
        public const ushort JumpIfAccAZero = 0x08A;
        public const ushort JumpIfRqmClear = 0x0BC;
        public const ushort JumpIfRqmSet = 0x0BE;

        // Assembles <program> at word 0 of a DSP-1-shaped firmware image,
        // padding the rest with an unconditional branch back to itself.
        public static NecDspFirmware Dsp1(params uint[] program) => Dsp1WithData(program, new ushort[0]);

        public static NecDspFirmware Dsp1WithData(uint[] program, ushort[] dataRom)
        {
            NecDspProfile profile = NecDspProfile.For(NecDspVariant.Dsp1);
            byte[] blob = new byte[profile.FirmwareBytes];

            int words = profile.ProgramBytes / 3;
            for (int i = 0; i < words; i++)
            {
                // Anything past the supplied program spins in place, so a
                // runaway PC can't wander into whatever follows.
                uint opcode = i < program.Length ? program[i] : Jp(JumpAlways, (ushort)i);
                blob[i * 3] = (byte)opcode;
                blob[(i * 3) + 1] = (byte)(opcode >> 8);
                blob[(i * 3) + 2] = (byte)(opcode >> 16);
            }

            for (int i = 0; i < dataRom.Length && (profile.ProgramBytes + (i * 2) + 1) < blob.Length; i++)
            {
                blob[profile.ProgramBytes + (i * 2)] = (byte)dataRom[i];
                blob[profile.ProgramBytes + (i * 2) + 1] = (byte)(dataRom[i] >> 8);
            }

            return NecDspFirmware.FromBlob(profile, blob)!;
        }

        public static byte[] Blob(NecDspFirmware firmware)
        {
            NecDspProfile profile = NecDspProfile.For(NecDspVariant.Dsp1);
            byte[] blob = new byte[profile.FirmwareBytes];
            Array.Copy(firmware.Program, blob, firmware.Program.Length);
            for (int i = 0; i < firmware.DataRom.Length; i++)
            {
                blob[profile.ProgramBytes + (i * 2)] = (byte)firmware.DataRom[i];
                blob[profile.ProgramBytes + (i * 2) + 1] = (byte)(firmware.DataRom[i] >> 8);
            }
            return blob;
        }
    }
}
