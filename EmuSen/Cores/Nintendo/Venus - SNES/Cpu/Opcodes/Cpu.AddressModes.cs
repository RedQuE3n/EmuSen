using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    public partial class Cpu
    {
        // --- Addressing Modes ---

        private uint AddrImplied() { return 0; }

        private uint AddrStackRelative()
        {
            // Fetch the 8-bit offset from the instruction
            byte offset = Fetch8();
            
            // Stack Relative addressing always accesses Bank 0. 
            // The effective address is the Stack Pointer + offset.
            return (uint)((S + offset) & 0xFFFF);
        }

        // (sr,S),Y: the 16-bit pointer lives on the stack (always Bank 0, same
        // as plain stack-relative above), then gets combined with DB and
        // indexed by Y - same combine-then-index pattern as (dp),Y
        // (AddrDirectIndirectY below). Verified against the documented 65816
        // opcode matrix (oxyron.de, cross-checked against softpixel's table)
        // for the eight 0x_3 opcodes that use it: ORA/AND/EOR/ADC/STA/LDA/
        // CMP/SBC (sr,S),Y.
        // Real 65816 hardware: any direct-page addressing mode costs one
        // extra cycle when D's low byte is nonzero (the CPU has to add the
        // 8-bit offset to a non-page-aligned D and can't just concatenate
        // bytes for free). Reported via _addrModeExtraCycles - see its
        // field comment in Cpu.cs - rather than a per-opcode table entry,
        // since it's a property of the addressing mode itself, not the
        // opcode using it.
        private void ChargeDirectPagePenalty()
        {
            if ((D & 0xFF) != 0) _addrModeExtraCycles++;
        }

        // Real 65816 hardware: indexed addressing that carries out of the
        // low byte of the 16-bit offset (i.e. crosses a page) costs one
        // extra cycle - the CPU speculatively reads the wrong page first,
        // then re-reads once it notices the carry. Approximated here as
        // applying unconditionally on a crossing regardless of 8-bit vs
        // 16-bit index width or read-vs-write instruction (the real
        // hardware rule is somewhat narrower - e.g. 16-bit index and pure
        // stores don't always take it), which is a known, accepted
        // simplification rather than threading opcode-level read/write and
        // index-width flags through every addressing mode for it.
        private void ChargePageCrossingPenalty(uint baseAddr, int index)
        {
            if ((baseAddr & 0xFF00) != ((baseAddr + index) & 0xFF00)) _addrModeExtraCycles++;
        }

        private uint AddrStackRelativeIndirectY()
        {
            byte offset = Fetch8();
            ushort sAddr = (ushort)((S + offset) & 0xFFFF);

            uint low = _bus.Read8(sAddr);
            uint high = _bus.Read8((uint)((sAddr + 1) & 0xFFFF));

            uint baseAddr = ((uint)DB << 16) | (high << 8) | low;
            return (baseAddr + Y) & 0xFFFFFF;
        }

        private uint AddrBlockMove()
        {
            // The instruction format is: Opcode, Destination Bank, Source Bank
            byte destBank = Fetch8();
            byte srcBank = Fetch8();
            
            // Pack them into a single uint so our Operate method can easily extract them
            return (uint)((destBank << 8) | srcBank);
        }

        private uint AddrDirectIndirectY()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            ushort dpAddr = (ushort)((D + dpOffset) & 0xFFFF);

            // Read the 16-bit pointer from the Direct Page
            uint low = _bus.Read8(dpAddr);
            uint high = _bus.Read8((uint)((dpAddr + 1) & 0xFFFF));

            // Combine with Data Bank (DB), then add Y
            uint baseAddr = ((uint)DB << 16) | (high << 8) | low;
            ChargePageCrossingPenalty(baseAddr, Y);
            return (baseAddr + Y) & 0xFFFFFF;
        }

        private uint AddrDirectIndirectX()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            ushort dpAddr = (ushort)((D + dpOffset + X) & 0xFFFF);

            uint low = _bus.Read8(dpAddr);
            uint high = _bus.Read8((uint)((dpAddr + 1) & 0xFFFF));

            return ((uint)DB << 16) | (high << 8) | low;
        }

        private uint AddrDirectIndirect()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            ushort dpAddr = (ushort)((D + dpOffset) & 0xFFFF);

            uint low = _bus.Read8(dpAddr);
            uint high = _bus.Read8((uint)((dpAddr + 1) & 0xFFFF));

            return ((uint)DB << 16) | (high << 8) | low;
        }

        private uint AddrAbsolute()
        {
            ushort offset = Fetch16();
            return ((uint)DB << 16) | offset;
        }

        private uint AddrAbsoluteX()
        {
            ushort offset = Fetch16();
            uint baseAddr = ((uint)DB << 16) | offset;
            ChargePageCrossingPenalty(baseAddr, X);
            return (baseAddr + X) & 0xFFFFFF;
        }

        private uint AddrAbsoluteY()
        {
            ushort offset = Fetch16();
            uint baseAddr = ((uint)DB << 16) | offset;
            ChargePageCrossingPenalty(baseAddr, Y);
            return (baseAddr + Y) & 0xFFFFFF;
        }

        private uint AddrAbsolutePB()
        {
            ushort offset = Fetch16();
            return ((uint)PB << 16) | offset;
        }

        private uint AddrAbsoluteLong()
        {
            ushort offset = Fetch16();
            byte bank = Fetch8();
            return ((uint)bank << 16) | offset;
        }

        private uint AddrAbsoluteLongX()
        {
            ushort offset = Fetch16();
            byte bank = Fetch8();
            uint baseAddr = ((uint)bank << 16) | offset;
            return (baseAddr + X) & 0xFFFFFF;
        }

        private uint AddrAbsoluteIndirect()
        {
            // JMP (abs): 16-bit pointer lives in bank 0; the 16-bit target read from it
            // is combined with the current program bank.
            ushort ptr = Fetch16();
            ushort target = (ushort)(_bus.Read8(ptr) | (_bus.Read8((uint)((ptr + 1) & 0xFFFF)) << 8));
            return ((uint)PB << 16) | target;
        }

        private uint AddrAbsoluteIndirectLong()
        {
            // JML [abs]: 16-bit pointer lives in bank 0; a full 24-bit target
            // (offset low, offset high, bank) is read from it.
            ushort ptr = Fetch16();
            uint low = _bus.Read8(ptr);
            uint high = _bus.Read8((uint)((ptr + 1) & 0xFFFF));
            uint bank = _bus.Read8((uint)((ptr + 2) & 0xFFFF));
            return (bank << 16) | (high << 8) | low; // Safe, bank is already a uint
        }

        private uint AddrAbsoluteIndexedIndirect()
        {
            // JMP/JSR (abs,X): pointer address is (abs + X) inside the CURRENT program
            // bank, and the 16-bit target read from it also stays in the program bank.
            ushort offset = Fetch16();
            uint ptr = ((uint)PB << 16) | (uint)((offset + X) & 0xFFFF);
            uint ptrNext = ((uint)PB << 16) | (uint)((offset + X + 1) & 0xFFFF);
            ushort target = (ushort)(_bus.Read8(ptr) | (_bus.Read8(ptrNext) << 8));
            return ((uint)PB << 16) | target;
        }

        private uint AddrImmediateM()
        {
            uint address = ((uint)PB << 16) | PC;
            PC += (ushort)(IsMemory8Bit ? 1 : 2);
            return address;
        }

        private uint AddrImmediateX()
        {
            uint address = ((uint)PB << 16) | PC;
            PC += (ushort)(IsIndex8Bit ? 1 : 2);
            return address;
        }

        private uint AddrImmediate8()
        {
            uint address = ((uint)PB << 16) | PC;
            PC++;
            return address;
        }

        private uint AddrRelative()
        {
            sbyte offset = (sbyte)Fetch8();
            return ((uint)PB << 16) | (ushort)(PC + offset);
        }

        private uint AddrRelativeLong()
        {
            // BRL's offset is 16-bit signed (not 8-bit like BRA/AddrRelative), which
            // is exactly why it exists - to reach branch targets too far away for the
            // short form. Cast through short so the sign extends correctly before
            // adding to PC.
            short offset = (short)Fetch16();
            return ((uint)PB << 16) | (ushort)(PC + offset);
        }

        private uint AddrDirectPageX()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            return (uint)((D + dpOffset + X) & 0xFFFF);
        }

        private uint AddrDirectPageY()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            return (uint)((D + dpOffset + Y) & 0xFFFF);
        }

        private uint AddrDirectPage()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            return (uint)((D + dpOffset) & 0xFFFF);
        }

        private uint AddrDirectIndirectLongY()
        {
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            ushort dpAddr = (ushort)((D + dpOffset) & 0xFFFF);

            uint low = _bus.Read8(dpAddr);
            uint high = _bus.Read8((uint)((dpAddr + 1) & 0xFFFF));
            uint bank = _bus.Read8((uint)((dpAddr + 2) & 0xFFFF));

            uint baseAddr = (bank << 16) | (high << 8) | low; // Safe, bank is already a uint

            return (baseAddr + Y) & 0xFFFFFF;
        }

        private uint AddrDirectIndirectLong()
        {
            // [dp]: 24-bit pointer is read from the direct page, no Y indexing
            ChargeDirectPagePenalty();
            byte dpOffset = Fetch8();
            ushort dpAddr = (ushort)((D + dpOffset) & 0xFFFF);

            uint low = _bus.Read8(dpAddr);
            uint high = _bus.Read8((uint)((dpAddr + 1) & 0xFFFF));
            uint bank = _bus.Read8((uint)((dpAddr + 2) & 0xFFFF));

            return (bank << 16) | (high << 8) | low; // Safe, bank is already a uint
        }

    }
}
