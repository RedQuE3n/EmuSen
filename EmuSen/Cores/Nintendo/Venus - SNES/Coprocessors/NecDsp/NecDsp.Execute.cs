using System;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // The four instruction forms - OP, RT (op then return), JP, and LD - and
    // the ALU they share. See Venus_NecDSP.md §5.
    public sealed partial class NecDsp
    {
        // --- Instruction forms ---

        private void ExecOp()
        {
            byte aluOperation = (byte)((_opcode >> 16) & 0x0F);
            ushort source = GetSourceValue((byte)((_opcode >> 4) & 0x0F));

            if (aluOperation != 0) RunAluOp(aluOperation, source);

            byte dest = (byte)(_opcode & 0x0F);
            Load(dest, source);

            // DP and RP are only stepped when the instruction didn't just load them.
            if (dest != 0x04) UpdateDataPointer();
            if (((_opcode >> 8) & 0x01) != 0 && dest != 0x05) _rp--;
        }

        private void ExecAndReturn()
        {
            ExecOp();
            _sp = (byte)((_sp - 1) & _stackMask);
            _pc = _stack[_sp];
            ReturnObserver?.Invoke();
        }

        private void Jump()
        {
            // The $2000 bit is inherited from PC, so a conditional branch stays
            // in the 8KB half it started in - see Venus_NecDSP.md §5.4.
            ushort target = (ushort)((_pc & 0x2000u) | ((_opcode & 0x03) << 11) | ((_opcode >> 2) & 0x7FF));
            ushort jumpType = (ushort)((_opcode >> 13) & 0x1FF);
            bool taken = false;

            switch (jumpType)
            {
                case 0x000: _pc = _serialOut; break;

                case 0x080: taken = !_carry[0]; break;
                case 0x082: taken = _carry[0]; break;
                case 0x084: taken = !_carry[1]; break;
                case 0x086: taken = _carry[1]; break;

                case 0x088: taken = !_zero[0]; break;
                case 0x08A: taken = _zero[0]; break;
                case 0x08C: taken = !_zero[1]; break;
                case 0x08E: taken = _zero[1]; break;

                case 0x090: taken = !_overflow0[0]; break;
                case 0x092: taken = _overflow0[0]; break;
                case 0x094: taken = !_overflow0[1]; break;
                case 0x096: taken = _overflow0[1]; break;
                case 0x098: taken = !_overflow1[0]; break;
                case 0x09A: taken = _overflow1[0]; break;
                case 0x09C: taken = !_overflow1[1]; break;
                case 0x09E: taken = _overflow1[1]; break;

                case 0x0A0: taken = !_sign0[0]; break;
                case 0x0A2: taken = _sign0[0]; break;
                case 0x0A4: taken = !_sign0[1]; break;
                case 0x0A6: taken = _sign0[1]; break;
                case 0x0A8: taken = !_sign1[0]; break;
                case 0x0AA: taken = _sign1[0]; break;
                case 0x0AC: taken = !_sign1[1]; break;
                case 0x0AE: taken = _sign1[1]; break;

                case 0x0B0: taken = (_dp & 0x0F) == 0; break;
                case 0x0B1: taken = (_dp & 0x0F) != 0; break;
                case 0x0B2: taken = (_dp & 0x0F) == 0x0F; break;
                case 0x0B3: taken = (_dp & 0x0F) != 0x0F; break;

                case 0x0B4: taken = (_sr & SerialInControl) == 0; break;
                case 0x0B6: taken = (_sr & SerialInControl) != 0; break;
                case 0x0B8: taken = (_sr & SerialOutControl) == 0; break;
                case 0x0BA: taken = (_sr & SerialOutControl) != 0; break;
                case 0x0BC: taken = (_sr & RequestForMaster) == 0; break;
                case 0x0BE: taken = (_sr & RequestForMaster) != 0; break;

                case 0x100: _pc = (ushort)(target & ~0x2000); break;
                case 0x101: _pc = (ushort)(target | 0x2000); break;

                case 0x140:
                    _stack[_sp] = _pc;
                    _sp = (byte)((_sp + 1) & _stackMask);
                    CallObserver?.Invoke((_pc - 1) & _programMask, target & ~0x2000);
                    _pc = (ushort)(target & ~0x2000);
                    break;

                case 0x141:
                    _stack[_sp] = _pc;
                    _sp = (byte)((_sp + 1) & _stackMask);
                    CallObserver?.Invoke((_pc - 1) & _programMask, target | 0x2000);
                    _pc = (ushort)(target | 0x2000);
                    break;
            }

            if (!taken) return;

            // A one-instruction branch back onto itself testing RQM is the
            // firmware idling on the host - see Venus_NecDSP.md §4.3.
            if (_pc - 1 == target && (jumpType == 0x0BC || jumpType == 0x0BE)) _inRqmLoop = true;
            _pc = target;
        }

        // --- ALU ---

        private void RunAluOp(byte aluOperation, ushort source)
        {
            int sel = (int)((_opcode >> 15) & 0x01);
            ushort acc = _acc[sel];
            bool otherCarry = _carry[sel ^ 1];

            // The second operand: RAM, the instruction's own source, or either
            // half of the multiplier's output.
            ushort p = ((_opcode >> 20) & 0x03) switch
            {
                0 => ReadRam(_dp),
                1 => source,
                2 => _m,
                _ => _n,
            };

            ushort result = 0;
            switch (aluOperation)
            {
                case 0x00: break;
                case 0x01: result = (ushort)(acc | p); break;
                case 0x02: result = (ushort)(acc & p); break;
                case 0x03: result = (ushort)(acc ^ p); break;
                case 0x04: result = (ushort)(acc - p); break;
                case 0x05: result = (ushort)(acc + p); break;
                case 0x06: result = (ushort)(acc - p - (otherCarry ? 1 : 0)); break;
                case 0x07: result = (ushort)(acc + p + (otherCarry ? 1 : 0)); break;

                // DEC/INC borrow the add/subtract overflow maths with p forced to 1.
                case 0x08: result = (ushort)(acc - 1); p = 1; break;
                case 0x09: result = (ushort)(acc + 1); p = 1; break;

                case 0x0A: result = (ushort)~acc; break;
                case 0x0B: result = (ushort)((acc >> 1) | (acc & 0x8000)); break;
                case 0x0C: result = (ushort)((acc << 1) | (otherCarry ? 1 : 0)); break;
                case 0x0D: result = (ushort)((acc << 2) | 0x03); break;
                case 0x0E: result = (ushort)((acc << 4) | 0x0F); break;
                case 0x0F: result = (ushort)((acc << 8) | (acc >> 8)); break;
            }

            bool zero = result == 0;
            bool sign0 = (result & 0x8000) != 0;
            bool sign1 = _sign1[sel];
            bool overflow0 = _overflow0[sel];
            bool overflow1 = _overflow1[sel];
            bool carry = _carry[sel];

            // S1 is a latched sign that only tracks S0 while OV1 is clear.
            if (!overflow1) sign1 = sign0;

            switch (aluOperation)
            {
                case 0x00:
                case 0x01:
                case 0x02:
                case 0x03:
                case 0x0A:
                case 0x0D:
                case 0x0E:
                case 0x0F:
                    carry = false;
                    overflow0 = false;
                    overflow1 = false;
                    break;

                case 0x04:
                case 0x05:
                case 0x06:
                case 0x07:
                case 0x08:
                case 0x09:
                {
                    // Odd operations are adds, even ones subtracts, which is
                    // what picks the second term of the overflow expression.
                    ushort overflow = (ushort)((acc ^ result) & (p ^ ((aluOperation & 0x01) != 0 ? result : acc)));
                    overflow0 = (overflow & 0x8000) != 0;
                    if (overflow0 && overflow1) overflow1 = sign0 == sign1;
                    else overflow1 |= overflow0;
                    carry = ((acc ^ p ^ result ^ overflow) & 0x8000) != 0;
                    break;
                }

                case 0x0B:
                    carry = (acc & 0x01) != 0;
                    overflow0 = false;
                    overflow1 = false;
                    break;

                case 0x0C:
                    carry = (acc & 0x8000) != 0;
                    overflow0 = false;
                    overflow1 = false;
                    break;
            }

            _acc[sel] = result;
            _zero[sel] = zero;
            _sign0[sel] = sign0;
            _sign1[sel] = sign1;
            _overflow0[sel] = overflow0;
            _overflow1[sel] = overflow1;
            _carry[sel] = carry;
        }

        // DP's low nibble steps without carrying into the high nibble, and the
        // high nibble is XOR-modified rather than assigned.
        private void UpdateDataPointer()
        {
            ushort dp = _dp;
            switch ((_opcode >> 13) & 0x03)
            {
                case 0: break;
                case 1: dp = (ushort)((dp & 0xF0) | ((dp + 1) & 0x0F)); break;
                case 2: dp = (ushort)((dp & 0xF0) | ((dp - 1) & 0x0F)); break;
                default: dp &= 0xF0; break;
            }

            _dp = (ushort)(dp ^ (((_opcode >> 9) & 0x0F) << 4));
        }

        // --- Operand transfer ---

        private void Load(byte dest, ushort value)
        {
            switch (dest)
            {
                case 0x00: break;
                case 0x01: _acc[0] = value; break;
                case 0x02: _acc[1] = value; break;
                case 0x03: _tr = value; break;
                case 0x04: _dp = value; break;
                case 0x05: _rp = value; break;

                case 0x06:
                    _dr = value;
                    _sr |= RequestForMaster;
                    break;

                // The high bits of SR belong to the chip, not the firmware.
                case 0x07: _sr = (ushort)((_sr & 0x907C) | (value & ~0x907C)); break;

                case 0x08:
                case 0x09: _serialOut = value; break;

                case 0x0A: _k = value; break;

                // The paired loads fetch the other multiplier input at the same time.
                case 0x0B:
                    _k = value;
                    _l = ReadDataRom(_rp);
                    break;

                case 0x0C:
                    _l = value;
                    _k = ReadRam((ushort)(_dp | 0x40));
                    break;

                case 0x0D: _l = value; break;
                case 0x0E: _trb = value; break;
                default: WriteRam(_dp, value); break;
            }
        }

        private ushort GetSourceValue(byte source)
        {
            switch (source)
            {
                case 0x00: return _trb;
                case 0x01: return _acc[0];
                case 0x02: return _acc[1];
                case 0x03: return _tr;
                case 0x04: return _dp;
                case 0x05: return _rp;
                case 0x06: return ReadDataRom(_rp);

                // Saturating constant used to clamp a signed result.
                case 0x07: return (ushort)(0x8000 - (_sign1[0] ? 1 : 0));

                case 0x08:
                    _sr |= RequestForMaster;
                    return _dr;

                case 0x09: return _dr;
                case 0x0A: return _sr;
                case 0x0B:
                case 0x0C: return _serialIn;
                case 0x0D: return _k;
                case 0x0E: return _l;
                case 0x0F: return ReadRam(_dp);
            }

            throw new InvalidOperationException($"NecDsp: invalid source operand {source}.");
        }

        // --- Internal memory ---

        private ushort ReadDataRom(ushort addr) => _dataRom[addr & _dataRomMask];

        private ushort ReadRam(ushort addr) => Ram[addr & _ramMask];

        private void WriteRam(ushort addr, ushort value) => Ram[addr & _ramMask] = value;
    }
}
