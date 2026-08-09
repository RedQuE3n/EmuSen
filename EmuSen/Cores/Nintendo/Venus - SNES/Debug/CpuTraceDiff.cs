using EmuSen.Cores.Debug;
using System.Collections.Generic;
using System.IO;

namespace EmuSen.Cores.Nintendo.Venus.Debug
{
    // The S-CPU half of TraceDiff: the ESCT record layout and how to read it - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
    public static class CpuTraceDiff
    {
        public readonly record struct Step(uint Addr, byte Opcode, byte Kind,
            ushort A, ushort X, ushort Y, ushort S, ushort D, byte Db, byte P, bool E, uint Cost)
            : ITraceStep<Step>
        {
            public bool SameRegisters(Step o) =>
                A == o.A && X == o.X && Y == o.Y && S == o.S && D == o.D && Db == o.Db && P == o.P && E == o.E;

            public string Registers => $"A={A:X4} X={X:X4} Y={Y:X4} S={S:X4} D={D:X4} DB={Db:X2} P={P:X2}{(E ? " E" : "")}";
        }

        public static Step[] Parse(byte[] blob)
        {
            if (blob.Length < CpuBinaryTrace.HeaderBytes)
            {
                throw new InvalidDataException("Not a CPU trace: file is shorter than its header.");
            }
            for (int i = 0; i < CpuBinaryTrace.Magic.Length; i++)
            {
                if (blob[i] != CpuBinaryTrace.Magic[i])
                {
                    throw new InvalidDataException("Not a CPU trace, or written by a different record layout (bad ESCT header).");
                }
            }

            int count = (blob.Length - CpuBinaryTrace.HeaderBytes) / CpuBinaryTrace.RecordBytes;
            var steps = new Step[count];
            for (int i = 0; i < count; i++)
            {
                int o = CpuBinaryTrace.HeaderBytes + i * CpuBinaryTrace.RecordBytes;
                steps[i] = new Step(
                    (uint)(blob[o] | (blob[o + 1] << 8) | (blob[o + 2] << 16)),
                    blob[o + 4], blob[o + 5],
                    (ushort)(blob[o + 6] | (blob[o + 7] << 8)),
                    (ushort)(blob[o + 8] | (blob[o + 9] << 8)),
                    (ushort)(blob[o + 10] | (blob[o + 11] << 8)),
                    (ushort)(blob[o + 12] | (blob[o + 13] << 8)),
                    (ushort)(blob[o + 14] | (blob[o + 15] << 8)),
                    blob[o + 16], blob[o + 17], blob[o + 18] != 0,
                    (uint)(blob[o + 20] | (blob[o + 21] << 8) | (blob[o + 22] << 16) | (blob[o + 23] << 24)));
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
