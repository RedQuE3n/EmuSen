using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // A Harvard 16-bit DSP running masked-in firmware the cartridge never exposes - see Venus_NecDSP.md.
    public sealed partial class NecDsp
    {
        // LoadRom re-reads the dump before any state load, so neither ROM belongs in a save state.
        [SkipInState] private readonly uint[] _program;
        [SkipInState] private readonly ushort[] _dataRom;

        [SkipInState] private readonly NecDspVariant _variant;
        [SkipInState] private readonly int _programMask;
        [SkipInState] private readonly int _dataRomMask;
        [SkipInState] private readonly int _ramMask;
        [SkipInState] private readonly int _stackMask;
        [SkipInState] private readonly int _clockHz;

        // Which address bit selects SR over DR, set by the memory map the cartridge uses - see Venus_NecDSP.md §3.
        [SkipInState] private readonly ushort _registerMask;
        [SkipInState] private readonly bool _hiRomMap;

        // Set from the cartridge's region, since the DSP's own crystal converts against master clocks.
        [SkipInState] public int MasterClockHz = 21477272;

        public ushort[] Ram;
        private ushort[] _stack;

        // Accumulators A and B, indexed by the opcode's accumulator-select bit.
        private ushort[] _acc = new ushort[2];
        private bool[] _carry = new bool[2];
        private bool[] _zero = new bool[2];
        private bool[] _overflow0 = new bool[2];
        private bool[] _overflow1 = new bool[2];
        private bool[] _sign0 = new bool[2];
        private bool[] _sign1 = new bool[2];

        private ushort _tr, _trb;   // temporary registers
        private ushort _pc;
        private ushort _rp;         // data ROM pointer
        private ushort _dp;         // data RAM pointer
        private ushort _dr;         // data register, the host's window into the chip
        private ushort _sr;         // status register
        private ushort _k, _l;      // multiplier inputs
        private ushort _m, _n;      // multiplier outputs
        private ushort _serialIn, _serialOut;
        private byte _sp;

        private uint _opcode;
        private long _clockBudget;

        // The DSP is spinning on RQM waiting for the host, so there is nothing to gain from stepping it until - see Venus_NecDSP.md §4.3.
        private bool _inRqmLoop;

        public NecDsp(NecDspVariant variant, NecDspFirmware firmware, bool hiRom)
        {
            NecDspProfile profile = NecDspProfile.For(variant);
            _variant = variant;
            _clockHz = profile.ClockHz;

            // Instructions are 24-bit, three bytes each, decoded once up front.
            _program = new uint[profile.ProgramBytes / 3];
            for (int i = 0; i < _program.Length; i++)
            {
                _program[i] = (uint)(firmware.Program[i * 3] | (firmware.Program[(i * 3) + 1] << 8) | (firmware.Program[(i * 3) + 2] << 16));
            }
            _programMask = _program.Length - 1;

            _dataRom = firmware.DataRom;
            _dataRomMask = _dataRom.Length - 1;

            Ram = new ushort[profile.RamWords];
            _ramMask = Ram.Length - 1;
            _stack = new ushort[profile.StackDepth];
            _stackMask = _stack.Length - 1;

            _hiRomMap = hiRom;
            _registerMask = SelectRegisterMask(variant, hiRom);
            Reset();
        }

        public string Name => _variant.ToString().ToUpperInvariant();

        // Only the uPD96050's 2KB of data RAM is battery-backed - see Venus_NecDSP.md §6.
        public bool HasBatteryRam => NecDspProfile.IsSt01x(_variant);

        // Side-effect-free views for the debug target - the host's own DR/SR reads advance the transfer - see Venus_NecDSP.md §3.2.
        public ushort DebugPc => _pc;
        public ushort DebugSr => _sr;
        public ushort DebugDr => _dr;
        public ushort DebugDp => _dp;
        public ushort DebugRp => _rp;

        // Firmware words for the disassembler, which decodes 24 bits at a time - see Venus_NecDSP.md §8.
        public int DebugProgramWords => _program.Length;
        public int DebugProgramWord(int index) => (int)_program[index & _programMask];

        // Same pull-hook shape as Sa1.BreakpointChecker, on the DSP's word PC - see Venus_NecDSP.md §9.
        [SkipInState] public System.Func<int, bool>? BreakpointChecker;

        // The chip's own CALL/RET pair, for a backtrace - see Venus_NecDSP.md §9.
        [SkipInState] public System.Action<int, int>? CallObserver;
        [SkipInState] public System.Action? ReturnObserver;

        [SkipInState] public bool HaltedAtBreakpoint;

        [SkipInState] public int HaltedAddress;

        // Skips one check after resuming, so `continue` leaves the breakpoint.
        [SkipInState] private bool _justResumedFromBreakpoint;

        // Called by VenusCore when re-entering a frame that halted on this chip.
        public void ResumeFromBreakpoint()
        {
            HaltedAtBreakpoint = false;
            _justResumedFromBreakpoint = true;
        }

        public void Reset()
        {
            System.Array.Clear(_acc);
            System.Array.Clear(_carry);
            System.Array.Clear(_zero);
            System.Array.Clear(_overflow0);
            System.Array.Clear(_overflow1);
            System.Array.Clear(_sign0);
            System.Array.Clear(_sign1);
            _tr = _trb = 0;
            _pc = _rp = _dp = _dr = _sr = 0;
            _k = _l = _m = _n = 0;
            _serialIn = _serialOut = 0;
            _sp = 0;
            _opcode = 0;
            _clockBudget = 0;
            _inRqmLoop = false;
        }

        // Advances the DSP by the master clocks the S-CPU just consumed - see Venus_NecDSP.md §4.1.
        public void Run(int masterClocks)
        {
            if (_inRqmLoop)
            {
                _clockBudget = 0;
                return;
            }

            _clockBudget += (long)masterClocks * _clockHz;
            while (_clockBudget >= MasterClockHz)
            {
                // Returns with _clockBudget intact, so resuming re-enters here - see Venus_NecDSP.md §9.
                if (!_justResumedFromBreakpoint && BreakpointChecker != null && BreakpointChecker(_pc & _programMask))
                {
                    HaltedAtBreakpoint = true;
                    HaltedAddress = _pc & _programMask;
                    return;
                }
                _justResumedFromBreakpoint = false;

                _clockBudget -= MasterClockHz;
                Step();
                if (_inRqmLoop)
                {
                    _clockBudget = 0;
                    return;
                }
            }
        }

        private void Step()
        {
            _opcode = _program[_pc & _programMask];
            _pc++;

            switch (_opcode & 0xC00000)
            {
                case 0x000000: ExecOp(); break;
                case 0x400000: ExecAndReturn(); break;
                case 0x800000: Jump(); break;
                default: Load((byte)(_opcode & 0x0F), (ushort)(_opcode >> 6)); break;
            }

            // The multiplier is combinational: every instruction re-latches it from whatever K and L now hold - see Venus_NecDSP.md §4.2.
            int product = (short)_k * (short)_l;
            _m = (ushort)(product >> 15);
            _n = (ushort)(product << 1);
        }
    }
}
