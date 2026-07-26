using System;
using EmuSen.Debug;
using EmuSen.DianaOS;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // The SNES hardware multiply/divide unit - see Venus_Memory.md §5.
    public class MathUnit
    {
        private byte _mpyA = 0xFF;             // WRMPYA ($4202), power-on default per docs
        private ushort _divDividend = 0xFFFF;  // WRDIV ($4204/4205), power-on default per docs
        private ushort _divQuotient;           // read via $4214/4215
        private ushort _rdMpy;                 // read via $4216/4217 - shared with multiply's product, see §5

        public void WriteMpyA(byte data) => _mpyA = data;

        public void WriteMpyBTrigger(byte data)
        {
            _rdMpy = (ushort)(_mpyA * data);
            if (DebugSettings.MathUnitLogging) Console.WriteLine($"[MATH] MPY {_mpyA} * {data} = {_rdMpy}");
        }

        public void WriteDivL(byte data) => _divDividend = (ushort)((_divDividend & 0xFF00) | data);
        public void WriteDivH(byte data) => _divDividend = (ushort)((_divDividend & 0x00FF) | (data << 8));

        public void WriteDivBTrigger(byte data)
        {
            if (data == 0)
            {
                // Divide by zero - see Venus_Memory.md §5.
                _divQuotient = 0xFFFF;
                _rdMpy = _divDividend;
            }
            else
            {
                _divQuotient = (ushort)(_divDividend / data);
                _rdMpy = (ushort)(_divDividend % data);
            }
            if (DebugSettings.MathUnitLogging) Console.WriteLine($"[MATH] DIV {_divDividend} / {data} = {_divQuotient} rem {_rdMpy}");
        }

        public byte ReadQuotientLow() => (byte)(_divQuotient & 0xFF);
        public byte ReadQuotientHigh() => (byte)((_divQuotient >> 8) & 0xFF);
        public byte ReadProductOrRemainderLow() => (byte)(_rdMpy & 0xFF);
        public byte ReadProductOrRemainderHigh() => (byte)((_rdMpy >> 8) & 0xFF);
    }
}
