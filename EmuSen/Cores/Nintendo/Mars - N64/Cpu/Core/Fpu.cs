using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The coprocessor-1 register file and its two control registers, with no arithmetic - see Mars_Fpu.md.
    public sealed partial class Cpu
    {
        public const int FpuImplementationRegister = 0;
        public const int FpuControlStatusRegister = 31;

        // What FCR0 reads on this part, and it ignores every write - see Mars_Fpu.md §4.
        public const uint FpuImplementation = 0x0000_0A00;

        // Bits 18-22 and 25-31 read back as zero however they were written - see Mars_Fpu.md §4.1.
        public const uint FcsrWritableMask = 0x0183_FFFF;

        public const uint FcsrMaskableCauses = 0x0001_F000;
        public const uint FcsrCauseUnimplemented = 1u << 17;
        public const uint FcsrEnables = 0x0000_0F80;

        // Each enable sits five bits below the cause it answers, which makes the overlap test a shift.
        private const int FcsrCauseToEnableShift = 5;

        public const ulong StatusCop1Usable = 1UL << 29;

        // Clear, the file is sixteen paired registers rather than thirty-two - see Mars_Fpu.md §2.
        public const ulong StatusFpuFullMode = 1UL << 26;

        public readonly ulong[] Fpr = new ulong[32];

        // Zero out of a hard reset, which is the only starting value the corpus will vouch for - see §4.2.
        public uint Fcsr;

        private bool FpuFullMode => (Cop0[StatusRegister] & StatusFpuFullMode) != 0;

        // In half mode a wide access loses the index's low bit, so odd and even name the same pair - see §2.
        private ulong ReadFpuWide(int index) => Fpr[FpuFullMode ? index : index & ~1];

        private void WriteFpuWide(int index, ulong value) => Fpr[FpuFullMode ? index : index & ~1] = value;

        // In half mode an odd index names the upper half of its pair; in full mode there are no halves - see §2.
        private uint ReadFpuWord(int index)
        {
            if (FpuFullMode) return (uint)Fpr[index];

            return (index & 1) != 0 ? (uint)(Fpr[index & ~1] >> 32) : (uint)Fpr[index & ~1];
        }

        // A word write leaves the other half of the register standing, which is not what a naive store does.
        private void WriteFpuWord(int index, uint value)
        {
            if (FpuFullMode)
            {
                Fpr[index] = (Fpr[index] & 0xFFFF_FFFF_0000_0000UL) | value;
                return;
            }

            int paired = index & ~1;

            Fpr[paired] = (index & 1) != 0
                ? (Fpr[paired] & 0x0000_0000_FFFF_FFFFUL) | ((ulong)value << 32)
                : (Fpr[paired] & 0xFFFF_FFFF_0000_0000UL) | value;
        }

        // Only two control registers exist; the rest read as zero, which is an assumption - see Mars_Fpu.md §4.3.
        private uint ReadFpuControl(int index) => index switch
        {
            FpuImplementationRegister => FpuImplementation,
            FpuControlStatusRegister => Fcsr,
            _ => 0,
        };

        // The write lands before the exception is considered, so a handler reads what the program wrote - see §4.4.
        private void WriteFpuControl(int index, uint value)
        {
            if (index != FpuControlStatusRegister) return;

            Fcsr = value & FcsrWritableMask;

            if (EnabledCause() != 0) throw Raise(ExceptionCode.FloatingPoint, CurrentPc);
        }

        // The unimplemented cause has no enable of its own: set by any route, it fires - see Mars_Fpu.md §4.4.
        private uint EnabledCause() =>
            ((Fcsr >> FcsrCauseToEnableShift) & Fcsr & FcsrEnables) | (Fcsr & FcsrCauseUnimplemented);

        // An operation that the part does not implement clears the maskable causes and fires regardless of them - see §5.
        private CpuException RaiseUnimplementedOperation()
        {
            Fcsr = (Fcsr & ~FcsrMaskableCauses) | FcsrCauseUnimplemented;

            return Raise(ExceptionCode.FloatingPoint, CurrentPc);
        }

        // Usability is answered before the decode, so an unusable coprocessor answers for its reserved forms too - see §3.
        private void RequireCop1()
        {
            if ((Cop0[StatusRegister] & StatusCop1Usable) != 0) return;

            throw Raise(ExceptionCode.CoprocessorUnusable, CurrentPc, coprocessor: 1);
        }
    }
}
