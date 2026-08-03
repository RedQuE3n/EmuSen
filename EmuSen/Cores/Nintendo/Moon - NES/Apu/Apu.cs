using System;

namespace EmuSen.Cores.Nintendo.Moon.Apu
{
    // Register surface and frame-counter IRQ only; no sound is synthesized - see Moon_APU.md.
    public sealed class Apu
    {
        public const int ChannelCount = 5;

        // $4000-$4013, kept verbatim so the debug target can report what a game wrote.
        public readonly byte[] Registers = new byte[0x14];

        public byte FrameCounter;
        public bool FrameIrqPending;

        // Length counters are the one piece of channel state $4015 actually reports.
        public readonly int[] LengthCounters = new int[ChannelCount];

        private int _frameStep;

        public bool IrqInhibited => (FrameCounter & 0x40) != 0;
        public bool FiveStepMode => (FrameCounter & 0x80) != 0;

        public void Reset()
        {
            Array.Clear(Registers);
            Array.Clear(LengthCounters);
            FrameCounter = 0;
            FrameIrqPending = false;
            _frameStep = 0;
        }

        public void WriteRegister(int address, byte value)
        {
            if (address >= 0x4000 && address <= 0x4013)
            {
                Registers[address - 0x4000] = value;
                return;
            }

            if (address == 0x4015)
            {
                for (int channel = 0; channel < ChannelCount; channel++)
                {
                    if ((value & (1 << channel)) == 0) LengthCounters[channel] = 0;
                }
                return;
            }

            if (address == 0x4017)
            {
                FrameCounter = value;
                _frameStep = 0;
                if (IrqInhibited) FrameIrqPending = false;
            }
        }

        // Reading the status register acknowledges the frame IRQ - see Moon_APU.md §2.
        public byte ReadStatus()
        {
            byte value = 0;

            for (int channel = 0; channel < ChannelCount; channel++)
            {
                if (LengthCounters[channel] > 0) value |= (byte)(1 << channel);
            }

            if (FrameIrqPending) value |= 0x40;
            FrameIrqPending = false;

            return value;
        }

        // Stepped four times per frame, which is the sequencer's real rate to within a scanline.
        public void StepFrameSequencer()
        {
            int steps = FiveStepMode ? 5 : 4;
            _frameStep = (_frameStep + 1) % steps;

            if (!FiveStepMode && _frameStep == 0 && !IrqInhibited) FrameIrqPending = true;
        }
    }
}
