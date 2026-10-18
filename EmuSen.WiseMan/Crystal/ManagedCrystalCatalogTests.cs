using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Cores;
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
            JsonElement n64 = Json(_f.Runtime.Systems()).EnumerateArray().First(s => s.GetProperty("id").GetString() == "n64").GetProperty("controllers");
            Assert.Contains((int)PadAxis.LeftX, n64.GetProperty("axes").EnumerateArray().Select(a => a.GetInt32()));
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
