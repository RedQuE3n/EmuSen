using System;
using System.Collections.Generic;

namespace EmuSen.Cauldron
{
    // One named register value, generic across wildly different register sets - see EmuSen_Cauldron.md §4.1.
    public readonly struct DebugRegisterValue : IEquatable<DebugRegisterValue>
    {
        public string Name { get; }
        public ulong Value { get; }
        public int BitWidth { get; }

        public DebugRegisterValue(string name, ulong value, int bitWidth)
        {
            Name = name;
            Value = value;
            BitWidth = bitWidth;
        }

        public bool Equals(DebugRegisterValue other) =>
            Value == other.Value && BitWidth == other.BitWidth && Name == other.Name;

        public override bool Equals(object? obj) => obj is DebugRegisterValue other && Equals(other);

        public override int GetHashCode() => HashCode.Combine(Name, Value, BitWidth);
    }

    // One sprite/OBJ entry, reduced to what any sprite viewer needs - see EmuSen_Cauldron.md §4.2.
    public readonly struct DebugSpriteInfo
    {
        public int Index { get; }
        public int X { get; }
        public int Y { get; }
        public int Width { get; }
        public int Height { get; }
        public int TileIndex { get; }
        public int PaletteIndex { get; }
        public int Priority { get; }
        public bool FlipX { get; }
        public bool FlipY { get; }

        public DebugSpriteInfo(int index, int x, int y, int width, int height, int tileIndex, int paletteIndex, int priority, bool flipX, bool flipY)
        {
            Index = index;
            X = x;
            Y = y;
            Width = width;
            Height = height;
            TileIndex = tileIndex;
            PaletteIndex = paletteIndex;
            Priority = priority;
            FlipX = flipX;
            FlipY = flipY;
        }
    }

    // One palette's colors, already resolved to display-ready RGB - see EmuSen_Cauldron.md §4.3.
    public readonly struct DebugPaletteInfo
    {
        public int Index { get; }
        public IReadOnlyList<(byte r, byte g, byte b)> Colors { get; }

        public DebugPaletteInfo(int index, IReadOnlyList<(byte r, byte g, byte b)> colors)
        {
            Index = index;
            Colors = colors;
        }
    }

    // One audio channel/voice, on a normalized 0-100 scale - see EmuSen_Cauldron.md §4.4.
    public readonly struct DebugAudioChannelInfo
    {
        public int Index { get; }
        public string Name { get; }
        public bool Active { get; }
        public int Level { get; }
        public bool Muted { get; }
        public string Info { get; }

        public DebugAudioChannelInfo(int index, string name, bool active, int level, bool muted, string info)
        {
            Index = index;
            Name = name;
            Active = active;
            Level = level;
            Muted = muted;
            Info = info;
        }
    }

    // One "how hard is this working" meter, 0-100 - see EmuSen_Cauldron.md §4.5.
    public readonly struct DebugLoadInfo
    {
        public string Name { get; }
        public double Percent { get; }

        public DebugLoadInfo(string name, double percent)
        {
            Name = name;
            Percent = percent;
        }
    }
}
