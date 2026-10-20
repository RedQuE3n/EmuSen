using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Crystal.Systems;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using Screen = EmuSen.Serenity.Shaders.ConsoleScreens.Screen;

namespace EmuSen.WiseMan.Mistress
{
    // Every console EmuSen supports has an accurate display filter made for it, and every console shown on a CRT is offered the CRT - see EmuSen_CRT.md §31.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConsoleScreenRuleTests
    {
        public ConsoleScreenRuleTests() => RuntimeCatalog.Forget();

        private static readonly Regex Dated = new(@"^\d{4}-\d{2}-\d{2}: \S");

        // The console a system's games run under in the frontend: its shelf's core's, or the system's own where no core of this build runs it.
        private static string RunsUnder(SystemEntry system) =>
            RuntimeCatalog.Shelves.FirstOrDefault(s => s.System == system.Id)?.Core.Console ?? system.Console;

        // Every way a row can break the rule, in words; empty when the rows keep it.
        public static List<string> Breaches(IReadOnlyList<Screen> screens, IReadOnlyList<SystemEntry> systems, Func<SystemEntry, string> runsUnder)
        {
            var breaches = new List<string>();
            foreach (SystemEntry system in systems)
            {
                Screen[] rows = screens.Where(s => s.Console == system.Console).ToArray();
                if (rows.Length != 1) { breaches.Add($"{system.Id} ({system.Console}) has {rows.Length} screen rows, where it must have one"); continue; }
                Screen screen = rows[0];
                if (screen.Gap is { } gap && !Dated.IsMatch(gap)) breaches.Add($"{system.Console}'s gap is not a dated note: \"{gap}\"");
                bool own = screen.Kind == ConsoleScreens.Kind.Lcd ? screens.Count(s => s.Profile == screen.Profile) == 1
                    : screen.Profile == screen.Console || screen.HostsPicture && system.AttachedTo == screen.Profile;
                if (!own && screen.Gap is null) breaches.Add($"{system.Console} borrows {screen.Profile}'s profile with no dated gap");
                if (screen.HostsPicture && system.AttachedTo is null) breaches.Add($"{system.Console} takes its host's picture, and is attached to none");
                string under = runsUnder(system), host = RuntimeCatalog.HostOf(under);
                IReadOnlyList<string> offered = ShaderCatalog.BuiltIns(under).Select(e => e.Stored).ToArray();
                if (screen.Kind == ConsoleScreens.Kind.Television)
                {
                    if (!CrtFilter.ConsoleTiming.ContainsKey(screen.Profile) || screen.Profile == ScreenFilter.AnyConsole) breaches.Add($"{system.Console}'s profile {screen.Profile} is no row of the CRT's");
                    if (!offered.Contains(CrtFilter.Name) || !offered.Contains(CrtFilters.Lottes.Name)) breaches.Add($"{system.Console}, a television, is not offered both tubes under {under}");
                    if (!GraphicsSettingsWindow.HasOverscan(under)) breaches.Add($"{system.Console}, a television, has no overscan row under {under}");
                    if (CrtFilter.Filter.ConsoleDefaults is not { } defaults || !defaults.TryGetValue(host, out var drawn) || !CrtFilter.ConsoleTiming.TryGetValue(screen.Profile, out var profile) || !ReferenceEquals(drawn, profile))
                        breaches.Add($"{system.Console} is drawn on {host}'s row, not on its profile {screen.Profile}'s");
                }
                else
                {
                    ScreenFilterChoice choice = ScreenFilters.Find(screen.Profile);
                    if (choice.Name != screen.Profile || choice.Filter is null) breaches.Add($"{system.Console}'s LCD profile {screen.Profile} is no filter");
                    else if (!offered.Contains(screen.Profile)) breaches.Add($"{system.Console}'s LCD filter {screen.Profile} is not offered under {under}");
                    if (offered.Contains(CrtFilter.Name) || GraphicsSettingsWindow.HasOverscan(under)) breaches.Add($"{system.Console}, an LCD, is offered a tube or an overscan under {under}");
                }
            }
            foreach (Screen screen in screens.Where(s => systems.All(e => e.Console != s.Console))) breaches.Add($"{screen.Console} has a screen row and is no system of the table");
            foreach (string row in CrtFilter.ConsoleTiming.Keys.Where(k => k != ScreenFilter.AnyConsole && screens.All(s => s.Profile != k))) breaches.Add($"the CRT's row {row} is no console's profile");
            return breaches;
        }

        private static IReadOnlyList<SystemEntry> Systems => SystemPacks.All.Select(p => p.Entry).ToArray();

        [Fact]
        public void Every_system_has_an_accurate_screen_of_its_own_and_every_television_is_offered_the_crt()
        {
            List<string> breaches = Breaches(ConsoleScreens.All, Systems, RunsUnder);
            Assert.True(breaches.Count == 0, "the rule of EmuSen_CRT.md §31 is broken:\n" + string.Join("\n", breaches));
        }

        // The tubes' list is the screens' televisions with rows of their own; a gap is never silent.
        [Fact]
        public void The_tubes_are_offered_for_the_televisions_with_rows_of_their_own_and_the_gaps_are_the_ones_on_record()
        {
            Assert.Equal(new[] { "NES", "SNES", "Genesis", "N64", "PlayStation", "Master System", "SG-1000" }, CrtFilter.TelevisionConsoles);
            Assert.Equal(CrtFilter.TelevisionConsoles, CrtFilter.Filter.Consoles);
            Assert.Equal(CrtFilter.TelevisionConsoles, CrtFilters.Lottes.Consoles);
            Assert.Equal(new[] { "32X", "GB", "GBC" }, ConsoleScreens.All.Where(s => s.Gap is not null).Select(s => s.Console));
        }

        // The consoles that had the tubes keep exactly the filters they had; the two new ones are offered the same list.
        [Fact]
        public void The_consoles_that_had_filters_keep_them_and_the_master_system_and_the_sg_1000_get_a_television_s()
        {
            string[] television = { ScreenFilters.None, CrtFilter.Name, CrtFilters.Lottes.Name, "Scanlines", "Simple CRT" };
            foreach (string console in new[] { "NES", "SNES", "Genesis", "Sega CD", "N64", "PlayStation", "Master System", "SG-1000" })
                Assert.Equal(television, ShaderCatalog.BuiltIns(console).Select(e => e.Stored));
            Assert.Equal(new[] { ScreenFilters.None, "Game Boy LCD", "Game Boy Pocket LCD", "Game Boy Light LCD", "Game Boy Color LCD", "Scanlines", "Simple CRT" }, ShaderCatalog.BuiltIns("GB").Select(e => e.Stored));
            foreach (string console in new[] { "Master System", "SG-1000" }) Assert.True(GraphicsSettingsWindow.HasOverscan(console), console);
        }

        // Each way the rule can be broken is caught: a system with no row, a borrowed profile, a television not offered the tube, an LCD that is no filter, an undated gap.
        [Fact]
        public void The_rule_catches_each_way_of_breaking_it()
        {
            IReadOnlyList<Screen> all = ConsoleScreens.All;
            IReadOnlyList<Screen> With(string console, Func<Screen, Screen> change) => all.Select(s => s.Console == console ? change(s) : s).ToArray();
            var cases = new (string What, IReadOnlyList<Screen> Screens, IReadOnlyList<SystemEntry> Systems, string Expected)[]
            {
                ("a system with no row", all.Where(s => s.Console != "SG-1000").ToArray(), Systems, "sg1000 (SG-1000) has 0 screen rows"),
                ("a new system nobody decided", all, Systems.Append(Systems[0] with { Id = "gg", Console = "Game Gear" }).ToArray(), "gg (Game Gear) has 0 screen rows"),
                ("a profile borrowed", With("Master System", s => s with { Profile = "Genesis" }), Systems, "Master System borrows Genesis's profile with no dated gap"),
                ("an attachment's own output taken for its host's", With("32X", s => s with { Gap = null }), Systems, "32X borrows Genesis's profile with no dated gap"),
                ("a gap with no date", With("32X", s => s with { Gap = "no encoder found" }), Systems, "32X's gap is not a dated note"),
                ("a television drawn as an LCD", With("SG-1000", s => s with { Kind = ConsoleScreens.Kind.Lcd }), Systems, "SG-1000's LCD profile SG-1000 is no filter"),
                ("a panel borrowed", With("GBC", s => s with { Profile = "Game Boy LCD", Gap = null }), Systems, "GBC borrows Game Boy LCD's profile with no dated gap"),
                ("a handheld given a tube's row", With("GB", s => s with { Kind = ConsoleScreens.Kind.Television }), Systems, "GB's profile Game Boy LCD is no row of the CRT's"),
                ("a host taken by a console attached to none", With("Master System", s => s with { Profile = "Genesis", HostsPicture = true }), Systems, "Master System takes its host's picture, and is attached to none"),
            };
            foreach (var (what, screens, systems, expected) in cases)
                Assert.True(Breaches(screens, systems, RunsUnder).Any(b => b.StartsWith(expected, StringComparison.Ordinal)), $"{what}: not caught; got\n{string.Join("\n", Breaches(screens, systems, RunsUnder))}");
        }
    }
}
