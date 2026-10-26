using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Game Package window in its three states from the game options sheet: a game with no package, a long-named game whose package holds two soft patches with two more beside it, and a long-named self-contained package scraped as patched - see EmuSen_GamePackages.md §15 and §19.
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

        // A self-contained package for game: the game copied in, its list holding one patch, scraped as patched.
        internal static string SelfContainedWithPatch(string game, string console)
        {
            string folder = Path.Combine(DataStore.Packages, console, Path.GetFileName(game));
            Directory.CreateDirectory(folder);
            string copy = Path.Combine(folder, Path.GetFileName(game));
            File.Copy(game, copy, overwrite: true);
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), "snes", "self-contained", new PackageFile("image", "package", Path.GetFileName(game)));
            p.Document["sources"] = new JsonArray(new JsonObject { ["role"] = "image", ["where"] = "absolute", ["path"] = Path.GetFullPath(game) });
            string patch = Path.GetFileNameWithoutExtension(game) + ".bps";
            File.WriteAllBytes(Path.Combine(folder, patch), "BPS1"u8.ToArray());
            p.Document["patches"] = new JsonArray(new JsonObject { ["file"] = patch, ["format"] = "bps" });
            p.Document["identity"] = "patched";
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
            return copy;
        }

        private static void ShowGamePackage(MainWindow window, string romDirectory, string which)
        {
            if (which == "GamePackageSelfContained")
            {
                string library = Path.Combine(romDirectory, LongTitle + " (Self-Contained).sfc");
                if (!File.Exists(library)) File.WriteAllBytes(library, new byte[0x8000]);
                string held = Path.Combine(DataStore.Packages, "SNES", Path.GetFileName(library));
                if (Directory.Exists(held)) Directory.Delete(held, recursive: true);
                string copy = SelfContainedWithPatch(library, "SNES");
                Call(window, "ShowGamePackage", copy, LongTitle);
                Settle(window);
                var contained = (GamePackageWindow)Sheets(window).Current!;
                Assert.True(contained.SelfContained && contained.Identity == "patched");
                Settle(window);
                return;
            }
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
