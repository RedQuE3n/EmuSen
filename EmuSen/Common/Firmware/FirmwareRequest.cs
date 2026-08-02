using System;
using System.Collections.Generic;

namespace EmuSen.Common.Firmware
{
    // One firmware image a core needs before it can emulate some chip. Kept
    // core-agnostic on purpose: a PlayStation BIOS and a DSP-1 dump differ
    // only in the values below, so the discovery, validation and prompt
    // machinery is written once. See EmuSen_Firmware.md §1.
    public sealed record FirmwareRequest(
        string CoreName,
        string ChipName,
        string FileName,
        int Size,
        string Purpose)
    {
        // Other filenames the same dump is distributed under. Checked after
        // FileName, and never written to - an install always lands under the
        // canonical name so the next launch finds it first.
        public IReadOnlyList<string> AlternateNames { get; init; } = Array.Empty<string>();

        // What the picker shows the user - see EmuSen_Firmware.md §3.
        public string Describe() => $"{CoreName} {ChipName} firmware - {FileName}, {Size:N0} bytes";
    }
}
