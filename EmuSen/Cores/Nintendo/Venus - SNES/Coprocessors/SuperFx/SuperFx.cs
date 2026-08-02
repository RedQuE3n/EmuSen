using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // Nintendo SuperFX / GSU: a custom 16-bit RISC processor with 16 general
    // registers, a 512-byte instruction cache, and dedicated pixel-plotting
    // hardware that renders straight into Game Pak RAM. Unlike the SA-1 this
    // is not a 65816, so it gets its own interpreter - see Venus_SuperFX.md.
    public sealed partial class SuperFx
    {
        public const int CacheSize = 512;
        private const int CacheLines = CacheSize / 16;

        [SkipInState] private readonly byte[] _rom;

        // Game Pak RAM: the GSU's work RAM, its framebuffer, and (on a game
        // that saves) the battery-backed save data, all the same chip.
        [SkipInState] private readonly byte[] _ram;

        // R15 is the program counter; R0 is the default accumulator - see Venus_SuperFX.md §3.1.
        public ushort[] R = new ushort[16];

        // Status/flag register, $3030 - see Venus_SuperFX.md §3.2.
        private ushort _sfr;

        private byte _pbr;      // $3034 program bank
        private byte _rombr;    // $3036 ROM data bank
        private byte _rambr;    // $303C RAM data bank
        private ushort _cbr;    // $303E cache base
        private byte _scbr;     // $3038 screen base
        private byte _scmr;     // $303A screen mode
        private byte _cfgr;     // $3037 config
        private byte _clsr;     // $3039 clock select
        private byte _bramr;    // $3033 backup RAM enable

        // Plot hardware state, driven by COLOR/CMODE rather than by MMIO.
        private byte _colr;
        private byte _por;

        public byte[] Cache = new byte[CacheSize];
        private bool[] _cacheValid = new bool[CacheLines];

        // One-byte instruction prefetch - see Venus_SuperFX.md §4.1.
        private byte _pipeline;

        // A jump has retargeted R15 but its delay-slot byte is still in the
        // pipeline and has to be consumed first.
        private bool _jumpPending;

        // Source and destination register selection, reset to R0 after every
        // instruction unless a TO/WITH/FROM prefix set them - see Venus_SUperFX.md §4.2.
        private int _sreg, _dreg;

        // R14 writes start a ROM fetch that later reads collect - see Venus_SuperFX.md §5.2.
        private byte _romBuffer;

        private int _clockBudget;

        public SuperFx(byte[] rom, byte[] ram)
        {
            _rom = rom;
            _ram = ram;
            Reset();
        }

        public int RamSize => _ram.Length;

        // --- Status flags ---

        private const ushort FlagZ = 0x0002;
        private const ushort FlagCy = 0x0004;
        private const ushort FlagS = 0x0008;
        private const ushort FlagOv = 0x0010;
        private const ushort FlagGo = 0x0020;
        private const ushort FlagRomPending = 0x0040;
        private const ushort FlagAlt1 = 0x0100;
        private const ushort FlagAlt2 = 0x0200;
        private const ushort FlagImmLow = 0x0400;
        private const ushort FlagImmHigh = 0x0800;
        private const ushort FlagWith = 0x1000;
        private const ushort FlagIrq = 0x8000;

        private bool GetFlag(ushort flag) => (_sfr & flag) != 0;

        private void SetFlag(ushort flag, bool value)
        {
            if (value) _sfr |= flag;
            else _sfr = (ushort)(_sfr & ~flag);
        }

        public bool Running => GetFlag(FlagGo);

        // Side-effect-free views for the debug target - ReadRegister($3031)
        // acknowledges the interrupt, so a debugger must not go through it.
        // Work RAM, framebuffer and save data all at once - see §1.
        public byte[] DebugRam => _ram;

        public ushort DebugSfr => _sfr;
        public byte DebugPbr => _pbr;
        public ushort DebugCbr => _cbr;
        public byte DebugScbr => _scbr;
        public byte DebugScmr => _scmr;
        public byte DebugRombr => _rombr;
        public byte DebugRambr => _rambr;

        // POR carries the OBJ-mode bit that picks the framebuffer layout, so it
        // is as load-bearing as SCMR - see Venus_SuperFX.md §6.2.
        public byte DebugPor => _por;
        public byte DebugColr => _colr;

        // The GSU's IRQ line into the S-CPU, masked by CFGR bit 7 - see Venus_SuperFX.md §3.2.
        public bool ScpuIrqPending => GetFlag(FlagIrq) && (_cfgr & 0x80) == 0;

        public void Reset()
        {
            System.Array.Clear(R);
            _sfr = 0;
            _pbr = _rombr = _rambr = _scbr = _scmr = _cfgr = _clsr = _bramr = 0;
            _cbr = 0;
            _colr = _por = 0;
            _sreg = _dreg = 0;
            _pipeline = 0x01; // NOP, so a stray step before a real fetch does nothing
            _romBuffer = 0;
            _clockBudget = 0;
            InvalidateCache();
            ResetPixelCache();
        }

        private void InvalidateCache() => System.Array.Clear(_cacheValid);

        // GSU-1 runs at half the master clock, GSU-2 at the full rate; CLSR
        // bit 0 picks - see Venus_SuperFX.md §2.2.
        private int MasterClocksPerCycle => (_clsr & 0x01) != 0 ? 1 : 2;

        // Advances the GSU by the master clocks the S-CPU just consumed.
        public void Run(int masterClocks)
        {
            _clockBudget += masterClocks;
            if (!Running)
            {
                _clockBudget = 0;
                return;
            }

            int perCycle = MasterClocksPerCycle;
            if (EmuSen.Debug.DebugSettings.SuperFxSpeedDivisor > 1) perCycle *= EmuSen.Debug.DebugSettings.SuperFxSpeedDivisor;
            while (_clockBudget > 0 && Running)
            {
                int cycles = StepInstruction();
                _clockBudget -= cycles * perCycle;
            }

            if (!Running) _clockBudget = 0;
        }

        // Starting the GSU is a side effect of the S-CPU writing R15's high
        // byte - see Venus_SuperFX.md §3.3.
        private void Start()
        {
            SetFlag(FlagGo, true);
            SetFlag(FlagIrq, false);
            ReloadPipeline();
        }

        private void Stop()
        {
            // Anything still in the pixel cache has to reach RAM before the
            // S-CPU starts reading the framebuffer back.
            FlushPixelCache();
            SetFlag(FlagGo, false);
            SetFlag(FlagIrq, true);
            _sreg = _dreg = 0;
        }
    }
}
