using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Systems
{
    // A pack's debugger views answered by emusen-systems, each reading the engine's memory in the rounds the view asks for; a view the library has not for the system answers ISystemDebugViews' default - see EmuSen_Backend.md §13.5, §40.1.
    public sealed class NativeDebugViews : ISystemDebugViews
    {
        private readonly string _system;
        private readonly Lazy<int> _stride;
        private readonly Lazy<IReadOnlyList<InterruptVector>> _vectors;
        private readonly Lazy<IReadOnlyList<string>> _symbols;
        private readonly Lazy<int> _maxSprites;

        private static readonly IReadOnlyList<IDebugMemorySpace> NoSpaces = Array.Empty<IDebugMemorySpace>();
        private static readonly (byte[] Rgba, int Width, int Height) NoPicture = (Array.Empty<byte>(), 0, 0);

        public NativeDebugViews(string system)
        {
            _system = system;
            _stride = new(() => Ask("stride", NoSpaces) is { } a ? a.Root.GetProperty("stride").GetInt32() : 0);
            _vectors = new(() => Ask("vectors", NoSpaces) is { } a
                ? a.Root.GetProperty("vectors").EnumerateArray().Select(v => new InterruptVector(v.GetProperty("name").GetString()!, v.GetProperty("address").GetInt32(), v.GetProperty("width").GetInt32(), v.GetProperty("mode").GetString()!)).ToArray()
                : Array.Empty<InterruptVector>());
            _symbols = new(() => Ask("symbol_names", NoSpaces) is { } a ? a.Root.GetProperty("names").EnumerateArray().Select(n => n.GetString()!).ToArray() : Array.Empty<string>());
            _maxSprites = new(() => Ask("max_sprites", NoSpaces) is { } a ? a.Root.GetProperty("max_sprites").GetInt32() : 0);
        }

        internal sealed record Answer(JsonElement Root, byte[] Payload);

        private Answer? Ask(string view, IReadOnlyList<IDebugMemorySpace> spaces, Action<Utf8JsonWriter>? fields = null) => Request(_system, view, spaces, fields);

        private static string M(string text) => PlatformText.Mend(text);

        // A request for one view of a system's over the spaces given, each with its size and whether a read disturbs it; null where the library has no such view.
        internal static Answer? Request(string system, string view, IReadOnlyList<IDebugMemorySpace> spaces, Action<Utf8JsonWriter>? fields = null)
        {
            var stream = new MemoryStream();
            using (var w = new Utf8JsonWriter(stream))
            {
                w.WriteStartObject();
                w.WriteString("system", system);
                w.WriteString("view", view);
                w.WriteStartArray("spaces");
                foreach (IDebugMemorySpace s in spaces)
                {
                    w.WriteStartObject();
                    w.WriteString("name", M(s.Name));
                    w.WriteNumber("size", s.Size);
                    w.WriteBoolean("side_effects", s.HasSideEffects);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                fields?.Invoke(w);
                w.WriteEndObject();
            }
            var named = spaces.Select(s => PlatformText.Crosses(s.Name) ? s : new MendedSpace(s)).ToArray();
            if (SystemCode.View(stream.ToArray(), named) is not var (document, payload)) return null;
            using (document) return new Answer(document.RootElement.Clone(), payload);
        }

        // A space whose name holds a lone surrogate, under the name the library was told.
        private sealed class MendedSpace(IDebugMemorySpace space) : IDebugMemorySpace
        {
            public string Name => M(space.Name);
            public int Size => space.Size;
            public bool IsWritable => space.IsWritable;
            public bool HasSideEffects => space.HasSideEffects;
            public byte Read(int address) => space.Read(address);
            public void Write(int address, byte value) => space.Write(address, value);
        }

        private static (byte[] Rgba, int Width, int Height) Picture(Answer a) => (a.Payload, a.Root.GetProperty("width").GetInt32(), a.Root.GetProperty("height").GetInt32());

        private static IReadOnlyList<DebugPaletteInfo> PalettesOf(Answer a) =>
            a.Root.GetProperty("palettes").EnumerateArray().Select(p => new DebugPaletteInfo(p.GetProperty("index").GetInt32(),
                p.GetProperty("colors").EnumerateArray().Select(c => (c[0].GetByte(), c[1].GetByte(), c[2].GetByte())).ToArray())).ToArray();

        private static IReadOnlyList<DebugDmaChannel> ChannelsOf(Answer a) =>
            a.Root.GetProperty("channels").EnumerateArray().Select(c => new DebugDmaChannel(c.GetProperty("index").GetInt32(), c.GetProperty("general").GetBoolean(),
                c.GetProperty("hdma").GetBoolean(), c.GetProperty("control").GetByte(), c.GetProperty("destination").GetByte(), c.GetProperty("source").GetInt32(),
                c.GetProperty("size").GetInt32(), c.GetProperty("hdma_active").GetBoolean(), c.GetProperty("table").GetInt32(), c.GetProperty("line_counter").GetByte(),
                c.GetProperty("indirect").GetInt32())).ToArray();

        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Ask("palettes", spaces) is { } a ? PalettesOf(a) : Array.Empty<DebugPaletteInfo>();

        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Ask("palette_swatch", spaces) is { } a ? Picture(a) : NoPicture;

        public (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Ask("tile_sheet", spaces) is { } a ? Picture(a) : NoPicture;

        public int TilemapEntryStride => _stride.Value;

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) =>
            Ask("tilemap_entry", new[] { space }, w => { w.WriteString("space", M(space.Name)); w.WriteNumber("address", address); }) is { } a ? a.Root.GetProperty("text").GetString()! : "";

        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (Ask("tile_pixels", new[] { space }, w => { w.WriteString("space", M(space.Name)); w.WriteNumber("address", address); w.WriteNumber("bpp", bpp); }) is not { } a) return Array.Empty<byte>();
            if (a.Root.TryGetProperty("refused", out JsonElement words))
                throw a.Root.GetProperty("param").ValueKind == JsonValueKind.Null ? new ArgumentException(words.GetString()) : new ArgumentException(words.GetString(), a.Root.GetProperty("param").GetString());
            return a.Payload;
        }

        public IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Ask("dma_channels", spaces) is { } a ? ChannelsOf(a) : Array.Empty<DebugDmaChannel>();

        public string? NameDmaDestination(byte register) =>
            Ask("dma_destination", NoSpaces, w => w.WriteNumber("register", register)) is { } a ? a.Root.GetProperty("text").GetString() : null;

        public IReadOnlyList<InterruptVector> InterruptVectors => _vectors.Value;

        public IReadOnlyList<string> SymbolNames => _symbols.Value;

        private static void Registers(Utf8JsonWriter w, IReadOnlyList<DebugRegisterValue> registers)
        {
            w.WriteStartArray("registers");
            foreach (DebugRegisterValue r in registers)
            {
                w.WriteStartObject();
                w.WriteString("name", M(r.Name));
                w.WriteNumber("value", r.Value);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        }

        public long? Symbol(string name, IReadOnlyList<DebugRegisterValue> registers)
        {
            ArgumentNullException.ThrowIfNull(name);
            if (Ask("symbol", NoSpaces, w => { w.WriteString("name", M(name)); Registers(w, registers); }) is not { } a) return null;
            JsonElement v = a.Root.GetProperty("value");
            return v.ValueKind == JsonValueKind.Null ? null : v.GetInt64();
        }

        private static void Unit(Utf8JsonWriter w, DebugUnit u)
        {
            w.WriteStartObject();
            w.WriteString("name", M(u.Name));
            w.WriteString("kind", M(u.Kind));
            Registers(w, u.Registers);
            w.WriteEndObject();
        }

        // The units as a view of the library reads them: each one's name and kind, and its registers named with their values.
        private static void Units(Utf8JsonWriter w, IReadOnlyList<DebugUnit> units)
        {
            w.WriteStartArray("units");
            foreach (DebugUnit u in units) Unit(w, u);
            w.WriteEndArray();
        }

        // A console whose palettes are registers reads the units too; with none the request is the palettes' as before.
        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            units.Count == 0 ? Palettes(spaces) : Ask("palettes", spaces, w => Units(w, units)) is { } a ? PalettesOf(a) : Array.Empty<DebugPaletteInfo>();

        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            units.Count == 0 ? PaletteSwatch(spaces) : Ask("palette_swatch", spaces, w => Units(w, units)) is { } a ? Picture(a) : NoPicture;

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            units.Count == 0 ? DecodeTilemapEntry(space, address)
            : Ask("tilemap_entry", new[] { space }, w => { w.WriteString("space", M(space.Name)); w.WriteNumber("address", address); Units(w, units); }) is { } a ? a.Root.GetProperty("text").GetString()! : "";

        public IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            units.Count == 0 ? DmaChannels(spaces) : Ask("dma_channels", spaces, w => Units(w, units)) is { } a ? ChannelsOf(a) : Array.Empty<DebugDmaChannel>();

        public int MaxSprites => _maxSprites.Value;

        public IReadOnlyList<DebugSpriteInfo> Sprites(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            Ask("sprites", spaces, w => Units(w, units)) is { } a
                ? a.Root.GetProperty("sprites").EnumerateArray().Select(x => new DebugSpriteInfo(x.GetProperty("index").GetInt32(), x.GetProperty("x").GetInt32(), x.GetProperty("y").GetInt32(),
                    x.GetProperty("width").GetInt32(), x.GetProperty("height").GetInt32(), x.GetProperty("tile").GetInt32(), x.GetProperty("palette").GetInt32(), x.GetProperty("priority").GetInt32(),
                    x.GetProperty("flip_x").GetBoolean(), x.GetProperty("flip_y").GetBoolean())).ToArray()
                : Array.Empty<DebugSpriteInfo>();

        public IReadOnlyList<DebugAudioChannelInfo> AudioChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units, uint mutes) =>
            Ask("audio_channels", spaces, w => { Units(w, units); w.WriteNumber("mutes", mutes); }) is { } a
                ? a.Root.GetProperty("channels").EnumerateArray().Select(c => new DebugAudioChannelInfo(c.GetProperty("index").GetInt32(), c.GetProperty("name").GetString()!,
                    c.GetProperty("active").GetBoolean(), c.GetProperty("level").GetInt32(), c.GetProperty("muted").GetBoolean(), c.GetProperty("info").GetString()!)).ToArray()
                : Array.Empty<DebugAudioChannelInfo>();

        public PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units)
        {
            if (Ask("physical", spaces, w => { w.WriteNumber("address", address); Units(w, units); }) is not { } a) return null;
            JsonElement space = a.Root.GetProperty("space");
            return space.ValueKind == JsonValueKind.Null ? null : new PhysicalAddress(space.GetString()!, a.Root.GetProperty("offset").GetInt32(), a.Root.GetProperty("addressable").GetBoolean());
        }

        public string? UnitLabel(DebugUnit unit) =>
            Ask("unit_label", NoSpaces, w => { w.WritePropertyName("unit"); Unit(w, unit); }) is { } a ? a.Root.GetProperty("text").GetString() : null;

        public string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            units.Count == 0 ? Summary(registers) : Ask("summary", spaces, w => { Registers(w, registers); Units(w, units); }) is { } a ? a.Root.GetProperty("text").GetString() : null;

        public string? Summary(IReadOnlyList<DebugRegisterValue> registers) =>
            Ask("summary", NoSpaces, w => Registers(w, registers)) is { } a ? a.Root.GetProperty("text").GetString() : null;
    }
}
