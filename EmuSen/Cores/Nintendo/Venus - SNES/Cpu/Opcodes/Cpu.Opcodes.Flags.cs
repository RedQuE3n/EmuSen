using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Every opcode whose job is the P register or E (emulation-mode) flag itself: the plain.
    public partial class Cpu
    {
        private void OpCLI(uint address) { SetFlag(CpuFlags.I, false); }
        private void OpSEI(uint address) { SetFlag(CpuFlags.I, true); }
        private void OpCLC(uint address) { SetFlag(CpuFlags.C, false); }
        private void OpSEC(uint address) { SetFlag(CpuFlags.C, true); }
        private void OpCLV(uint address) { SetFlag(CpuFlags.V, false); }
        private void OpCLD(uint address) { SetFlag(CpuFlags.D, false); }
        private void OpSED(uint address) { SetFlag(CpuFlags.D, true); }

        private void OpXCE(uint address)
        {
            bool oldCarry = GetFlag(CpuFlags.C);
            SetFlag(CpuFlags.C, E);
            E = oldCarry;

            if (E)
            {
                SetFlag(CpuFlags.M, true);
                SetFlag(CpuFlags.X, true);
                X &= 0x00FF; // Clear X high byte
                Y &= 0x00FF; // Clear Y high byte
                S = (ushort)(0x0100 | (S & 0xFF)); // Force stack to Page 1
                Console.WriteLine("[CPU] Switched to Emulation Mode (E=1). M and X flags forced to 1.");
            }
        }

        private void OpREP(uint address)
        {
            byte val = _bus.Read8(address);
            P &= (byte)~val;

            if (DebugSettings.CpuVerboseLogging && (!IsMemory8Bit || !IsIndex8Bit))
            {
                Console.WriteLine($"[DEBUG] REP executed. M={GetFlag(CpuFlags.M)}, X={GetFlag(CpuFlags.X)}");
            }
        }

        private void OpSEP(uint address)
        {
            byte val = _bus.Read8(address);
            P |= val;

            if (GetFlag(CpuFlags.X))
            {
                X &= 0x00FF;
                Y &= 0x00FF;
            }
        }
    }
}
