using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Processor;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // Nintendo SA-1: a second 65C816 at 10.74MHz plus 2KB I-RAM, a bank
    // controller over ROM/BW-RAM, and four accelerators. Full register map,
    // both memory maps, and the boot handshake: Venus_SA1.md.
    public sealed partial class Sa1
    {
        public const int IRamSize = 0x800;

        // One SA-1 cycle is exactly two master clocks (10.738MHz vs 21.477MHz) - see Venus_SA1.md §2.2.
        public const int MasterClocksPerCycle = 2;

        [SkipInState] private readonly byte[] _rom;

        // Cartridge already serializes this same array - see SkipInStateAttribute case 2.
        [SkipInState] private readonly byte[] _bwRam;

        [SkipInState] private readonly Sa1Bus _bus;

        public byte[] IRam = new byte[IRamSize];

        // The SA-1's own 65816. Same class as the S-CPU, pointed at Sa1Bus - see Venus_SA1.md §2.1.
        public Cpu Cpu;

        public Sa1Math Math = new Sa1Math();
        public Sa1BitStream BitStream;
        public Sa1Dma Dma;

        // Super MMC ROM bank registers $2220-$2223 - see Venus_SA1.md §3.1.
        private byte _cxb = 0x00, _dxb = 0x01, _exb = 0x02, _fxb = 0x03;

        // BW-RAM $6000-$7FFF window selects, one per side - see Venus_SA1.md §3.2.
        private byte _bmaps, _bmap;

        // Write-protection registers: stored so reads are faithful, not enforced - see Venus_SA1.md §3.4.
        private byte _sbwe, _cbwe, _bwpa, _siwp, _ciwp;

        // BW-RAM bitmap format $223F, bit 7: 0 = 4bpp, 1 = 2bpp.
        private byte _bbf;

        // $2200 CCNT, latched so the RESB/RDYB edges below can be detected.
        private byte _ccnt = 0x20;

        // $2209 SCNT - the SA-1's control over the S-CPU, incl. vector overrides.
        private byte _scnt;

        // Interrupt enables: $2201 SIE (S-CPU side), $220A CIE (SA-1 side).
        private byte _sie, _cie;

        // SA-1 CPU vectors $2203-$2208; the SA-1 always boots from CRV.
        private ushort _crv, _cnv, _civ;

        // S-CPU vector overrides $220C-$220F, gated by SCNT bits 5/4.
        private ushort _snv, _siv;

        // Pending-interrupt latches. Cleared by the matching $2202/$220B write.
        private bool _sa1IrqToScpu, _dmaIrqToScpu;
        private bool _scpuIrqToSa1, _scpuNmiToSa1, _timerIrqToSa1, _dmaIrqToSa1;

        // Edge latches so a level-held request only enters the handler once.
        private bool _sa1IrqTaken, _sa1NmiTaken;

        // RESB (bit 5) holds the SA-1 CPU in reset; RDYB (bit 6) parks it.
        private bool Halted => (_ccnt & 0x60) != 0;

        // Unspent master clocks carried between Run() calls.
        private int _clockBudget;

        public Sa1(byte[] rom, byte[] bwRam)
        {
            _rom = rom;
            _bwRam = bwRam;
            _bus = new Sa1Bus(this);
            BitStream = new Sa1BitStream(rom);
            Dma = new Sa1Dma(this);
            Cpu = new Cpu(_bus, "SA1", logReset: false);
        }

        public int BwRamSize => _bwRam.Length;

        // True while the SA-1 is asserting an IRQ the S-CPU has enabled - polled by VenusCore.RunFrame.
        public bool ScpuIrqPending =>
            ((_sa1IrqToScpu && (_sie & 0x80) != 0) || (_dmaIrqToScpu && (_sie & 0x20) != 0));

        // Advances the SA-1 by however many master clocks the S-CPU just
        // consumed, so both cores share one timebase - see Venus_SA1.md §2.2.
        public void Run(int masterClocks)
        {
            _clockBudget += masterClocks;
            if (_clockBudget <= 0) return;

            if (Halted)
            {
                // Still clock the timer while the CPU is parked: a game can arm
                // the timer, halt the SA-1, and wait on the IRQ to restart it.
                StepTimer(_clockBudget);
                _clockBudget = 0;
                return;
            }

            while (_clockBudget > 0)
            {
                ServiceInterrupts();
                int spent = Cpu.Step();
                StepTimer(spent);
                _clockBudget -= spent;
            }
        }

        private void ServiceInterrupts()
        {
            bool nmi = _scpuNmiToSa1 && (_cie & 0x10) != 0;
            if (nmi && !_sa1NmiTaken)
            {
                _sa1NmiTaken = true;
                Cpu.Nmi();
                return;
            }
            if (!nmi) _sa1NmiTaken = false;

            bool irq = ((_scpuIrqToSa1 && (_cie & 0x80) != 0)
                     || (_timerIrqToSa1 && (_cie & 0x40) != 0)
                     || (_dmaIrqToSa1 && (_cie & 0x20) != 0));
            if (irq && !_sa1IrqTaken)
            {
                // Irq() itself declines while the I flag is set, so only latch when it actually entered.
                if (Cpu.Irq()) _sa1IrqTaken = true;
            }
            else if (!irq)
            {
                _sa1IrqTaken = false;
            }
        }

        // RESB 1->0 restarts the SA-1 CPU at CRV - see Venus_SA1.md §4.1.
        private void OnControlWrite(byte previous, byte value)
        {
            bool wasReset = (previous & 0x20) != 0;
            bool nowReset = (value & 0x20) != 0;
            if (wasReset && !nowReset)
            {
                Cpu.Reset();
                _sa1IrqTaken = false;
                _sa1NmiTaken = false;
                _clockBudget = 0;
            }

            if ((value & 0x80) != 0) _scpuIrqToSa1 = true;
            if ((value & 0x10) != 0) _scpuNmiToSa1 = true;
        }
    }
}
