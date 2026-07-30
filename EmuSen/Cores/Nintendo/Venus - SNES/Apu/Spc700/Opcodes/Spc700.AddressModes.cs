using System;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    public partial class Spc700
    {
        // --- Addressing Modes ---

        private ushort AddrImplied() { return 0; }

        // Direct-page 16-bit accesses (CMPW/ADDW/SUBW/MOVW/INCW/DECW dp,
        // and the pointer fetch in [dp]+Y indirect-indexed addressing)
        // read/write two consecutive bytes, but real hardware wraps the
        // high byte's offset within the SAME 256-byte page rather than
        // spilling into the next one - e.g. dp=$FF, P=0 reads its high
        // byte from $00, not $100. AddrDirectPageIndexedXIndirect (just
        // below) already gets this right by wrapping the offset as a
        // byte before adding the page base back in; every direct-page
        // word op below this comment previously used a plain
        // `address + 1`, which is correct for PC-relative and absolute
        // addressing (no page to wrap within) but wrong for an address
        // that's already page-based - confirmed via the TomHarte/
        // ProcessorTests spc700 ground-truth suite, where every failure
        // left unexplained by this project's memory-mapped $F0-FF I/O
        // registers (a separate, expected divergence from that suite's
        // flat-RAM model) turned out to be exactly this: a direct-page
        // word op whose test case happened to use a dp offset near a
        // page boundary ($xFF/$x00).
        private static ushort DpWrapNextByte(ushort address)
        {
            return (ushort)((address & 0xFF00) | (byte)(address + 1));
        }

        // --- Addressing Modes ---
        private ushort AddrIndX() 
        {
            // If the P flag is set, the direct page is 0x0100. Otherwise, it's 0x0000.
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            
            // Add the X register to the base to get the effective address
            return (ushort)(dpBase + X);
        }
        
        private ushort AddrDirectPageIndexedXIndirect()
        {
            byte dpOffset = Read8(PC);
            PC++;
    
            byte pointerAddr = (byte)(dpOffset + X);
            byte low = Read8((ushort)(pointerAddr + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000)));
            byte high = Read8((ushort)((byte)(pointerAddr + 1) + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000)));
    
            return (ushort)((high << 8) | low);
        }
        
        private ushort AddrDirectPageX()
        {
            byte dpOffset = Read8(PC);
            PC++;
            
            byte wrappedOffset = (byte)(dpOffset + X);
            
            return (ushort)(wrappedOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
        }

        // d+Y: same as d+X above but indexed by Y instead of X. Used by the
        // handful of MOV opcodes that index by Y on the direct page (MOV
        // d+Y,X and MOV X,d+Y) rather than X.
        private ushort AddrDirectPageY()
        {
            byte dpOffset = Read8(PC);
            PC++;

            byte wrappedOffset = (byte)(dpOffset + Y);

            return (ushort)(wrappedOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
        }

        // m.b operand packing - see Venus_APU.md §2.3.
        private ushort AddrMemBit()
        {
            byte low = Read8(PC);
            byte high = Read8((ushort)(PC + 1));
            PC += 2;
            return (ushort)((high << 8) | low);
        }

        private ushort AddrAbsoluteIndexedX()
        {
            byte low = Read8(PC);
            byte high = Read8((ushort)(PC + 1));
            PC += 2;
            ushort baseAddress = (ushort)((high << 8) | low);
            
            return (ushort)(baseAddress + X);
        }

        private ushort AddrAbsoluteIndexedY()
        {
            byte low = Read8(PC);
            byte high = Read8((ushort)(PC + 1));
            PC += 2;
            ushort baseAddress = (ushort)((high << 8) | low);

            return (ushort)(baseAddress + Y);
        }

        private ushort AddrAbsolute()
        {
            byte low = Read8(PC);
            byte high = Read8((ushort)(PC + 1));
            PC += 2;
            return (ushort)((high << 8) | low);
        }
        
        private ushort AddrImmediate() 
        { 
            ushort address = PC;
            PC++;
            return address; 
        }

        private ushort AddrRelative() 
        { 
            ushort address = PC;
            PC++;
            return address; 
        }

        private ushort AddrIndirectX() 
        { 
            return (ushort)(X + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000)); 
        }

        private ushort AddrImmediateToDirectPage() 
        { 
            ushort address = PC;
            PC += 2; 
            return address; 
        }

        private ushort AddrDirectPage()
        {
            byte dpOffset = Read8(PC);
            PC++;
            return (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
        }

        private ushort AddrDirectIndirectIndexedY()
        {
            byte dpOffset = Read8(PC);
            PC++;
            ushort dpAddress = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
            
            byte low = Read8(dpAddress);
            byte high = Read8(DpWrapNextByte(dpAddress));
            ushort baseAddress = (ushort)((high << 8) | low);
            
            return (ushort)(baseAddress + Y);
        }

        private ushort AddrAbsoluteIndirectX()
        {
            byte low = Read8(PC);
            byte high = Read8((ushort)(PC + 1));
            PC += 2;
            ushort baseAddr = (ushort)((high << 8) | low);
            
            ushort ptr = (ushort)(baseAddr + X);
            
            byte targetLow = Read8(ptr);
            byte targetHigh = Read8((ushort)(ptr + 1));
            
            return (ushort)((targetHigh << 8) | targetLow);
        }

    }
}
