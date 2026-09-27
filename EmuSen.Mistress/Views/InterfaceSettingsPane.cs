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

        public static readonly (string Value, string Text)[] ScreensaverChoices =
        [
            (BigPictureInterface.SaverDim, "Dim"),
            (BigPictureInterface.SaverBlack, "Black"),
            (BigPictureInterface.SaverSlideshow, "Slideshow"),
            (BigPictureInterface.SaverVideo, "Video"),
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
            // ES-DE's "Screensaver settings" and "Slideshow screensaver settings", under UI settings as there - see EmuSen_Settings_Reference.md §4.75.
            Ui.Header("Screensaver"),
            new FieldRow
            {
                Label = "Start Screensaver After",
                Hint = "Minutes with no button or key pressed in the library before the screensaver starts; 0 is never. ES-DE's default is 5.",
                Content = Steps("ScreensaverTimer", 0, 30, 1, S.ScreensaverTimer / 60000, m => m == 0 ? "Never" : $"{m} min", m => S.ScreensaverTimer = m * 60000),
            },
            Choice("ScreensaverType", "Screensaver Type", "Dim greys and darkens the view, Black blanks it, Slideshow shows the library's pictures. Video shows Dim until videos are supported.",
                ScreensaverChoices, S.ScreensaverType, v => S.ScreensaverType = v),
            new FieldRow
            {
                Label = "Screensaver Controls",
                Hint = "X in the system view starts it; in a slideshow left and right show another game, A starts the game shown and Y goes to it. Off, any button only wakes the screen.",
                Content = Switch("ScreensaverControls", "Enable screensaver controls", S.ScreensaverControls, on => S.ScreensaverControls = on),
            },
            new FieldRow
            {
                Label = "In Game Mode",
                Hint = "Whether it starts in a Steam Game Mode session, where Steam dims and sleeps the screen itself.",
                Content = Switch("ScreensaverInGameMode", "Start the screensaver in Game Mode", S.ScreensaverInGameMode, on => S.ScreensaverInGameMode = on),
            },
            new FieldRow
            {
                Label = "Swap Images After",
                Hint = "How long the slideshow shows each picture, 2 to 120 seconds.",
                Content = Steps("ScreensaverSwapImageTimeout", 2, 120, 2, S.ScreensaverSwapImageTimeout / 1000, s => $"{s} s", s => S.ScreensaverSwapImageTimeout = s * 1000),
            },
            new FieldRow
            {
                Label = "Slideshow",
                Hint = "Which games it shows, how, and whether their name and system show in the upper left corner. Custom images replace the library's pictures.",
                Content = Ui.Stack(4,
                    Switch("ScreensaverSlideshowOnlyFavorites", "Only include favorite games", S.ScreensaverSlideshowOnlyFavorites, on => S.ScreensaverSlideshowOnlyFavorites = on),
                    Switch("ScreensaverStretchImages", "Stretch images to screen resolution", S.ScreensaverStretchImages, on => S.ScreensaverStretchImages = on),
                    Switch("ScreensaverSlideshowGameInfo", "Display game info overlay", S.ScreensaverSlideshowGameInfo, on => S.ScreensaverSlideshowGameInfo = on),
                    Switch("ScreensaverSlideshowCustomImages", "Use custom images", S.ScreensaverSlideshowCustomImages, on => S.ScreensaverSlideshowCustomImages = on),
                    Switch("ScreensaverSlideshowRecurse", "Custom image directory recursive search", S.ScreensaverSlideshowRecurse, on => S.ScreensaverSlideshowRecurse = on)),
            },
            new FieldRow
            {
                Label = "Custom Image Directory",
                Hint = "JPG, PNG, WebP, SVG and GIF pictures. ~ is the home folder and %ROMPATH% the ROM folder.",
                Content = CustomFolder(),
            },
        ];

        private PathPickerRow CustomFolder()
        {
            var picker = new PathPickerRow
            {
                Name = "ScreensaverSlideshowCustomDir", Placeholder = "No folder chosen", BrowseTitle = "Custom image directory", Mode = PathPickerMode.Folder,
                Path = S.ScreensaverSlideshowCustomDir, IsEditable = true,
            };
            picker.PathPicked += picked =>
            {
                S.ScreensaverSlideshowCustomDir = picked;
                _changed();
            };
            return picker;
        }

        // A slider of whole steps with its value beside it, as the navigation volume's in Preferences.
        private Control Steps(string name, int min, int max, int step, int current, Func<int, string> text, Action<int> store)
        {
            var slider = new Slider
            {
                Name = name, Minimum = min, Maximum = max, SmallChange = step, LargeChange = step, TickFrequency = step, IsSnapToTickEnabled = true, MinWidth = 240,
                Value = Math.Clamp(current, min, max),
            };
            var label = new TextBlock { Name = name + "Text", VerticalAlignment = VerticalAlignment.Center, MinWidth = 60, Text = text((int)slider.Value) };
            slider.ValueChanged += (_, _) =>
            {
                int value = (int)Math.Round(slider.Value);
                label.Text = text(value);
                store(value);
                _changed();
            };
            return Ui.Row(12, slider, label);
        }

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
