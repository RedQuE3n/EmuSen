using System;
using System.Collections.Generic;
using System.Linq;
using SDL3;
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

        // Players 2 on, by console then player; a player missing here plays with player 1's map - see EmuSen_Input.md §8.4.
        private readonly Dictionary<string, Dictionary<int, GamepadBindingMap>> _byPlayer = new(StringComparer.OrdinalIgnoreCase);

        private Dictionary<int, GamepadBindingMap> PlayersOf(string console)
        {
            if (!_byPlayer.TryGetValue(console, out Dictionary<int, GamepadBindingMap>? players)) _byPlayer[console] = players = new();
            return players;
        }

        public GamepadBindingMap For(string console, int player) =>
            player > 1 && _byPlayer.TryGetValue(console, out var players) && players.TryGetValue(player, out GamepadBindingMap? own) ? own : For(console);

        public bool HasOwn(string console, int player) => player > 1 && _byPlayer.TryGetValue(console, out var players) && players.ContainsKey(player);

        // The player's own map, begun as a copy of player 1's the first time it is changed.
        public GamepadBindingMap Own(string console, int player)
        {
            if (player <= 1) return For(console);
            Dictionary<int, GamepadBindingMap> players = PlayersOf(console);
            if (!players.TryGetValue(player, out GamepadBindingMap? own)) players[player] = own = Copy(For(console));
            return own;
        }

        // Back to player 1's map.
        public void Forget(string console, int player)
        {
            if (_byPlayer.TryGetValue(console, out var players)) players.Remove(player);
        }

        // Default's maps back to the defaults; the named profiles are the player's and stay.
        public void ResetToDefaults()
        {
            foreach (GamepadBindingMap map in _byConsole.Values) map.ResetToDefaults();
            _byPlayer.Clear();
        }

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
            public Dictionary<string, string> Games { get; } = new(StringComparer.Ordinal);
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
            foreach (string game in all.Games.Where(g => string.Equals(g.Value, profile.Name, StringComparison.OrdinalIgnoreCase)).Select(g => g.Key).ToList()) all.Games[game] = newName;
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
            foreach (string game in all.Games.Where(g => string.Equals(g.Value, profile.Name, StringComparison.OrdinalIgnoreCase)).Select(g => g.Key).ToList()) all.Games.Remove(game);
            return true;
        }

        // A game's own profile by its key (the file's stem, as its own settings are kept), null where it follows the console's default; store 1.2.
        public string? GameProfile(string console, string game) =>
            _profiles.TryGetValue(console, out ConsoleProfiles? all) && all.Games.TryGetValue(game, out string? name) ? name : null;

        // Null forgets the game's own choice; a profile the console does not have is refused.
        public bool SetGameProfile(string console, string game, string? profile)
        {
            if (game.Length == 0) return false;
            if (profile is null)
            {
                if (_profiles.TryGetValue(console, out ConsoleProfiles? had)) had.Games.Remove(game);
                return true;
            }
            string? name = IsDefault(profile) ? DefaultProfile : Named(console, profile)?.Name;
            if (name is null) return false;
            ProfilesOf(console).Games[game] = name;
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

        // gamepadbindings.json as the platform library hands it over: each console's map, then each player's who has one.
        internal sealed class PadFile
        {
            public Dictionary<string, Dictionary<PadButton, SDL.GamepadButton>> Consoles { get; set; } = new();
            public Dictionary<string, Dictionary<int, Dictionary<PadButton, SDL.GamepadButton>>> Players { get; set; } = new();
        }

        // gamepadprofiles.json likewise: by console, the chosen default and each named profile's maps by player; Default stays in gamepadbindings.json.
        internal sealed class ProfileFile
        {
            public string? Default { get; set; }
            public Dictionary<string, Dictionary<int, Dictionary<PadButton, SDL.GamepadButton>>> Profiles { get; set; } = new();
            public Dictionary<string, string>? Games { get; set; }
        }

        private static GamepadBindingMap MapOf(Dictionary<PadButton, SDL.GamepadButton> buttons)
        {
            var map = new GamepadBindingMap();
            map.Replace(buttons);
            return map;
        }

        // Both files are Endymion's in the platform library, their one reader and writer - see EmuSen_Input.md §5.2.
        public bool Save()
        {
            var pads = new PadFile
            {
                Consoles = _byConsole.ToDictionary(kv => kv.Key, kv => kv.Value.ButtonToPad),
                Players = _byPlayer.ToDictionary(kv => kv.Key, kv => kv.Value.ToDictionary(p => p.Key, p => p.Value.ButtonToPad)),
            };
            var profiles = _profiles.ToDictionary(kv => kv.Key, kv => new ProfileFile
            {
                Default = kv.Value.Chosen,
                Profiles = kv.Value.Named.ToDictionary(p => p.Name, p => p.Maps.ToDictionary(m => m.Key, m => m.Value.ButtonToPad)),
                Games = kv.Value.Games.Count > 0 ? new Dictionary<string, string>(kv.Value.Games) : null,
            });
            return BindingFiles.Save(BindingFile.Gamepad, pads) & BindingFiles.Save(BindingFile.Profiles, profiles);
        }

        public static GamepadBindings Load(IEnumerable<string> consoles)
        {
            var bindings = new GamepadBindings(consoles);
            PadFile pads = BindingFiles.Load<PadFile>(BindingFile.Gamepad, bindings._byConsole.Keys);
            foreach (var (console, map) in pads.Consoles) bindings.For(console).Replace(map);
            foreach (var (console, players) in pads.Players)
                foreach (var (player, map) in players) bindings.PlayersOf(console)[player] = MapOf(map);
            foreach (var (console, file) in BindingFiles.Load<Dictionary<string, ProfileFile>>(BindingFile.Profiles, []))
            {
                ConsoleProfiles all = bindings.ProfilesOf(console);
                foreach (var (name, maps) in file.Profiles)
                {
                    var profile = new Profile(name);
                    foreach (var (player, map) in maps) profile.Maps[player] = MapOf(map);
                    all.Named.Add(profile);
                }
                all.Chosen = file.Default;
                foreach (var (game, name) in file.Games ?? new()) all.Games[game] = name;
            }
            return bindings;
        }
    }
}
