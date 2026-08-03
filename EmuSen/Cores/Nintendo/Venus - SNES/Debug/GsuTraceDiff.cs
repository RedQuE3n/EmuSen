using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // The GSU half of TraceDiff: the ESGT record layout and how to read it - see EmuSen_Debugging_Tools_Reference_v5.md §3.41.
    public static class GsuTraceDiff
    {
        // Z/Cy/S/Ov and the four prefix bits; Go, ROM-pending and IRQ are excluded - see §3.41.
        public const ushort ComparedSfrBits = 0x1F1E;

        // R15 is the address, and the two emulators label it by different conventions - see §3.41.
        public const int ComparedRegisters = 15;

        public readonly record struct Step(uint Addr, byte Opcode, ushort Sfr, byte Sreg, byte Dreg,
            ushort[] Regs, int RegBase, uint Cost) : ITraceStep<Step>
        {
            public byte Kind => 0;

            public ushort R(int i) => Regs[RegBase + i];

            public bool SameRegisters(Step o)
            {
                if ((Sfr & ComparedSfrBits) != (o.Sfr & ComparedSfrBits)) return false;
                if (Sreg != o.Sreg || Dreg != o.Dreg) return false;
                for (int i = 0; i < ComparedRegisters; i++)
                {
                    if (Regs[RegBase + i] != o.Regs[o.RegBase + i]) return false;
                }
                return true;
            }

            public string Registers
            {
                get
                {
                    var sb = new System.Text.StringBuilder();
                    for (int i = 0; i < ComparedRegisters; i++) sb.Append($"R{i}={R(i):X4} ");
                    return sb.Append($"SFR={Sfr:X4} s={Sreg:X1} d={Dreg:X1}").ToString();
                }
            }
        }

        public static Step[] Parse(byte[] blob)
        {
            if (blob.Length < GsuBinaryTrace.HeaderBytes)
            {
                throw new InvalidDataException("Not a GSU trace: file is shorter than its header.");
            }
            for (int i = 0; i < GsuBinaryTrace.Magic.Length; i++)
            {
                if (blob[i] != GsuBinaryTrace.Magic[i])
                {
                    throw new InvalidDataException("Not a GSU trace, or written by a different record layout (bad ESGT header).");
                }
            }

            int count = (blob.Length - GsuBinaryTrace.HeaderBytes) / GsuBinaryTrace.RecordBytes;

            // One flat register array for the whole stream; a ushort[16] per step would be all header.
            var regs = new ushort[count * 16];
            var steps = new Step[count];
            for (int i = 0; i < count; i++)
            {
                int o = GsuBinaryTrace.HeaderBytes + i * GsuBinaryTrace.RecordBytes;
                for (int k = 0; k < 16; k++)
                {
                    regs[i * 16 + k] = (ushort)(blob[o + 8 + k * 2] | (blob[o + 9 + k * 2] << 8));
                }
                steps[i] = new Step(
                    (uint)(blob[o] | (blob[o + 1] << 8) | (blob[o + 2] << 16)),
                    blob[o + 4],
                    (ushort)(blob[o + 6] | (blob[o + 7] << 8)),
                    (byte)(blob[o + 5] >> 4), (byte)(blob[o + 5] & 0x0F),
                    regs, i * 16,
                    (uint)(blob[o + 40] | (blob[o + 41] << 8) | (blob[o + 42] << 16) | (blob[o + 43] << 24)));
            }
            return steps;
        }

        public static Step[] Load(string path) => Parse(File.ReadAllBytes(path));

        public static List<TraceDiff.Node> Collapse(Step[] steps, int maxPeriod = TraceDiff.MaxLoopPeriod) =>
            TraceDiff.Collapse(steps, maxPeriod);

        public static TraceDiff.Result Compare(Step[] left, Step[] right, int maxFindings = 24) =>
            TraceDiff.Compare(left, right, maxFindings);

        public static string Report(TraceDiff.Result r, string leftLabel, string rightLabel) =>
            TraceDiff.Report(r, leftLabel, rightLabel);
    }
}
