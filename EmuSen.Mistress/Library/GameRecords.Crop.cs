using System.Globalization;
using Microsoft.Data.Sqlite;

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
        // Null for a game with no crop of its own, which is drawn with its console's.
        public GameCrop? Crop(string path)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = "SELECT crop_left, crop_right, crop_top, crop_bottom FROM game WHERE path = $path AND crop_left IS NOT NULL";
            command.Parameters.AddWithValue("$path", path);
            using SqliteDataReader row = command.ExecuteReader();
            return row.Read() ? new GameCrop(row.GetDouble(0), row.GetDouble(1), row.GetDouble(2), row.GetDouble(3)) : null;
        }

        public void SetCrop(string path, GameCrop crop)
        {
            using SqliteCommand command = _db.CreateCommand();
            command.CommandText = """
                INSERT INTO game (path, crop_left, crop_right, crop_top, crop_bottom) VALUES ($path, $left, $right, $top, $bottom)
                ON CONFLICT(path) DO UPDATE SET crop_left = excluded.crop_left, crop_right = excluded.crop_right, crop_top = excluded.crop_top, crop_bottom = excluded.crop_bottom
                """;
            command.Parameters.AddWithValue("$path", path);
            command.Parameters.AddWithValue("$left", crop.Left);
            command.Parameters.AddWithValue("$right", crop.Right);
            command.Parameters.AddWithValue("$top", crop.Top);
            command.Parameters.AddWithValue("$bottom", crop.Bottom);
            command.ExecuteNonQuery();
        }

        public void ClearCrop(string path) => Write(path, "UPDATE game SET crop_left = NULL, crop_right = NULL, crop_top = NULL, crop_bottom = NULL WHERE path = $path");
    }
}
