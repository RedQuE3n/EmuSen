namespace EmuSen.Cores
{
    // What a frontend decides about a core when it makes it, honoured by every core from its load on - see EmuSen_Multicore.md §6.
    public sealed record CoreOptions
    {
        // --nobattery: the cartridge save is neither read nor written.
        public bool BatteryRamDisabled { get; init; }

        // The most undrained samples a core keeps before the oldest pair goes - see EmuSen_Settings_Reference.md §4.85.2.
        public int AudioBufferMaxSamples { get; init; } = Common.SampleQueue.DefaultLimit;

        public static CoreOptions Default { get; } = new();

        // What audio.json asks of the cores, as the frontends read it at start.
        public static CoreOptions From(Galaxia.Models.AudioConfig audio) => new() { AudioBufferMaxSamples = audio.Bounded().AudioBufferMaxSamples };
    }
}
