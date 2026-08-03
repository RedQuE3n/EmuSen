namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Effective-address computation, including the dummy reads that make the cycle counts real - see Moon_CPU.md §3.
    public partial class Cpu
    {
        // An implied or accumulator opcode still fetches the next byte, then throws it away.
        private void ConsumeImplied() => Read(PC);

        private ushort AddrImmediate() => PC++;

        private ushort AddrZeroPage() => Read(PC++);

        private ushort AddrZeroPageX() => AddrZeroPageIndexed(X);

        private ushort AddrZeroPageY() => AddrZeroPageIndexed(Y);

        // The index is added inside the zero page, so the address wraps at 0xFF rather than carrying.
        private ushort AddrZeroPageIndexed(byte index)
        {
            byte pointer = Read(PC++);
            Read(pointer);
            return (byte)(pointer + index);
        }

        private ushort AddrAbsolute()
        {
            byte lo = Read(PC++);
            byte hi = Read(PC++);
            return (ushort)(lo | (hi << 8));
        }

        private ushort AddrAbsoluteX(bool alwaysFixup) => AddrAbsoluteIndexed(X, alwaysFixup);

        private ushort AddrAbsoluteY(bool alwaysFixup) => AddrAbsoluteIndexed(Y, alwaysFixup);

        // <alwaysFixup> is set for writes and RMW, which pay the fixup cycle even without a page cross.
        private ushort AddrAbsoluteIndexed(byte index, bool alwaysFixup)
        {
            byte lo = Read(PC++);
            byte hi = Read(PC++);
            ushort baseAddress = (ushort)(lo | (hi << 8));
            ushort effective = (ushort)(baseAddress + index);

            if (alwaysFixup || (effective & 0xFF00) != (baseAddress & 0xFF00))
            {
                Read((ushort)((baseAddress & 0xFF00) | (effective & 0x00FF)));
            }

            return effective;
        }

        private ushort AddrIndexedIndirect()
        {
            byte pointer = Read(PC++);
            Read(pointer);
            byte indexed = (byte)(pointer + X);
            byte lo = Read(indexed);
            byte hi = Read((byte)(indexed + 1));
            return (ushort)(lo | (hi << 8));
        }

        // <alwaysFixup> as above; the pointer itself wraps inside the zero page.
        private ushort AddrIndirectIndexed(bool alwaysFixup)
        {
            byte pointer = Read(PC++);
            byte lo = Read(pointer);
            byte hi = Read((byte)(pointer + 1));
            ushort baseAddress = (ushort)(lo | (hi << 8));
            ushort effective = (ushort)(baseAddress + Y);

            if (alwaysFixup || (effective & 0xFF00) != (baseAddress & 0xFF00))
            {
                Read((ushort)((baseAddress & 0xFF00) | (effective & 0x00FF)));
            }

            return effective;
        }

        // The high byte of the base address is what the unstable stores AND against - see Moon_CPU.md §6.3.
        private ushort AddrAbsoluteIndexedUnstable(byte index, out byte baseHigh, out bool crossed)
        {
            byte lo = Read(PC++);
            byte hi = Read(PC++);
            ushort baseAddress = (ushort)(lo | (hi << 8));
            ushort effective = (ushort)(baseAddress + index);

            baseHigh = hi;
            crossed = (effective & 0xFF00) != (baseAddress & 0xFF00);
            Read((ushort)((baseAddress & 0xFF00) | (effective & 0x00FF)));
            return effective;
        }

        private ushort AddrIndirectIndexedUnstable(out byte baseHigh, out bool crossed)
        {
            byte pointer = Read(PC++);
            byte lo = Read(pointer);
            byte hi = Read((byte)(pointer + 1));
            ushort baseAddress = (ushort)(lo | (hi << 8));
            ushort effective = (ushort)(baseAddress + Y);

            baseHigh = hi;
            crossed = (effective & 0xFF00) != (baseAddress & 0xFF00);
            Read((ushort)((baseAddress & 0xFF00) | (effective & 0x00FF)));
            return effective;
        }

        // The real chip writes the untouched value back before the modified one - see Moon_CPU.md §3.3.
        private byte ReadModifyWriteFetch(ushort address)
        {
            byte value = Read(address);
            Write(address, value);
            return value;
        }
    }
}
