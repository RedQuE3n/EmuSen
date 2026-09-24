using System;
using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Mercury.Memory;

namespace EmuSen.Cores.Nintendo.Mercury
{
    // Which console a cartridge runs on: the one its header asks for, or the one the player chose - see Mercury_Model.md §1.
    public enum GbModel
    {
        Auto,
        GameBoy,
        GameBoyColor,
    }

    // The Model setting, which both engines honour at the next load - see Mercury_Model.md §1 and EmuSen_Settings_Reference.md §4.47.
    public sealed partial class MercuryCore
    {
        public const string ModelKey = "Model";

        public const string ModelAuto = "Auto";
        public const string ModelGameBoy = "Game Boy";
        public const string ModelGameBoyColor = "Game Boy Color";

        public static readonly IReadOnlyList<global::EmuSen.Cores.CoreSetting> ModelSettings = new global::EmuSen.Cores.CoreSetting[]
        {
            new(ModelKey, "Model",
                "Which console a game runs on. Auto follows the cartridge: a Game Boy Color game on a Game Boy Color, the rest on a Game Boy. Game Boy runs every game on a Game Boy: a colour game in its own monochrome mode, and a game made only for the Color shows whatever that game shows a Game Boy. Game Boy Color runs every game on a Game Boy Color: a Game Boy game in the colours the Color's start-up picks for it from its title. Takes effect when a game is next loaded; a save state resumes on the console it was made on.",
                global::EmuSen.Cores.CoreSettingKind.Choice, ModelAuto, Choices: new[] { ModelAuto, ModelGameBoy, ModelGameBoyColor }),
        };

        private GbModel _model;

        // The console the next load builds; changed before the first frame, the game is loaded again at once, as Mars's Expansion Pak is.
        public GbModel Model
        {
            get => _model;
            set
            {
                if (value == _model) return;
                _model = value;
                if (Bus is not null && TotalFrames == 0 && Cart is { RomPath.Length: > 0 } cart) LoadRom(cart.RomPath);
            }
        }

        // True for a Game Boy Color: the chosen console, or under Auto the one the header asks for.
        public static bool ConsoleFor(GbModel model, CgbSupport header) => model switch
        {
            GbModel.GameBoy => false,
            GbModel.GameBoyColor => true,
            _ => header != CgbSupport.None,
        };

        public static GbModel ParseModel(string text) => text switch
        {
            ModelAuto => GbModel.Auto,
            ModelGameBoy => GbModel.GameBoy,
            ModelGameBoyColor => GbModel.GameBoyColor,
            _ => throw new ArgumentException($"{text} is not a Game Boy console."),
        };

        public static string ModelName(GbModel model) => model switch
        {
            GbModel.GameBoy => ModelGameBoy,
            GbModel.GameBoyColor => ModelGameBoyColor,
            _ => ModelAuto,
        };

        IReadOnlyList<global::EmuSen.Cores.CoreSetting> global::EmuSen.Cores.ICoreSettings.Settings => ModelSettings;

        public string Get(string key) => key == ModelKey ? ModelName(Model) : throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));

        public void Set(string key, string value)
        {
            if (key != ModelKey) throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));
            Model = ParseModel(value);
        }
    }
}
