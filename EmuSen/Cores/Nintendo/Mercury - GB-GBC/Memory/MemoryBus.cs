using System;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mercury.Cpu.Core;
using EmuSen.Cores.Nintendo.Mercury.Audio;
using EmuSen.Cores.Nintendo.Mercury.Input;
using EmuSen.Cores.Nintendo.Mercury.Video;

namespace EmuSen.Cores.Nintendo.Mercury.Memory
{
    // The five interrupt sources, in priority order - the bit index each occupies in IE and IF.
    public enum Interrupt
    {
        VBlank = 0,
        LcdStat = 1,
        Timer = 2,
        Serial = 3,
        Joypad = 4,
    }

    // Everything at the far end of the CPU's address pins - see Mercury_Memory.md §3.
    public sealed partial class MemoryBus : ICpuBus
    {
        public const int WramBankSize = 0x1000;
        public const int VramBankSize = 0x2000;

        private readonly Cartridge _cart;

        // Colour mode is decided once at construction from the header - see Mercury_Cgb.md §1.
        [SkipInState] public readonly bool Cgb;

        public byte[] Vram;
        public byte[] Wram;
        public byte[] Oam = new byte[0xA0];
        public byte[] HighRam = new byte[0x7F];

        // $FF00-$FF7F verbatim, for the registers no device has claimed yet.
        public byte[] Io = new byte[0x80];

        public byte InterruptEnable;
        public byte InterruptFlags;

        [SkipInState] public IWriteObserver? WriteObserver;

        // Where a Game Genie code lands: every cartridge-routed read passes through it - see Mercury_Cheats.md §2.
        [SkipInState] public EmuSen.Cores.IRomReadPatcher? RomPatcher;
        [SkipInState] public Joypad Joypad = new();

        public readonly Ppu Ppu;
        public readonly Apu Apu = new();

        private byte _serialData;
        private byte _serialControl;

        private byte _oamDmaPage;
        private int _oamDmaCyclesLeft;
        private int _oamDmaIndex;

        // Cycles a DMA took from the CPU, drained by the core after the instruction - see Mercury_Cgb.md §4.1.
        private int _stallCycles;

        // Not machine state: what the test corpus printed, for the harness to read - see Mercury_Memory.md §10.
        [SkipInState] private readonly System.Collections.Generic.List<byte> _serialLog = new();

        // The 16-bit counter DIV is the top half of - see Mercury_Memory.md §5.
        private ushort _divCounter;
        private byte _tima;
        private byte _tma;
        private byte _tac;
        private bool _lastTimerEdge;
        private int _timaReloadDelay;

        public MemoryBus(Cartridge cart)
        {
            _cart = cart;
            Cgb = cart.Cgb != CgbSupport.None;

            Vram = new byte[VramBankSize * (Cgb ? 2 : 1)];
            Wram = new byte[WramBankSize * (Cgb ? 8 : 2)];

            Ppu = new Ppu(this);
        }

        public Cartridge Cart => _cart;

        public byte Div => (byte)(_divCounter >> 8);
        public byte Tima => _tima;
        public byte Tma => _tma;
        public byte Tac => _tac;

        public void Request(Interrupt source) => InterruptFlags |= (byte)(1 << (int)source);

        // Every byte the ROM has clocked out of the link port, oldest first - see Mercury_Memory.md §10.
        public System.Collections.Generic.IReadOnlyList<byte> SerialLog => _serialLog;

        public string SerialText => string.Concat(Array.ConvertAll(_serialLog.ToArray(), b => (char)b));

        public void ClearSerialLog() => _serialLog.Clear();

        public bool OamDmaActive => _oamDmaCyclesLeft > 0;

        // The CPU loses VRAM while the PPU is drawing from it - see Mercury_Ppu.md §7.
        public bool VramAccessibleToCpu => !Ppu.LcdEnabled || Ppu.Mode != PpuMode.Drawing;

        // ...and OAM for the scan as well, plus the whole of an OAM DMA - see Mercury_Ppu.md §7.
        public bool OamAccessibleToCpu =>
            !OamDmaActive && (!Ppu.LcdEnabled || Ppu.Mode is not (PpuMode.OamScan or PpuMode.Drawing));

        // What a DMA charged the CPU, taken once and cleared - see Mercury_Cgb.md §4.1.
        public int TakePendingStall()
        {
            int stall = _stallCycles;
            _stallCycles = 0;
            return stall;
        }

        public byte Read(ushort address)
        {
            switch (address)
            {
                case < 0x8000:
                {
                    byte value = _cart.Mapper.ReadRom(address);
                    if (RomPatcher is not null && RomPatcher.TryPatch(address, value, out byte patched)) value = patched;
                    return value;
                }

                case < 0xA000:
                    return VramAccessibleToCpu ? Vram[VramOffset(address)] : (byte)0xFF;

                case < 0xC000:
                    return _cart.Mapper.ReadRam(address);

                // $E000-$FDFF mirrors WRAM; real hardware wires the address lines straight through.
                case < 0xFE00:
                    return Wram[WramOffset(address)];

                case < 0xFEA0:
                    return OamAccessibleToCpu ? Oam[address - 0xFE00] : (byte)0xFF;

                // Prohibited on a DMG, and it reads back as zero rather than open bus.
                case < 0xFF00:
                    return 0x00;

                case < 0xFF80:
                    return ReadIo(address);

                case < 0xFFFF:
                    return HighRam[address - 0xFF80];

                default:
                    return InterruptEnable;
            }
        }

        public void Write(ushort address, byte data)
        {
            switch (address)
            {
                case < 0x8000:
                    _cart.Mapper.WriteRom(address, data);
                    return;

                case < 0xA000:
                {
                    if (!VramAccessibleToCpu) return;

                    int offset = VramOffset(address);
                    Vram[offset] = data;
                    WriteObserver?.OnWrite(MercuryCore.SpaceVram, offset, data);
                    return;
                }

                case < 0xC000:
                    _cart.Mapper.WriteRam(address, data);
                    WriteObserver?.OnWrite(MercuryCore.SpaceCartRam, address - 0xA000, data);
                    return;

                case < 0xFE00:
                {
                    int offset = WramOffset(address);
                    Wram[offset] = data;
                    WriteObserver?.OnWrite(MercuryCore.SpaceWram, offset, data);
                    return;
                }

                case < 0xFEA0:
                    if (!OamAccessibleToCpu) return;

                    Oam[address - 0xFE00] = data;
                    WriteObserver?.OnWrite(MercuryCore.SpaceOam, address - 0xFE00, data);
                    return;

                case < 0xFF00:
                    return;

                case < 0xFF80:
                    WriteIo(address, data);
                    return;

                case < 0xFFFF:
                    HighRam[address - 0xFF80] = data;
                    WriteObserver?.OnWrite(MercuryCore.SpaceHram, address - 0xFF80, data);
                    return;

                default:
                    InterruptEnable = data;
                    return;
            }
        }

        private byte ReadIo(ushort address) => address switch
        {
            0xFF00 => Joypad.Read(Io[0x00]),
            0xFF01 => _serialData,

            // Bit 1 is the CGB's clock-speed select; on a DMG it reads back as one with the rest.
            0xFF02 => (byte)(_serialControl | (Cgb ? 0x7C : 0x7E)),
            0xFF04 => Div,
            0xFF05 => _tima,
            0xFF06 => _tma,
            0xFF07 => (byte)(_tac | 0xF8),
            0xFF0F => (byte)(InterruptFlags | 0xE0),
            >= 0xFF10 and <= 0xFF26 => Apu.ReadRegister(address),
            >= 0xFF30 and <= 0xFF3F => Apu.ReadRegister(address),
            0xFF40 => Ppu.Lcdc,
            0xFF41 => Ppu.ReadStat(),
            0xFF42 => Ppu.Scy,
            0xFF43 => Ppu.Scx,
            0xFF44 => Ppu.Ly,
            0xFF45 => Ppu.Lyc,
            0xFF47 => Ppu.Bgp,
            0xFF48 => Ppu.Obp0,
            0xFF49 => Ppu.Obp1,
            0xFF4A => Ppu.Wy,
            0xFF4B => Ppu.Wx,
            _ => Cgb ? ReadCgbIo(address) : Io[address - 0xFF00],
        };

        private void WriteIo(ushort address, byte data)
        {
            switch (address)
            {
                // Only the two select bits are writable; the button lines are inputs.
                case 0xFF00:
                    Io[0x00] = (byte)(data & 0x30);
                    return;

                case 0xFF01:
                    _serialData = data;
                    return;

                case 0xFF02:
                    WriteSerialControl(data);
                    return;

                // Any write zeroes the whole counter, which is also how a game resets the timer's phase.
                case 0xFF04:
                    _divCounter = 0;
                    return;

                case 0xFF05:
                    _tima = data;
                    _timaReloadDelay = 0;
                    return;

                case 0xFF06:
                    _tma = data;
                    return;

                case 0xFF07:
                    _tac = (byte)(data & 0x07);
                    return;

                case 0xFF0F:
                    InterruptFlags = (byte)(data & 0x1F);
                    return;

                case >= 0xFF10 and <= 0xFF26:
                case >= 0xFF30 and <= 0xFF3F:
                    Apu.WriteRegister(address, data);
                    return;

                case 0xFF40:
                    Ppu.WriteLcdc(data);
                    return;

                case 0xFF41:
                    Ppu.WriteStat(data);
                    return;

                case 0xFF42:
                    Ppu.Scy = data;
                    return;

                case 0xFF43:
                    Ppu.Scx = data;
                    return;

                // LY is the counter itself; a write is a reset request on hardware, and Mercury ignores it.
                case 0xFF44:
                    return;

                case 0xFF45:
                    Ppu.WriteLyc(data);
                    return;

                case 0xFF46:
                    Io[0x46] = data;
                    StartOamDma(data);
                    return;

                case 0xFF47:
                    Ppu.Bgp = data;
                    return;

                case 0xFF48:
                    Ppu.Obp0 = data;
                    return;

                case 0xFF49:
                    Ppu.Obp1 = data;
                    return;

                case 0xFF4A:
                    Ppu.Wy = data;
                    return;

                case 0xFF4B:
                    Ppu.Wx = data;
                    return;

                default:
                    if (Cgb && WriteCgbIo(address, data)) return;
                    Io[address - 0xFF00] = data;
                    return;
            }
        }

        // A sink, not a link: an internally clocked transfer completes at once and shifts in $FF - see Mercury_Memory.md §10.
        private void WriteSerialControl(byte data)
        {
            _serialControl = (byte)(data & (Cgb ? 0x83 : 0x81));

            // Bit 0 low means the partner supplies the clock, and an unconnected port never gets one.
            if ((_serialControl & 0x81) != 0x81) return;

            _serialLog.Add(_serialData);
            _serialData = 0xFF;
            _serialControl &= 0x7F;
            Request(Interrupt.Serial);
        }

        // 160 machine cycles, one OAM byte each, with OAM closed to the CPU throughout - see Mercury_Memory.md §6.
        private void StartOamDma(byte page)
        {
            _oamDmaPage = page;
            _oamDmaIndex = 0;
            _oamDmaCyclesLeft = Oam.Length * 4;
        }

        private void StepOamDma()
        {
            if (_oamDmaCyclesLeft == 0) return;

            _oamDmaCyclesLeft--;

            // One byte per machine cycle, which lands the 160th on the last cycle of the transfer.
            if ((_oamDmaCyclesLeft & 3) != 0 || _oamDmaIndex >= Oam.Length) return;

            Oam[_oamDmaIndex] = ReadForDma((ushort)((_oamDmaPage << 8) + _oamDmaIndex));
            _oamDmaIndex++;
        }

        // The DMA controller drives the bus itself, so the PPU's CPU-side blocking does not apply to it.
        private byte ReadForDma(ushort address) => address switch
        {
            < 0x8000 => _cart.Mapper.ReadRom(address),
            < 0xA000 => Vram[VramOffset(address)],
            < 0xC000 => _cart.Mapper.ReadRam(address),
            _ => Wram[WramOffset((ushort)(address & 0xDFFF))],
        };

        public void Tick(int cycles)
        {
            for (int i = 0; i < cycles; i++) StepOneCycle();
            _cart.Mapper.Tick(cycles);
        }

        // TIMA counts falling edges of one selected bit of the DIV counter - see Mercury_Memory.md §5.
        private void StepOneCycle()
        {
            // The CPU and the timer double; the LCD and the sound hardware do not - see Mercury_Cgb.md §5.
            if (DoubleSpeed)
            {
                _baseClockPhase = !_baseClockPhase;
                if (_baseClockPhase) StepBaseClock();
            }
            else
            {
                StepBaseClock();
            }

            StepOamDma();

            _divCounter++;

            // The frame sequencer watches a DIV bit, so a DIV write can clock it early - see Mercury_Apu.md §2.
            Apu.OnDivBit((_divCounter & (DoubleSpeed ? 0x2000 : 0x1000)) != 0);

            if (_timaReloadDelay > 0 && --_timaReloadDelay == 0)
            {
                _tima = _tma;
                Request(Interrupt.Timer);
            }

            bool edge = (_divCounter & TimerBitMask) != 0 && (_tac & 0x04) != 0;

            if (_lastTimerEdge && !edge && ++_tima == 0)
            {
                // The reload is not instant: TIMA reads 0 for four cycles before TMA lands.
                _timaReloadDelay = 4;
            }

            _lastTimerEdge = edge;
        }

        private void StepBaseClock()
        {
            Ppu.Tick();
            Apu.Tick();
        }

        private int TimerBitMask => (_tac & 0x03) switch
        {
            0 => 1 << 9,
            1 => 1 << 3,
            2 => 1 << 5,
            _ => 1 << 7,
        };

        public void Reset()
        {
            Array.Clear(Vram);
            Array.Clear(Wram);
            Array.Clear(Oam);
            Array.Clear(HighRam);
            Array.Clear(Io);

            InterruptEnable = 0;
            InterruptFlags = 0;
            _serialData = 0;
            _serialControl = 0;
            _serialLog.Clear();
            _oamDmaPage = 0;
            _oamDmaCyclesLeft = 0;
            _oamDmaIndex = 0;
            _stallCycles = 0;
            _divCounter = 0;
            _tima = 0;
            _tma = 0;
            _tac = 0;
            _lastTimerEdge = false;
            _timaReloadDelay = 0;

            // What the DMG boot ROM leaves behind, since Mercury starts past it - see Mercury_Cpu.md §5.
            Io[0x00] = 0x30;

            ResetCgb();
            Ppu.Reset();
            Apu.Reset();
        }
    }
}
