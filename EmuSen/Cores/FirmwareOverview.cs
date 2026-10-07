using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.Galaxia.Models;

namespace EmuSen.Cores
{
    // One firmware file a system's engine can use, in a player's words: what it is, which version runs, and what the open one changes - see EmuSen_Settings_Reference.md §4.89.
    public sealed record FirmwareItem(string Label, string FileName, int Size, bool Present, bool WrongSize, string? Effect, string? Cost)
    {
        public const string OpenVersion = "Using EmuSen's open version", OwnFile = "Using your own file", NoVersion = "No open version yet";
        public const string NeedsFile = "Games that need this chip need your own file.", Exact = "Exact: games run the same either way.", AsTheConsole = "Runs as on the console.";

        // Whether the engine carries an open replacement that runs the chip, exact or at a cost.
        public bool HasOpenVersion => Effect is "exact" or "accuracy";

        public string Title => $"{Label} ({FileName})";

        public string InUse => Present ? OwnFile : HasOpenVersion ? OpenVersion : NoVersion;

        // The same in a word or two, for a menu row's value.
        public string InUseShort => Present ? "Your own file" : HasOpenVersion ? "Open version" : "None yet";

        // What the open version changes: the first sentence of the cost the core states, or the fixed words where there is nothing to quote.
        public string Change => Present ? AsTheConsole : Effect == "exact" ? Exact : HasOpenVersion && FirstSentence(Cost) is { } said ? said : NeedsFile;

        // Said of a file that is in the folder and not used, since size is the library's one test - see EmuSen_Firmware.md §2.1.
        public string? Note => WrongSize ? $"The file here is not {Size:N0} bytes, so it is not used." : null;

        public static string? FirstSentence(string? text)
        {
            if (string.IsNullOrWhiteSpace(text)) return null;
            int end = text.IndexOf(". ", StringComparison.Ordinal);
            return end < 0 ? text.Trim() : text[..(end + 1)];
        }
    }

    // A system the build runs and the engine that runs it, with the firmware that engine declares; none for most.
    public sealed record FirmwareSystem(string Name, string Engine, IReadOnlyList<FirmwareItem> Items);

    // What every shipped system's firmware is doing, read from the cores' own declarations and the firmware folder, never a table of its own - see EmuSen_Settings_Reference.md §4.89.
    public static class FirmwareOverview
    {
        public const string NoFirmware = "Needs no firmware files.";
        public const string Optional = "Every file here is optional, and EmuSen never downloads firmware. A file of your own in the firmware folder is used in place of EmuSen's open version the next time a game starts.";

        public static string Folder => FirmwareLibrary.Directory;

        // The catalog's consoles on the engine each will run, then any system only a discovered core runs; a core in development is left out.
        public static IReadOnlyList<FirmwareSystem> Build(GraphicsConfig? config = null)
        {
            config ??= GraphicsConfig.Load();
            var systems = new List<FirmwareSystem>();
            foreach (var console in CoreCatalog.ConsolesInReleaseOrder)
            {
                if (CoreCatalog.IsDiscovered(console)) continue;
                string name = string.Join(" and ", CoreCatalog.ShelvesInReleaseOrder.Where(s => ReferenceEquals(s.Core, console)).Select(s => s.EsdeFullName));
                string? chosen = CoreCatalog.EngineChosen(console.Console, config.Value(console.Console, CoreCatalog.EngineKey));
                if (Shipped(CoreDiscovery.ByEngineName(chosen)) is { } v1)
                    systems.Add(new FirmwareSystem(name, v1.EngineName, v1.Info.Systems.Where(s => CoreCatalog.SystemIdsFor(console.Console).Contains(s.Id)).SelectMany(s => s.Firmware)
                        .DistinctBy(f => f.Name).Select(f => Item(CoreEngine.Request(v1.Info.Name, f), f.Label)).ToArray()));
                else
                    systems.Add(new FirmwareSystem(name, CoreCatalog.ReferenceEngine(console.Console) ?? console.DisplayName, ReferenceFirmware(console.Console).Select(r => Item(r, $"{r.CoreName} {r.ChipName} chip")).ToArray()));
            }
            foreach (DiscoveredCore core in CoreDiscovery.Found)
            {
                if (Shipped(core) is null) continue;
                foreach (CoreSystem system in core.Info.Systems.Where(s => !s.Development && CoreCatalog.ConsoleForSystem(s.Id) is null && systems.All(known => known.Name != s.Name)))
                    systems.Add(new FirmwareSystem(system.Name, core.EngineName, system.Firmware.Select(f => Item(CoreEngine.Request(core.Info.Name, f), f.Label)).ToArray()));
            }
            return systems;
        }

        // A discovered engine a player is offered and whose library is there to run; one still in development never is, whatever discovery lists.
        private static DiscoveredCore? Shipped(DiscoveredCore? core) => core is { Sidecar.Development: false } && File.Exists(core.Sidecar.LibraryPath) ? core : null;

        // What a C# reference core reads from the firmware folder, asked of the core's own request for each chip it knows.
        private static IEnumerable<FirmwareRequest> ReferenceFirmware(string console) =>
            string.Equals(console, "SNES", StringComparison.OrdinalIgnoreCase) ? Enum.GetValues<NecDspVariant>().Select(NecDspFirmware.RequestFor).DistinctBy(r => r.FileName) : [];

        private static FirmwareItem Item(FirmwareRequest request, string label)
        {
            bool present = FirmwareLibrary.IsInstalled(request);
            string whole = FirmwareLibrary.PathFor(request);
            return new FirmwareItem(label, request.FileName, request.Size, present, !present && File.Exists(whole) && new FileInfo(whole).Length != request.Size, request.ReplacementEffect, request.ReplacementCost);
        }
    }
}
