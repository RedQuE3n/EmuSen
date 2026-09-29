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
            Ppu.Step(DotsPerCpuCycle);
            Cpu?.SetNmiLine(Ppu.NmiOutput);

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
                // Driven on the chip's internal bus only: bit 5 floats, and the external bus keeps its value - see Moon_Native.md §3.8.
                return (byte)(Apu.ReadStatus() | (OpenBus & 0x20));
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
        private void RunDmcDma(ushort address)
        {
            HaltedRead(address);
            HaltedRead(address);
            if (!Apu.NextCycleIsGet) HaltedRead(address);
            DmcGet(address);
        }

        // The halt cycle, one alignment cycle if the next is a put, then a get read and a put write per byte: 513 or 514 cycles.
        private void RunOamDma(ushort address)
        {
            OamDmaPending = false;
            int source = OamDmaPage << 8;
            HaltedRead(address);
            if (!Apu.NextCycleIsGet) HaltedRead(address);

            for (int i = 0; i < 256; i++)
            {
                // The DMC takes a get cycle it asks for, and the copy realigns on the put after it - see Moon_Native.md §3.9.
                while (Apu.Dmc.DmaRequested)
                {
                    DmcGet(address);
                    Cpu!.BeginHaltedCycle();
                    Cpu.EndHaltedCycle();
                }

                Cpu!.BeginHaltedCycle();
                byte value = Read((ushort)(source + i));
                Cpu.EndHaltedCycle();

                Cpu.BeginHaltedCycle();
                Ppu.Oam[(byte)(Ppu.OamAddress + i)] = value;
                Cpu.EndHaltedCycle();
            }
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
            ushort fetch = Apu.Dmc.DmaAddress;
            Apu.Dmc.CompleteDma(Read(fetch));
            if (halted >= 0x4000 && halted < 0x4020) ConflictWithApuRegister(fetch);
            Cpu.EndHaltedCycle();
        }

        // A CPU halted on $4000-$401F keeps the 2A03's registers enabled, so the fetch's low five bits select one too - see Moon_Native.md §3.9.
        private void ConflictWithApuRegister(ushort fetch)
        {
            ushort register = (ushort)(0x4000 | (fetch & 0x1F));
            if (register == 0x4015) Apu.ReadStatus();
            if (register is 0x4016 or 0x4017)
            {
                _lastReadAddress = register;
                _lastReadCycle = Cpu?.Cycles ?? 0;
            }
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
            _strobeLatch = false;
            _strobeOut = false;
            OpenBus = 0;
            Controller1.Reset();
            Controller2.Reset();
        }
    }
}
