namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // The GSU instruction set. One byte per opcode, with four meanings per
    // slot selected by the ALT1/ALT2 prefix flags, plus TO/WITH/FROM prefixes
    // that redirect the source and destination registers - see Venus_SuperFX.md §4.
    public sealed partial class SuperFx
    {
        // Set by the prefix opcodes so the dispatcher leaves the prefix state alone.
        private bool _prefixInstruction;

        // The last RAM address a load/store touched, which SBK writes back to.
        private ushort _lastRamAddress;

        private ushort Src => R[_sreg];

        private bool Alt1 => GetFlag(FlagAlt1);
        private bool Alt2 => GetFlag(FlagAlt2);

        private void ClearPrefix()
        {
            SetFlag(FlagAlt1, false);
            SetFlag(FlagAlt2, false);
            SetFlag(FlagWith, false);
            _sreg = 0;
            _dreg = 0;
        }

        // R15 is the program counter, so writing it is a jump; the already
        // prefetched byte still executes as a delay slot - see Venus_SuperFX.md §4.1.
        private void WriteReg(int index, ushort value)
        {
            if (index == 15) { SetPc(value); return; }

            R[index] = value;

            // Writing R14 starts a ROM fetch that GETB and friends collect.
            if (index == 14) _romBuffer = ReadRomBuffer();
        }

        private void Dst(ushort value) => WriteReg(_dreg, value);

        // R15 always names the byte sitting in the pipeline, so software that
        // reads it - the `WITH R15 : TO R13` idiom that captures a loop start,
        // or LINK - sees the next instruction's address rather than one past
        // it. See Venus_SuperFX.md §4.1.
        private byte Pipe()
        {
            byte result = _pipeline;

            // After a jump R15 already names the destination; the byte being
            // consumed right now is the delay slot, so don't advance past it.
            if (_jumpPending) _jumpPending = false;
            else R[15]++;

            _pipelineAddress = R[15];
            _pipeline = FetchProgramByte(R[15]);
            return result;
        }

        private void ReloadPipeline()
        {
            _pipelineAddress = R[15];
            _pipeline = FetchProgramByte(R[15]);
            _jumpPending = false;
        }

        // Every write to R15 is a jump, and every jump runs one delay-slot
        // instruction before it takes effect.
        private void SetPc(ushort target)
        {
            R[15] = target;
            _jumpPending = true;
        }

        private void SetZS(ushort value)
        {
            SetFlag(FlagZ, value == 0);
            SetFlag(FlagS, (value & 0x8000) != 0);
        }

        private int RamAddress(ushort address) => ((_rambr & 0x1F) << 16) | address;

        private byte ReadRamByte(ushort address) => ReadRam(RamAddress(address));

        private void WriteRamByte(ushort address, byte data) => WriteRam(RamAddress(address), data);

        // The high byte lands at address ^ 1, not address + 1 - see Venus_SuperFX.md §5.1a.
        private ushort ReadRamWord(ushort address) =>
            (ushort)(ReadRamByte(address) | (ReadRamByte((ushort)(address ^ 1)) << 8));

        private void WriteRamWord(ushort address, ushort data)
        {
            WriteRamByte(address, (byte)data);
            WriteRamByte((ushort)(address ^ 1), (byte)(data >> 8));
        }

        private int StepInstruction()
        {
            if (EmuSen.Debug.DebugSettings.SuperFxTraceCountdown > 0)
            {
                EmuSen.Debug.DebugSettings.SuperFxTraceCountdown--;
                System.Console.WriteLine(
                    $"[GSU] {_pbr:X2}:{_pipelineAddress:X4} op={_pipeline:X2} sfr={_sfr:X4} cbr={_cbr:X4} "
                    + $"R0={R[0]:X4} R1={R[1]:X4} R2={R[2]:X4} R11={R[11]:X4} R13={R[13]:X4} R14={R[14]:X4}");
            }

            if (EmuSen.Debug.DebugSettings.SuperFxPlotTraceSkip == 0
                && EmuSen.Debug.DebugSettings.SuperFxPlotTraceInstr > 0)
            {
                EmuSen.Debug.DebugSettings.SuperFxPlotTraceInstr--;
                System.Console.WriteLine(
                    $"[I] {_pbr:X2}:{_pipelineAddress:X4} {_pipeline:X2} R0={R[0]:X4} R1={R[1]:X4} R2={R[2]:X4} "
                    + $"R3={R[3]:X4} R4={R[4]:X4} R5={R[5]:X4} R10={R[10]:X4} R11={R[11]:X4} R12={R[12]:X4} R14={R[14]:X4} rb={_romBuffer:X2} rombr={_rombr:X2} sfr={_sfr:X4}");
            }


            CoverageRecorder?.Invoke((_pbr << 16) | _pipelineAddress);

            // Before Pipe(), so registers are this instruction's inputs - the point Mesen records at.
            bool tracing = EmuSen.Cores.Nintendo.Venus.Debug.GsuBinaryTrace.Enabled;
            if (tracing)
            {
                EmuSen.Cores.Nintendo.Venus.Debug.GsuBinaryTrace.Record(
                    (uint)((_pbr << 16) | _pipelineAddress), _pipeline, _sfr, _sreg, _dreg, R);
            }

            byte opcode = Pipe();
            _prefixInstruction = false;
            int cycles = Execute(opcode);
            if (!_prefixInstruction) ClearPrefix();
            if (tracing) EmuSen.Cores.Nintendo.Venus.Debug.GsuBinaryTrace.SetLastCost(cycles * MasterClocksPerCycle);
            return cycles;
        }

        private int Execute(byte opcode)
        {
            int n = opcode & 0x0F;

            switch (opcode)
            {
                case 0x00: Stop(); return 1;
                case 0x01: return 1; // NOP

                // Point the cache window at the current address, flushing if it moved.
                case 0x02:
                {
                    ushort target = (ushort)(R[15] & 0xFFF0);
                    if (_cbr != target) { _cbr = target; InvalidateCache(); }
                    return 1;
                }

                case 0x03: return OpLsr();
                case 0x04: return OpRol();

                case 0x05: return Branch(true);
                // $06 is BGE and $07 is BLT, not the reverse - see Venus_SuperFX.md §9.
                case 0x06: return Branch(GetFlag(FlagS) == GetFlag(FlagOv));
                case 0x07: return Branch(GetFlag(FlagS) != GetFlag(FlagOv));
                case 0x08: return Branch(!GetFlag(FlagZ));
                case 0x09: return Branch(GetFlag(FlagZ));
                case 0x0A: return Branch(!GetFlag(FlagS));
                case 0x0B: return Branch(GetFlag(FlagS));
                case 0x0C: return Branch(!GetFlag(FlagCy));
                case 0x0D: return Branch(GetFlag(FlagCy));
                case 0x0E: return Branch(!GetFlag(FlagOv));
                case 0x0F: return Branch(GetFlag(FlagOv));

                case >= 0x10 and <= 0x1F: return OpToOrMove(n);
                case >= 0x20 and <= 0x2F: return OpWith(n);
                case >= 0x30 and <= 0x3B: return OpStore(n);

                case 0x3C: return OpLoop();
                // Each ALT supersedes a pending WITH - see Venus_SuperFX.md §4.2.
                case 0x3D: SetFlag(FlagWith, false); SetFlag(FlagAlt1, true); _prefixInstruction = true; return 1;
                case 0x3E: SetFlag(FlagWith, false); SetFlag(FlagAlt2, true); _prefixInstruction = true; return 1;
                case 0x3F: SetFlag(FlagWith, false); SetFlag(FlagAlt1, true); SetFlag(FlagAlt2, true); _prefixInstruction = true; return 1;

                case >= 0x40 and <= 0x4B: return OpLoad(n);

                case 0x4C: return Alt1 ? OpRpix() : OpPlot();
                case 0x4D: return OpSwap();
                case 0x4E: return Alt1 ? OpCmode() : OpColor();
                case 0x4F: return OpNot();

                case >= 0x50 and <= 0x5F: return OpAdd(n);
                case >= 0x60 and <= 0x6F: return OpSub(n);

                case 0x70: return OpMerge();
                case >= 0x71 and <= 0x7F: return OpAnd(n);
                case >= 0x80 and <= 0x8F: return OpMult(n);

                case 0x90: WriteRamWord(_lastRamAddress, Src); return 4;
                case >= 0x91 and <= 0x94: WriteReg(11, (ushort)(R[15] + n)); return 1;
                case 0x95: Dst((ushort)(sbyte)Src); SetZS((ushort)(sbyte)Src); return 1;
                case 0x96: return Alt1 ? OpDiv2() : OpAsr();
                case 0x97: return OpRor();

                case >= 0x98 and <= 0x9D: return Alt1 ? OpLjmp(n) : OpJmp(n);

                case 0x9E: return OpLob();
                case 0x9F: return OpFmult();

                case >= 0xA0 and <= 0xAF: return OpImmediateByteOrShort(n);
                case >= 0xB0 and <= 0xBF: return OpFromOrMoves(n);

                case 0xC0: return OpHib();
                case >= 0xC1 and <= 0xCF: return OpOr(n);

                case >= 0xD0 and <= 0xDE: return OpInc(n);
                case 0xDF: return OpGetcRambRomb();

                case >= 0xE0 and <= 0xEE: return OpDec(n);
                case 0xEF: return OpGetb();

                default: return OpIwtLmSm(n); // $F0-$FF
            }
        }

        // Branches take their displacement from the instruction stream and are
        // relative to the delay-slot instruction that follows - see Venus_SuperFX.md §4.1.
        // A branch is the one non-prefix instruction that does NOT clear the prefix
        // state, so TO/FROM/WITH/ALT ahead of it apply to the delay slot - see §4.2.
        private int Branch(bool take)
        {
            sbyte displacement = (sbyte)Pipe();
            if (take) SetPc((ushort)(R[15] + displacement));
            _prefixInstruction = true;
            return 1;
        }

        private int OpToOrMove(int n)
        {
            if (GetFlag(FlagWith))
            {
                WriteReg(n, Src); // MOVE sets no flags
                return 1;
            }
            _dreg = n;
            _prefixInstruction = true;
            return 1;
        }

        private int OpWith(int n)
        {
            _sreg = n;
            _dreg = n;
            SetFlag(FlagWith, true);
            _prefixInstruction = true;
            return 1;
        }

        private int OpFromOrMoves(int n)
        {
            if (GetFlag(FlagWith))
            {
                ushort value = R[n];
                Dst(value);
                SetFlag(FlagOv, (value & 0x80) != 0);
                SetZS(value);
                return 1;
            }
            _sreg = n;
            _prefixInstruction = true;
            return 1;
        }

        private int OpStore(int n)
        {
            _lastRamAddress = R[n];
            if (Alt1) WriteRamByte(R[n], (byte)Src);
            else WriteRamWord(R[n], Src);
            return 4;
        }

        private int OpLoad(int n)
        {
            _lastRamAddress = R[n];
            ushort value = Alt1 ? ReadRamByte(R[n]) : ReadRamWord(R[n]);
            // LDW/LDB have no flag effects - see Venus_SuperFX.md §4.4.
            Dst(value);
            return 5;
        }

        private int OpLoop()
        {
            ushort counter = (ushort)(R[12] - 1);
            WriteReg(12, counter);
            SetZS(counter);
            if (counter != 0) SetPc(R[13]);
            return 1;
        }

        private int OpLsr()
        {
            ushort value = Src;
            SetFlag(FlagCy, (value & 1) != 0);
            ushort result = (ushort)(value >> 1);
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpAsr()
        {
            ushort value = Src;
            SetFlag(FlagCy, (value & 1) != 0);
            ushort result = (ushort)((short)value >> 1);
            Dst(result);
            SetZS(result);
            return 1;
        }

        // DIV2 is ASR with the one case that would round -1 to -1 forced to 0.
        private int OpDiv2()
        {
            ushort value = Src;
            SetFlag(FlagCy, (value & 1) != 0);
            ushort result = value == 0xFFFF ? (ushort)0 : (ushort)((short)value >> 1);
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpRol()
        {
            ushort value = Src;
            ushort result = (ushort)((value << 1) | (GetFlag(FlagCy) ? 1 : 0));
            SetFlag(FlagCy, (value & 0x8000) != 0);
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpRor()
        {
            ushort value = Src;
            ushort result = (ushort)((value >> 1) | (GetFlag(FlagCy) ? 0x8000 : 0));
            SetFlag(FlagCy, (value & 1) != 0);
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpAdd(int n)
        {
            ushort a = Src;
            ushort b = Alt2 ? (ushort)n : R[n];
            int carry = (Alt1 && GetFlag(FlagCy)) ? 1 : 0;
            int result = a + b + carry;

            SetFlag(FlagCy, result > 0xFFFF);
            SetFlag(FlagOv, ((~(a ^ b)) & (a ^ result) & 0x8000) != 0);
            Dst((ushort)result);
            SetZS((ushort)result);
            return 1;
        }

        private int OpSub(int n)
        {
            ushort a = Src;
            bool compare = Alt1 && Alt2; // ALT3 on this slot is CMP
            ushort b = (Alt2 && !compare) ? (ushort)n : R[n];
            int borrow = (Alt1 && !compare && !GetFlag(FlagCy)) ? 1 : 0;
            int result = a - b - borrow;

            SetFlag(FlagCy, result >= 0);
            SetFlag(FlagOv, ((a ^ b) & (a ^ result) & 0x8000) != 0);
            if (!compare) Dst((ushort)result);
            SetZS((ushort)result);
            return 1;
        }

        private int OpAnd(int n)
        {
            ushort b = Alt2 ? (ushort)n : R[n];
            ushort result = (ushort)(Alt1 ? Src & ~b : Src & b);
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpOr(int n)
        {
            ushort b = Alt2 ? (ushort)n : R[n];
            ushort result = (ushort)(Alt1 ? Src ^ b : Src | b);
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpNot()
        {
            ushort result = (ushort)~Src;
            Dst(result);
            SetZS(result);
            return 1;
        }

        // 8x8 multiply, signed unless the ALT1 form is used.
        private int OpMult(int n)
        {
            int b = Alt2 ? n : R[n];
            int result = Alt1
                ? (byte)Src * (byte)b
                : (sbyte)Src * (sbyte)b;
            Dst((ushort)result);
            SetZS((ushort)result);
            return 1;
        }

        // Fractional 16x16 multiply against R6, keeping the high word.
        private int OpFmult()
        {
            int result = (short)Src * (short)R[6];
            if (Alt1) WriteReg(4, (ushort)result); // LMULT also keeps the low word in R4
            ushort high = (ushort)(result >> 16);
            Dst(high);
            SetFlag(FlagCy, ((result >> 15) & 1) != 0);
            SetZS(high);
            return Alt1 ? 8 : 8;
        }

        private int OpSwap()
        {
            ushort result = (ushort)((Src >> 8) | (Src << 8));
            Dst(result);
            SetZS(result);
            return 1;
        }

        private int OpLob()
        {
            ushort result = (ushort)(Src & 0x00FF);
            Dst(result);
            SetFlag(FlagZ, result == 0);
            SetFlag(FlagS, (result & 0x80) != 0);
            return 1;
        }

        private int OpHib()
        {
            ushort result = (ushort)(Src >> 8);
            Dst(result);
            SetFlag(FlagZ, result == 0);
            SetFlag(FlagS, (result & 0x80) != 0);
            return 1;
        }

        // Builds a clipping test out of R7/R8's high bytes; every flag is a
        // different threshold on the same value.
        private int OpMerge()
        {
            ushort result = (ushort)((R[7] & 0xFF00) | (R[8] >> 8));
            Dst(result);
            SetFlag(FlagS, (result & 0x8080) != 0);
            SetFlag(FlagOv, (result & 0xC0C0) != 0);
            SetFlag(FlagCy, (result & 0xE0E0) != 0);
            SetFlag(FlagZ, (result & 0xF0F0) != 0);
            return 1;
        }

        private int OpJmp(int n)
        {
            SetPc(R[n]);
            return 1;
        }

        // The long form additionally reloads the program bank, which moves the
        // cache window and invalidates everything in it. Rn carries the bank
        // and Sreg the address, not the other way round - see Venus_SuperFX.md §9.
        private int OpLjmp(int n)
        {
            _pbr = (byte)(R[n] & 0x7F);
            SetPc(Src);
            _cbr = (ushort)(R[15] & 0xFFF0);
            InvalidateCache();
            return 1;
        }

        private int OpInc(int n)
        {
            ushort result = (ushort)(R[n] + 1);
            WriteReg(n, result);
            SetZS(result);
            return 1;
        }

        private int OpDec(int n)
        {
            ushort result = (ushort)(R[n] - 1);
            WriteReg(n, result);
            SetZS(result);
            return 1;
        }

        private int OpGetcRambRomb()
        {
            if (Alt2 && Alt1) { _rombr = (byte)(Src & 0x7F); return 1; } // ROMB
            if (Alt2) { _rambr = (byte)Src; return 1; }                  // RAMB
            LoadColorFromRomBuffer();
            return 1;
        }

        private int OpGetb()
        {
            ushort result = (Alt1, Alt2) switch
            {
                (false, false) => _romBuffer,                                   // GETB
                (true, false) => (ushort)((Src & 0x00FF) | (_romBuffer << 8)),  // GETBH
                (false, true) => (ushort)((Src & 0xFF00) | _romBuffer),         // GETBL
                (true, true) => (ushort)(sbyte)_romBuffer,                      // GETBS
            };
            // No flag effects on any of the four - see Venus_SuperFX.md §4.4.
            Dst(result);
            return 1;
        }

        // $A0-$AF: an 8-bit immediate, or a short-address RAM load/store whose
        // operand indexes words rather than bytes.
        private int OpImmediateByteOrShort(int n)
        {
            if (Alt1 && !Alt2)
            {
                ushort address = (ushort)(Pipe() << 1);
                _lastRamAddress = address;
                WriteReg(n, ReadRamWord(address));
                return 6;
            }
            if (Alt2 && !Alt1)
            {
                ushort address = (ushort)(Pipe() << 1);
                _lastRamAddress = address;
                WriteRamWord(address, R[n]);
                return 6;
            }

            WriteReg(n, (ushort)(sbyte)Pipe());
            return 2;
        }

        // $F0-$FF: a 16-bit immediate, or a full-address RAM load/store.
        private int OpIwtLmSm(int n)
        {
            ushort operand = (ushort)(Pipe() | (Pipe() << 8));

            if (Alt1 && !Alt2)
            {
                _lastRamAddress = operand;
                WriteReg(n, ReadRamWord(operand));
                return 7;
            }
            if (Alt2 && !Alt1)
            {
                _lastRamAddress = operand;
                WriteRamWord(operand, R[n]);
                return 7;
            }

            WriteReg(n, operand);
            return 3;
        }
    }
}
