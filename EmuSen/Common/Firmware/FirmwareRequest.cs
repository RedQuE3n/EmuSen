using System;
using System.Collections.Generic;

namespace EmuSen.Common.Firmware
{
    // One firmware image a core needs before it can emulate some chip - see EmuSen_Firmware.md §1.
    public sealed record FirmwareRequest(
        string CoreName,
        string ChipName,
        string FileName,
        int Size,
        string Purpose)
    {
        // Other filenames the same dump is distributed under.
        public IReadOnlyList<string> AlternateNames { get; init; } = Array.Empty<string>();

        // Split forms of the same dump, each a list of files that together make it, passed as files from the core's number upwards - see EmuSen_CoreAPI.md §6.2.
        public IReadOnlyList<IReadOnlyList<string>> Parts { get; init; } = Array.Empty<IReadOnlyList<string>>();

        // False when the game runs without the file, on the core's replacement or without the chip; only a required request is prompted for - see VenusRT_DspHle.md §7.3.
        public bool Required { get; init; } = true;

        // The core's replacement for the file: its effect (exact, accuracy or none) and cost, or null for none offered - see EmuSen_CoreAPI.md §6.2.
        public string? ReplacementEffect { get; init; }
        public string? ReplacementCost { get; init; }

        // What the picker shows the player - see EmuSen_Firmware.md §3.
        public string Describe() => $"{CoreName} {ChipName} firmware - {FileName}, {Size:N0} bytes";
    }
}
