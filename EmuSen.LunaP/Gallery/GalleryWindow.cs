using System.Collections.Generic;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;

namespace EmuSen.LunaP.Gallery
{
    // Every control in the kit, once, with sample data - the visual reference, and what one render test covers - see EmuSen_LunaP.md §7.
    public class GalleryWindow : Window
    {
        public GalleryWindow()
        {
            Title = "LunaP gallery";
            Width = 520;
            Height = 860;
            Background = LunaPalette.Surface;

            var console = new ConsolePane { Prompt = "DianaOS #: ", HistorySource = () => new[] { "help", "coretop" } };
            console.Submitted += line => console.AppendLine("DianaOS #: " + line);

            var swatch = new RgbaImageView { Stretch = Stretch.None };
            swatch.SetFrame(Ramp(256, 32), 256, 32);

            var meters = new MeterList
            {
                Meters = new List<MeterEntry>
                {
                    new("S-CPU", 24, "24.0%"),
                    new("S-PPU", 68, "68.0%"),
                    new("SuperFX", 91, "91.0%"),
                    new("A name long enough to be trimmed by the label column", 5, "5.0%"),
                },
            };

            Content = Ui.Scroll(Ui.Stack(10,
                Ui.Section("Text", Ui.Stack(6,
                    Ui.Mono("PC=0x008123  A=0x0000  X=0x01FF"),
                    Ui.Hint("Grey, 11pt, wrapping - the explanatory line under a label or a checkbox."))),

                Ui.Section("Meters", meters),

                Ui.Section("Image view", swatch),

                Ui.Section("Settings fields", Ui.Stack(10,
                    new FieldRow
                    {
                        Label = "ROM Directory",
                        Hint = "Default folder for Open ROM... and the ROM list.",
                        Content = new PathPickerRow { Placeholder = "(not set)", BrowseTitle = "Choose ROM Directory" },
                    },
                    new FieldRow
                    {
                        Label = "Emulator Core",
                        Content = new ComboBox { ItemsSource = new[] { "SNES", "NES" }, SelectedIndex = 0 }.Grow(),
                    })),

                Ui.Section("Console", console.Height(160)),

                Ui.Section("Bottom bar", new StatusBar
                {
                    Status = "Ready.",
                    Content = Ui.Buttons(
                        Ui.Button("Apply", () => { }),
                        Ui.Button("Close", Close)),
                })).Margin(12));

            console.AppendLine("DianaOS #: help");
            console.AppendLine("Type a command. This pane knows nothing about DianaOS.");
        }

        // Real pixels, so the image view is not just showing a flat rectangle.
        private static byte[] Ramp(int width, int height)
        {
            var rgba = new byte[width * height * 4];
            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    int i = ((y * width) + x) * 4;
                    rgba[i] = (byte)x;
                    rgba[i + 1] = (byte)(y * 8);
                    rgba[i + 2] = (byte)(255 - x);
                    rgba[i + 3] = 255;
                }
            }

            return rgba;
        }
    }
}
