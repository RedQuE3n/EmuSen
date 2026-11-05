using System;
using System.Collections.Generic;
using EmuSen.Galaxia.Library;
using System.Text.Json;

namespace EmuSen.Mistress.Library
{
    // One game's own crop, a percentage of its picture hidden at each edge, which replaces its console's while it runs - see EmuSen_Settings_Reference.md §4.100.
    public readonly record struct GameCrop(double Left, double Right, double Top, double Bottom)
    {
        public EmuSen.Serenity.PictureCrop Picture => EmuSen.Serenity.PictureCrop.Percent(Left, Top, Right, Bottom);

        // A crop as the four percentages the boxes show, to one decimal place.
        public static GameCrop From(EmuSen.Serenity.PictureCrop crop) => new(Shown(crop.Left), Shown(crop.Right), Shown(crop.Top), Shown(crop.Bottom));

        private static double Shown(double share) => System.Math.Round(share * 100, 1);

        public GameCrop With(int edge, double percent) => edge switch
        {
            0 => this with { Left = percent },
            1 => this with { Right = percent },
            2 => this with { Top = percent },
            _ => this with { Bottom = percent },
        };

        public double this[int edge] => edge switch { 0 => Left, 1 => Right, 2 => Top, _ => Bottom };
    }

    public sealed partial class GameRecords
    {
        // A packaged game's crop is its package's settings.json's, this file's row read until the package holds one; a package's folder reads its game's - see EmuSen_GamePackages.md §18.2.
        public Func<string, (string Game, string? Package)> OpenedAs { get; set; } = path => EmuSen.Galaxia.Media.GamePackages.OpenedAs(path);

        // The system whose plain settings a package's first settings.json copies in.
        public Func<string, string?> SystemOf { get; set; } = path => RuntimeCatalog.Route(path).System;

        // Null for a game with no crop of its own, which is drawn with its console's.
        public GameCrop? Crop(string path)
        {
            (string game, string? root) = OpenedAs(path);
            if (root is not null && GameSettings.Read(null, game, root).TryGetValue(GameSettings.CropKey, out string? kept))
                return GameSettings.ParseCrop(kept) is { } c ? new GameCrop(c.Left, c.Right, c.Top, c.Bottom) : null;
            return RowCrop(root is null ? path : game);
        }

        private GameCrop? RowCrop(string path)
        {
            using JsonDocument d = Get("crop", w => w.WriteString("path", path));
            JsonElement c = d.RootElement;
            return c.ValueKind == JsonValueKind.Null ? null : new GameCrop(Real(c.GetProperty("left")), Real(c.GetProperty("right")), Real(c.GetProperty("top")), Real(c.GetProperty("bottom")));
        }

        private void SetRowCrop(string path, GameCrop crop) =>
            Change("set_crop", w => { w.WriteString("path", path); Real(w, "left", crop.Left); Real(w, "right", crop.Right); Real(w, "top", crop.Top); Real(w, "bottom", crop.Bottom); });

        private void ClearRowCrop(string path) => Change("clear_crop", w => w.WriteString("path", path));

        public void SetCrop(string path, GameCrop crop)
        {
            if (OpenedAs(path) is (string game, string root)) { SetPackagedCrop(game, root, GameSettings.CropText(crop.Left, crop.Right, crop.Top, crop.Bottom)); return; }
            SetRowCrop(path, crop);
        }

        private void SetPackagedCrop(string path, string root, string text)
        {
            string? system = SystemOf(path);
            Dictionary<string, string> values = GameSettings.Read(system, path, root);
            values[GameSettings.CropKey] = text;
            GameSettings.Write(system, path, root, values);
        }

        public void ClearCrop(string path)
        {
            if (OpenedAs(path) is (string game, string root)) SetPackagedCrop(game, root, "");
            else ClearRowCrop(path);
        }
    }
}
