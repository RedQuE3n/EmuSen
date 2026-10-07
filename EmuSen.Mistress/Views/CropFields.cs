using System;
using System.Collections.Generic;
using System.Globalization;
using Avalonia.Controls;
using Avalonia.Layout;

namespace EmuSen.Mistress.Views
{
    // Four boxes for the edges of a crop, each a percentage of the picture, for a console's crop and for a game's - see EmuSen_Settings_Reference.md §4.99.
    public sealed class CropFields
    {
        // The edges in the order they are shown, with the key a console's value is stored under.
        public static readonly (string Key, string Label)[] Edges = { ("CropLeft", "Left"), ("CropRight", "Right"), ("CropTop", "Top"), ("CropBottom", "Bottom") };

        public const string Hint = "How much of the picture's width or height to hide at each edge, up to 25.";

        private readonly List<TextBox> _boxes = new();
        private readonly Func<int, double> _stored;

        public WrapPanel Row { get; } = new() { Orientation = Orientation.Horizontal };

        // stored answers an edge's percentage as kept now; changed is told a typed one that differs from it.
        public CropFields(string name, Func<int, double> stored, Action<int, double> changed)
        {
            _stored = stored;
            for (int i = 0; i < Edges.Length; i++)
            {
                int edge = i;
                var box = new TextBox { Name = $"{name}.{Edges[i].Key}", Width = 64, VerticalAlignment = VerticalAlignment.Center, Margin = new Avalonia.Thickness(0, 0, 14, 0) };
                Avalonia.Automation.AutomationProperties.SetName(box, $"{Edges[i].Label}, percent");
                // The event comes after the text is set, so a box filled from the stored value is told from a typed one by its value, not by a flag.
                box.TextChanged += (_, _) => { if (ParsePercent(box.Text) is { } percent && percent != _stored(edge)) changed(edge, percent); };
                box.LostFocus += (_, _) => box.Text = Text(_stored(edge));
                _boxes.Add(box);
                Row.Children.Add(new TextBlock { Text = Edges[i].Label, VerticalAlignment = VerticalAlignment.Center, Margin = new Avalonia.Thickness(0, 0, 6, 0) });
                Row.Children.Add(box);
            }
            Show();
        }

        // Every box as stored; not called while one is typed in, whose text it would replace.
        public void Show()
        {
            for (int i = 0; i < _boxes.Count; i++) _boxes[i].Text = Text(_stored(i));
        }

        // A percentage as typed, with a point or a comma, held to what one edge may hide; null when it is no number.
        public static double? ParsePercent(string? text) =>
            double.TryParse(text?.Trim().Replace(',', '.'), NumberStyles.Float, CultureInfo.InvariantCulture, out double value) && double.IsFinite(value)
                ? Math.Clamp(Math.Round(value, 1), 0, EmuSen.Serenity.PictureCrop.MostAtAnEdge * 100) : null;

        public static string Text(double percent) => percent.ToString("0.#", CultureInfo.InvariantCulture);
    }
}
