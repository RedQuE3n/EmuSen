using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Models;

namespace EmuSen.Cores
{
    // One firmware file a system's engine can use: its file, whether a folder holds it at its size, and what the engine's open version costs; a frontend words it - see EmuSen_Settings_Reference.md §4.89.
    public sealed record FirmwareItem(string Label, string FileName, int Size, bool Present, bool WrongSize, string? Effect, string? Cost)
    {
        public const string YourFolder = "your folder", EmuSensFolder = "EmuSen's folder";

        // Which folder the file in use was found in, as YourFolder or EmuSensFolder, or null when none is - see EmuSen_Firmware.md §2.3.
        public string? FoundIn { get; init; }

        // The player's own file name when the file in use was recognised by its content, else null - see EmuSen_Firmware.md §7.
        public string? FoundAs { get; init; }

        // Which folder holds a file of the name and the wrong size, when no folder holds the right one.
        public string? WrongSizeIn { get; init; }
    }

    // A system the build runs and the engine that runs it, with the firmware that engine declares; none for most.
    public sealed record FirmwareSystem(string Name, string Engine, IReadOnlyList<FirmwareItem> Items);

    // What every shipped system's firmware is doing, read from the cores' own declarations and the firmware folder, never a table of its own - see EmuSen_Settings_Reference.md §4.89.
    public static class FirmwareOverview
    {

        // home/Firmware, which EmuSen makes and a file the player installs goes in.
        public static string Folder => FirmwareLibrary.Directory;

        // Every folder searched, the player's chosen one first when set - see EmuSen_Firmware.md §2.3.
        public static IReadOnlyList<string> Folders => FirmwareLibrary.Folders;

        // The catalog's consoles on the engine each will run, then any system only a discovered core runs; a core in development is left out.
        public static IReadOnlyList<FirmwareSystem> Build(ConsolesConfig? config = null)
        {
            config ??= ConsolesConfig.Load();
            IReadOnlyList<string> folders = FirmwareLibrary.Folders;
            var systems = new List<FirmwareSystem>();
            foreach (var console in CoreCatalog.ConsolesInReleaseOrder)
            {
                if (CoreCatalog.IsDiscovered(console)) continue;
                string name = string.Join(" and ", CoreCatalog.ShelvesInReleaseOrder.Where(s => ReferenceEquals(s.Core, console)).Select(s => s.EsdeFullName));
                string? chosen = CoreCatalog.EngineChosen(console.Console, config.Value(console.Console, CoreCatalog.EngineKey)) ?? CoreCatalog.SoleDiscoveredEngine(console.Console);
                if (Shipped(CoreDiscovery.ByEngineName(chosen)) is { } v1)
                    systems.Add(new FirmwareSystem(name, v1.EngineName, v1.Info.Systems.Where(s => CoreCatalog.SystemIdsFor(console.Console).Contains(s.Id)).SelectMany(s => s.Firmware)
                        .DistinctBy(f => f.Name).Select(f => Item(CoreEngine.Request(v1.Info.Name, f), f.Label, folders)).ToArray()));
                else
                    systems.Add(new FirmwareSystem(name, CoreCatalog.ReferenceEngine(console.Console) ?? console.DisplayName, Array.Empty<FirmwareItem>()));
            }
            foreach (DiscoveredCore core in CoreDiscovery.Found)
            {
                if (Shipped(core) is null) continue;
                foreach (CoreSystem system in core.Info.Systems.Where(s => !s.Development && CoreCatalog.ConsoleForSystem(s.Id) is null && systems.All(known => known.Name != s.Name)))
                    systems.Add(new FirmwareSystem(system.Name, core.EngineName, system.Firmware.Select(f => Item(CoreEngine.Request(core.Info.Name, f), f.Label, folders)).ToArray()));
            }
            return systems;
        }

        // A discovered engine a player is offered and whose library is there to run; one still in development never is, whatever discovery lists.
        private static DiscoveredCore? Shipped(DiscoveredCore? core) => core is { Sidecar.Development: false } && File.Exists(core.Sidecar.LibraryPath) ? core : null;

        private static FirmwareItem Item(FirmwareRequest request, string label, IReadOnlyList<string> folders)
        {
            var (found, wrong) = FirmwareLibrary.Locate(request, folders);
            return new FirmwareItem(label, request.FileName, request.Size, found is not null, wrong is not null, request.ReplacementEffect, request.ReplacementCost)
            {
                FoundIn = Role(found?.Folder), WrongSizeIn = Role(wrong), FoundAs = found is { ByContent: true } ? found.Names[0] : null,
            };
        }

        // home/Firmware is EmuSen's folder; any other folder searched is the player's chosen one.
        private static string? Role(string? folder) => folder is null ? null : folder == FirmwareLibrary.Directory ? FirmwareItem.EmuSensFolder : FirmwareItem.YourFolder;
    }
}
