using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Crystal.Systems
{
    // A pack's debugger views answered by emusen-systems, each reading the engine's memory in the rounds the view asks for; a view the library has not is the C# views' it fronts - see EmuSen_Backend.md §13.5.
    public sealed class NativeDebugViews : ISystemDebugViews
    {
        private readonly string _system;
        private readonly Lazy<int> _stride;
        private readonly Lazy<IReadOnlyList<InterruptVector>> _vectors;
        private readonly Lazy<IReadOnlyList<string>> _symbols;

        public NativeDebugViews(string system, ISystemDebugViews managed)
        {
            _system = system;
            Managed = managed;
            _stride = new(() => Ask("stride", Array.Empty<IDebugMemorySpace>()) is { } a ? a.Root.GetProperty("stride").GetInt32() : Managed.TilemapEntryStride);
            _vectors = new(() => Ask("vectors", Array.Empty<IDebugMemorySpace>()) is { } a
                ? a.Root.GetProperty("vectors").EnumerateArray().Select(v => new InterruptVector(v.GetProperty("name").GetString()!, v.GetProperty("address").GetInt32(), v.GetProperty("width").GetInt32(), v.GetProperty("mode").GetString()!)).ToArray()
                : Managed.InterruptVectors);
            _symbols = new(() => Ask("symbol_names", Array.Empty<IDebugMemorySpace>()) is { } a
                ? a.Root.GetProperty("names").EnumerateArray().Select(n => n.GetString()!).ToArray()
                : Managed.SymbolNames);
        }

        // The C# views this one fronts, the failover and the parity class's oracle.
        public ISystemDebugViews Managed { get; }

        internal sealed record Answer(JsonElement Root, byte[] Payload);

        private Answer? Ask(string view, IReadOnlyList<IDebugMemorySpace> spaces, Action<Utf8JsonWriter>? fields = null) => Request(_system, view, spaces, fields);

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
                    w.WriteString("name", s.Name);
                    w.WriteNumber("size", s.Size);
                    w.WriteBoolean("side_effects", s.HasSideEffects);
                    w.WriteEndObject();
                }
                w.WriteEndArray();
                fields?.Invoke(w);
                w.WriteEndObject();
            }
            if (SystemCode.View(stream.ToArray(), spaces) is not var (document, payload)) return null;
            using (document) return new Answer(document.RootElement.Clone(), payload);
        }

        private static bool Crosses(IEnumerable<IDebugMemorySpace> spaces) => spaces.All(s => SystemCode.Takes(s.Name));

        private static (byte[] Rgba, int Width, int Height) Picture(Answer a) => (a.Payload, a.Root.GetProperty("width").GetInt32(), a.Root.GetProperty("height").GetInt32());

        public IReadOnlyList<DebugPaletteInfo> Palettes(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Crosses(spaces) && Ask("palettes", spaces) is { } a
                ? a.Root.GetProperty("palettes").EnumerateArray().Select(p => new DebugPaletteInfo(p.GetProperty("index").GetInt32(),
                    p.GetProperty("colors").EnumerateArray().Select(c => (c[0].GetByte(), c[1].GetByte(), c[2].GetByte())).ToArray())).ToArray()
                : Managed.Palettes(spaces);

        public (byte[] Rgba, int Width, int Height) PaletteSwatch(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Crosses(spaces) && Ask("palette_swatch", spaces) is { } a ? Picture(a) : Managed.PaletteSwatch(spaces);

        public (byte[] Rgba, int Width, int Height) TileSheet(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Crosses(spaces) && Ask("tile_sheet", spaces) is { } a ? Picture(a) : Managed.TileSheet(spaces);

        public int TilemapEntryStride => _stride.Value;

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) =>
            Crosses(new[] { space }) && Ask("tilemap_entry", new[] { space }, w => { w.WriteString("space", space.Name); w.WriteNumber("address", address); }) is { } a
                ? a.Root.GetProperty("text").GetString()!
                : Managed.DecodeTilemapEntry(space, address);

        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (!Crosses(new[] { space }) || Ask("tile_pixels", new[] { space }, w => { w.WriteString("space", space.Name); w.WriteNumber("address", address); w.WriteNumber("bpp", bpp); }) is not { } a)
                return Managed.DecodeTilePixels(space, address, bpp);
            if (a.Root.TryGetProperty("refused", out JsonElement words))
                throw a.Root.GetProperty("param").ValueKind == JsonValueKind.Null ? new ArgumentException(words.GetString()) : new ArgumentException(words.GetString(), a.Root.GetProperty("param").GetString());
            return a.Payload;
        }

        public IReadOnlyList<DebugDmaChannel> DmaChannels(IReadOnlyList<IDebugMemorySpace> spaces) =>
            Crosses(spaces) && Ask("dma_channels", spaces) is { } a
                ? a.Root.GetProperty("channels").EnumerateArray().Select(c => new DebugDmaChannel(c.GetProperty("index").GetInt32(), c.GetProperty("general").GetBoolean(),
                    c.GetProperty("hdma").GetBoolean(), c.GetProperty("control").GetByte(), c.GetProperty("destination").GetByte(), c.GetProperty("source").GetInt32(),
                    c.GetProperty("size").GetInt32(), c.GetProperty("hdma_active").GetBoolean(), c.GetProperty("table").GetInt32(), c.GetProperty("line_counter").GetByte(),
                    c.GetProperty("indirect").GetInt32())).ToArray()
                : Managed.DmaChannels(spaces);

        public string? NameDmaDestination(byte register) =>
            Ask("dma_destination", Array.Empty<IDebugMemorySpace>(), w => w.WriteNumber("register", register)) is { } a ? a.Root.GetProperty("text").GetString() : Managed.NameDmaDestination(register);

        public IReadOnlyList<InterruptVector> InterruptVectors => _vectors.Value;

        public IReadOnlyList<string> SymbolNames => _symbols.Value;

        private static void Registers(Utf8JsonWriter w, IReadOnlyList<DebugRegisterValue> registers)
        {
            w.WriteStartArray("registers");
            foreach (DebugRegisterValue r in registers)
            {
                w.WriteStartObject();
                w.WriteString("name", r.Name);
                w.WriteNumber("value", r.Value);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        }

        private static bool Crosses(string name, IReadOnlyList<DebugRegisterValue> registers) => SystemCode.Takes(name) && registers.All(r => SystemCode.Takes(r.Name));

        public long? Symbol(string name, IReadOnlyList<DebugRegisterValue> registers)
        {
            if (name is null || !Crosses(name, registers) || Ask("symbol", Array.Empty<IDebugMemorySpace>(), w => { w.WriteString("name", name); Registers(w, registers); }) is not { } a)
                return Managed.Symbol(name!, registers);
            JsonElement v = a.Root.GetProperty("value");
            return v.ValueKind == JsonValueKind.Null ? null : v.GetInt64();
        }

        // The views no console's library answers yet, the C# views' until a console's fill adds their twins.
        public int MaxSprites => Managed.MaxSprites;

        public IReadOnlyList<DebugSpriteInfo> Sprites(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => Managed.Sprites(spaces, units);

        public IReadOnlyList<DebugAudioChannelInfo> AudioChannels(IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units, uint mutes) => Managed.AudioChannels(spaces, units, mutes);

        public PhysicalAddress? ResolvePhysical(int address, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) => Managed.ResolvePhysical(address, spaces, units);

        public string? UnitLabel(DebugUnit unit) => Managed.UnitLabel(unit);

        // A summary that reads units is the C# views' until its twin, one that reads only the processor's registers the library's as before.
        public string? Summary(IReadOnlyList<DebugRegisterValue> registers, IReadOnlyList<IDebugMemorySpace> spaces, IReadOnlyList<DebugUnit> units) =>
            units.Count > 0 ? Managed.Summary(registers, spaces, units) : Summary(registers);

        public string? Summary(IReadOnlyList<DebugRegisterValue> registers) =>
            Crosses("", registers) && Ask("summary", Array.Empty<IDebugMemorySpace>(), w => Registers(w, registers)) is { } a ? a.Root.GetProperty("text").GetString() : Managed.Summary(registers);
    }
}
