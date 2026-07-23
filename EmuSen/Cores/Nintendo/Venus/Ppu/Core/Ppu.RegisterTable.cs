using System;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Video
{
    public partial class Ppu
    {
        private void BuildRegisterTable()
        {
            // The PPU maps from $2100 to $213F (64 registers total)
            _registers = new PpuRegister[0x40];

            for (int i = 0; i < 0x40; i++)
            {
                _registers[i] = new PpuRegister 
                { 
                    Name = $"UNK_21{i:X2}", 
                    Write = WriteUnmapped, 
                    Read = ReadUnmapped 
                };
            }

            // --- Display & OAM ---
            _registers[0x00] = new PpuRegister { Name = "INIDISP", Write = WriteINIDISP, Read = ReadUnmapped };
            _registers[0x01] = new PpuRegister { Name = "OBSEL", Write = WriteOBSEL, Read = ReadUnmapped };
            _registers[0x02] = new PpuRegister { Name = "OAMADDL", Write = WriteOAMADDL, Read = ReadUnmapped };
            _registers[0x03] = new PpuRegister { Name = "OAMADDH", Write = WriteOAMADDH, Read = ReadUnmapped };
            _registers[0x04] = new PpuRegister { Name = "OAMDATA", Write = WriteOAMDATA, Read = ReadUnmapped };
            
            // --- Backgrounds ---
            _registers[0x05] = new PpuRegister { Name = "BGMODE", Write = WriteBGMODE, Read = ReadUnmapped };
            _registers[0x06] = new PpuRegister { Name = "MOSAIC", Write = WriteMOSAIC, Read = ReadUnmapped };
            _registers[0x07] = new PpuRegister { Name = "BG1SC", Write = WriteBGSC, Read = ReadUnmapped };
            _registers[0x08] = new PpuRegister { Name = "BG2SC", Write = WriteBGSC, Read = ReadUnmapped };
            _registers[0x09] = new PpuRegister { Name = "BG3SC", Write = WriteBGSC, Read = ReadUnmapped };
            _registers[0x0A] = new PpuRegister { Name = "BG4SC", Write = WriteBGSC, Read = ReadUnmapped };
            _registers[0x0B] = new PpuRegister { Name = "BG12NBA", Write = WriteBGNBA, Read = ReadUnmapped };
            _registers[0x0C] = new PpuRegister { Name = "BG34NBA", Write = WriteBGNBA, Read = ReadUnmapped };
            
            // --- Scrolling ---
            _registers[0x0D] = new PpuRegister { Name = "BG1HOFS", Write = WriteBGHOFS, Read = ReadUnmapped };
            _registers[0x0E] = new PpuRegister { Name = "BG1VOFS", Write = WriteBGVOFS, Read = ReadUnmapped };
            _registers[0x0F] = new PpuRegister { Name = "BG2HOFS", Write = WriteBGHOFS, Read = ReadUnmapped };
            _registers[0x10] = new PpuRegister { Name = "BG2VOFS", Write = WriteBGVOFS, Read = ReadUnmapped };
            _registers[0x11] = new PpuRegister { Name = "BG3HOFS", Write = WriteBGHOFS, Read = ReadUnmapped };
            _registers[0x12] = new PpuRegister { Name = "BG3VOFS", Write = WriteBGVOFS, Read = ReadUnmapped };
            _registers[0x13] = new PpuRegister { Name = "BG4HOFS", Write = WriteBGHOFS, Read = ReadUnmapped };
            _registers[0x14] = new PpuRegister { Name = "BG4VOFS", Write = WriteBGVOFS, Read = ReadUnmapped };

            // --- VRAM ---
            _registers[0x15] = new PpuRegister { Name = "VMAIN", Write = WriteVMAIN, Read = ReadUnmapped };
            _registers[0x16] = new PpuRegister { Name = "VMADDL", Write = WriteVMADDL, Read = ReadUnmapped };
            _registers[0x17] = new PpuRegister { Name = "VMADDH", Write = WriteVMADDH, Read = ReadUnmapped };
            _registers[0x18] = new PpuRegister { Name = "VMDATAL", Write = WriteVMDATAL, Read = ReadUnmapped };
            _registers[0x19] = new PpuRegister { Name = "VMDATAH", Write = WriteVMDATAH, Read = ReadUnmapped };

            // --- Mode 7 ---
            _registers[0x1A] = new PpuRegister { Name = "M7SEL", Write = WriteM7SEL, Read = ReadUnmapped };
            _registers[0x1B] = new PpuRegister { Name = "M7A", Write = WriteM7A, Read = ReadUnmapped };
            _registers[0x1C] = new PpuRegister { Name = "M7B", Write = WriteM7B, Read = ReadUnmapped };
            _registers[0x1D] = new PpuRegister { Name = "M7C", Write = WriteM7C, Read = ReadUnmapped };
            _registers[0x1E] = new PpuRegister { Name = "M7D", Write = WriteM7D, Read = ReadUnmapped };
            _registers[0x1F] = new PpuRegister { Name = "M7X", Write = WriteM7X, Read = ReadUnmapped };
            _registers[0x20] = new PpuRegister { Name = "M7Y", Write = WriteM7Y, Read = ReadUnmapped };

            // --- CGRAM ---
            _registers[0x21] = new PpuRegister { Name = "CGADD", Write = WriteCGADD, Read = ReadUnmapped };
            _registers[0x22] = new PpuRegister { Name = "CGDATA", Write = WriteCGDATA, Read = ReadUnmapped };
            
            // --- Screen Designation ---
            _registers[0x2C] = new PpuRegister { Name = "TM", Write = WriteTM, Read = ReadUnmapped };
            _registers[0x2D] = new PpuRegister { Name = "TS", Write = WriteTS, Read = ReadUnmapped }; 
            _registers[0x23] = new PpuRegister { Name = "W12SEL", Write = WriteW12Sel, Read = ReadUnmapped };
            _registers[0x24] = new PpuRegister { Name = "W34SEL", Write = WriteW34Sel, Read = ReadUnmapped };
            _registers[0x25] = new PpuRegister { Name = "WOBJSEL", Write = WriteWObjSel, Read = ReadUnmapped };
            _registers[0x26] = new PpuRegister { Name = "WH0", Write = WriteWh0, Read = ReadUnmapped };
            _registers[0x27] = new PpuRegister { Name = "WH1", Write = WriteWh1, Read = ReadUnmapped };
            _registers[0x28] = new PpuRegister { Name = "WH2", Write = WriteWh2, Read = ReadUnmapped };
            _registers[0x29] = new PpuRegister { Name = "WH3", Write = WriteWh3, Read = ReadUnmapped };
            _registers[0x2A] = new PpuRegister { Name = "WBGLOG", Write = WriteWBgLog, Read = ReadUnmapped };
            _registers[0x2B] = new PpuRegister { Name = "WOBJLOG", Write = WriteWObjLog, Read = ReadUnmapped };
            _registers[0x2E] = new PpuRegister { Name = "TMW", Write = WriteTmw, Read = ReadUnmapped };
            _registers[0x2F] = new PpuRegister { Name = "TSW", Write = WriteTsw, Read = ReadUnmapped };
            _registers[0x30] = new PpuRegister { Name = "CGWSEL", Write = WriteCGWSEL, Read = ReadUnmapped };
            _registers[0x31] = new PpuRegister { Name = "CGADSUB", Write = WriteCGADSUB, Read = ReadUnmapped };
            _registers[0x33] = new PpuRegister { Name = "SETINI", Write = WriteSETINI, Read = ReadUnmapped };

            // --- Read Ports ---
            _registers[0x34] = new PpuRegister { Name = "MPYL", Write = WriteUnmapped, Read = ReadMPYL };
            _registers[0x35] = new PpuRegister { Name = "MPYM", Write = WriteUnmapped, Read = ReadMPYM };
            _registers[0x36] = new PpuRegister { Name = "MPYH", Write = WriteUnmapped, Read = ReadMPYH };
            _registers[0x37] = new PpuRegister { Name = "SLHV", Write = WriteUnmapped, Read = ReadSLHV };
            _registers[0x38] = new PpuRegister { Name = "OAMDATAREAD", Write = WriteUnmapped, Read = ReadOAMDATA };
            _registers[0x39] = new PpuRegister { Name = "VMDATALREAD", Write = WriteUnmapped, Read = ReadVMDATAL };
            _registers[0x3A] = new PpuRegister { Name = "VMDATAHREAD", Write = WriteUnmapped, Read = ReadVMDATAH };
            _registers[0x3B] = new PpuRegister { Name = "CGDATAREAD", Write = WriteUnmapped, Read = ReadCGDATA };
            _registers[0x3C] = new PpuRegister { Name = "OPHCT", Write = WriteUnmapped, Read = ReadOPHCT };
            _registers[0x3D] = new PpuRegister { Name = "OPVCT", Write = WriteUnmapped, Read = ReadOPVCT };
            _registers[0x3E] = new PpuRegister { Name = "STAT77", Write = WriteUnmapped, Read = ReadSTAT77 };
            _registers[0x3F] = new PpuRegister { Name = "STAT78", Write = WriteUnmapped, Read = ReadSTAT78 };
        }

    }
}
