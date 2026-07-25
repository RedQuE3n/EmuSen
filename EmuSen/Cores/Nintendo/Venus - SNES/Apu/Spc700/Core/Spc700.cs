using System;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    [Flags]
    public enum SpcFlags : byte
    {
        None = 0,
        C = (1 << 0), // Carry
        Z = (1 << 1), // Zero
        I = (1 << 2), // Interrupt Enable
        H = (1 << 3), // Half Carry
        B = (1 << 4), // Break
        P = (1 << 5), // Direct Page (0 = $0000, 1 = $0100)
        V = (1 << 6), // Overflow
        N = (1 << 7)  // Negative
    }

    public struct SpcInstruction
    {
        public string Name;
        public Func<ushort> AddrMode;    
        public Action<ushort> Operate;   
        public byte Cycles;            
    }

    public partial class Spc700
    {
        // Logs CPU<->APU port traffic on value changes only - see Venus_APU.md §1.2.
        public bool LogPortTraffic = false;
        private byte[] _lastCpuWrite = new byte[] { 0xFF, 0xFF, 0xFF, 0xFF };
        private byte[] _lastSpcWrite = new byte[] { 0xFF, 0xFF, 0xFF, 0xFF };

        public int CycleBudget { get; set; }
        public int TotalBytesStored { get; private set; }

        public byte[] Ram = new byte[65536]; 
        
        // --- S-DSP Instance ---
        public SDsp Dsp { get; private set; } = new SDsp();
        
        public byte A;
        public byte X;
        public byte Y;
        public byte SP;
        public ushort PC;
        public byte PSW; 

        // Set by SLEEP/STOP - see Venus_APU.md §1.4.
        private bool _halted;

        private byte[] _inPorts = new byte[4];
        private byte[] _outPorts = new byte[4];
        
        // --- Timer State ---
        private byte _timerControl; // $00F1
        
        private int _timer0Cycles;
        private byte _timer0Internal;
        private byte _timer0Target; // $00FA
        private byte _timer0Counter; // $00FD

        private int _timer1Cycles;
        private byte _timer1Internal;
        private byte _timer1Target; // $00FB
        private byte _timer1Counter; // $00FE

        private int _timer2Cycles;
        private byte _timer2Internal;
        private byte _timer2Target; // $00FC
        private byte _timer2Counter; // $00FF

        [EmuSen.Common.SkipInState] private SpcInstruction[] _instructions = null!;

        // Real SNES boot ROM, hardcoded - see Venus_APU.md §1.1.
        private static readonly byte[] IplRom = new byte[]
        {
            0xCD, 0xEF, 0xBD, 0xE8, 0x00, 0xC6, 0x1D, 0xD0, 0xFC, 0x8F, 0xAA, 0xF4, 0x8F, 0xBB, 0xF5, 0x78,
            0xCC, 0xF4, 0xD0, 0xFB, 0x2F, 0x19, 0xEB, 0xF4, 0xD0, 0xFC, 0x7E, 0xF4, 0xD0, 0x0B, 0xE4, 0xF5,
            0xCB, 0xF4, 0xD7, 0x00, 0xFC, 0xD0, 0xF3, 0xAB, 0x01, 0x10, 0xEF, 0x7E, 0xF4, 0x10, 0xEB, 0xBA,
            0xF6, 0xDA, 0x00, 0xBA, 0xF4, 0xC4, 0xF4, 0xDD, 0x5D, 0xD0, 0xDB, 0x1F, 0x00, 0x00, 0xC0, 0xFF
        };

        // Equality key for one executed instruction, used only by
        // _verboseTrace to detect repeating polling/delay loops (the APU
        // handshake being the canonical example - see the comment near
        // Step()'s use of Spc700VerboseLogging). Same rationale as Cpu's
        // own StepKey: compare a small struct instead of the rendered
        // string, and only look Name up again in Render() for lines that
        // actually get emitted.
        private readonly struct StepKey : IEquatable<StepKey>
        {
            public readonly ushort Pc;
            public readonly byte Opcode;
            public readonly ushort TargetAddr;

            public StepKey(ushort pc, byte opcode, ushort targetAddr)
            {
                Pc = pc;
                Opcode = opcode;
                TargetAddr = targetAddr;
            }

            public bool Equals(StepKey other) =>
                Pc == other.Pc && Opcode == other.Opcode && TargetAddr == other.TargetAddr;

            public override bool Equals(object? obj) => obj is StepKey other && Equals(other);

            public override int GetHashCode() => HashCode.Combine(Pc, Opcode, TargetAddr);
        }

        private readonly EmuSen.Debug.DebugTools.RepeatCollapsingTrace<StepKey> _verboseTrace;
        private bool _wasVerboseLogging;

        public Spc700()
        {
            BuildOpcodeTable();
            Dsp.AttachMemory(Ram);
            _verboseTrace = new EmuSen.Debug.DebugTools.RepeatCollapsingTrace<StepKey>(
                Console.WriteLine,
                key => $"[SPC700] 0x{key.Pc:X4}: {_instructions[key.Opcode].Name} (Opcode 0x{key.Opcode:X2}) -> Target Addr: 0x{key.TargetAddr:X4}",
                (cycleLength, repeats) => cycleLength == 1
                    ? $"[SPC700]     ^ repeated {repeats}x total"
                    : $"[SPC700]     ^ {cycleLength}-instruction loop above repeated {repeats}x total");
            Reset();
        }

        // Step() already flushes automatically the moment it notices
        // Spc700VerboseLogging went from on to off (see _wasVerboseLogging)
        // - this is for the one case Step() can't see coming: the process
        // exiting while logging is still on. Program.cs's shutdown path
        // calls this before disposing the log writer.
        public void FlushVerboseTrace() => _verboseTrace.Flush();

        public void Reset()
        {
            Array.Copy(IplRom, 0, Ram, 0xFFC0, IplRom.Length);

            PC = 0xFFC0; 
            A = 0x00;
            X = 0x00;
            Y = 0x00;
            SP = 0xEF; 
            PSW = 0x00;
            _halted = false;
            
            Array.Clear(_inPorts, 0, 4);
            Array.Clear(_outPorts, 0, 4);

            _timerControl = 0;
            _timer0Cycles = _timer1Cycles = _timer2Cycles = 0;
            _timer0Internal = _timer1Internal = _timer2Internal = 0;
            _timer0Target = _timer1Target = _timer2Target = 0;
            _timer0Counter = _timer1Counter = _timer2Counter = 0;

            // Reset the DSP
            Dsp.Reset();
        }

        public byte ReadPort(byte port)
        {
            return _outPorts[port & 0x03];
        }

        // Debug visibility: what the CPU most recently wrote into each APU-side port.
        public byte GetInPort(int port) => _inPorts[port & 0x03];

        private int[] _milestoneCounts = new int[4];

        private void TraceMilestone(int idx, string name, string detail)
        {
            if (_milestoneCounts[idx] >= 6) return;
            _milestoneCounts[idx]++;
            Console.WriteLine($"[TRACE] Reached {name} {detail} (hit #{_milestoneCounts[idx]})");
        }

        public void WritePort(byte port, byte data)
        {
            port &= 0x03;
            _inPorts[port] = data;
            TotalBytesStored++;

            if (LogPortTraffic && data != _lastCpuWrite[port])
            {
                _lastCpuWrite[port] = data;
                Console.WriteLine($"[PORT] CPU -> APU port {port}: 0x{data:X2} (SPC PC=0x{PC:X4})");
            }
        }

        // Public purely for testability - see SpcValidation harness (or
        // wherever this ends up documented). A raw poke into Ram[] isn't
        // equivalent to a real write for $00F2-$00F7 (DSP register access,
        // APU communication ports) - those never touch Ram at all, they
        // live only in Dsp's register file / _inPorts/_outPorts - so a
        // test harness needs the real Read8/Write8 semantics to set up and
        // verify state for any test touching that range. No effect on
        // production callers, which already only ever called these from
        // within this class.
        public byte Read8(ushort address)
        {
            // Intercept APU Communication Ports
            if (address >= 0x00F4 && address <= 0x00F7)
            {
                return _inPorts[address - 0x00F4];
            }

            // Intercept S-DSP Read Registers
            if (address == 0x00F2) return Dsp.GetRegisterAddress();
            if (address == 0x00F3) return Dsp.ReadRegister();

            // Timer counters - reading clears them, see Venus_APU.md §1.3.
            switch (address)
            {
                case 0x00FD:
                    byte t0 = _timer0Counter;
                    _timer0Counter = 0;
                    return t0;
                case 0x00FE:
                    byte t1 = _timer1Counter;
                    _timer1Counter = 0;
                    return t1;
                case 0x00FF:
                    byte t2 = _timer2Counter;
                    _timer2Counter = 0;
                    return t2;
            }

            return Ram[address];
        }

        // Public for the same testability reason as Read8 above.
        public void Write8(ushort address, byte data)
        {
            // Intercept APU Communication Ports
            if (address >= 0x00F4 && address <= 0x00F7)
            {
                int portIdx = address - 0x00F4;
                _outPorts[portIdx] = data;

                if (LogPortTraffic && data != _lastSpcWrite[portIdx])
                {
                    _lastSpcWrite[portIdx] = data;
                    Console.WriteLine($"[PORT] APU -> CPU port {portIdx}: 0x{data:X2} (SPC PC=0x{PC:X4})");
                }

                return;
            }

            // Intercept S-DSP Write Registers
            if (address == 0x00F2) 
            { 
                Dsp.SetRegisterAddress(data); 
                return; 
            }
            if (address == 0x00F3) 
            { 
                Dsp.WriteRegister(data); 
                return; 
            }

            // Intercept Timer Setup and Targets
            switch (address)
            {
                case 0x00F1:
                    _timerControl = data;
                    // Disabling a timer resets its prescaler - see Venus_APU.md §1.3.
                    if ((data & 0x01) == 0) { _timer0Cycles = 0; _timer0Internal = 0; }
                    if ((data & 0x02) == 0) { _timer1Cycles = 0; _timer1Internal = 0; }
                    if ((data & 0x04) == 0) { _timer2Cycles = 0; _timer2Internal = 0; }

                    // Bits 4/5 clear the CPU->APU input latches - see Venus_APU.md §1.3.
                    if ((data & 0x10) != 0) { _inPorts[0] = 0; _inPorts[1] = 0; }
                    if ((data & 0x20) != 0) { _inPorts[2] = 0; _inPorts[3] = 0; }
                    break;
                case 0x00FA: _timer0Target = data; break;
                case 0x00FB: _timer1Target = data; break;
                case 0x00FC: _timer2Target = data; break;
            }

            Ram[address] = data; 
        }

        // --- Stack Helpers ---
        
        private void Push8(byte value)
        {
            Write8((ushort)(0x0100 | SP), value);
            SP--;
        }

        private byte Pop8()
        {
            SP++;
            return Read8((ushort)(0x0100 | SP));
        }

        // --- Flag Helpers ---
        
        public void SetFlag(SpcFlags flag, bool condition)
        {
            if (condition) PSW |= (byte)flag;
            else PSW &= (byte)~flag;
        }

        public bool GetFlag(SpcFlags flag) { return (PSW & (byte)flag) != 0; }

        private void UpdateZN(byte value)
        {
            SetFlag(SpcFlags.Z, value == 0);
            SetFlag(SpcFlags.N, (value & 0x80) != 0);
        }

        // --- Execution Engine ---
        
        public void Step()
        {
            if (CycleBudget <= 0) return;

            // SLEEP/STOP - see Venus_APU.md §1.4.
            if (_halted)
            {
                CycleBudget -= 2;
                return;
            }

            ushort executedAtPC = PC;
            byte opcode = Read8(PC);
            // One-shot dispatch-chain tracer - see Venus_APU.md §1.5.
            if (LogPortTraffic)
            {
                switch (PC)
                {
                    case 0x05A5: TraceMilestone(0, "port handler $05A5", $"X={X}"); break;
                    case 0x0816: TraceMilestone(1, "port-1 dispatcher $0816", $"A=0x{A:X2}"); break;
                    case 0x09E5: TraceMilestone(2, "port-0 dispatcher $09E5", $"A=0x{A:X2}"); break;
                    case 0xFFC0: TraceMilestone(3, "IPL restart $FFC0", ""); break;
                }
            }
            PC++;

            SpcInstruction inst = _instructions[opcode];

            if (inst.Name == "NOP/UNK")
            {
                throw new NotImplementedException($"Unimplemented SPC700 Opcode: 0x{opcode:X2} at PC: 0x{(PC - 1):X4}");
            }

            ushort targetAddr = inst.AddrMode();
            inst.Operate(targetAddr);

            if (DebugSettings.Spc700VerboseLogging)
            {
                _verboseTrace.Log(new StepKey(executedAtPC, opcode, targetAddr));
                _wasVerboseLogging = true;
            }
            else if (_wasVerboseLogging)
            {
                // Verbose logging just turned off - flush whatever
                // loop/tail _verboseTrace was still holding rather than
                // stranding it. See Cpu.Step()'s identical pattern.
                _verboseTrace.Flush();
                _wasVerboseLogging = false;
            }


            TickTimers(inst.Cycles);
            
            // Tick the DSP along with the timers
            Dsp.Tick(inst.Cycles);

            CycleBudget -= inst.Cycles;
        }

        private void TickTimers(int cycles)
        {
            // Timer 0 ticks every 128 cycles
            if ((_timerControl & 0x01) != 0)
            {
                _timer0Cycles += cycles;
                while (_timer0Cycles >= 128)
                {
                    _timer0Cycles -= 128;
                    _timer0Internal++;
                    if (_timer0Internal == _timer0Target)
                    {
                        _timer0Internal = 0;
                        _timer0Counter = (byte)((_timer0Counter + 1) & 0x0F);
                    }
                }
            }

            // Timer 1 ticks every 128 cycles
            if ((_timerControl & 0x02) != 0)
            {
                _timer1Cycles += cycles;
                while (_timer1Cycles >= 128)
                {
                    _timer1Cycles -= 128;
                    _timer1Internal++;
                    if (_timer1Internal == _timer1Target)
                    {
                        _timer1Internal = 0;
                        _timer1Counter = (byte)((_timer1Counter + 1) & 0x0F);
                    }
                }
            }

            // Timer 2 ticks every 16 cycles
            if ((_timerControl & 0x04) != 0)
            {
                _timer2Cycles += cycles;
                while (_timer2Cycles >= 16)
                {
                    _timer2Cycles -= 16;
                    _timer2Internal++;
                    if (_timer2Internal == _timer2Target)
                    {
                        _timer2Internal = 0;
                        _timer2Counter = (byte)((_timer2Counter + 1) & 0x0F);
                    }
                }
            }
        }

    }
}
