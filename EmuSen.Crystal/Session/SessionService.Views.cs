using System.Buffers.Binary;
using System.Text.Json;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using static EmuSen.Crystal.Session.SessionJson;

namespace EmuSen.Crystal.Session
{
    // The chip-view, expression, cheat, trace, telemetry, state, log and event groups of EmuSen_DianaSh.md §9.2.
    public sealed partial class SessionService
    {
        internal byte[] DmaChannels(uint session)
        {
            IDebugTarget t = Target(session);
            return Array(t.DmaChannels, (w, c) =>
            {
                w.WriteStartObject();
                w.WriteNumber("index", c.Index);
                w.WriteBoolean("general", c.GeneralEnabled);
                w.WriteBoolean("hdma", c.HdmaEnabled);
                w.WriteNumber("control", c.Control);
                w.WriteNumber("destination", c.DestinationRegister);
                NullableString(w, "destination_name", t.NameDmaDestination(c.DestinationRegister));
                w.WriteNumber("source", U(c.SourceAddress));
                w.WriteNumber("size", c.TransferSize);
                w.WriteBoolean("hdma_active", c.HdmaActive);
                w.WriteNumber("table", c.TableAddress);
                w.WriteNumber("line_counter", c.LineCounter);
                w.WriteNumber("indirect", c.IndirectAddress);
                w.WriteEndObject();
            });
        }

        internal byte[] DmaLogOp(uint session, uint op, ReadOnlySpan<byte> args, bool readOnly)
        {
            IDebugTarget t = Target(session);
            DmaLogRegistry log = Need(t.DmaLog, "This core does not record DMA transfers.");
            if (op > SessionAbi.DmaLogDestinations) throw NotSupported($"no DMA log op {op}");
            if (readOnly && op <= SessionAbi.DmaLogClear) throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session");
            JsonElement a = Parse(args);
            switch (op)
            {
                case SessionAbi.DmaLogArm:
                    if (OptionalInt(a, "capacity") is { } capacity) log.Arm(Int32(capacity, "capacity"));
                    else log.Arm();
                    return [];
                case SessionAbi.DmaLogDisarm: log.Disarm(); return [];
                case SessionAbi.DmaLogClear: log.Clear(); return [];
                case SessionAbi.DmaLogStatus:
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteBoolean("armed", log.IsArmed);
                        w.WriteNumber("capacity", log.Capacity);
                        w.WriteNumber("transfers", log.TotalTransfers);
                        w.WriteNumber("bytes", log.TotalBytes);
                        w.WriteNumber("first_frame", log.FirstFrame);
                        w.WriteNumber("last_frame", log.LastFrame);
                        w.WriteEndObject();
                    });
                case SessionAbi.DmaLogTail:
                    return Array(log.Tail(Int32(Int(a, "count"), "count")), (w, x) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("sequence", x.Sequence);
                        w.WriteNumber("frame", x.FrameNumber);
                        w.WriteNumber("scanline", x.Scanline);
                        w.WriteNumber("channel", x.Channel);
                        w.WriteString("kind", x.Kind == DmaTransferKind.Hdma ? "hdma" : "general");
                        w.WriteNumber("control", x.Control);
                        w.WriteNumber("destination", x.DestinationRegister);
                        NullableString(w, "destination_name", t.NameDmaDestination(x.DestinationRegister));
                        w.WriteNumber("source", U(x.SourceAddress));
                        w.WriteNumber("length", x.Length);
                        w.WriteEndObject();
                    });
                case SessionAbi.DmaLogChannels:
                    return Array(log.ChannelStats(), (w, x) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("channel", x.Channel);
                        w.WriteNumber("general_transfers", x.GeneralTransfers);
                        w.WriteNumber("general_bytes", x.GeneralBytes);
                        w.WriteNumber("hdma_transfers", x.HdmaTransfers);
                        w.WriteNumber("hdma_bytes", x.HdmaBytes);
                        w.WriteEndObject();
                    });
                default:
                    return Array(log.DestinationStats(), (w, x) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("destination", x.DestinationRegister);
                        NullableString(w, "destination_name", t.NameDmaDestination(x.DestinationRegister));
                        w.WriteNumber("transfers", x.Transfers);
                        w.WriteNumber("bytes", x.Bytes);
                        w.WriteNumber("channel_mask", x.ChannelMask);
                        w.WriteEndObject();
                    });
            }
        }

        internal byte[] RegisterFlowOp(uint session, uint op, ReadOnlySpan<byte> args, bool readOnly)
        {
            RegisterFlowRegistry flow = Need(Target(session).RegisterFlow, "This cartridge has no coprocessor register window to watch.");
            if (op > SessionAbi.FlowStats) throw NotSupported($"no register-flow op {op}");
            if (readOnly && op <= SessionAbi.FlowClear) throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session");
            JsonElement a = Parse(args);
            switch (op)
            {
                case SessionAbi.FlowArm:
                    string window = String(a, "window");
                    int size = Int32(Int(a, "size"), "size");
                    if (OptionalInt(a, "capacity") is { } capacity) flow.Arm(window, size, Int32(capacity, "capacity"));
                    else flow.Arm(window, size);
                    return [];
                case SessionAbi.FlowDisarm: flow.Disarm(); return [];
                case SessionAbi.FlowClear: flow.Clear(); return [];
                case SessionAbi.FlowStatus:
                    return Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteBoolean("armed", flow.IsArmed);
                        w.WriteString("window", flow.WindowName);
                        w.WriteNumber("size", flow.WindowSize);
                        w.WriteNumber("capacity", flow.Capacity);
                        w.WriteNumber("accesses", flow.TotalAccesses);
                        w.WriteStartObject("longest_poll");
                        w.WriteNumber("run", flow.LongestPollRun);
                        w.WriteNumber("register", flow.LongestPollRegister);
                        w.WriteNumber("value", flow.LongestPollValue);
                        w.WriteEndObject();
                        w.WriteStartObject("current_poll");
                        w.WriteNumber("run", flow.CurrentPollRun);
                        w.WriteNumber("register", flow.CurrentPollRegister);
                        w.WriteEndObject();
                        w.WriteEndObject();
                    });
                case SessionAbi.FlowTail:
                    return Array(flow.Tail(Int32(Int(a, "count"), "count")), (w, x) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("sequence", x.Sequence);
                        w.WriteNumber("frame", x.FrameNumber);
                        w.WriteNumber("register", x.Register);
                        w.WriteNumber("value", x.Value);
                        w.WriteBoolean("write", x.IsWrite);
                        w.WriteString("actor", x.Actor);
                        w.WriteEndObject();
                    });
                default:
                    return Array(flow.Stats(), (w, x) =>
                    {
                        w.WriteStartObject();
                        w.WriteNumber("register", x.Register);
                        w.WriteNumber("reads", x.Reads);
                        w.WriteNumber("writes", x.Writes);
                        w.WriteNumber("distinct_values", x.DistinctValues);
                        w.WriteNumber("last_value", x.LastValue);
                        w.WriteEndObject();
                    });
            }
        }

        internal byte[] CoprocessorHistory(uint session, uint count)
        {
            if (Target(session) is not IHistoricalCoprocessorTarget historical) throw NotSupported("This core does not keep coprocessor register history.");
            var provider = historical.CoprocessorHistory;
            var frames = provider.GetHistory();
            var shown = frames.Skip(Math.Max(0, frames.Count - Int32(count, "count")));
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("capacity", provider.Capacity);
                w.WriteNumber("refreshes", provider.RefreshCount);
                w.WriteNumber("unchanged_for", provider.RefreshesSinceChange);
                w.WriteStartArray("frames");
                foreach (var frame in shown) SessionJson.Registers(w, frame);
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal byte[] Vectors(uint session) => Array(Target(session).InterruptVectors, (w, v) =>
        {
            w.WriteStartObject();
            w.WriteString("name", v.Name);
            w.WriteNumber("address", U(v.Address));
            w.WriteNumber("width", v.Width);
            w.WriteString("mode", v.Mode);
            w.WriteEndObject();
        });

        internal byte[] TilemapEntry(uint session, string? space, uint address)
        {
            IDebugTarget t = Target(session);
            string text = t.DecodeTilemapEntry(Space(t, space), Addr(address));
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("stride", t.TilemapEntryStride);
                w.WriteString("text", text);
                w.WriteEndObject();
            });
        }

        internal byte[] TilePixels(uint session, string? space, uint address, uint bpp)
        {
            IDebugTarget t = Target(session);
            return t.DecodeTilePixels(Space(t, space), Addr(address), Int32(bpp, "bpp"));
        }

        private static void Sprite(Utf8JsonWriter w, DebugSpriteInfo s)
        {
            w.WriteStartObject();
            w.WriteNumber("index", s.Index);
            w.WriteNumber("x", s.X);
            w.WriteNumber("y", s.Y);
            w.WriteNumber("width", s.Width);
            w.WriteNumber("height", s.Height);
            w.WriteNumber("tile", s.TileIndex);
            w.WriteNumber("palette", s.PaletteIndex);
            w.WriteNumber("priority", s.Priority);
            w.WriteBoolean("flip_x", s.FlipX);
            w.WriteBoolean("flip_y", s.FlipY);
            w.WriteEndObject();
        }

        private static void Palette(Utf8JsonWriter w, DebugPaletteInfo p)
        {
            w.WriteStartObject();
            w.WriteNumber("index", p.Index);
            w.WriteStartArray("colors");
            foreach (var (r, g, b) in p.Colors)
            {
                w.WriteStartArray();
                w.WriteNumberValue(r);
                w.WriteNumberValue(g);
                w.WriteNumberValue(b);
                w.WriteEndArray();
            }
            w.WriteEndArray();
            w.WriteEndObject();
        }

        private static void Channel(Utf8JsonWriter w, DebugAudioChannelInfo c)
        {
            w.WriteStartObject();
            w.WriteNumber("index", c.Index);
            w.WriteString("name", c.Name);
            w.WriteBoolean("active", c.Active);
            w.WriteNumber("level", c.Level);
            w.WriteBoolean("muted", c.Muted);
            w.WriteString("info", c.Info);
            w.WriteEndObject();
        }

        internal byte[] Sprites(uint session)
        {
            IDebugTarget t = Target(session);
            var sprites = t.Sprites.Current;
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteNumber("max", t.MaxSprites);
                w.WriteStartArray("sprites");
                foreach (var s in sprites) Sprite(w, s);
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        internal byte[] Palettes(uint session) => Array(Target(session).Palettes.Current, Palette);

        internal byte[] AudioChannels(uint session) => Array(Target(session).AudioChannels.Current, Channel);

        // A channel crosses as a 32-bit pattern, as an address does, so a command's own index reaches the core as typed.
        internal void SetChannelMuted(uint session, uint channel, bool muted) => Target(session).SetChannelMuted(Addr(channel), muted);

        internal byte[] Evaluate(uint session, uint processor, string? text)
        {
            IExpressionContext context = ProcessorOf(Target(session), processor).Expressions;
            long value;
            try { value = new ExpressionEvaluator(context).Evaluate(text ?? ""); }
            catch (ExpressionException e) { throw new SessionException(SessionAbi.Parse, e.Message); }
            return Write(w => { w.WriteStartObject(); w.WriteNumber("value", value); w.WriteEndObject(); });
        }

        internal byte[] Symbols(uint session, uint processor, string? filter)
        {
            var names = ProcessorOf(Target(session), processor).Expressions.SymbolNames;
            return Strings(filter is null ? names : names.Where(n => n.Contains(filter, StringComparison.OrdinalIgnoreCase)));
        }

        private static readonly HashSet<uint> CheatChanges = new()
        {
            SessionAbi.CheatAdd, SessionAbi.CheatPoke, SessionAbi.CheatRomPatch, SessionAbi.CheatEnable, SessionAbi.CheatDisable,
            SessionAbi.CheatRemove, SessionAbi.CheatClear, SessionAbi.CheatImport,
        };

        // A read answers with a record; a change with its id, its count or 0, and copies nothing - see emusen_session.h.
        internal (byte[]? Record, long Value) CheatOp(uint session, uint op, ReadOnlySpan<byte> args, bool readOnly)
        {
            IDebugTarget t = Target(session);
            CheatRegistry cheats = t.Cheats;
            if (op > SessionAbi.CheatDbFind) throw NotSupported($"no cheat op {op}");
            JsonElement a = Parse(args);
            bool changes = CheatChanges.Contains(op) || (op == SessionAbi.CheatMaster && OptionalBool(a, "enabled") is not null);
            if (readOnly && changes) throw new SessionException(SessionAbi.ReadOnly, "refused: this is a read-only context, and the call changes the session");
            switch (op)
            {
                case SessionAbi.CheatList:
                    return (Write(w =>
                    {
                        w.WriteStartObject();
                        w.WriteBoolean("master", cheats.MasterEnabled);
                        w.WriteStartArray("cheats");
                        foreach (CheatInfo c in cheats.GetCheats())
                        {
                            w.WriteStartObject();
                            w.WriteNumber("id", c.Id);
                            w.WriteString("kind", c.Kind == CheatKind.RomPatch ? "rom" : "ram");
                            w.WriteString("description", c.Description);
                            w.WriteBoolean("enabled", c.Enabled);
                            NullableNumber(w, "compare", c.Compare);
                            w.WritePropertyName("writes");
                            Writes(w, c.Writes);
                            w.WriteEndObject();
                        }
                        w.WriteEndArray();
                        w.WriteEndObject();
                    }), 0);
                case SessionAbi.CheatAdd:
                {
                    CheatKind kind = String(a, "kind") switch
                    {
                        "ram" => CheatKind.RamPoke,
                        "rom" => CheatKind.RomPatch,
                        var other => throw NotSupported($"no cheat kind '{other}'"),
                    };
                    if (!a.TryGetProperty("writes", out JsonElement writes) || writes.ValueKind != JsonValueKind.Array)
                        throw new SessionException(SessionAbi.BadArgument, "a cheat has an array of writes");
                    var list = writes.EnumerateArray().Select(ParseWrite).ToList();
                    try { return (null, cheats.AddCheat(kind, list, CompareOf(a), OptionalString(a, "description") ?? "", OptionalBool(a, "enabled") ?? true)); }
                    catch (ArgumentException e) { throw new SessionException(SessionAbi.BadArgument, e.Message); }
                }
                case SessionAbi.CheatPoke:
                {
                    long value = Int(a, "value");
                    if (value is < 0 or > 0xFF) throw new SessionException(SessionAbi.BadArgument, "a poke's value is one byte");
                    return (null, cheats.AddRamPoke(Space(t, String(a, "space")).Name, Address(a, "address"), (byte)value,
                        OptionalString(a, "description") ?? "", OptionalBool(a, "enabled") ?? true));
                }
                case SessionAbi.CheatRomPatch:
                {
                    long value = Int(a, "value");
                    if (value is < 0 or > 0xFF) throw new SessionException(SessionAbi.BadArgument, "a patch's value is one byte");
                    return (null, cheats.AddRomPatch(Address(a, "address"), (byte)value, CompareOf(a), OptionalString(a, "description") ?? "",
                        OptionalBool(a, "enabled") ?? true));
                }
                case SessionAbi.CheatEnable:
                case SessionAbi.CheatDisable:
                {
                    long id = Int(a, "id");
                    if (!cheats.SetEnabled(Int32(id, "id"), op == SessionAbi.CheatEnable)) throw NoSuchId("cheat", id);
                    return (null, 0);
                }
                case SessionAbi.CheatRemove:
                {
                    long id = Int(a, "id");
                    if (!cheats.RemoveCheat(Int32(id, "id"))) throw NoSuchId("cheat", id);
                    return (null, 0);
                }
                case SessionAbi.CheatClear: cheats.Clear(); return (null, 0);
                case SessionAbi.CheatMaster:
                    if (OptionalBool(a, "enabled") is { } on) { cheats.MasterEnabled = on; return (null, 0); }
                    return (Write(w => { w.WriteStartObject(); w.WriteBoolean("enabled", cheats.MasterEnabled); w.WriteEndObject(); }), 0);
                case SessionAbi.CheatExport:
                    return (JsonSerializer.SerializeToUtf8Bytes(cheats.ToCheatFile(), ConfigJson.Options), 0);
                case SessionAbi.CheatImport:
                {
                    CheatFile file;
                    try { file = JsonSerializer.Deserialize<CheatFile>(args, ConfigJson.Options) ?? throw new JsonException("null"); }
                    catch (JsonException e) { throw new SessionException(SessionAbi.Parse, $"That is not a cheat file: {e.Message}"); }
                    return (null, cheats.LoadFrom(file).loaded);
                }
                case SessionAbi.CheatDecode: return (Decode(String(a, "code"), OptionalString(a, "codec") ?? "auto"), 0);
                case SessionAbi.CheatDbSystems:
                    return (Array(Database().Systems(), (w, s) =>
                    {
                        w.WriteStartObject();
                        w.WriteString("system", s.System);
                        w.WriteNumber("count", s.Count);
                        w.WriteEndObject();
                    }), 0);
                case SessionAbi.CheatDbGames: return (Entries(Database().Games(String(a, "system"), OptionalString(a, "filter"))), 0);
                default: return (Entries(Database().Find(String(a, "text"), Int32(OptionalInt(a, "limit") ?? 20, "limit"))), 0);
            }
        }

        private static byte? CompareOf(JsonElement a)
        {
            long? compare = OptionalInt(a, "compare");
            if (compare is < 0 or > 0xFF) throw new SessionException(SessionAbi.BadArgument, "a compare is one byte");
            return compare is { } c ? (byte)c : null;
        }

        private static void Writes(Utf8JsonWriter w, IReadOnlyList<CheatWrite> writes)
        {
            w.WriteStartArray();
            foreach (CheatWrite x in writes)
            {
                w.WriteStartObject();
                w.WriteString("space", x.Space);
                w.WriteNumber("address", U(x.Address));
                w.WriteNumber("value", x.Value);
                w.WriteNumber("width", x.Width);
                w.WriteString("type", x.Type.ToString());
                NullableNumber(w, "bit", x.BitPosition);
                w.WriteBoolean("big_endian", x.BigEndian);
                w.WriteNumber("repeat", x.RepeatCount);
                w.WriteNumber("repeat_address", x.RepeatAddAddress);
                w.WriteNumber("repeat_value", x.RepeatAddValue);
                w.WriteEndObject();
            }
            w.WriteEndArray();
        }

        private static CheatWrite ParseWrite(JsonElement x)
        {
            CheatWriteType type = CheatWriteType.Set;
            if (OptionalString(x, "type") is { } typeName && !CheatRegistry.TryParseWriteType(typeName, out type))
                throw new SessionException(SessionAbi.BadArgument, $"no cheat write type '{typeName}'");
            long value = Int(x, "value");
            long repeatValue = OptionalInt(x, "repeat_value") ?? 0;
            if (value is < 0 or > uint.MaxValue || repeatValue is < 0 or > uint.MaxValue) throw new SessionException(SessionAbi.BadArgument, "a write's value is 32 bits");
            long? bit = OptionalInt(x, "bit");
            return new CheatWrite
            {
                Space = OptionalString(x, "space") ?? "",
                Address = Address(x, "address"),
                Value = (uint)value,
                Width = Int32(OptionalInt(x, "width") ?? 1, "width"),
                Type = type,
                BitPosition = bit is { } b ? Int32(b, "bit") : null,
                BigEndian = OptionalBool(x, "big_endian") ?? false,
                RepeatCount = Int32(OptionalInt(x, "repeat") ?? 1, "repeat"),
                RepeatAddAddress = Int32(OptionalInt(x, "repeat_address") ?? 0, "repeat_address"),
                RepeatAddValue = (uint)repeatValue,
            };
        }

        private byte[] Decode(string code, string which)
        {
            ICheatCodeCodec codec = which switch
            {
                "auto" => _options.AutoDetectCodec ?? throw NotSupported("No cheat codec is registered for this target."),
                "explicit" => _options.ExplicitCodec ?? throw NotSupported("No Game Genie-style cheat codec is registered for this target."),
                _ => throw NotSupported($"no codec '{which}'"),
            };
            if (!codec.CanDecode(code)) throw new SessionException(SessionAbi.Parse, $"'{code}' is not a {codec.Name} code.");
            (int address, byte value) decoded;
            try { decoded = codec.Decode(code); }
            catch (Exception e) { throw new SessionException(SessionAbi.Parse, e.Message); }
            byte? compare = codec.DecodeCompare(code);
            var writes = codec.DecodeWrites(code);
            return Write(w =>
            {
                w.WriteStartObject();
                w.WriteString("codec", codec.Name);
                w.WriteString("kind", codec.Kind == CheatCodeKind.RomPatch ? "rom" : "ram");
                NullableString(w, "space", codec.SpaceName);
                w.WriteNumber("address", U(decoded.address));
                w.WriteNumber("value", decoded.value);
                NullableNumber(w, "compare", compare);
                if (writes is null) w.WriteNull("writes");
                else { w.WritePropertyName("writes"); Writes(w, writes); }
                w.WriteEndObject();
            });
        }

        private CheatDatabase Database()
            => _options.CheatDatabaseDirectory is { } dir ? new CheatDatabase(dir) : throw NotSupported("No cheat database is configured for this session.");

        private static byte[] Entries(IEnumerable<CheatDatabaseEntry> entries) => Array(entries, (w, e) =>
        {
            w.WriteStartObject();
            w.WriteString("system", e.System);
            w.WriteString("game", e.Game);
            w.WriteString("path", e.Path);
            w.WriteEndObject();
        });

        internal byte[] CheatCodecs(uint session)
        {
            Target(session);
            var systems = _options.CheatSystems?.Invoke() ?? System.Array.Empty<string>();
            return Write(w =>
            {
                w.WriteStartObject();
                Codec(w, "auto", _options.AutoDetectCodec);
                Codec(w, "explicit", _options.ExplicitCodec);
                w.WriteStartArray("systems");
                foreach (string s in systems) w.WriteStringValue(s);
                w.WriteEndArray();
                w.WriteEndObject();
            });
        }

        private static void Codec(Utf8JsonWriter w, string key, ICheatCodeCodec? codec)
        {
            if (codec is null) { w.WriteNull(key); return; }
            w.WriteStartObject(key);
            w.WriteString("name", codec.Name);
            w.WriteString("kind", codec.Kind == CheatCodeKind.RomPatch ? "rom" : "ram");
            NullableString(w, "space", codec.SpaceName);
            w.WriteEndObject();
        }

        private ICpuTraceSwitch Trace(uint session, uint processor)
        {
            Target(session);
            if (processor != 0) throw NotSupported("Only the main processor can be traced.");
            return _options.Trace ?? throw NotSupported("This core has no instruction trace.");
        }

        internal void TraceOn(uint session, uint processor, uint count)
        {
            if (count < 1) throw new SessionException(SessionAbi.BadArgument, "Trace count must be at least 1.");
            Trace(session, processor).Arm(Int32(count, "count"));
        }

        internal void TraceOff(uint session, uint processor) => Trace(session, processor).Disarm();

        internal byte[] Telemetry(uint session, uint kind)
        {
            IDebugTarget t = Target(session);
            return kind switch
            {
                SessionAbi.TelemetryCpu => Write(w => SessionJson.Registers(w, t.CpuRegisters.Current)),
                SessionAbi.TelemetryVideo => Write(w => SessionJson.Registers(w, t.VideoRegisters.Current)),
                SessionAbi.TelemetryApu => Write(w => SessionJson.Registers(w, t.ApuRegisters.Current)),
                SessionAbi.TelemetryCoprocessor => Write(w => SessionJson.Registers(w, t.CoprocessorRegisters.Current)),
                SessionAbi.TelemetrySprites => Array(t.Sprites.Current, Sprite),
                SessionAbi.TelemetryPalettes => Array(t.Palettes.Current, Palette),
                SessionAbi.TelemetryAudioChannels => Array(t.AudioChannels.Current, Channel),
                SessionAbi.TelemetryHardwareLoad => Array(t.HardwareLoad.Current, (w, l) =>
                {
                    w.WriteStartObject();
                    w.WriteString("name", l.Name);
                    w.WriteNumber("percent", l.Percent);
                    w.WriteString("kind", l.Kind == DebugLoadKind.GuestUtilization ? "guest_utilization" : "emulator_cost");
                    w.WriteEndObject();
                }),
                _ => throw NotSupported($"no telemetry kind {kind}"),
            };
        }

        private static byte[] Picture((byte[] Rgba, int Width, int Height) picture)
        {
            var bytes = new byte[8 + picture.Rgba.Length];
            BinaryPrimitives.WriteUInt32LittleEndian(bytes, (uint)picture.Width);
            BinaryPrimitives.WriteUInt32LittleEndian(bytes.AsSpan(4), (uint)picture.Height);
            picture.Rgba.CopyTo(bytes, 8);
            return bytes;
        }

        internal byte[] TileSheet(uint session) => Picture(Target(session).RenderTileSheet());

        internal byte[] PaletteSwatch(uint session) => Picture(Target(session).RenderPaletteSwatch());

        internal byte[] AudioPeek(uint session)
        {
            var (samples, rate) = Target(session).GetAudioSamples();
            var bytes = new byte[4 + samples.Length * 2];
            BinaryPrimitives.WriteUInt32LittleEndian(bytes, (uint)rate);
            for (int i = 0; i < samples.Length; i++) BinaryPrimitives.WriteInt16LittleEndian(bytes.AsSpan(4 + i * 2), samples[i]);
            return bytes;
        }

        private string StatePath(string? path)
            => path ?? _options.DefaultStatePath?.Invoke() ?? throw NotSupported("This host gives no default state path.");

        internal void StateSave(uint session, string? path)
        {
            Target(session);
            Action<string> save = _options.SaveState ?? throw NotSupported("This host does not save states.");
            save(StatePath(path));
        }

        internal void StateLoad(uint session, string? path)
        {
            Target(session);
            Action<string> load = _options.LoadState ?? throw NotSupported("This host does not load states.");
            load(StatePath(path));
        }

        internal byte[] StateDefaultPath(uint session)
        {
            Target(session);
            return Utf8(StatePath(null));
        }

        internal static void LogSwitch(bool on) => DebugLogging.MasterEnabled = on;

        internal static bool LogStatus() => DebugLogging.MasterEnabled;

        // Takes the waiting events of the kinds asked, only when the buffer holds them all.
        internal byte[] EventsPeek(uint session, uint kinds, out int taken)
        {
            if (_target is not null || session != 0) Target(session);
            if ((kinds & ~SessionAbi.EventRun) != 0) throw NotSupported($"unknown event kinds 0x{kinds:X}");
            List<SessionEvent> chosen;
            lock (_eventsGate) chosen = _events.Where(e => kinds == 0 || (kinds & SessionAbi.EventRun) != 0 && e.Kind == "run").ToList();
            taken = chosen.Count;
            return Array(chosen, (w, e) =>
            {
                w.WriteStartObject();
                w.WriteString("kind", e.Kind);
                w.WriteString("op", e.Op);
                w.WriteEndObject();
            });
        }

        internal void EventsTake(uint kinds, int count)
        {
            lock (_eventsGate)
            {
                int removed = 0;
                _events.RemoveAll(e => removed < count && (kinds == 0 || e.Kind == "run") && ++removed > 0);
            }
        }
    }
}
