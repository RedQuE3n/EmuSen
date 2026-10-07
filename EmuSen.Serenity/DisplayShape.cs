namespace EmuSen.Serenity
{
    // The shapes of the screens consoles were made for, width over height, which the cores report and the CRT filter's tube is drawn in - see EmuSen_Serenity.md §2.9.
    public static class DisplayShape
    {
        public const double Television = 4.0 / 3.0;

        // The Game Boy's LCD: 160 by 144 square pixels.
        public const double GameBoy = 10.0 / 9.0;
    }
}
