using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // Which badges a game shows, in THEMES.md's slots, and each as the theme's file or Mistress's own drawing - see EmuSen_BigPicture.md §29.
    public static class SceneBadges
    {
        public static readonly string[] BadgeSlots = ["collection", "folder", "favorite", "completed", "kidgame", "broken", "controller", "altemulator", "manual"];

        public static bool Has(SceneGame game, string slot) => slot switch
        {
            "collection" => game.InCollection, "folder" => game.Folder, "favorite" => game.Favorite, "completed" => game.Completed,
            "kidgame" => game.KidGame, "broken" => game.Broken, "altemulator" => game.AltEmulator, "controller" => game.Controller is not null,
            "manual" => game.Manual, _ => false,
        };

        // THEMES.md's "all": the slots a theme names first, in its order, then the rest in the documented order.
        public static IReadOnlyList<string> Slots(IReadOnlyList<string> named) =>
            named.Contains("all") ? named.Where(s => s != "all").Concat(BadgeSlots.Where(s => !named.Contains(s))).Distinct().ToList() : named;

        public static BadgeKind KindOf(string slot) => slot switch
        {
            "collection" => BadgeKind.Collection, "folder" => BadgeKind.Folder, "completed" => BadgeKind.Completed, "kidgame" => BadgeKind.KidGame,
            "broken" => BadgeKind.Broken, "controller" => BadgeKind.Controller, "altemulator" => BadgeKind.AltEmulator, "manual" => BadgeKind.Manual,
            _ => BadgeKind.Favorite,
        };

        // Mistress draws the controllers of its own consoles, a generic pad for any other pad, and the unknown shape for the rest (§29).
        public static ControllerShape ShapeOf(string? controller) => controller switch
        {
            "gamepad_nintendo_nes" => ControllerShape.Nes,
            "gamepad_nintendo_snes" => ControllerShape.Snes,
            "gamepad_nintendo_64" => ControllerShape.Nintendo64,
            { } c when c.StartsWith("gamepad_", StringComparison.Ordinal) => ControllerShape.Gamepad,
            _ => ControllerShape.Unknown,
        };

        // Each shown slot as the theme's file when it names one that exists, else Mistress's own drawing, as ES-DE falls back to its built-in badges (§29).
        public static IReadOnlyList<BadgeEntry> Entries(SceneGame game, ResolvedElement e)
        {
            IReadOnlyDictionary<string, ThemePath> icons = e.Keyed("customBadgeIcon"), controllers = e.Keyed("customControllerIcon");
            string? link = e.Path("customFolderLinkIcon") is { Exists: true } l ? l.Absolute : null;
            return Slots(e.List("slots")).Where(slot => Has(game, slot)).Select(slot => new BadgeEntry(KindOf(slot))
            {
                IconPath = icons.GetValueOrDefault(slot) is { Exists: true } icon ? icon.Absolute : null,
                Controller = slot == "controller" ? ShapeOf(game.Controller) : null,
                ControllerIconPath = slot == "controller" && game.Controller is { } c && controllers.GetValueOrDefault(c) is { Exists: true } own ? own.Absolute : null,
                Linked = slot == "folder" && game.FolderLink,
                LinkIconPath = link,
            }).ToList();
        }
    }
}
