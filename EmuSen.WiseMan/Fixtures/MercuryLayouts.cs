using EmuSen.Cores.Native;
using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.CompilerServices;
using EmuSen.Cores.Nintendo.MercuryRT;

namespace EmuSen.WiseMan.Fixtures
{
    // The committed layout listings, the state format's specification, and the one a state of a cartridge follows - see Mercury_Native.md §14.5.
    public static class MercuryLayouts
    {
        // Set to 1 to write a listing missing for a board, console and RAM size from MercuryRT's state_layout instead of failing.
        public const string RecordVariable = "EMUSEN_MERCURY_LAYOUT_RECORD";

        // The state version the committed listings describe; a new version records them again.
        public const int StateVersion = 11;

        public static string Directory([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "MercuryGoldens", "layouts");

        // The board a cartridge type selects; the listings are named by it.
        public static string Board(byte cartridgeType) => cartridgeType switch
        {
            0x00 or 0x08 or 0x09 => "none",
            >= 0x01 and <= 0x03 => "mbc1",
            0x05 or 0x06 => "mbc2",
            >= 0x0F and <= 0x13 => "mbc3",
            >= 0x19 and <= 0x1E => "mbc5",
            _ => throw new ArgumentException($"no board for cartridge type ${cartridgeType:X2}"),
        };

        public static string FileName(string board, bool colourConsole, int ramLength) => $"{board}-{(colourConsole ? "gbc" : "gb")}-ram{ramLength}.layout";

        // The listing a state of this cartridge follows: the board from the header, the console from the state's own header, the RAM size from its length.
        public static StateLayout For(byte[] rom, ReadOnlySpan<byte> state)
        {
            string board = Board(rom[0x147]);
            bool colour = state[8] != 0;
            string prefix = $"{board}-{(colour ? "gbc" : "gb")}-";
            lock (Chosen)
                if (Chosen.TryGetValue((prefix, state.Length), out StateLayout? known)) return known;
            string dir = Directory();
            foreach (string path in System.IO.Directory.Exists(dir) ? System.IO.Directory.GetFiles(dir, prefix + "*.layout") : Array.Empty<string>())
            {
                var layout = Load(path);
                if (layout.Length != state.Length) continue;
                lock (Chosen) Chosen[(prefix, state.Length)] = layout;
                return layout;
            }
            throw new InvalidDataException($"No committed layout for a {board} board on the {(colour ? "Game Boy Color" : "Game Boy")} with a {state.Length}-byte state; record one with {RecordVariable}=1.");
        }

        private static readonly Dictionary<string, StateLayout> Loaded = new(StringComparer.Ordinal);
        private static readonly Dictionary<(string, int), StateLayout> Chosen = new();

        public static StateLayout Load(string path)
        {
            lock (Loaded)
            {
                if (!Loaded.TryGetValue(path, out StateLayout? layout))
                    Loaded[path] = layout = StateLayout.FromText(Path.GetFileName(path), ReadListing(path));
                return layout;
            }
        }

        // A committed listing as the engine prints it: a Windows checkout gives its lines CRLF ends, which the engine never writes.
        public static string ReadListing(string path) => File.ReadAllText(path).ReplaceLineEndings("\n");

        // A listing written from the engine's state_layout, named for its board, console and RAM size; the commit that adds it says why.
        public static string Record(byte cartridgeType, string text)
        {
            var layout = StateLayout.FromText("", text);
            StateLayout.Field ram = layout["Cart.Ram"];
            StateLayout.Field console = layout["Console"];
            Assert.True(console.Length == 1, "a listing without the console byte");
            string name = FileName(Board(cartridgeType), text.Contains("u8[16384] Bus.Vram", StringComparison.Ordinal), ram.Length);
            System.IO.Directory.CreateDirectory(Directory());
            string path = Path.Combine(Directory(), name);
            File.WriteAllText(path, text);
            lock (Loaded) Loaded.Remove(path);
            lock (Chosen) Chosen.Clear();
            return name;
        }
    }
}
