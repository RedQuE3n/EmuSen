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

        // The copy costs 513 cycles, or 514 when it starts on an odd one - see Moon_Memory.md §5.1.
        public const int OamDmaCycles = 513;

        public readonly byte[] Ram = new byte[RamSize];

        [SkipInState] public readonly Cartridge Cart;
        [SkipInState] public readonly Ppu Ppu;
        [SkipInState] public readonly Apu.Apu Apu;
        [SkipInState] public readonly Controller Controller1 = new();
        [SkipInState] public readonly Controller Controller2 = new();

        // Set by the debug layer; the bus itself names no debug type - see Moon_Memory.md §6.
        [SkipInState] public IWriteObserver? WriteObserver;

        // Cycles a $4014 transfer stole, collected by the core's timing loop.
        public int PendingDmaCycles;

        // Stamped onto the cartridge so a board can reject back-to-back writes.
        [SkipInState] public Cpu? Cpu;

        // The last value the CPU put on the bus, returned for addresses nothing drives.
        public byte OpenBus;

        public MemoryBus(Cartridge cart, Ppu ppu, Apu.Apu apu)
        {
            Cart = cart;
            Ppu = ppu;
            Apu = apu;
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
                value = Apu.ReadStatus();
            }
            else if (address == 0x4016)
            {
                value = (byte)((OpenBus & 0xE0) | Controller1.Read());
            }
            else if (address == 0x4017)
            {
                value = (byte)((OpenBus & 0xE0) | Controller2.Read());
            }
            else if (address < 0x4020)
            {
                value = OpenBus;
            }
            else
            {
                value = Cart.Mapper.ReadPrg(address);
            }

            OpenBus = value;
            return value;
        }

        public void Write(ushort address, byte data)
        {
            OpenBus = data;

            if (address < 0x2000)
            {
                Ram[address & 0x07FF] = data;
                WriteObserver?.OnWrite("RAM", address & 0x07FF, data);
                return;
            }

            if (address < 0x4000)
            {
                int register = address & 0x07;
                Ppu.WriteRegister(register, data);
                WriteObserver?.OnWrite("PPUREG", register, data);
                return;
            }

            if (address == OamDmaRegister)
            {
                RunOamDma(data);
                return;
            }

            if (address == 0x4016)
            {
                bool strobe = (data & 0x01) != 0;
                Controller1.SetStrobe(strobe);
                Controller2.SetStrobe(strobe);
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

        // Reads go through the normal decode, so a page pointed at registers behaves as it would.
        private void RunOamDma(byte page)
        {
            int source = page << 8;

            for (int i = 0; i < 256; i++)
            {
                byte value = Read((ushort)(source + i));
                Ppu.Oam[(byte)(Ppu.OamAddress + i)] = value;
            }

            PendingDmaCycles += OamDmaCycles;
        }

        public int TakePendingDmaCycles()
        {
            int cycles = PendingDmaCycles;
            PendingDmaCycles = 0;
            return cycles;
        }

        public void Reset()
        {
            System.Array.Clear(Ram);
            PendingDmaCycles = 0;
            OpenBus = 0;
            Controller1.Reset();
            Controller2.Reset();
        }
    }
}
