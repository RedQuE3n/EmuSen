namespace EmuSen.Cores.Nintendo.Venus
{
    // Which console the cartridge expects - see Venus_CPU.md §8.5c.
    public enum ConsoleRegion
    {
        Ntsc,
        Pal
    }

    public static class ConsoleRegions
    {
        // Header country byte ($FFD9/$7FD9) - see Venus_Memory.md §2.5.
        public static ConsoleRegion FromCountryCode(byte country)
        {
            if (country >= 0x02 && country <= 0x0C) return ConsoleRegion.Pal;
            if (country == 0x11) return ConsoleRegion.Pal;
            return ConsoleRegion.Ntsc;
        }
    }
}
