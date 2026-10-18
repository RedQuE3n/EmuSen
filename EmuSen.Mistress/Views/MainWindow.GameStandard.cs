using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Mistress.Views
{
    // The television standard a game's picture is drawn to: the player's for this game, else the console's filter setting, else the core's answer - see EmuSen_CRT.md §26.
    public partial class MainWindow
    {
        public const string StandardRow = "Television Standard", StandardFollows = "The game's", StandardNtsc = "NTSC", StandardPal = "PAL";

        private static readonly (string Text, string? Value)[] StandardChoices = { (StandardFollows, null), (StandardNtsc, TelevisionStandards.Ntsc), (StandardPal, TelevisionStandards.Pal) };

        // Offered for a game of a console that drove a television, which every filter of a tube is offered for.
        private static bool StandardOffered(string path) =>
            EmuSen.Mistress.Library.RuntimeCatalog.ConsoleForRom(path) is { } console && CrtFilter.TelevisionConsoles.Contains(EmuSen.Mistress.Library.RuntimeCatalog.HostOf(console));

        // The values drawn with the running game's standard laid over them: its own choice wins over the console's, the core's answer only fills in.
        private Dictionary<string, float> RunningStandard(Dictionary<string, float> values) =>
            _currentRomPath is string rom && _records.StandardChoice(rom) is { } own
                ? TelevisionStandards.Apply(values, own, chosen: true)
                : TelevisionStandards.Apply(values, _session?.TelevisionStandard, chosen: false);

        // A choice row in the game's options, kept when the menu applies; a running game is drawn again with it.
        private GameOption StandardOption(string path)
        {
            var choice = new Dropdown { Name = "GameTelevisionStandard", HorizontalAlignment = HorizontalAlignment.Stretch };
            string? stored = _records.StandardChoice(path);
            choice.Fill(StandardChoices.Select(c => c.Text).ToArray(), StandardChoices.FirstOrDefault(c => c.Value == stored, StandardChoices[0]).Text);
            return new GameOption(StandardRow, () => { })
            {
                Row = () => new FieldRow { Label = StandardRow, Content = choice },
                MenuRow = () => MenuRows.Apply(choice, StandardRow),
                Apply = () =>
                {
                    string? chosen = StandardChoices.FirstOrDefault(c => c.Text == choice.SelectedItem as string, StandardChoices[0]).Value;
                    if (chosen == stored) return;
                    _records.SetStandardChoice(path, chosen, DateTime.Now);
                    if (IsRunning(path)) ApplyScreenFilter(_activeConsole);
                },
            };
        }
    }
}
