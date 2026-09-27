using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.Mistress.BigPicture;

namespace EmuSen.Mistress.Views
{
    // The Theme Settings sheet's Interface tab: ES-DE's UI settings and system status settings that are not the theme's own - see EmuSen_Settings_Reference.md §4.66.
    public sealed class InterfaceSettingsPane
    {
        public static readonly (string Value, string Text)[] QuickSelectChoices =
        [
            (BigPictureInterface.QuickSelectLeftRightOrShoulders, "Left/right or shoulders"),
            (BigPictureInterface.QuickSelectLeftRightOrTriggers, "Left/right or triggers"),
            (BigPictureInterface.QuickSelectShoulders, "Shoulders"),
            (BigPictureInterface.QuickSelectTriggers, "Triggers"),
            (BigPictureInterface.QuickSelectLeftRight, "Left/right"),
            (BigPictureInterface.QuickSelectDisabled, "Disabled"),
        ];

        public static readonly (string Value, string Text)[] ViewChoices = [(BigPictureInterface.ViewSystem, "System view"), (BigPictureInterface.ViewGamelist, "Gamelist view")];

        public static readonly (string Value, string Text)[] LaunchScreenChoices =
        [
            (BigPictureInterface.LaunchNormal, "Normal"),
            (BigPictureInterface.LaunchBrief, "Brief"),
            (BigPictureInterface.LaunchLong, "Long"),
            (BigPictureInterface.LaunchPopup, "Popup"),
            (BigPictureInterface.LaunchDisabled, "Disabled"),
        ];

        private readonly AppSettings _settings;
        private readonly Action _changed;

        public InterfaceSettingsPane(AppSettings settings, Action changed)
        {
            _settings = settings;
            _changed = changed;
        }

        private BigPictureInterface S => _settings.BigPictureInterface;

        // The startup choices: Default, then each system in the order the carousel shows them.
        public IReadOnlyList<(string Value, string Text)> StartupChoices() =>
            SystemsOrder.Sort(EmuSen.Cores.CoreCatalog.ShelvesInReleaseOrder, S.SystemsSorting, s => s.EsdeSystem, s => s.EsdeFullName)
                .Select(s => (s.EsdeSystem, s.EsdeFullName)).Prepend(("", "Default (the first system)")).ToList();

        public Control[] Rows() =>
        [
            Ui.Header("Navigation"),
            Choice("QuickSystemSelect", "Quick System Select", "The buttons that change the system in a game list. The two \"or\" choices give left and right to a list, and the shoulders or triggers to a grid, whose left and right move through the games.",
                QuickSelectChoices, S.QuickSystemSelect, v => S.QuickSystemSelect = v),
            Choice("StartupSystem", "System on Startup", "The system shown when big picture first opens in a session.", StartupChoices(), S.StartupSystem, v => S.StartupSystem = v),
            Choice("StartupView", "Startup View", "Whether big picture opens on the systems or inside that system's game list.", ViewChoices, S.StartupView, v => S.StartupView = v),
            Choice("SystemsSorting", "Systems Sorting", "The order of the systems in the carousel. Collections follow the systems whatever the order.", SystemsOrder.Choices, S.SystemsSorting, v => S.SystemsSorting = v),
            new FieldRow
            {
                Label = "Quick Scrolling Overlay",
                Hint = "While a game list is held down, the first two letters of the game passing, over a slight shade, or a star among favourites kept on top.",
                Content = Switch("ListScrollOverlay", "Enable quick scrolling overlay", S.ListScrollOverlay, on => S.ListScrollOverlay = on),
            },
            Ui.Header("On Screen"),
            Choice("LaunchScreenDuration", "Launch Screen Duration", "How long the game's name and marquee (else its cover) show before it starts: Normal 3 s, Brief 1.7 s, Long 4.5 s, as ES-DE measures; Popup, a notice at the top for 1.7 s; Disabled, at once.",
                LaunchScreenChoices, S.LaunchScreenDuration, v => S.LaunchScreenDuration = v),
            new FieldRow
            {
                Label = "Clock",
                Hint = "The time, where the theme places its clock. Off by default, as in ES-DE.",
                Content = Switch("DisplayClock", "Display clock", S.DisplayClock, on => S.DisplayClock = on),
            },
            new FieldRow
            {
                Label = "On-Screen Help",
                Hint = "The bar that names what each button does.",
                Content = Switch("DisplayHelp", "Display on-screen help", S.DisplayHelp, on => S.DisplayHelp = on),
            },
            new FieldRow
            {
                Label = "System Status",
                Hint = "The indicators the theme shows. Each shows only when this device has what it reports.",
                Content = Ui.Stack(4,
                    Switch("StatusBluetooth", "Display Bluetooth status indicator", S.StatusBluetooth, on => S.StatusBluetooth = on),
                    Switch("StatusWifi", "Display Wi-Fi status indicator", S.StatusWifi, on => S.StatusWifi = on),
                    Switch("StatusBattery", "Display battery status indicator", S.StatusBattery, on => S.StatusBattery = on),
                    Switch("StatusBatteryPercentage", "Display battery charge percentage", S.StatusBatteryPercentage, on => S.StatusBatteryPercentage = on)),
            },
        ];

        private LunaSwitch Switch(string name, string label, bool on, Action<bool> store)
        {
            var box = new LunaSwitch { Name = name, Label = label, IsChecked = on };
            box.IsCheckedChanged += (_, _) =>
            {
                store(box.IsChecked == true);
                _changed();
            };
            return box;
        }

        private FieldRow Choice(string name, string label, string hint, IReadOnlyList<(string Value, string Text)> entries, string current, Action<string> store)
        {
            var dropdown = new Dropdown { Name = name, HorizontalAlignment = HorizontalAlignment.Stretch };
            string[] texts = entries.Select(e => e.Text).ToArray();
            dropdown.Fill(texts, entries.FirstOrDefault(e => e.Value == current, entries[0]).Text);
            dropdown.Chose += chosen =>
            {
                int i = Array.IndexOf(texts, chosen as string);
                if (i < 0) return;
                store(entries[i].Value);
                _changed();
            };
            return new FieldRow { Label = label, Hint = hint, Content = dropdown };
        }
    }
}
