using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Managed
{
    // The systems, the engines discovery found and a file's route, answered from the systems table and the sidecars without running a core - see EmuSen_Backend.md §3.5 and §14.
    internal sealed partial class ManagedRuntime
    {
        public static byte[] SystemsJson() => CrystalJson.Write(w =>
        {
            Galaxia.Models.ConsolesConfig stored = Galaxia.Models.ConsolesConfig.Load();
            var empty = new HashSet<string>(CoreCatalog.ConsolesThatStartEmpty.Select(e => e.SystemId), StringComparer.Ordinal);
            var listed = new HashSet<string>(StringComparer.Ordinal);
            w.WriteStartArray();
            foreach (SystemEntry e in SystemPacks.All.Select(p => p.Entry))
            {
                listed.Add(e.Id);
                w.WriteStartObject();
                w.WriteString("id", e.Id);
                w.WriteString("name", e.Name);
                w.WriteString("console", e.Console);
                w.WriteString("manufacturer", e.Manufacturer);
                w.WriteNumber("release_year", e.ReleaseYear);
                CrystalJson.Strings(w, "extensions", e.Extensions);
                CrystalJson.Strings(w, "cheat_systems", e.CheatSystems);
                CrystalJson.Strings(w, "openvgdb_systems", e.OpenVgdbSystems);
                w.WriteNumber("cover_aspect", e.CoverAspect);
                w.WriteString("esde_system", e.EsdeSystem);
                w.WriteString("esde_full_name", e.EsdeFullName);
                w.WriteBoolean("self_delimiting_images", e.SelfDelimitingImages);
                CrystalJson.StringOrNull(w, "attached_to", e.AttachedTo);
                if (e.ScreenScraperSystem is int screenScraper) w.WriteNumber("screenscraper_system", screenScraper);
                else w.WriteNull("screenscraper_system");
                CrystalJson.Strings(w, "engines", EnginesOf(e.Console, e.Id));
                w.WriteBoolean("starts_empty", empty.Contains(e.Id));
                CoreOf(w, e.Console, e.Extensions);
                Controllers(w, e.Console, stored);
                w.WriteEndObject();
            }
            foreach (var group in CoreDiscovery.Found.SelectMany(c => c.Info.Systems.Where(CoreDiscovery.Offered).Select(s => (Core: c, System: s))).Where(p => !listed.Contains(p.System.Id)).GroupBy(p => p.System.Id))
            {
                CoreSystem s = group.First().System;
                w.WriteStartObject();
                w.WriteString("id", s.Id);
                w.WriteString("name", s.Name);
                CrystalJson.Strings(w, "extensions", s.Extensions);
                string[] engines = group.Select(p => p.Core.EngineName).Distinct().ToArray();
                CrystalJson.Strings(w, "engines", engines);
                w.WriteBoolean("starts_empty", group.Any(p => p.System.StartsEmpty));
                w.WriteStartObject("core");
                CrystalJson.Strings(w, "names", new[] { s.Id }.Concat(engines.Where(n => !n.Any(char.IsWhiteSpace)).Select(n => n.ToLowerInvariant())).Distinct());
                w.WriteString("display_name", $"{s.Name} ({string.Join(", ", engines)})");
                w.WriteString("console", s.Id);
                CrystalJson.Strings(w, "extensions", s.Extensions);
                w.WriteEndObject();
                Controllers(w, CoreCatalog.DiscoveredConsoleForSystem(s.Id) ?? s.Id, stored);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        // A console's pad, its most ports and each player's controller as its stored settings stand: the engine's facts, from core info - see EmuSen_Backend.md §14.13.
        private static void Controllers(Utf8JsonWriter w, string console, Galaxia.Models.ConsolesConfig stored)
        {
            Func<string, string?> value = key => stored.Value(console, key);
            w.WriteStartObject("controllers");
            w.WriteStartArray("buttons");
            foreach (Galaxia.Input.PadButton b in CoreCatalog.ButtonsFor(console)) w.WriteNumberValue((int)b);
            w.WriteEndArray();
            w.WriteStartArray("axes");
            foreach (Galaxia.Input.PadAxis a in CoreCatalog.AxesFor(console)) w.WriteNumberValue((int)a);
            w.WriteEndArray();
            w.WriteNumber("ports", ControllerPorts.ForConsole(console));
            w.WriteStartArray("players");
            for (int p = 0; p < ControllerPorts.ForConsole(console, value); p++)
            {
                if (CoreCatalog.ControllerFor(console, p, value) is { } id) w.WriteStringValue(id);
                else w.WriteNullValue();
            }
            w.WriteEndArray();
            w.WriteEndObject();
        }

        // The library's shelves in release order, each a system of a console's core, named for the core - see EmuSen_Backend.md §14.13.
        public static byte[] ShelvesJson() => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            foreach (CoreCatalog.LibraryShelf s in CoreCatalog.ShelvesInReleaseOrder)
            {
                w.WriteStartObject();
                w.WriteString("name", s.Name);
                w.WriteString("label", s.Label);
                w.WriteString("system", s.System);
                w.WriteString("esde_system", s.EsdeSystem);
                w.WriteString("esde_full_name", s.EsdeFullName);
                w.WriteStartObject("core");
                w.WriteString("display_name", s.Core.DisplayName);
                w.WriteString("console", s.Core.Console);
                CrystalJson.Strings(w, "extensions", s.Core.Extensions);
                w.WriteEndObject();
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        // The names `core <name>` knows a system's core by, and that core's own display name, console and extensions, from the catalog's registry - see EmuSen_Backend.md §14.11.
        private static void CoreOf(Utf8JsonWriter w, string console, IReadOnlyList<string> extensions)
        {
            var named = CoreCatalog.Registry.Where(kv => kv.Value.Console == console).ToArray();
            if (named.Length == 0) named = CoreCatalog.Registry.Where(kv => extensions.All(kv.Value.SupportsExtension)).ToArray();
            if (named.Length == 0)
            {
                w.WriteNull("core");
                return;
            }
            CoreDescriptor core = named[0].Value;
            w.WriteStartObject("core");
            CrystalJson.Strings(w, "names", named.Where(kv => ReferenceEquals(kv.Value, core)).Select(kv => kv.Key));
            w.WriteString("display_name", core.DisplayName);
            w.WriteString("console", core.Console);
            CrystalJson.Strings(w, "extensions", core.Extensions);
            w.WriteEndObject();
        }

        // The engines that run a system: the console's choice where it has one, else what discovery found for it.
        private static IReadOnlyList<string> EnginesOf(string console, string systemId) =>
            CoreCatalog.EngineFor(console)?.Choices is { Count: > 0 } choices ? choices
            : CoreCatalog.SoleDiscoveredEngine(console) is { } sole ? new[] { sole }
            : CoreDiscovery.ForSystem(systemId).Select(c => c.EngineName).Distinct().ToArray();

        public static byte[] EnginesJson() => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            foreach (DiscoveredCore c in CoreDiscovery.Found)
            {
                w.WriteStartObject();
                w.WriteString("name", c.EngineName);
                w.WriteString("id", c.Info.Id);
                w.WriteString("version", c.Info.Version);
                CrystalJson.Strings(w, "systems", c.Info.Systems.Where(CoreDiscovery.Offered).Select(s => s.Id));
                w.WriteBoolean("available", File.Exists(c.Sidecar.LibraryPath));
                w.WriteString("report", c.Report);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        public static byte[] CoresJson() => CrystalJson.Write(w =>
        {
            w.WriteStartArray();
            foreach (DiscoveredCore c in CoreDiscovery.Found)
            {
                w.WriteStartObject();
                w.WriteString("library", c.Sidecar.LibraryPath);
                w.WriteString("sidecar", c.Sidecar.SidecarPath);
                w.WriteString("sha256", c.Sidecar.Sha256);
                w.WriteString("abi", c.Sidecar.Abi);
                w.WriteBoolean("development", c.Sidecar.Development);
                w.WriteString("engine", c.EngineName);
                w.WriteString("report", c.Report);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        });

        // The system a file runs as, the systems table's id or an engine's own: what routing decides, without running a core.
        public static string? SystemOfFile(string path) => Routed(path).System;

        private static (string? System, CoreDescriptor? Core, Routed? Decision) Routed(string path)
        {
            var (core, routed) = CoreCatalog.Route(path);
            string extension = CoreCatalog.ImageExtension(path);
            string? system = routed?.System?.Id
                ?? (core is not null ? CoreCatalog.SystemIdsFor(core.Console).FirstOrDefault() ?? SystemPacks.All.FirstOrDefault(p => p.Entry.Console == core.Console)?.Entry.Id : null)
                ?? (routed is null ? CoreDiscovery.ForExtension(extension).Select(c => c.Info.SystemFor(extension)?.Id).FirstOrDefault(id => id is not null) : null);
            return (system, core, routed);
        }

        public static byte[] RouteJson(string path) => CrystalJson.Write(w =>
        {
            var (system, core, routed) = Routed(path);
            string extension = CoreCatalog.ImageExtension(path);
            string? console = core?.Console ?? routed?.System?.Console ?? (system is not null ? CoreCatalog.DiscoveredConsoleForSystem(system) : null);
            string? engine = system is null ? null : EngineFor(path) ?? (console is not null ? CoreCatalog.SoleDiscoveredEngine(console) : null)
                ?? CoreDiscovery.ForSystem(system).Select(c => c.EngineName).FirstOrDefault();
            w.WriteStartObject();
            CrystalJson.StringOrNull(w, "system", system);
            CrystalJson.StringOrNull(w, "console", console);
            CrystalJson.StringOrNull(w, "engine", engine);
            CrystalJson.StringOrNull(w, "reason", system is null ? routed?.Reason ?? $"no engine in this build takes {extension} files" : null);
            Cartridge(w, system, path);
            CrystalJson.StringOrNull(w, "shelf", CoreCatalog.ShelfFor(path, core));
            w.WriteEndObject();
        });

        // What a system pack's header reader says of the file, the NES's iNES header today; null for a system whose pack reads none or a file it cannot read.
        private static void Cartridge(System.Text.Json.Utf8JsonWriter w, string? system, string path)
        {
            if (system != "nes" || !File.Exists(path))
            {
                w.WriteNull("cartridge");
                return;
            }
            try
            {
                var header = Crystal.Systems.Nes.NesHeader.Read(File.ReadAllBytes(path));
                w.WriteStartObject("cartridge");
                w.WriteNumber("board", header.Mapper);
                w.WriteString("header_trust", header.Trust switch { Crystal.Systems.Nes.NesHeaderTrust.Archaic => "archaic", Crystal.Systems.Nes.NesHeaderTrust.Unverifiable => "unverifiable", _ => "clean" });
                w.WriteNumber("prg_bytes", (long)header.PrgBanks * Crystal.Systems.Nes.NesHeader.PrgBankSize);
                w.WriteNumber("chr_bytes", header.ChrIsRam ? Crystal.Systems.Nes.NesHeader.ChrBankSize : (long)header.ChrBanks * Crystal.Systems.Nes.NesHeader.ChrBankSize);
                w.WriteBoolean("battery", header.HasBattery);
                w.WriteEndObject();
            }
            catch (Exception e) when (e is InvalidDataException or IOException or UnauthorizedAccessException)
            {
                w.WriteNull("cartridge");
            }
        }
    }
}
