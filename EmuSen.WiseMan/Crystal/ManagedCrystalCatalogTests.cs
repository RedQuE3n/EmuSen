using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The catalog's facts 1.1 gives a library with no session: the shelves, each system's controllers and a file's shelf, held equal to the catalog that answers them - see EmuSen_Backend.md §14.13.
    [Collection(TestCollections.ProcessGlobals)]
    public class ManagedCrystalCatalogTests : System.IDisposable
    {
        private readonly CrystalFixture _f = new();

        public void Dispose() => _f.Dispose();

        private static JsonElement Json(string text) => JsonDocument.Parse(text).RootElement;

        [Fact]
        public void The_shelves_are_the_catalog_s_in_release_order()
        {
            var shelves = Json(_f.Runtime.Shelves()).EnumerateArray().ToArray();
            Assert.Equal(CoreCatalog.ShelvesInReleaseOrder.Select(s => s.Name), shelves.Select(s => s.GetProperty("name").GetString()));
            foreach ((CoreCatalog.LibraryShelf want, JsonElement got) in CoreCatalog.ShelvesInReleaseOrder.Zip(shelves))
            {
                Assert.Equal((want.Label, want.System, want.EsdeSystem, want.EsdeFullName),
                    (got.GetProperty("label").GetString(), got.GetProperty("system").GetString(), got.GetProperty("esde_system").GetString(), got.GetProperty("esde_full_name").GetString()));
                JsonElement core = got.GetProperty("core");
                Assert.Equal((want.Core.DisplayName, want.Core.Console), (core.GetProperty("display_name").GetString(), core.GetProperty("console").GetString()));
                Assert.Equal(want.Core.Extensions, core.GetProperty("extensions").EnumerateArray().Select(e => e.GetString()));
            }
            Assert.Contains(shelves, s => s.GetProperty("name").GetString() == CoreCatalog.GameBoyColorShelf && s.GetProperty("system").GetString() == "gbc");
        }

        // The pad, the most ports and the players as consoles.json stands, for every system the table lists.
        [Fact]
        public void Each_system_s_controllers_are_the_engine_s_as_the_settings_stand()
        {
            foreach (JsonElement system in Json(_f.Runtime.Systems()).EnumerateArray().Where(s => s.TryGetProperty("console", out _)))
            {
                string console = system.GetProperty("console").GetString()!;
                JsonElement c = system.GetProperty("controllers");
                Assert.Equal(CoreCatalog.ButtonsFor(console).Select(b => (int)b), c.GetProperty("buttons").EnumerateArray().Select(b => b.GetInt32()));
                Assert.Equal(CoreCatalog.AxesFor(console).Select(a => (int)a), c.GetProperty("axes").EnumerateArray().Select(a => a.GetInt32()));
                Assert.Equal(ControllerPorts.ForConsole(console), c.GetProperty("ports").GetInt32());
                System.Func<string, string?> stored = key => ConsolesConfig.Load().Value(console, key);
                Assert.Equal(ControllerPorts.ForConsole(console, stored), c.GetProperty("players").GetArrayLength());
            }
        }

        // A catalog console's pad, ports and players are its sole engine's core info, read from the sidecars beside the program (§17.8 Q11) - see EmuSen_Backend.md §18.10.10.
        [CSharpImplementationFact("it moves the C# discovery's directories, which the Rust takes from its config; CrystalParityTests compares the systems' JSON")]
        public void Each_catalog_console_s_controllers_are_its_engine_s_core_info()
        {
            CoreDiscovery.UseDirectories(null);
            var seen = new System.Collections.Generic.List<string>();
            foreach (JsonElement system in Json(_f.Runtime.Systems()).EnumerateArray().Where(s => s.TryGetProperty("console", out _)))
            {
                string id = system.GetProperty("id").GetString()!, console = system.GetProperty("console").GetString()!;
                if (CoreCatalog.SoleDiscoveredEngine(console) is not { } engine) continue;
                CoreSystem info = Assert.Single(Assert.IsType<DiscoveredCore>(CoreDiscovery.ByEngineName(engine)).Info.Systems, s => s.Id == id);
                JsonElement c = system.GetProperty("controllers");
                int ports = info.Controllers.SelectMany(k => k.Ports).Select(p => (int)p + 1).Max();
                Assert.Equal(info.Controllers.SelectMany(k => k.Buttons).Select(b => b.Control).OfType<PadButton>().Distinct().Select(b => (int)b), c.GetProperty("buttons").EnumerateArray().Select(b => b.GetInt32()));
                Assert.Equal(info.Controllers.SelectMany(k => k.Axes).Select(a => a.Control).OfType<PadAxis>().Distinct().Select(a => (int)a), c.GetProperty("axes").EnumerateArray().Select(a => a.GetInt32()));
                Assert.Equal(ports, c.GetProperty("ports").GetInt32());
                Assert.Equal(ports, c.GetProperty("players").GetArrayLength());
                seen.Add(id);
            }
            Assert.Superset(new System.Collections.Generic.HashSet<string> { "nes", "snes", "gb", "n64" }, seen.ToHashSet());
            JsonElement n64 = Json(_f.Runtime.Systems()).EnumerateArray().First(s => s.GetProperty("id").GetString() == "n64").GetProperty("controllers");
            Assert.Contains((int)PadAxis.LeftX, n64.GetProperty("axes").EnumerateArray().Select(a => a.GetInt32()));
        }

        // The NES's and SNES's pad and ports move with their engine's sidecar, so the core info is their one owner - see EmuSen_Backend.md §18.10.12.
        [CSharpImplementationFact("it moves the C# discovery's directories, which the Rust takes from its config; CrystalParityTests compares the systems' JSON")]
        public void The_nes_and_snes_pads_follow_their_engine_s_sidecar()
        {
            CoreDiscovery.UseDirectories(null);
            string edited = Path.Combine(_f.Root, "edited-sidecars");
            Directory.CreateDirectory(edited);
            var want = new System.Collections.Generic.Dictionary<string, PadButton[]>();
            foreach (string console in new[] { "NES", "SNES" })
            {
                DiscoveredCore core = Assert.IsType<DiscoveredCore>(CoreDiscovery.ByEngineName(CoreCatalog.SoleDiscoveredEngine(console)));
                var sidecar = (System.Text.Json.Nodes.JsonObject)System.Text.Json.Nodes.JsonNode.Parse(File.ReadAllText(core.Sidecar.SidecarPath))!;
                var controller = sidecar["info"]!["systems"]![0]!["controllers"]![0]!.AsObject();
                controller["buttons"] = new System.Text.Json.Nodes.JsonArray(controller["buttons"]!.AsArray().Reverse().Select(b => b!.DeepClone()).ToArray());
                controller["ports"] = new System.Text.Json.Nodes.JsonArray(0);
                File.WriteAllText(Path.Combine(edited, Path.GetFileName(core.Sidecar.SidecarPath)), sidecar.ToJsonString());
                want[console] = core.Info.Systems[0].Controllers[0].Buttons.Select(b => b.Control).OfType<PadButton>().Reverse().ToArray();
            }
            CoreDiscovery.UseDirectories(new[] { edited });
            JsonElement[] systems = Json(_f.Runtime.Systems()).EnumerateArray().ToArray();
            foreach ((string console, PadButton[] buttons) in want)
            {
                Assert.Equal(buttons, CoreCatalog.ButtonsFor(console));
                Assert.Equal(1, ControllerPorts.ForConsole(console));
                JsonElement c = systems.Single(s => s.TryGetProperty("console", out JsonElement n) && n.GetString() == console).GetProperty("controllers");
                Assert.Equal(buttons.Select(b => (int)b), c.GetProperty("buttons").EnumerateArray().Select(b => b.GetInt32()));
                Assert.Equal((1, 1), (c.GetProperty("ports").GetInt32(), c.GetProperty("players").GetArrayLength()));
            }
        }

        // A Game Boy Color game sits on the Color's shelf, a Game Boy game on the Game Boy's, and a file no core takes on none.
        [Fact]
        public void Route_names_the_file_s_shelf()
        {
            string gbc = Path.Combine(_f.Root, "Color.gbc"), gb = Path.Combine(_f.Root, "Mono.gb"), none = Path.Combine(_f.Root, "Notes.txt");
            byte[] cart = new byte[0x8000];
            File.WriteAllBytes(gbc, cart);
            File.WriteAllBytes(gb, cart);
            File.WriteAllText(none, "not a game");
            Assert.Equal(CoreCatalog.GameBoyColorShelf, Json(_f.Runtime.Route(gbc)).GetProperty("shelf").GetString());
            Assert.Equal(CoreCatalog.ShelfFor(gb), Json(_f.Runtime.Route(gb)).GetProperty("shelf").GetString());
            Assert.Equal(JsonValueKind.Null, Json(_f.Runtime.Route(none)).GetProperty("shelf").ValueKind);
        }
    }
}
