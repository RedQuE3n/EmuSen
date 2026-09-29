using EmuSen.Common;
using EmuSen.Cores.Nintendo.Moon.Apu;
using EmuSen.Cores.Nintendo.Moon.Input;
using EmuSen.Cores.Nintendo.Moon.Processor;
using EmuSen.Cores.Nintendo.Moon.Video;

namespace EmuSen.Cores.Nintendo.Moon.Memory
{
    // The CPU's address decode. Everything above $4020 belongs to the board - see Moon_Memory.md §1.
    public sealed class MemoryBus : ICpuBus
    {
        public const int RamSize = 0x0800;
        public const ushort OamDmaRegister = 0x4014;


        public readonly byte[] Ram = new byte[RamSize];

        [SkipInState] public readonly Cartridge Cart;
        [SkipInState] public readonly Ppu Ppu;
        [SkipInState] public readonly Apu.Apu Apu;
        [SkipInState] public readonly Controller Controller1 = new();
        [SkipInState] public readonly Controller Controller2 = new();

        // Set by the debug layer; the bus itself names no debug type - see Moon_Memory.md §6.
        [SkipInState] public IWriteObserver? WriteObserver;

        // Stamped on each APU register-write record so a log aligns with the reference's - see §3.46.
        [SkipInState] public uint ApuTraceFrame;

        // Game Genie's edge-connector intercept, on cartridge reads only - see Moon_Cheats.md §3.
        [SkipInState] public IRomReadPatcher? RomPatcher;

        // Always 0 since DMA became a halt of the CPU; kept so version 3's layout is unchanged - see Moon_Native.md §3.9.
        public int PendingDmaCycles;
        public int StolenCycles;

        // A $4014 write waiting for the CPU's next read; written in version 4's tail - see Moon_Native.md §3.9.
        [SkipInState] public bool OamDmaPending;
        [SkipInState] public byte OamDmaPage;

        // The last read's cycle and address: a pad clocks once for reads on consecutive cycles - see Moon_Native.md §3.9.
        [SkipInState] private long _lastReadCycle = -2;
        [SkipInState] private ushort _lastReadAddress;

        // $4016's OUT0 as written and as the pads see it, which follows the write only at a get cycle's end; not state, as the pads are not (§6.2 D3).
        [SkipInState] private bool _strobeLatch;

        // The 2A03's internal data bus: every access's value, but a $4015 read's own; not state, as it equals OpenBus but after such a read - see Moon_Native.md §3.11.
        [SkipInState] public byte InternalBus;
        [SkipInState] private bool _strobeOut;

        // Stamped onto the cartridge so a board can reject back-to-back writes.
        [SkipInState] public Cpu? Cpu;

        // The last value the CPU put on the bus, returned for addresses nothing drives.
        public byte OpenBus;

        // Asked once, because most boards say no and the check is on the hottest path there is.
        [SkipInState] private readonly bool _mapperClocksOnCpu;

        public MemoryBus(Cartridge cart, Ppu ppu, Apu.Apu apu)
        {
            Cart = cart;
            Ppu = ppu;
            Apu = apu;
            _mapperClocksOnCpu = cart.Mapper.ClocksOnCpuCycle;
        }

        // Three PPU dots to a CPU cycle on NTSC - see Moon_PPU.md §1.
        public const int DotsPerCpuCycle = 3;

        // One CPU cycle of everything clocked from the CPU's own clock - see Moon_CPU.md §5.5.
        public void Tick()
        {
            // Two dots before the access and the third after it, which is where the CPU's access falls - see Moon_Native.md §3.10.
            Ppu.Step(DotsPerCpuCycle - 1);

            Apu.Step(1);
            if (_mapperClocksOnCpu) Cart.Mapper.OnCpuCycle();

            // The pads take OUT0 at a get cycle's end, the edge the RTL calls put_ce, so this put cycle sees it - see Moon_Native.md §3.9.
            if (_strobeLatch != _strobeOut && !Apu.IsGetCycle)
            {
                _strobeOut = _strobeLatch;
                Controller1.SetStrobe(_strobeOut);
                Controller2.SetStrobe(_strobeOut);
            }

            Cpu?.SetIrqLine(Apu.IrqAsserted || Cart.Mapper.IrqPending);
        }

        // The CPU samples /NMI at the cycle's end, after this cycle's $2002 read or $2000 write - see Moon_Native.md §3.10.
        public void EndCycle()
        {
            Ppu.Step(1);
            Cpu?.SetNmiLine(Ppu.NmiOutput);
        }

        public byte Read(ushort address)
        {
            byte value;

            if (address < 0x2000)
            {
                value = Ram[address & 0x07FF];
            }
            else if (address < 0x4000)
            {
                value = Ppu.ReadRegister(address & 0x07);
            }
            else if (address == 0x4015)
            {
                // Driven on the chip's internal bus only: bit 5 is that bus's, and the external bus keeps its value - see Moon_Native.md §3.8.
                byte status = (byte)(Apu.ReadStatus() | (InternalBus & 0x20));
                InternalBus = status;
                return status;
            }
            else if (address == 0x4016)
            {
                value = (byte)((OpenBus & 0xE0) | Controller1.Read(clock: !HeldRead(address)));
            }
            else if (address == 0x4017)
            {
                value = (byte)((OpenBus & 0xE0) | Controller2.Read(clock: !HeldRead(address)));
            }
            else if (address < 0x4020)
            {
                value = OpenBus;
            }
            else if (address < 0x6000)
            {
                // No board here drives the expansion area on a read, so it floats - see Moon_Native.md §3.8.
                value = OpenBus;
            }
            else
            {
                value = Cart.Mapper.ReadPrg(address);

                // The CPU address, not a ROM offset: that is what a Game Genie sees.
                if (RomPatcher is not null && RomPatcher.TryPatch(address, value, out byte patched)) value = patched;
            }

            OpenBus = value;
            InternalBus = value;
            return value;
        }

        // The pad clocks on its select's falling edge, so a read on the cycle after one of the same register does not - see Moon_Native.md §3.9.
        private bool HeldRead(ushort address)
        {
            long cycle = Cpu?.Cycles ?? 0;
            bool held = address == _lastReadAddress && cycle == _lastReadCycle + 1;
            _lastReadAddress = address;
            _lastReadCycle = cycle;
            return held;
        }

        public void Write(ushort address, byte data)
        {
            OpenBus = data;
            InternalBus = data;

            // At the top of the funnel, where the reference's hook is: $4014 and $4016 are handled by their own - see §3.46.
            if (Debug.ApuWriteTrace.Enabled && address >= 0x4000 && address <= 0x4017)
            {
                Debug.ApuWriteTrace.Record(ApuTraceFrame, Cpu?.PC ?? 0, address, data);
            }

            if (address < 0x2000)
            {
                Ram[address & 0x07FF] = data;
                WriteObserver?.OnWrite("RAM", address & 0x07FF, data);
                return;
            }

            if (address < 0x4000)
            {
                int register = address & 0x07;
                // Stamped before the write, so the dot is the one the PPU was on when it landed - see Moon_PPU.md §7.
                if (EmuSen.Debug.DebugSettings.NesPpuWriteLogging)
                {
                    Console.WriteLine($"[PPUW] f{Ppu.FrameCount} line {Ppu.Scanline,3} dot {Ppu.Cycle,3}  $200{register} = ${data:X2}");
                }
                Ppu.WriteRegister(register, data);
                WriteObserver?.OnWrite("PPUREG", register, data);
                return;
            }

            if (address == OamDmaRegister)
            {
                OamDmaPage = data;
                OamDmaPending = true;
                return;
            }

            if (address == 0x4016)
            {
                _strobeLatch = (data & 0x01) != 0;
                return;
            }

            if (address < 0x4020)
            {
                Apu.WriteRegister(address, data);
                WriteObserver?.OnWrite("APUREG", address - 0x4000, data);
                return;
            }

            Cart.CpuCycle = Cpu?.Cycles ?? 0;
            Cart.Mapper.WritePrg(address, data);

            if (address < 0x8000) WriteObserver?.OnWrite("PRGRAM", address & 0x1FFF, data);
        }

        // A DMA waits for the CPU's next read cycle, which it halts - see Moon_Native.md §3.9.
        public bool DmaPending => OamDmaPending || Apu.Dmc.DmaRequested;

        // Runs every DMA that wants the bus while the CPU is halted on a read of <address>; each cycle clocks the whole machine.
        public void RunDma(ushort address)
        {
            while (true)
            {
                if (OamDmaPending) RunOamDma(address);
                else if (Apu.Dmc.DmaRequested) RunDmcDma(address);
                else return;
            }
        }

        // Halt and dummy cycles repeat the CPU's read; one more aligns the fetch to a get cycle - see Moon_Native.md §3.9.
        // The DMA commits after its halt: a request still up then is served whole, one gone costs the halt alone - see Moon_Native.md §3.11.
        private void RunDmcDma(ushort address)
        {
            HaltedRead(address);
            if (!Apu.Dmc.DmaRequested) return;
            HaltedRead(address);
            if (!Apu.NextCycleIsGet) HaltedRead(address);
            DmcGet(address);
        }

        // The halt, an alignment cycle before a put, then a get read and a put write per byte; a DMC fetch runs alongside - see Moon_Native.md §3.11.
        private void RunOamDma(ushort address)
        {
            OamDmaPending = false;
            int source = OamDmaPage << 8;
            long dmcHalt = -1;

            NoteDmcHalt(ref dmcHalt);
            HaltedRead(address);
            if (!Apu.NextCycleIsGet)
            {
                NoteDmcHalt(ref dmcHalt);
                HaltedRead(address);
            }

            for (int i = 0; i < 256; i++)
            {
                // The DMC's halt and dummy overlap the copy; its get takes a get at least two cycles later, and the copy realigns on the put after it.
                NoteDmcHalt(ref dmcHalt);
                while (dmcHalt >= 0 && Cpu!.Cycles + 1 >= dmcHalt + 2)
                {
                    DmcGet(address);
                    dmcHalt = -1;
                    NoteDmcHalt(ref dmcHalt);
                    Cpu.BeginHaltedCycle();
                    Cpu.EndHaltedCycle();
                    NoteDmcHalt(ref dmcHalt);
                }

                Cpu!.BeginHaltedCycle();
                byte value = DmaRead((ushort)(source + i), address, forOam: true);
                Cpu.EndHaltedCycle();

                NoteDmcHalt(ref dmcHalt);
                Cpu.BeginHaltedCycle();
                Ppu.Oam[(byte)(Ppu.OamAddress + i)] = value;
                Cpu.EndHaltedCycle();
            }

            // A fetch the copy outlived goes on alone, from whichever of its cycles it had reached.
            if (dmcHalt < 0 || !Apu.Dmc.DmaRequested) return;
            if (Cpu!.Cycles - dmcHalt + 1 < 2) HaltedRead(address);
            if (!Apu.NextCycleIsGet) HaltedRead(address);
            DmcGet(address);
        }

        // The cycle about to run is a DMC request's halt, if one has just risen.
        private void NoteDmcHalt(ref long dmcHalt)
        {
            if (dmcHalt < 0 && Apu.Dmc.DmaRequested) dmcHalt = Cpu!.Cycles + 1;
        }

        // A DMA's read: the 2A03's registers answer only while the halted CPU's address is in $4000-$401F, chosen by this address's low five bits - see Moon_Native.md §3.11.
        private byte DmaRead(ushort address, ushort halted, bool forOam)
        {
            // Nothing on these boards drives $4000-$5FFF for a DMA, so there a register's answer is the bus.
            bool undriven = address >= 0x4000 && address < 0x6000;
            byte internalBefore = InternalBus;
            byte value = undriven ? OpenBus : Read(address);
            if (halted < 0x4000 || halted >= 0x4020)
            {
                if (undriven) InternalBus = value;
                return value;
            }

            ushort register = (ushort)(0x4000 | (address & 0x1F));
            if (register == 0x4015)
            {
                byte status = (byte)(Apu.ReadStatus() | (internalBefore & 0x20));
                InternalBus = status;
                if (undriven) OpenBus = status;
                return undriven || forOam ? status : value;
            }
            if (register is 0x4016 or 0x4017)
            {
                Controller pad = register == 0x4016 ? Controller1 : Controller2;
                byte merged = (byte)((value & 0xE0) | pad.Read(clock: !HeldRead(register)));
                OpenBus = merged;
                InternalBus = merged;
                return forOam && !undriven ? value : merged;
            }
            if (undriven) InternalBus = value;
            return value;
        }

        private void HaltedRead(ushort address)
        {
            Cpu!.BeginHaltedCycle();
            Read(address);
            Cpu.EndHaltedCycle();
        }

        // The get cycle: the DMC's read puts its byte on the external bus, as any read does.
        private void DmcGet(ushort halted)
        {
            Cpu!.BeginHaltedCycle();
            Apu.Dmc.CompleteDma(DmaRead(Apu.Dmc.DmaAddress, halted, forOam: false));
            Cpu.EndHaltedCycle();
        }

        public int TakePendingDmaCycles()
        {
            int cycles = PendingDmaCycles;
            PendingDmaCycles = 0;
            return cycles;
        }

        public int TakeStolenCycles()
        {
            int cycles = StolenCycles;
            StolenCycles = 0;
            return cycles;
        }

        public void Reset()
        {
            System.Array.Clear(Ram);
            SoftReset();
        }

        // The RESET line does not clear work RAM; only the transient bus state goes - see Moon_Core.md §6.
        // A loaded state starts with no read on the cycle before it.
        public void ForgetLastRead() => _lastReadCycle = -2;

        public void SoftReset()
        {
            PendingDmaCycles = 0;
            OamDmaPending = false;
            _lastReadCycle = -2;
            InternalBus = 0;
            _strobeLatch = false;
            _strobeOut = false;
            OpenBus = 0;
            Controller1.Reset();
            Controller2.Reset();
        }
    }
}
