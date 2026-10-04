using System;
using System.Collections.Generic;
using System.Text.Json;
using SDL3;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;

namespace EmuSen.Endymion.Input
{
    // The gamepad counterpart to ControllerKeyBindings: one map per console,
    // loaded and saved as one file - see EmuSen_Input.md §5.1.
    public sealed class GamepadBindings
    {
        private readonly Dictionary<string, GamepadBindingMap> _byConsole = new(StringComparer.OrdinalIgnoreCase);

        public GamepadBindings(IEnumerable<string> consoles)
        {
            foreach (string console in consoles) _byConsole[console] = new GamepadBindingMap();
        }

        public IReadOnlyDictionary<string, GamepadBindingMap> ByConsole => _byConsole;

        public GamepadBindingMap For(string console)
        {
            if (!_byConsole.TryGetValue(console, out GamepadBindingMap? map))
            {
                _byConsole[console] = map = new GamepadBindingMap();
            }
            return map;
        }

        // Players 2 on, by their file key; a player missing here plays with player 1's map - see EmuSen_Input.md §8.4.
        private readonly Dictionary<string, GamepadBindingMap> _byPlayer = new(StringComparer.OrdinalIgnoreCase);

        private const string PlayerWord = " Player ";

        // The file key of a player's map beside the console's own, so a build that predates players keeps it as an unknown console.
        public static string PlayerKey(string console, int player) => $"{console}{PlayerWord}{player}";

        public GamepadBindingMap For(string console, int player) =>
            player > 1 && _byPlayer.TryGetValue(PlayerKey(console, player), out GamepadBindingMap? own) ? own : For(console);

        public bool HasOwn(string console, int player) => player > 1 && _byPlayer.ContainsKey(PlayerKey(console, player));

        // The player's own map, begun as a copy of player 1's the first time it is changed.
        public GamepadBindingMap Own(string console, int player)
        {
            if (player <= 1) return For(console);
            string key = PlayerKey(console, player);
            if (!_byPlayer.TryGetValue(key, out GamepadBindingMap? own))
            {
                own = new GamepadBindingMap();
                own.Replace(new Dictionary<PadButton, SDL.GamepadButton>(For(console).ButtonToPad));
                _byPlayer[key] = own;
            }
            return own;
        }

        // Back to player 1's map.
        public void Forget(string console, int player) => _byPlayer.Remove(PlayerKey(console, player));

        public void ResetToDefaults()
        {
            foreach (GamepadBindingMap map in _byConsole.Values) map.ResetToDefaults();
            _byPlayer.Clear();
        }

        private static readonly ConfigFile<Dictionary<string, Dictionary<PadButton, SDL.GamepadButton>>> File =
            new("gamepadbindings.json");

        // Only ever asks "is this the flat shape or the per-console one" - JsonElement
        // accepts any value, so a bad enum still reaches the typed load below and is
        // reported there exactly once - see EmuSen_Config_Reference.md §3.6.
        private static readonly ConfigFile<Dictionary<string, JsonElement>> ShapeProbe = new("gamepadbindings.json");

        private static readonly ConfigFile<Dictionary<PadButton, SDL.GamepadButton>> LegacyFile =
            new("gamepadbindings.json");

        public bool Save()
        {
            var payload = new Dictionary<string, Dictionary<PadButton, SDL.GamepadButton>>(StringComparer.OrdinalIgnoreCase);
            foreach (var kv in _byConsole) payload[kv.Key] = kv.Value.ButtonToPad;
            foreach (var kv in _byPlayer) payload[kv.Key] = kv.Value.ButtonToPad;
            return File.Save(payload);
        }

        public static GamepadBindings Load(IEnumerable<string> consoles)
        {
            var bindings = new GamepadBindings(consoles);
            if (ShapeProbe.Load() is not { Count: > 0 } probe) return bindings;

            // A flat file predates per-console bindings, so every console inherits it - that is what it used to mean.
            if (IsLegacyFlat(probe))
            {
                if (LegacyFile.Load() is { Count: > 0 } shared)
                {
                    foreach (GamepadBindingMap map in bindings._byConsole.Values)
                    {
                        map.Replace(new Dictionary<PadButton, SDL.GamepadButton>(shared));
                        map.AddDefaultsForNewButtons();
                    }
                }
                return bindings;
            }

            if (File.Load() is not { } loaded) return bindings;

            foreach (var kv in loaded)
            {
                if (kv.Value is not { Count: > 0 }) continue;
                GamepadBindingMap map = IsPlayerKey(kv.Key) ? bindings._byPlayer[kv.Key] = new GamepadBindingMap() : bindings.For(kv.Key);
                map.Replace(kv.Value);
                map.AddDefaultsForNewButtons();
            }
            return bindings;
        }

        private static bool IsPlayerKey(string key)
        {
            int at = key.LastIndexOf(PlayerWord, StringComparison.Ordinal);
            return at > 0 && int.TryParse(key.AsSpan(at + PlayerWord.Length), System.Globalization.NumberStyles.None, null, out int player) && player > 1;
        }

        // The values are pad buttons in the old shape and objects in the new one.
        private static bool IsLegacyFlat(Dictionary<string, JsonElement> probe)
        {
            foreach (var kv in probe) return kv.Value.ValueKind != JsonValueKind.Object;
            return false;
        }
    }
}
