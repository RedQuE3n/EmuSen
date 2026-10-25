using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Game Package window in its two states from the game options sheet: a game with no package, and a long-named game whose package holds two soft patches with two more beside it - see EmuSen_GamePackages.md §15.
    public partial class WindowFitAuditTests
    {
        internal const string LongPatch = "A Very Long Translation Patch Name, Version 2.1 With Every Fix Applied (English).bps";

        // A linked package for game naming it by its full path, its list holding the patches named, each file written beside the manifest.
        internal static string PackageWithPatches(string game, string console, params string[] patches)
        {
            string folder = Path.Combine(DataStore.Packages, console, Path.GetFileName(game));
            Directory.CreateDirectory(folder);
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), "snes", "linked", new PackageFile("image", "absolute", Path.GetFullPath(game)));
            var list = new JsonArray();
            foreach (string name in patches)
            {
                File.WriteAllBytes(Path.Combine(folder, name), "PATCHEOF"u8.ToArray());
                list.Add(new JsonObject { ["file"] = name, ["format"] = Path.GetExtension(name).TrimStart('.').TrimEnd("0123456789".ToCharArray()) });
            }
            p.Document["patches"] = list;
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
            return folder;
        }

        private static void ShowGamePackage(MainWindow window, string romDirectory, string which)
        {
            bool packaged = which == "GamePackageLongTitle";
            string title = packaged ? LongTitle : "Cobalt Harbor (Synthetic)";
            string game = Path.Combine(romDirectory, title + ".sfc");
            if (!File.Exists(game)) File.WriteAllBytes(game, new byte[0x8000]);
            string package = Path.Combine(DataStore.Packages, "SNES", Path.GetFileName(game));
            if (Directory.Exists(package)) Directory.Delete(package, recursive: true);
            if (packaged)
            {
                PackageWithPatches(game, "SNES", LongTitle + ".ips", LongTitle + ".ips1");
                foreach (string beside in new[] { LongPatch, "Restoration (v1.0).ips" }) File.WriteAllBytes(Path.Combine(romDirectory, beside), "PATCHEOF"u8.ToArray());
            }
            Call(window, "ShowGamePackage", game, title);
            Settle(window);
            var shown = (GamePackageWindow)Sheets(window).Current!;
            if (packaged)
            {
                shown.Apply(JsonSerializer.Serialize(new { @event = "package", index = 1, of = 1, package = "SNES/" + Path.GetFileName(game), state = "added", patch = LongTitle + ".ips1", retroarch = true }));
                Assert.Equal(2, shown.Patches().Count);
            }
            Settle(window);
            Assert.Equal(packaged, shown.Package is not null);
        }
    }
}
