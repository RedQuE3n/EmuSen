namespace EmuSen.Cores
{
    // Run switches a frontend sets before LoadRom, honoured by every core - see EmuSen_Multicore.md §6.
    public static class CoreOptions
    {
        // --nobattery: the cartridge save is neither read nor written.
        public static bool BatteryRamDisabled;
    }
}
