using System;
using System.Collections.Generic;
using System.Linq;
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

        // Default's maps back to the defaults; the named profiles are the player's and stay.
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

        // --- Profiles, store 1.1: named sets beside Default, which is the console's maps above - see EmuSen_Input.md §10. ---

        public const string DefaultProfile = "Default";

        // A named profile's maps by player; player 1's is always there.
        private sealed class Profile(string name)
        {
            public string Name { get; set; } = name;
            public Dictionary<int, GamepadBindingMap> Maps { get; } = new();
        }

        private sealed class ConsoleProfiles
        {
            public List<Profile> Named { get; } = new();
            public string? Chosen { get; set; }
        }

        private readonly Dictionary<string, ConsoleProfiles> _profiles = new(StringComparer.OrdinalIgnoreCase);

        private static bool IsDefault(string name) => string.Equals(name, DefaultProfile, StringComparison.OrdinalIgnoreCase);

        private Profile? Named(string console, string name) =>
            _profiles.TryGetValue(console, out ConsoleProfiles? all) ? all.Named.Find(p => string.Equals(p.Name, name, StringComparison.OrdinalIgnoreCase)) : null;

        private ConsoleProfiles ProfilesOf(string console)
        {
            if (!_profiles.TryGetValue(console, out ConsoleProfiles? all)) _profiles[console] = all = new ConsoleProfiles();
            return all;
        }

        private static GamepadBindingMap Copy(GamepadBindingMap map)
        {
            var copy = new GamepadBindingMap();
            copy.Replace(new Dictionary<PadButton, SDL.GamepadButton>(map.ButtonToPad));
            return copy;
        }

        // Default first, then the console's named profiles in the order they were made.
        public IReadOnlyList<string> Profiles(string console) =>
            _profiles.TryGetValue(console, out ConsoleProfiles? all) ? [DefaultProfile, .. all.Named.Select(p => p.Name)] : [DefaultProfile];

        // A new profile begun as a copy of another, Default unless named; false for a blank name or one taken in any case.
        public bool CreateProfile(string console, string name, string? copyOf = null)
        {
            name = name.Trim();
            if (name.Length == 0 || IsDefault(name) || Named(console, name) is not null) return false;
            string from = copyOf ?? DefaultProfile;
            if (!IsDefault(from) && Named(console, from) is null) return false;
            var made = new Profile(name);
            for (int player = 1; player <= PlayerSlots.MaxPlayers; player++)
                if (player == 1 || HasOwn(console, player, from)) made.Maps[player] = Copy(For(console, player, from));
            ProfilesOf(console).Named.Add(made);
            return true;
        }

        // Default is never renamed, and a name taken in any case is refused; the console's choice follows the rename.
        public bool RenameProfile(string console, string name, string newName)
        {
            newName = newName.Trim();
            if (IsDefault(name) || newName.Length == 0 || IsDefault(newName) || Named(console, name) is not { } profile) return false;
            if (Named(console, newName) is { } other && other != profile) return false;
            ConsoleProfiles all = ProfilesOf(console);
            if (all.Chosen is not null && string.Equals(all.Chosen, profile.Name, StringComparison.OrdinalIgnoreCase)) all.Chosen = newName;
            profile.Name = newName;
            return true;
        }

        // Default is never deleted; a console whose chosen profile goes starts its games with Default.
        public bool DeleteProfile(string console, string name)
        {
            if (IsDefault(name) || Named(console, name) is not { } profile) return false;
            ConsoleProfiles all = ProfilesOf(console);
            all.Named.Remove(profile);
            if (all.Chosen is not null && string.Equals(all.Chosen, profile.Name, StringComparison.OrdinalIgnoreCase)) all.Chosen = null;
            return true;
        }

        // The profile the console's games start with unless a game names its own.
        public bool ChooseDefault(string console, string name)
        {
            Profile? profile = IsDefault(name) ? null : Named(console, name);
            if (profile is null && !IsDefault(name)) return false;
            ProfilesOf(console).Chosen = profile?.Name;
            return true;
        }

        public string ConsoleDefault(string console) =>
            _profiles.TryGetValue(console, out ConsoleProfiles? all) && all.Chosen is { } chosen && Named(console, chosen) is { } profile ? profile.Name : DefaultProfile;

        // The profile a game starts with: its own where it names one that exists, else the console's default.
        public string Resolve(string console, string? gameProfile = null)
        {
            if (gameProfile is not null && IsDefault(gameProfile)) return DefaultProfile;
            if (gameProfile is not null && Named(console, gameProfile) is { } own) return own.Name;
            return ConsoleDefault(console);
        }

        // A player's map in a profile; a player with none of their own plays with player 1's, and an unknown profile is Default.
        public GamepadBindingMap For(string console, int player, string profile)
        {
            if (IsDefault(profile) || Named(console, profile) is not { } named) return For(console, player);
            return player > 1 && named.Maps.TryGetValue(player, out GamepadBindingMap? own) ? own : named.Maps[1];
        }

        public bool HasOwn(string console, int player, string profile) =>
            IsDefault(profile) || Named(console, profile) is not { } named ? HasOwn(console, player) : player > 1 && named.Maps.ContainsKey(player);

        // The player's own map in a profile, begun as a copy of player 1's there the first time it is changed.
        public GamepadBindingMap Own(string console, int player, string profile)
        {
            if (IsDefault(profile) || Named(console, profile) is not { } named) return Own(console, player);
            if (!named.Maps.TryGetValue(player, out GamepadBindingMap? own)) named.Maps[player] = own = Copy(named.Maps[1]);
            return own;
        }

        public void Forget(string console, int player, string profile)
        {
            if (IsDefault(profile) || Named(console, profile) is not { } named) Forget(console, player);
            else if (player > 1) named.Maps.Remove(player);
        }

        // gamepadprofiles.json: by console, the chosen default and each named profile's maps by player; Default stays in gamepadbindings.json.
        internal sealed class ProfileFile
        {
            public string? Default { get; set; }
            public Dictionary<string, Dictionary<int, Dictionary<PadButton, SDL.GamepadButton>>> Profiles { get; set; } = new();
        }

        private static readonly ConfigFile<Dictionary<string, ProfileFile>> ProfilesFile = new("gamepadprofiles.json");

        public bool Save()
        {
            var payload = new Dictionary<string, Dictionary<PadButton, SDL.GamepadButton>>(StringComparer.OrdinalIgnoreCase);
            foreach (var kv in _byConsole) payload[kv.Key] = kv.Value.ButtonToPad;
            foreach (var kv in _byPlayer) payload[kv.Key] = kv.Value.ButtonToPad;
            var profiles = new Dictionary<string, ProfileFile>(StringComparer.OrdinalIgnoreCase);
            foreach (var (console, all) in _profiles)
            {
                if (all.Named.Count == 0) continue;
                profiles[console] = new ProfileFile
                {
                    Default = all.Chosen,
                    Profiles = all.Named.ToDictionary(p => p.Name, p => p.Maps.ToDictionary(m => m.Key, m => m.Value.ButtonToPad)),
                };
            }
            return File.Save(payload) & ProfilesFile.Save(profiles);
        }

        private void LoadProfiles()
        {
            if (ProfilesFile.Load() is not { } loaded) return;
            foreach (var (console, file) in loaded)
            {
                ConsoleProfiles all = ProfilesOf(console);
                foreach (var (name, maps) in file.Profiles ?? new())
                {
                    string trimmed = name.Trim();
                    if (trimmed.Length == 0 || IsDefault(trimmed) || Named(console, trimmed) is not null || !maps.ContainsKey(1)) continue;
                    var profile = new Profile(trimmed);
                    foreach (var (player, buttons) in maps)
                    {
                        if (player < 1 || player > PlayerSlots.MaxPlayers || buttons is null) continue;
                        var map = new GamepadBindingMap();
                        map.Replace(buttons);
                        map.AddDefaultsForNewButtons();
                        profile.Maps[player] = map;
                    }
                    all.Named.Add(profile);
                }
                all.Chosen = file.Default is { } chosen && Named(console, chosen) is { } named ? named.Name : null;
            }
        }

        public static GamepadBindings Load(IEnumerable<string> consoles)
        {
            var bindings = new GamepadBindings(consoles);
            bindings.LoadProfiles();
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
