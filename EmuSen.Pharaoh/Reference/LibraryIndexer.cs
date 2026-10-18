using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Library.Catalogue;
using EmuSen.Galaxia.Models;

namespace EmuSen.Pharaoh.Reference
{
    // Walks the ROM library once and writes what it learned - see EmuSen_Galaxia.md §7.
    //
    // Read-only on the library, without exception. Nothing here opens a ROM for
    // writing, moves one, or deletes one.
    public static class LibraryIndexer
    {
        public static int Run(string[] args)
        {
            string root = args.Length >= 2 ? args[1] : RomDirectory();
            if (!Directory.Exists(root))
            {
                Console.WriteLine($"[ERROR] no ROM library at {root}");
                return 1;
            }

            string dbPath = args.Length >= 3
                ? args[2]
                : Path.Combine(DataStore.UsrHome, "catalogue.db");

            Console.WriteLine($"[INFO] indexing {root}");
            using SqliteCatalogue catalogue = SqliteCatalogue.Open(dbPath);
            using var runtime = new CrystalRuntime();

            var entries = new List<RomEntry>();
            int read = 0, failed = 0;

            foreach (string path in Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories))
            {
                string extension = Path.GetExtension(path).ToLowerInvariant();
                if (extension is not (".nes" or ".smc" or ".sfc" or ".gb" or ".gbc")) continue;

                try
                {
                    entries.Add(Describe(runtime, path, extension));
                    read++;
                }
                catch (Exception)
                {
                    // A dump this program cannot parse is still a fact about the
                    // library, so it is recorded as unplayable rather than skipped.
                    var info = new FileInfo(path);
                    entries.Add(new RomEntry(path, Path.GetFileNameWithoutExtension(path),
                        SystemFor(extension), info.Length, null, null, null, null, 0, 0, false, null));
                    failed++;
                }
            }

            catalogue.PutAll(entries);
            Console.WriteLine($"[INFO] {read} readable, {failed} unreadable, {catalogue.Count} rows in {dbPath}");
            return 0;
        }

        // Only the NES loader is asked for detail today; the others are catalogued
        // by name and size until their cores can answer the same questions.
        private static RomEntry Describe(CrystalRuntime runtime, string path, string extension)
        {
            var info = new FileInfo(path);
            string name = Path.GetFileNameWithoutExtension(path);
            string system = SystemFor(extension);

            if (extension != ".nes")
            {
                return new RomEntry(path, name, system, info.Length, null, null, null, null, 0, 0, false, null);
            }

            // The board as the NES pack reads the header, catalogued even where unimplemented; playable is whether the engine opens it.
            JsonElement cart = JsonDocument.Parse(runtime.Route(path)).RootElement.GetProperty("cartridge");
            if (cart.ValueKind != JsonValueKind.Object) throw new InvalidDataException($"{path} has no header the NES pack reads");
            bool playable;
            try
            {
                using CrystalSession session = runtime.Open(new CrystalOpen { Path = path, NoBattery = true });
                playable = true;
            }
            catch (CrystalException)
            {
                playable = false;
            }

            return new RomEntry(
                path, name, system, info.Length, null,
                cart.GetProperty("board").GetInt32().ToString(CultureInfo.InvariantCulture),
                cart.GetProperty("header_trust").GetString(),
                null, cart.GetProperty("prg_bytes").GetInt64(), cart.GetProperty("chr_bytes").GetInt64(), playable, null);
        }

        private static string SystemFor(string extension) => extension switch
        {
            ".nes" => "nes",
            ".smc" or ".sfc" => "snes",
            _ => "gb",
        };

        // AppSettings.RomDirectory is the only authoritative answer to where the
        // library is; nothing here resolves a game through a hardcoded path.
        private static string RomDirectory() => SettingFolders.Roms(AppSettings.Load()) ?? "";
    }
}
