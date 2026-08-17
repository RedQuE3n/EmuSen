namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // Which NEC DSP a cartridge carries - see Venus_NecDSP.md §1.
    public enum NecDspVariant
    {
        Dsp1,
        Dsp1B,
        Dsp2,
        Dsp3,
        Dsp4,
        St010,
        St011,
    }

    // The per-variant constants that shape a chip: clock, memory sizes, and the address bit that picks SR - see Venus_NecDSP.md §1.
    public readonly struct NecDspProfile
    {
        public readonly int ClockHz;
        public readonly int ProgramBytes;
        public readonly int DataRomBytes;
        public readonly int RamWords;
        public readonly int StackDepth;
        public readonly string FirmwareName;

        private NecDspProfile(int clockHz, int programBytes, int dataRomBytes, int ramWords, int stackDepth, string firmwareName)
        {
            ClockHz = clockHz;
            ProgramBytes = programBytes;
            DataRomBytes = dataRomBytes;
            RamWords = ramWords;
            StackDepth = stackDepth;
            FirmwareName = firmwareName;
        }

        public int FirmwareBytes => ProgramBytes + DataRomBytes;

        // uPD7725: 2048 24-bit instructions, 1024 data words, 256 words of RAM.
        private static NecDspProfile Upd7725(string name) => new(7600000, 0x1800, 0x800, 0x100, 4, name);

        // uPD96050: 16384 instructions, 2048 data words, 2KB of battery-backed RAM.
        private static NecDspProfile Upd96050(int clockHz, string name) => new(clockHz, 0xC000, 0x1000, 0x800, 8, name);

        public static NecDspProfile For(NecDspVariant variant) => variant switch
        {
            NecDspVariant.Dsp1 => Upd7725("dsp1"),
            NecDspVariant.Dsp1B => Upd7725("dsp1b"),
            NecDspVariant.Dsp2 => Upd7725("dsp2"),
            NecDspVariant.Dsp3 => Upd7725("dsp3"),
            NecDspVariant.Dsp4 => Upd7725("dsp4"),
            NecDspVariant.St010 => Upd96050(11000000, "st010"),
            _ => Upd96050(22000000, "st011"),
        };

        public static bool IsSt01x(NecDspVariant variant) => variant is NecDspVariant.St010 or NecDspVariant.St011;
    }
}
