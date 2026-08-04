namespace EmuSen.Cores.Nintendo.Mercury.Cpu.Core
{
    // The arithmetic, logic and shift primitives every opcode file below builds on - see Mercury_Cpu.md §6.
    public sealed partial class Cpu
    {
        private void Add8(byte value, bool withCarry = false)
        {
            int carry = withCarry && Flag(FlagC) ? 1 : 0;
            int result = A + value + carry;

            SetFlag(FlagH, (A & 0x0F) + (value & 0x0F) + carry > 0x0F);
            SetFlag(FlagC, result > 0xFF);
            A = (byte)result;
            SetFlag(FlagZ, A == 0);
            SetFlag(FlagN, false);
        }

        private void Sub8(byte value, bool withCarry = false)
        {
            int carry = withCarry && Flag(FlagC) ? 1 : 0;
            int result = A - value - carry;

            SetFlag(FlagH, (A & 0x0F) - (value & 0x0F) - carry < 0);
            SetFlag(FlagC, result < 0);
            A = (byte)result;
            SetFlag(FlagZ, A == 0);
            SetFlag(FlagN, true);
        }

        private void And8(byte value)
        {
            A &= value;
            F = 0;
            SetFlag(FlagH, true);
            SetFlag(FlagZ, A == 0);
        }

        private void Or8(byte value)
        {
            A |= value;
            F = 0;
            SetFlag(FlagZ, A == 0);
        }

        private void Xor8(byte value)
        {
            A ^= value;
            F = 0;
            SetFlag(FlagZ, A == 0);
        }

        // SUB that keeps the flags and throws the difference away.
        private void Cp8(byte value)
        {
            byte saved = A;
            Sub8(value);
            A = saved;
        }

        // Carry is deliberately untouched, which is what makes INC usable inside a multi-byte add.
        private byte Inc8(byte value)
        {
            byte result = (byte)(value + 1);
            SetFlag(FlagH, (value & 0x0F) == 0x0F);
            SetFlag(FlagZ, result == 0);
            SetFlag(FlagN, false);
            return result;
        }

        private byte Dec8(byte value)
        {
            byte result = (byte)(value - 1);
            SetFlag(FlagH, (value & 0x0F) == 0);
            SetFlag(FlagZ, result == 0);
            SetFlag(FlagN, true);
            return result;
        }

        // Zero is preserved here, unlike every 8-bit add.
        private void AddHl(ushort value)
        {
            int result = HL + value;
            SetFlag(FlagH, (HL & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
            SetFlag(FlagC, result > 0xFFFF);
            SetFlag(FlagN, false);
            HL = (ushort)result;
        }

        // Both half- and full-carry come from the low byte, even though the operand is signed.
        private ushort AddSpOffset()
        {
            sbyte offset = (sbyte)Fetch();
            int result = SP + offset;

            F = 0;
            SetFlag(FlagH, (SP & 0x0F) + (offset & 0x0F) > 0x0F);
            SetFlag(FlagC, (SP & 0xFF) + (offset & 0xFF) > 0xFF);
            return (ushort)result;
        }

        // Fixes up a BCD result after the fact, which is why the N flag exists at all.
        private void Daa()
        {
            int a = A;

            if (!Flag(FlagN))
            {
                if (Flag(FlagC) || a > 0x99) { a += 0x60; SetFlag(FlagC, true); }
                if (Flag(FlagH) || (a & 0x0F) > 0x09) a += 0x06;
            }
            else
            {
                if (Flag(FlagC)) a -= 0x60;
                if (Flag(FlagH)) a -= 0x06;
            }

            A = (byte)a;
            SetFlag(FlagZ, A == 0);
            SetFlag(FlagH, false);
        }

        private byte Rlc(byte value, bool setZero = true)
        {
            byte result = (byte)((value << 1) | (value >> 7));
            F = 0;
            SetFlag(FlagC, (value & 0x80) != 0);
            SetFlag(FlagZ, setZero && result == 0);
            return result;
        }

        private byte Rrc(byte value, bool setZero = true)
        {
            byte result = (byte)((value >> 1) | (value << 7));
            F = 0;
            SetFlag(FlagC, (value & 0x01) != 0);
            SetFlag(FlagZ, setZero && result == 0);
            return result;
        }

        private byte Rl(byte value, bool setZero = true)
        {
            byte result = (byte)((value << 1) | (Flag(FlagC) ? 1 : 0));
            F = 0;
            SetFlag(FlagC, (value & 0x80) != 0);
            SetFlag(FlagZ, setZero && result == 0);
            return result;
        }

        private byte Rr(byte value, bool setZero = true)
        {
            byte result = (byte)((value >> 1) | (Flag(FlagC) ? 0x80 : 0));
            F = 0;
            SetFlag(FlagC, (value & 0x01) != 0);
            SetFlag(FlagZ, setZero && result == 0);
            return result;
        }

        private byte Sla(byte value)
        {
            byte result = (byte)(value << 1);
            F = 0;
            SetFlag(FlagC, (value & 0x80) != 0);
            SetFlag(FlagZ, result == 0);
            return result;
        }

        // Arithmetic shift: bit 7 is copied back into itself, so a negative stays negative.
        private byte Sra(byte value)
        {
            byte result = (byte)((value >> 1) | (value & 0x80));
            F = 0;
            SetFlag(FlagC, (value & 0x01) != 0);
            SetFlag(FlagZ, result == 0);
            return result;
        }

        private byte Srl(byte value)
        {
            byte result = (byte)(value >> 1);
            F = 0;
            SetFlag(FlagC, (value & 0x01) != 0);
            SetFlag(FlagZ, result == 0);
            return result;
        }

        private byte Swap(byte value)
        {
            byte result = (byte)((value >> 4) | (value << 4));
            F = 0;
            SetFlag(FlagZ, result == 0);
            return result;
        }

        // Carry is the one flag BIT leaves alone.
        private void Bit(byte value, int index)
        {
            SetFlag(FlagZ, (value & (1 << index)) == 0);
            SetFlag(FlagN, false);
            SetFlag(FlagH, true);
        }
    }
}
