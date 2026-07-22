using System;
using EmuSen.Debug;

namespace EmuSen.Apu
{
    public partial class Spc700
    {
        // --- Addressing Modes ---

        private ushort AddrImplied() { return 0; }

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

        // m.b: the operand for AND1/OR1/EOR1/MOV1/NOT1 packs a 13-bit
        // absolute memory address (bits 0-12) and a 3-bit bit index (bits
        // 13-15) into one 16-bit operand word - verified against documented
        // hardware encoding (sneslab.net's MOV1 page: "the low 13 bits of
        // the operand byte specify an absolute address, the high 3 bits
        // specify which bit"). That's a perfect fit for the existing
        // Func<ushort> AddrMode signature, so no changes needed there -
        // callers just unpack it: `addr = raw & 0x1FFF`, `bit = (raw >> 13)
        // & 0x07`. See the Op*1 handlers in Spc700.Opcodes.cs.
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
            byte high = Read8((ushort)(dpAddress + 1));
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
