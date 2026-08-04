namespace EmuSen.Cores.Nintendo.Mercury.Cpu.Core
{
    // The $CB-prefixed table: rotates, shifts and the bit instructions - see Mercury_Cpu.md §6.2.
    public sealed partial class Cpu
    {
        // Cycles returned include the prefix byte, which is why a plain register op is 8 and not 4.
        private int ExecuteCb(byte opcode)
        {
            int operand = opcode & 0x07;
            int index = (opcode >> 3) & 0x07;
            bool memory = operand == 6;

            // BIT never writes back, so it costs one fewer bus cycle than the rest of the $40+ block.
            if (opcode is >= 0x40 and <= 0x7F)
            {
                Bit(GetOperand(operand), index);
                return memory ? 12 : 8;
            }

            byte value = GetOperand(operand);

            byte result = opcode switch
            {
                <= 0x07 => Rlc(value),
                <= 0x0F => Rrc(value),
                <= 0x17 => Rl(value),
                <= 0x1F => Rr(value),
                <= 0x27 => Sla(value),
                <= 0x2F => Sra(value),
                <= 0x37 => Swap(value),
                <= 0x3F => Srl(value),
                <= 0xBF => (byte)(value & ~(1 << index)),
                _ => (byte)(value | (1 << index)),
            };

            SetOperand(operand, result);
            return memory ? 16 : 8;
        }
    }
}
