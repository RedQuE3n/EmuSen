using System;
using System.Collections.Generic;
using System.Text;
using System.Text.Json;
using EmuSen.Endymion.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Native;

namespace EmuSen.Endymion.Input
{
    // The four binding files, by emusen_platform.h's EMUSEN_ENDYMION_BINDINGS_* numbers.
    public enum BindingFile : uint
    {
        Keys,
        Gamepad,
        Profiles,
        Hotkeys,
    }

    // A profile, in its own spelling, and the player whose map in it a player plays with - see EmuSen_Input.md §5.2.
    public sealed record BindingChoice(string Profile, int MapPlayer);

    // The platform library reads and writes the binding files, its one reader and writer; this binds what crosses - see EmuSen_Input.md §5.2.
    public static unsafe class BindingFiles
    {
        // A file's model as a program starts with it, for the consoles it knows; a file that would not read is its defaults, and says why.
        public static T Load<T>(BindingFile file, IEnumerable<string> consoles)
        {
            EndymionNative.Up();
            var names = new StringBuilder();
            foreach (string console in consoles) names.Append(console).Append('\0');
            byte[] joined = Encoding.UTF8.GetBytes(names.ToString());
            string text = DeviceNative.Text((o, c) =>
            {
                fixed (byte* n = joined) return EndymionNative.BindingsLoad((uint)file, n, (nuint)joined.Length, o, c);
            })!;
            PlatformLibrary.ReportDiagnostics();
            return JsonSerializer.Deserialize<T>(text, ConfigJson.Options)!;
        }

        // False when the file could not be written; a change that cannot reach the disk just won't survive a restart.
        public static bool Save<T>(BindingFile file, T model)
        {
            EndymionNative.Up();
            byte[] document = JsonSerializer.SerializeToUtf8Bytes(model, ConfigJson.Options);
            fixed (byte* d = document) return EndymionNative.BindingsSave((uint)file, d, (nuint)document.Length) == 0;
        }

        // One map at its defaults: a console's keys, a console's gamepad buttons, or the hotkeys; a new one each call.
        public static T Defaults<T>(BindingFile file)
        {
            EndymionNative.Up();
            return JsonSerializer.Deserialize<T>(DeviceNative.Text((o, c) => EndymionNative.BindingsDefaults((uint)file, o, c))!, ConfigJson.Options)!;
        }

        // Which map a player plays with, by Endymion's rule over the two models passed: in the profile named, else the one that applies to the game's path.
        public static BindingChoice Choose(object gamepad, object profiles, string console, int player, string? game, string? profile)
        {
            EndymionNative.Up();
            byte[] pads = JsonSerializer.SerializeToUtf8Bytes(gamepad, ConfigJson.Options), sets = JsonSerializer.SerializeToUtf8Bytes(profiles, ConfigJson.Options);
            byte[] name = Encoding.UTF8.GetBytes(console);
            byte[]? path = game is null ? null : Encoding.UTF8.GetBytes(game), chosen = profile is null ? null : Encoding.UTF8.GetBytes(profile);
            string text = DeviceNative.Text((o, c) =>
            {
                fixed (byte* g = pads, s = sets, n = name, p = path, f = chosen)
                    return EndymionNative.BindingsChoose(g, (nuint)pads.Length, s, (nuint)sets.Length, n, (nuint)name.Length, player, p, (nuint)(path?.Length ?? 0), f, (nuint)(chosen?.Length ?? 0), o, c);
            })!;
            return JsonSerializer.Deserialize<BindingChoice>(text, ConfigJson.Options)!;
        }
    }
}
