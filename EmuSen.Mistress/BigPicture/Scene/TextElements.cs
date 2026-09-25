using System;
using System.Globalization;
using System.IO;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // text and datetime as a FontText: what the words are, then how they are set - see EmuSen_BigPicture.md §13.2.
    internal static class TextElements
    {
        internal static Control? Text(SceneBuilder b, ResolvedElement e)
        {
            string? value = e.String("text") ?? Metadata(b, e) ?? SystemData(b, e);
            if (string.IsNullOrEmpty(value)) value = e.String("defaultValue");
            if (string.IsNullOrEmpty(value)) return null;
            return Set(b, e, value);
        }

        internal static Control? DateTime(SceneBuilder b, ResolvedElement e)
        {
            string? field = e.String("metadata");
            System.DateTime? date = b.Data.Game is { } g ? field switch { "releasedate" => g.ReleaseDate, "lastplayed" => g.LastPlayed, _ => null } : null;
            string? value = date is { } d
                ? e.Bool("displayRelative") == true ? SceneUnits.Relative(d, b.Data.Now) : SceneUnits.Format(d, e.String("format") ?? "%Y-%m-%d")
                : e.String("defaultValue");
            if (string.IsNullOrEmpty(value)) return null;
            return Set(b, e, value);
        }

        // A game's field as the text shows it; the words for flags and counts are Mistress's own (§3.6).
        private static string? Metadata(SceneBuilder b, ResolvedElement e)
        {
            if (e.String("metadata") is not { } field) return null;
            SceneSystem system = b.Data.System;
            if (field is "systemName") return system.System.Name;
            if (field is "systemFullname") return system.System.FullName;
            if (field is "sourceSystemName") return system.System.Name;
            if (field is "sourceSystemFullname") return system.System.FullName;
            if (b.Data.Game is not { } g) return null;
            static string YesNo(bool v) => v ? "Yes" : "No";
            string? value = field switch
            {
                "name" => g.Name,
                "description" => g.Description,
                "rating" => g.Rating is { } r ? (r * 5).ToString("0.#", CultureInfo.InvariantCulture) + "/5" : null,
                "developer" => g.Developer,
                "publisher" => g.Publisher,
                "genre" => g.Genre,
                "players" => g.Players,
                "favorite" => YesNo(g.Favorite),
                "completed" => YesNo(g.Completed),
                "kidgame" => YesNo(g.KidGame),
                "broken" => YesNo(g.Broken),
                "playcount" => g.PlayCount.ToString(CultureInfo.InvariantCulture),
                "playtime" => g.PlayTime is { } t && t > TimeSpan.Zero ? SceneUnits.PlayTime(t) : null,
                "altemulator" or "emulator" => g.Emulator,
                "physicalName" => Path.GetFileNameWithoutExtension(g.File),
                "physicalNameExtension" => Path.GetFileName(g.File),
                _ => null,
            };
            if (field == "name" && value is not null && e.Bool("systemNameSuffix") == true && system.System.Kind != ThemeSystemKind.Regular)
                value += " [" + SceneUnits.Cased(system.System.Name, e.String("letterCaseSystemNameSuffix") ?? "uppercase") + "]";
            return value;
        }

        // The system view's own numbers, in Mistress's words.
        private static string? SystemData(SceneBuilder b, ResolvedElement e)
        {
            if (e.String("systemdata") is not { } field) return null;
            SceneSystem s = b.Data.System;
            int games = s.Games.Count(g => !g.Folder), favorites = s.Games.Count(g => g.Favorite);
            return field switch
            {
                "name" => s.System.Name,
                "fullname" => s.System.FullName,
                "gamecount" => $"{games} games available" + (favorites > 0 ? $", {favorites} favorites" : ""),
                "gamecountGames" => $"{games} games",
                "gamecountGamesNoText" => games.ToString(CultureInfo.InvariantCulture),
                "gamecountFavorites" => $"{favorites} favorites",
                "gamecountFavoritesNoText" => favorites.ToString(CultureInfo.InvariantCulture),
                _ => null,
            };
        }

        // The typesetting both elements share: font, size, colour, alignment, case, spacing, background and the box rules.
        private static FontText Set(SceneBuilder b, ResolvedElement e, string value)
        {
            var text = new FontText
            {
                Text = value,
                FontPath = ImageElements.Existing(e.Path("fontPath")),
                FontSize = SceneUnits.Px(e.Float("fontSize") ?? 0.045f, b.H),
                Foreground = new SolidColorBrush(SceneUnits.ToColor(e.Color("color"), Colors.Black)),
                TextAlignment = SceneUnits.Horizontal(e.String("horizontalAlignment")),
                TextVerticalAlignment = SceneUnits.Vertical(e.String("verticalAlignment")),
                LineSpacing = e.Float("lineSpacing") ?? 1.5f,
                LetterCase = SceneUnits.Case(e.String("letterCase")),
                Ellipsis = "...",
            };

            Size size = SceneUnits.ToSize(e.Pair("size"));
            text.Wrap = size.Width > 0;
            if (e.Bool("container") == true)
            {
                text.Ellipsis = null;
                if (e.String("containerType") == "horizontal") text.Wrap = false;
                else text.TextVerticalAlignment = Avalonia.Layout.VerticalAlignment.Top;
            }

            if (e.Color("backgroundColor") is { A: > 0 } background)
            {
                text.Background = new SolidColorBrush(SceneUnits.ToColor(background));
                text.BackgroundCornerRadius = SceneUnits.Px(e.Float("backgroundCornerRadius") ?? 0, b.W);
                Size margins = SceneUnits.ToSize(e.Pair("backgroundMargins"));
                text.Padding = new Thickness(SceneUnits.Px(margins.Width, b.W), 0, SceneUnits.Px(margins.Height, b.W), 0);
                text.Margin = new Thickness(-SceneUnits.Px(margins.Width, b.W), 0, -SceneUnits.Px(margins.Height, b.W), 0);
            }

            NormalizedCanvas.SetSize(text, size);
            return text;
        }
    }
}
