using System;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // NMI, H/V-IRQ and vblank status, kept as one subsystem - see Venus_Memory.md §4.
    public class InterruptController
    {
        public bool NmiEnabled;
        public bool InVBlank;
        private bool _vblankFlag;

        // NMI rising-edge quirk - see Venus_Memory.md §4.2.
        public bool PendingImmediateNmi;

        // $4200 bit 0 - gates the once-per-frame auto-joypad read, see Venus_Memory.md §4.4.
        public bool AutoJoypadEnabled;

        // H/V-IRQ (the $4200 bits 4-5 / $4207-$420A timer, distinct from NMI).
        public bool HIrqEnabled;
        public bool VIrqEnabled;
        public ushort HTime = 0x1FF; // 9-bit; power-on default per hardware docs
        public ushort VTime = 0x1FF; // 9-bit; power-on default per hardware docs
        private bool _irqFlag;       // $4211 bit 7, set on trigger, cleared on read

        // The auto-joypad-read busy window, in master clocks after the vblank scanline starts - see Venus_Memory.md §4.4.
        public const int AutoJoypadScanline = 225;
        private const int AutoJoypadBusyStart = 258;
        public const int AutoJoypadBusyEnd = 4482;

        // Latching at this scanline's end lands at master clock 4092, the last per-scanline tick inside the window - see Venus_Memory.md §4.4.
        public const int AutoJoypadLatchScanline = 227;

        // True while a game's "wait for $4212 bit 0 to clear" loop should still be spinning - see Venus_Memory.md §4.4.
        public static bool InAutoJoypadWindow(int scanline, int lineCycles)
        {
            long clock = (long)(scanline - AutoJoypadScanline) * VenusCore.CyclesPerScanline + lineCycles;
            return clock >= AutoJoypadBusyStart && clock < AutoJoypadBusyEnd;
        }

        public void RaiseVBlank()
        {
            _vblankFlag = true;
        }

        // Auto-clears the vblank flag at end of vblank - see Venus_Memory.md §4.3.
        public void EndVBlank()
        {
            InVBlank = false;
            _vblankFlag = false;
        }

        // Called by the main loop when the H/V-IRQ timer's trigger condition (scanline/dot position) has been.
        public void RaiseTimerIrq()
        {
            _irqFlag = true;
        }

        // $4210 RDNMI - see Venus_Memory.md §4.4.
        public byte ReadRDNMI(byte lastBusValue)
        {
            byte val = (byte)(0x02 | (lastBusValue & 0x70));
            if (_vblankFlag) val |= 0x80;
            _vblankFlag = false;
            return val;
        }

        // $4212 HVBJOY - see Venus_Memory.md §4.4.
        public byte ReadHVBJOY(bool inHBlank, bool inAutoJoypadWindow, byte lastBusValue)
        {
            bool busy = AutoJoypadEnabled && inAutoJoypadWindow;
            return (byte)((InVBlank ? 0x80 : 0x00) | (inHBlank ? 0x40 : 0x00) | (lastBusValue & 0x3E) | (busy ? 0x01 : 0x00));
        }

        // $4211 TIMEUP - see Venus_Memory.md §4.4.
        public byte ReadTIMEUP(byte lastBusValue)
        {
            byte val = (byte)(lastBusValue & 0x7F);
            if (_irqFlag) val |= 0x80;
            _irqFlag = false;
            return val;
        }

        public void Write4200(byte data)
        {
            bool oldNmiEnabled = NmiEnabled;
            NmiEnabled = (data & 0x80) != 0;
            AutoJoypadEnabled = (data & 0x01) != 0;

            // See PendingImmediateNmi / Venus_Memory.md §4.2.
            if (!oldNmiEnabled && NmiEnabled && _vblankFlag)
            {
                PendingImmediateNmi = true;
            }

            bool newH = (data & 0x10) != 0;
            bool newV = (data & 0x20) != 0;
            if ((newH != HIrqEnabled || newV != VIrqEnabled) && DebugSettings.HvIrqChangeLogging)
            {
                Console.WriteLine($"[HVIRQ] $4200 write=0x{data:X2} -> HIrqEnabled {HIrqEnabled}->{newH}, VIrqEnabled {VIrqEnabled}->{newV}, HTime={HTime} VTime={VTime}");
            }
            // Disabling an IRQ also acknowledges a pending one (not NMI).
            if ((HIrqEnabled && !newH) || (VIrqEnabled && !newV)) _irqFlag = false;
            HIrqEnabled = newH;
            VIrqEnabled = newV;
        }

        public void WriteHTimeL(byte data)
        {
            ushort old = HTime;
            HTime = (ushort)((HTime & 0x100) | data);
            if (HTime != old && DebugSettings.HvIrqChangeLogging) Console.WriteLine($"[HVIRQ] HTIMEL write=0x{data:X2} -> HTime={HTime}");
        }

        public void WriteHTimeH(byte data)
        {
            ushort old = HTime;
            HTime = (ushort)((HTime & 0x0FF) | ((data & 0x01) << 8));
            if (HTime != old && DebugSettings.HvIrqChangeLogging) Console.WriteLine($"[HVIRQ] HTIMEH write=0x{data:X2} -> HTime={HTime}");
        }

        public void WriteVTimeL(byte data)
        {
            ushort old = VTime;
            VTime = (ushort)((VTime & 0x100) | data);
            if (VTime != old && DebugSettings.HvIrqChangeLogging) Console.WriteLine($"[HVIRQ] VTIMEL write=0x{data:X2} -> VTime={VTime}");
        }

        public void WriteVTimeH(byte data)
        {
            ushort old = VTime;
            VTime = (ushort)((VTime & 0x0FF) | ((data & 0x01) << 8));
            if (VTime != old && DebugSettings.HvIrqChangeLogging) Console.WriteLine($"[HVIRQ] VTIMEH write=0x{data:X2} -> VTime={VTime}");
        }
    }
}
