namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Jumps, subroutine linkage, the software interrupt, and the do-nothings.
    public partial class Cpu
    {
        // The byte after the opcode is fetched and discarded, which is why BRK is a two-byte instruction.
        private void OpBRK()
        {
            Read(PC++);
            Push((byte)(PC >> 8));
            Push((byte)PC);
            Push((byte)(P | 0x30));
            SetFlag(CpuFlags.I, true);
            PC = ReadVector(IrqVector);
        }

        private void OpRTI()
        {
            ConsumeImplied();
            byte pulled = PullWithDummy();
            P = (byte)((pulled | (byte)CpuFlags.U) & ~(byte)CpuFlags.B);

            byte lo = Pull();
            byte hi = Pull();
            PC = (ushort)(lo | (hi << 8));
        }

        // JSR pushes the address of its own last byte, which is why RTS has to add one back.
        private void OpJSR()
        {
            byte lo = Read(PC++);
            Read((ushort)(0x0100 | S));
            Push((byte)(PC >> 8));
            Push((byte)PC);
            byte hi = Read(PC);
            PC = (ushort)(lo | (hi << 8));
        }

        private void OpRTS()
        {
            ConsumeImplied();
            byte lo = PullWithDummy();
            byte hi = Pull();
            PC = (ushort)(lo | (hi << 8));
            Read(PC);
            PC++;
        }

        private void OpJMPIndirect()
        {
            ushort pointer = AddrAbsolute();
            byte lo = Read(pointer);
            // The pointer's high byte never carries, so 0x02FF reads its top byte from 0x0200.
            byte hi = Read((ushort)((pointer & 0xFF00) | ((pointer + 1) & 0x00FF)));
            PC = (ushort)(lo | (hi << 8));
        }

        // A KIL/JAM opcode wedges the chip after re-reading the operand twice - see Moon_CPU.md §6.4.
        private void OpJAM()
        {
            Read(PC);
            Read(PC);
            PC--;
            Jammed = true;
        }
    }
}
