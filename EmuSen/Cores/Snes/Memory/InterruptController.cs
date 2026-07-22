using System;
using EmuSen.Debug;

namespace EmuSen.Memory
{
    // The SNES's NMI / H-V-IRQ / vblank-status subsystem ($4200, $4207-
    // $420A write; $4210, $4211, $4212 read; RaiseVBlank/EndVBlank/
    // RaiseTimerIrq called by whichever frontend loop is running).
    //
    // Extracted out of MemoryBus as ONE cohesive unit rather than split
    // further. An earlier architecture review proposed a separate
    // "IrqController" (H/V-IRQ only) alongside NMI/vblank state staying on
    // MemoryBus - on closer inspection that split would have been
    // artificial: the vblank flag feeds BOTH the $4210 (RDNMI) read AND
    // the $4200-write NMI-rising-edge check, and a single $4200 write sets
    // NMI enable and H/V-IRQ enable together, on real hardware. Splitting
    // those apart would have added cross-object coupling (each half
    // needing to reach into the other) rather than removing any. This
    // class is the corrected version of that proposal: everything that's
    // genuinely one hardware subsystem, extracted together, so MemoryBus
    // doesn't have to carry it directly.
    //
    // Deliberately does NOT own LineCycles/CurrentScanline - those mirror
    // into Ppu for SLHV/OPHCT/OPVCT and are a PPU-timing concern, not an
    // interrupt-controller one, even though HVBJOY's H-blank bit needs a
    // value derived from LineCycles. MemoryBus computes that bool itself
    // and passes it in to ReadHVBJOY, keeping this class from needing to
    // know about PPU-timing-mirror state at all.
    public class InterruptController
    {
        public bool NmiEnabled;
        public bool InVBlank;
        private bool _vblankFlag;

        // Documented hardware quirk (confirmed via fullsnes, the SNESdev
        // wiki's MMIO_registers page, AND its Errata page independently -
        // three sources, all agreeing): NMI actually fires on the RISING
        // EDGE of (NMITIMEN bit 7 AND RDNMI's vblank flag), not just "once
        // at the start of vblank if already enabled". If a game disables
        // NMI, does some work, then re-enables it via $4200 WHILE the
        // vblank flag is still set (i.e. still during vblank, hasn't read
        // $4210 yet), an NMI fires immediately at that write - not at the
        // next frame's vblank start. This can also mean MORE than one NMI
        // in a single vblank. Set in Write4200, consumed (and cleared) once
        // per scanline boundary in the main loop, same granularity already
        // used for H/V-IRQ checks - a known, already-accepted scanline-
        // level timing simplification in this project, not cycle-exact,
        // but far better than not implementing this at all.
        public bool PendingImmediateNmi;

        // H/V-IRQ (the $4200 bits 4-5 / $4207-$420A timer, distinct from NMI).
        public bool HIrqEnabled;
        public bool VIrqEnabled;
        public ushort HTime = 0x1FF; // 9-bit; power-on default per hardware docs
        public ushort VTime = 0x1FF; // 9-bit; power-on default per hardware docs
        private bool _irqFlag;       // $4211 bit 7, set on trigger, cleared on read

        public void RaiseVBlank()
        {
            _vblankFlag = true;
        }

        // Real hardware auto-clears RDNMI's vblank flag at the END of
        // vblank, regardless of whether the game ever read $4210 during that
        // frame - confirmed via fullsnes ("The flag gets reset automatically
        // at end of Vblank, and gets also reset after reading from this
        // register") and the SNESdev wiki's MMIO_registers page independently.
        // Previously this only ever cleared on a $4210 read, so a game that
        // skipped reading it in one frame would carry a stale, incorrectly-
        // set flag into the next frame's active display period. Called from
        // the main loop at the same point InVBlank flips false.
        public void EndVBlank()
        {
            InVBlank = false;
            _vblankFlag = false;
        }

        // Called by the main loop when the H/V-IRQ timer's trigger condition
        // (scanline/dot position) has been reached, so $4211 correctly reflects it.
        public void RaiseTimerIrq()
        {
            _irqFlag = true;
        }

        // $4210 (RDNMI): Nxxx VVVV - bit 7 = vblank flag, bits 4-6 = open
        // bus, bits 0-3 = CPU version (2, unchanged since the original
        // S-CPU). lastBusValue supplies the open-bus bits from the
        // caller (MemoryBus), which is the thing that actually tracks it.
        public byte ReadRDNMI(byte lastBusValue)
        {
            byte val = (byte)(0x02 | (lastBusValue & 0x70));
            if (_vblankFlag) val |= 0x80;
            _vblankFlag = false;
            return val;
        }

        // $4212 (HVBJOY): VHxx xxxJ - bit 7 = vblank, bit 6 = hblank, bits
        // 1-5 = open bus, bit 0 = joypad auto-read in-progress (not
        // modeled - auto-joypad read itself isn't implemented as its own
        // timed process in this project, a separate, already-known gap
        // from this one; defaults to 0 = "not in progress", which is
        // correct outside the ~3-scanline window right after vblank
        // starts where real hardware would briefly report 1).
        public byte ReadHVBJOY(bool inHBlank, byte lastBusValue)
        {
            return (byte)((InVBlank ? 0x80 : 0x00) | (inHBlank ? 0x40 : 0x00) | (lastBusValue & 0x3E));
        }

        // $4211 (TIMEUP): Txxx xxxx - bit 7 = timer/IRQ flag, bits 0-6 =
        // open bus.
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

            // Documented quirk - see PendingImmediateNmi's own comment for
            // the full citation: NMI fires on the rising edge of (enable
            // AND vblank flag), not just at vblank's start.
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
            // Disabling IRQs also acknowledges a pending one - documented
            // hardware behavior (fullsnes), not something NMI does.
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
