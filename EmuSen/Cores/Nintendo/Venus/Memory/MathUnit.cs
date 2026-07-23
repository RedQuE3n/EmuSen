using System;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    // The SNES hardware multiply/divide unit ($4202-$4206 write, $4214-
    // $4217 read) - a genuinely distinct piece of hardware, extracted out
    // of MemoryBus so the bus doesn't have to carry register-specific
    // state and logic that has nothing to do with its actual job (address
    // decode/dispatch). Computed instantly on the triggering write
    // (WRMPYB/WRDIVB) rather than modeling the real 8/16-cycle delay -
    // correct for any game that waits before reading the result, which is
    // how this hardware is always used in practice.
    public class MathUnit
    {
        private byte _mpyA = 0xFF;             // WRMPYA ($4202), power-on default per docs
        private ushort _divDividend = 0xFFFF;  // WRDIV ($4204/4205), power-on default per docs
        private ushort _divQuotient;           // read via $4214/4215
        private ushort _rdMpy;                 // read via $4216/4217 - holds the last multiply's
                                                // product OR the last divide's remainder,
                                                // whichever operation ran most recently, since
                                                // they're the same physical registers on real hardware

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
                // Documented hardware behavior: divide by zero gives a
                // quotient of $FFFF and a remainder equal to the dividend.
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
