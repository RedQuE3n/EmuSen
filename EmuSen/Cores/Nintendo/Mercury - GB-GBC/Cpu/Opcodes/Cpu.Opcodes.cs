using System;

namespace EmuSen.Cores.Nintendo.Mercury.Cpu.Core
{
    // The unprefixed table - see Mercury_Cpu.md §6.
    public sealed partial class Cpu
    {
        // Operand order for the register-shaped blocks; index 6 is the byte at HL, not a register.
        private byte GetOperand(int index) => index switch
        {
            0 => B,
            1 => C,
            2 => D,
            3 => E,
            4 => H,
            5 => L,
            6 => _bus.Read(HL),
            _ => A,
        };

        private void SetOperand(int index, byte value)
        {
            switch (index)
            {
                case 0: B = value; break;
                case 1: C = value; break;
                case 2: D = value; break;
                case 3: E = value; break;
                case 4: H = value; break;
                case 5: L = value; break;
                case 6: _bus.Write(HL, value); break;
                default: A = value; break;
            }
        }

        private int Execute(byte opcode)
        {
            // $40-$BF are two dense blocks: register-to-register moves, then A-with-operand arithmetic.
            if (opcode is >= 0x40 and <= 0x7F && opcode != 0x76) return ExecuteLoadBlock(opcode);
            if (opcode is >= 0x80 and <= 0xBF) return ExecuteAluBlock(opcode);

            switch (opcode)
            {
                case 0x00: return 4;

                // Two bytes on real hardware even though the second is ignored.
                case 0x10: Fetch(); return 4;

                case 0x76: return Halt();

                case 0x01: BC = Fetch16(); return 12;
                case 0x11: DE = Fetch16(); return 12;
                case 0x21: HL = Fetch16(); return 12;
                case 0x31: SP = Fetch16(); return 12;

                case 0x02: _bus.Write(BC, A); return 8;
                case 0x12: _bus.Write(DE, A); return 8;
                case 0x22: _bus.Write(HL, A); HL++; return 8;
                case 0x32: _bus.Write(HL, A); HL--; return 8;

                case 0x0A: A = _bus.Read(BC); return 8;
                case 0x1A: A = _bus.Read(DE); return 8;
                case 0x2A: A = _bus.Read(HL); HL++; return 8;
                case 0x3A: A = _bus.Read(HL); HL--; return 8;

                case 0x03: BC++; return 8;
                case 0x13: DE++; return 8;
                case 0x23: HL++; return 8;
                case 0x33: SP++; return 8;

                case 0x0B: BC--; return 8;
                case 0x1B: DE--; return 8;
                case 0x2B: HL--; return 8;
                case 0x3B: SP--; return 8;

                case 0x04: B = Inc8(B); return 4;
                case 0x0C: C = Inc8(C); return 4;
                case 0x14: D = Inc8(D); return 4;
                case 0x1C: E = Inc8(E); return 4;
                case 0x24: H = Inc8(H); return 4;
                case 0x2C: L = Inc8(L); return 4;
                case 0x34: _bus.Write(HL, Inc8(_bus.Read(HL))); return 12;
                case 0x3C: A = Inc8(A); return 4;

                case 0x05: B = Dec8(B); return 4;
                case 0x0D: C = Dec8(C); return 4;
                case 0x15: D = Dec8(D); return 4;
                case 0x1D: E = Dec8(E); return 4;
                case 0x25: H = Dec8(H); return 4;
                case 0x2D: L = Dec8(L); return 4;
                case 0x35: _bus.Write(HL, Dec8(_bus.Read(HL))); return 12;
                case 0x3D: A = Dec8(A); return 4;

                case 0x06: B = Fetch(); return 8;
                case 0x0E: C = Fetch(); return 8;
                case 0x16: D = Fetch(); return 8;
                case 0x1E: E = Fetch(); return 8;
                case 0x26: H = Fetch(); return 8;
                case 0x2E: L = Fetch(); return 8;
                case 0x36: _bus.Write(HL, Fetch()); return 12;
                case 0x3E: A = Fetch(); return 8;

                case 0x09: AddHl(BC); return 8;
                case 0x19: AddHl(DE); return 8;
                case 0x29: AddHl(HL); return 8;
                case 0x39: AddHl(SP); return 8;

                // The accumulator rotates always clear Z, unlike their CB-prefixed twins.
                case 0x07: A = Rlc(A, setZero: false); return 4;
                case 0x0F: A = Rrc(A, setZero: false); return 4;
                case 0x17: A = Rl(A, setZero: false); return 4;
                case 0x1F: A = Rr(A, setZero: false); return 4;

                case 0x27: Daa(); return 4;
                case 0x2F: A = (byte)~A; SetFlag(FlagN, true); SetFlag(FlagH, true); return 4;
                case 0x37: SetFlag(FlagC, true); SetFlag(FlagN, false); SetFlag(FlagH, false); return 4;
                case 0x3F: SetFlag(FlagC, !Flag(FlagC)); SetFlag(FlagN, false); SetFlag(FlagH, false); return 4;

                case 0x08:
                {
                    ushort target = Fetch16();
                    _bus.Write(target, (byte)SP);
                    _bus.Write((ushort)(target + 1), (byte)(SP >> 8));
                    return 20;
                }

                case 0x18: return JumpRelative(true);
                case 0x20: return JumpRelative(!Flag(FlagZ));
                case 0x28: return JumpRelative(Flag(FlagZ));
                case 0x30: return JumpRelative(!Flag(FlagC));
                case 0x38: return JumpRelative(Flag(FlagC));

                case 0xC3: return Jump(true);
                case 0xC2: return Jump(!Flag(FlagZ));
                case 0xCA: return Jump(Flag(FlagZ));
                case 0xD2: return Jump(!Flag(FlagC));
                case 0xDA: return Jump(Flag(FlagC));
                case 0xE9: PC = HL; return 4;

                case 0xCD: return Call(true);
                case 0xC4: return Call(!Flag(FlagZ));
                case 0xCC: return Call(Flag(FlagZ));
                case 0xD4: return Call(!Flag(FlagC));
                case 0xDC: return Call(Flag(FlagC));

                case 0xC9: PC = Pop(); return 16;
                case 0xC0: return ReturnIf(!Flag(FlagZ));
                case 0xC8: return ReturnIf(Flag(FlagZ));
                case 0xD0: return ReturnIf(!Flag(FlagC));
                case 0xD8: return ReturnIf(Flag(FlagC));

                // The only instruction that re-enables interrupts with no one-instruction delay.
                case 0xD9: PC = Pop(); Ime = true; return 16;

                case 0xC7: return Restart(0x00);
                case 0xCF: return Restart(0x08);
                case 0xD7: return Restart(0x10);
                case 0xDF: return Restart(0x18);
                case 0xE7: return Restart(0x20);
                case 0xEF: return Restart(0x28);
                case 0xF7: return Restart(0x30);
                case 0xFF: return Restart(0x38);

                case 0xC1: BC = Pop(); return 12;
                case 0xD1: DE = Pop(); return 12;
                case 0xE1: HL = Pop(); return 12;
                case 0xF1: AF = Pop(); return 12;

                case 0xC5: Push(BC); return 16;
                case 0xD5: Push(DE); return 16;
                case 0xE5: Push(HL); return 16;
                case 0xF5: Push(AF); return 16;

                case 0xC6: Add8(Fetch()); return 8;
                case 0xCE: Add8(Fetch(), withCarry: true); return 8;
                case 0xD6: Sub8(Fetch()); return 8;
                case 0xDE: Sub8(Fetch(), withCarry: true); return 8;
                case 0xE6: And8(Fetch()); return 8;
                case 0xEE: Xor8(Fetch()); return 8;
                case 0xF6: Or8(Fetch()); return 8;
                case 0xFE: Cp8(Fetch()); return 8;

                // The $FF00 page is reachable by an 8-bit operand, which is why the I/O registers live there.
                case 0xE0: _bus.Write((ushort)(0xFF00 + Fetch()), A); return 12;
                case 0xF0: A = _bus.Read((ushort)(0xFF00 + Fetch())); return 12;
                case 0xE2: _bus.Write((ushort)(0xFF00 + C), A); return 8;
                case 0xF2: A = _bus.Read((ushort)(0xFF00 + C)); return 8;

                case 0xEA: _bus.Write(Fetch16(), A); return 16;
                case 0xFA: A = _bus.Read(Fetch16()); return 16;

                case 0xE8: SP = AddSpOffset(); return 16;
                case 0xF8: HL = AddSpOffset(); return 12;
                case 0xF9: SP = HL; return 8;

                case 0xF3: Ime = false; _imeScheduled = false; return 4;
                case 0xFB: _imeScheduled = true; return 4;

                case 0xCB: return ExecuteCb(Fetch());

                // Nothing is wired to these; real hardware locks the CPU up until reset.
                default:
                    throw new NotSupportedException(
                        $"Opcode ${opcode:X2} at ${LastInstructionPC:X4} is not a real SM83 instruction - see Mercury_Cpu.md §6.1.");
            }
        }

        private int ExecuteLoadBlock(byte opcode)
        {
            int destination = (opcode >> 3) & 0x07;
            int source = opcode & 0x07;

            SetOperand(destination, GetOperand(source));
            return destination == 6 || source == 6 ? 8 : 4;
        }

        private int ExecuteAluBlock(byte opcode)
        {
            int source = opcode & 0x07;
            byte value = GetOperand(source);

            switch ((opcode >> 3) & 0x07)
            {
                case 0: Add8(value); break;
                case 1: Add8(value, withCarry: true); break;
                case 2: Sub8(value); break;
                case 3: Sub8(value, withCarry: true); break;
                case 4: And8(value); break;
                case 5: Xor8(value); break;
                case 6: Or8(value); break;
                default: Cp8(value); break;
            }

            return source == 6 ? 8 : 4;
        }

        // With IME clear and something already pending, the CPU never sleeps and re-reads the next byte.
        private int Halt()
        {
            if (!Ime && _interruptPending) _haltBug = true;
            else Halted = true;

            return 4;
        }

        private int JumpRelative(bool taken)
        {
            sbyte offset = (sbyte)Fetch();
            if (!taken) return 8;

            PC = (ushort)(PC + offset);
            return 12;
        }

        // The operand is read either way; only the branch itself is conditional.
        private int Jump(bool taken)
        {
            ushort target = Fetch16();
            if (!taken) return 12;

            PC = target;
            return 16;
        }

        private int Call(bool taken)
        {
            ushort target = Fetch16();
            if (!taken) return 12;

            Push(PC);
            PC = target;
            return 24;
        }

        private int ReturnIf(bool taken)
        {
            if (!taken) return 8;

            PC = Pop();
            return 20;
        }

        private int Restart(ushort vector)
        {
            Push(PC);
            PC = vector;
            return 16;
        }
    }
}
