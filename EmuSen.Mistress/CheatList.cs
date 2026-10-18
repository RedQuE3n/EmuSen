using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;

namespace EmuSen.Mistress
{
    // A cheat as the windows list it, its writes formatted as `cheat list` prints them.
    public sealed record CheatLine(int Id, bool RomPatch, string Description, bool Enabled, string Detail, int Writes);

    // The cheats the windows show and the game runs: the running session's own, through cheat_op, or with no game a list held here, which the next game starts with - see EmuSen_Backend.md §14.13.
    public sealed class CheatList
    {
        private CrystalSession? _session;
        private CheatFile _held = new();
        private bool _heldMaster = true;

        // Whether a session holds the list, so a code can be decoded and a change reaches the game.
        public bool Live => _session is not null;

        // The session's list from now on, starting with what was held.
        public void Attach(CrystalSession session)
        {
            _session = session;
            if (_held.Cheats.Count > 0) Import(_held);
            Master = _heldMaster;
            _held = new CheatFile();
        }

        // The session's list kept here as it closes, for the next game to start with.
        public void Detach()
        {
            if (_session is { } s)
            {
                try
                {
                    _held = ToFile();
                    _heldMaster = Master;
                }
                catch (CrystalException) { }
            }
            _session = null;
        }

        private string Op(uint op, object? args = null) =>
            _session!.CheatOp(op, args is null ? "{}" : JsonSerializer.Serialize(args));

        private long Change(uint op, object? args = null) =>
            _session!.CheatChange(op, args is null ? "{}" : JsonSerializer.Serialize(args));

        public IReadOnlyList<CheatLine> Cheats
        {
            get
            {
                if (_session is null) return _held.Cheats.Select(Held).ToArray();
                using JsonDocument list = JsonDocument.Parse(Op(CrystalSessionAbi.CheatList));
                return list.RootElement.GetProperty("cheats").EnumerateArray().Select(c =>
                {
                    bool rom = c.GetProperty("kind").GetString() == "rom";
                    JsonElement[] writes = c.GetProperty("writes").EnumerateArray().ToArray();
                    string detail = writes.Length == 1 ? Format(Write.Of(writes[0]), rom) : $"{writes.Length} writes";
                    return new CheatLine(c.GetProperty("id").GetInt32(), rom, c.GetProperty("description").GetString() ?? "", c.GetProperty("enabled").GetBoolean(), detail, writes.Length);
                }).ToArray();
            }
        }

        private static CheatLine Held(CheatFileEntry e, int index)
        {
            IReadOnlyList<CheatFileWrite> writes = e.EffectiveWrites;
            string detail = writes.Count == 1 && Write.Of(writes[0]) is { } one ? Format(one, e.IsRomPatch) : $"{writes.Count} writes";
            return new CheatLine(index, e.IsRomPatch, e.Description, e.Enabled, detail, writes.Count);
        }

        public bool Master
        {
            get
            {
                if (_session is null) return _heldMaster;
                using JsonDocument master = JsonDocument.Parse(Op(CrystalSessionAbi.CheatMaster));
                return master.RootElement.GetProperty("enabled").GetBoolean();
            }
            set
            {
                if (_session is null) _heldMaster = value;
                else Change(CrystalSessionAbi.CheatMaster, new { enabled = value });
            }
        }

        public void SetEnabled(int id, bool on)
        {
            if (_session is null)
            {
                if (id >= 0 && id < _held.Cheats.Count) _held.Cheats[id].Enabled = on;
                return;
            }
            Change(on ? CrystalSessionAbi.CheatEnable : CrystalSessionAbi.CheatDisable, new { id });
        }

        public void Remove(int id)
        {
            if (_session is null)
            {
                if (id >= 0 && id < _held.Cheats.Count) _held.Cheats.RemoveAt(id);
                return;
            }
            Change(CrystalSessionAbi.CheatRemove, new { id });
        }

        public void Clear()
        {
            if (_session is null) _held = new CheatFile();
            else Change(CrystalSessionAbi.CheatClear);
        }

        // As the game's saved list writes it.
        public CheatFile ToFile() =>
            _session is null ? Copy(_held) : JsonSerializer.Deserialize<CheatFile>(Op(CrystalSessionAbi.CheatExport), ConfigJson.Options) ?? new CheatFile();

        private static CheatFile Copy(CheatFile file) =>
            JsonSerializer.Deserialize<CheatFile>(JsonSerializer.Serialize(file, ConfigJson.Options), ConfigJson.Options) ?? new CheatFile();

        // Added to what is there; how many were taken and how many could not be read.
        public (int Loaded, int Skipped) Import(CheatFile file)
        {
            if (_session is null)
            {
                _held.Cheats.AddRange(Copy(file).Cheats);
                return (file.Cheats.Count, 0);
            }
            int loaded = (int)_session.CheatChange(CrystalSessionAbi.CheatImport, JsonSerializer.Serialize(file, ConfigJson.Options));
            return (loaded, file.Cheats.Count - loaded);
        }

        // In place of what is there, so what is shown is what the file says.
        public (int Loaded, int Skipped) Replace(CheatFile file)
        {
            Clear();
            return Import(file);
        }

        public int Count => _session is null ? _held.Cheats.Count : Cheats.Count;

        // The running game's codecs, the way a code is read: {"auto","explicit"}, each {"name","kind","space"} or null.
        public (string? Auto, string? Explicit) CodecNames()
        {
            if (_session is null) return (null, null);
            using JsonDocument codecs = JsonDocument.Parse(_session.CheatCodecs());
            string? Name(string key) => codecs.RootElement.GetProperty(key) is { ValueKind: JsonValueKind.Object } c ? c.GetProperty("name").GetString() : null;
            return (Name("auto"), Name("explicit"));
        }

        // A typed code decoded by the running game's codecs and added enabled; the codec's name, or the words why not.
        public (bool Added, string Words) AddCode(string code, string description)
        {
            if (_session is null) return (false, "Start a game to add a code: a code is read by the running game's codecs.");
            var (auto, explicit_) = CodecNames();
            if (auto is null && explicit_ is null) return (false, "This console has no cheat-code decoder in this build.");
            string codec = PrefersExplicit(code, auto is not null, explicit_ is not null) ? "explicit" : "auto";
            string name = (codec == "explicit" ? explicit_ : auto)!;
            JsonNode decoded;
            // Unchecked, as `cheat add` decodes: a code no codec claims gets the codec's own words for what is wrong with it.
            try { decoded = JsonNode.Parse(Op(CrystalSessionAbi.CheatDecode, new { code, codec, @unchecked = true }))!; }
            catch (CrystalException e) { return (false, $"Couldn't decode '{code}' as {name}: {e.Message}"); }
            bool rom = (decoded["kind"]?.GetValue<string>() ?? (codec == "explicit" ? "rom" : "ram")) == "rom";
            try
            {
                if (decoded["writes"] is JsonArray writes)
                    _session.CheatChange(CrystalSessionAbi.CheatAdd, new JsonObject { ["kind"] = rom ? "rom" : "ram", ["writes"] = writes.DeepClone(), ["compare"] = decoded["compare"]?.DeepClone(), ["description"] = description, ["enabled"] = true }.ToJsonString());
                else if (rom)
                    _session.CheatChange(CrystalSessionAbi.CheatRomPatch, new JsonObject { ["address"] = decoded["address"]?.DeepClone(), ["value"] = decoded["value"]?.DeepClone(), ["compare"] = decoded["compare"]?.DeepClone(), ["description"] = description, ["enabled"] = true }.ToJsonString());
                else
                    _session.CheatChange(CrystalSessionAbi.CheatPoke, new JsonObject { ["space"] = decoded["space"]?.DeepClone(), ["address"] = decoded["address"]?.DeepClone(), ["value"] = decoded["value"]?.DeepClone(), ["description"] = description, ["enabled"] = true }.ToJsonString());
            }
            catch (CrystalException e) { return (false, $"Couldn't add '{code}' as {name}: {e.Message}"); }
            return (true, name);
        }

        // The explicit codec where it alone claims the code, or both or neither do and it is written as a Game Genie code is - `cheat add`'s rule.
        private bool PrefersExplicit(string code, bool hasAuto, bool hasExplicit)
        {
            if (!hasAuto || !hasExplicit) return hasExplicit;
            using JsonDocument claims = JsonDocument.Parse(Op(CrystalSessionAbi.CheatClaims, new { code }));
            bool Claims(string key) => claims.RootElement.GetProperty(key) is { ValueKind: JsonValueKind.True };
            bool explicitClaims = Claims("explicit"), autoClaims = Claims("auto");
            return explicitClaims != autoClaims ? explicitClaims : EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.CheatCommand.LooksLikeGameGenieFormat(code);
        }

        // A write as the list's words show it, whichever form it came in.
        private sealed record Write(string Space, long Address, uint Value, int Width, string Type, int? Bit, bool BigEndian, int Repeat, long RepeatAddress, uint RepeatValue)
        {
            public static Write Of(JsonElement w) => new(w.GetProperty("space").GetString() ?? "", w.GetProperty("address").GetInt64(), w.GetProperty("value").GetUInt32(),
                w.GetProperty("width").GetInt32(), w.GetProperty("type").GetString() ?? "Set", w.GetProperty("bit") is { ValueKind: JsonValueKind.Number } b ? b.GetInt32() : null,
                w.GetProperty("big_endian").GetBoolean(), w.GetProperty("repeat").GetInt32(), w.GetProperty("repeat_address").GetInt64(), w.GetProperty("repeat_value").GetUInt32());

            public static Write? Of(CheatFileWrite w) =>
                w.TryParseNumbers(out uint address, out uint value, out uint repeatAddress, out uint repeatValue)
                    ? new Write(w.Space, address, value, w.Width, w.Type, w.BitPosition, w.BigEndian, w.RepeatCount, repeatAddress, repeatValue) : null;
        }

        private static string Format(Write? w, bool romPatch)
        {
            if (w is null) return "unreadable write";
            string where = romPatch ? $"0x{w.Address:X6}" : $"{w.Space} 0x{w.Address:X}";
            string op = w.Type.ToLowerInvariant() switch
            {
                "increase" => "+=",
                "decrease" => "-=",
                "ifequal" => "==",
                "ifnotequal" => "!=",
                "ifless" => "<",
                "ifgreater" => ">",
                _ => "=",
            };
            if (w.Type.StartsWith("If", StringComparison.OrdinalIgnoreCase)) where = "if " + where;
            if (w.Bit is int bit) return $"{where} bit{bit} {op} {w.Value & 1}";
            int width = Math.Max(1, w.Width);
            string value = $"0x{w.Value.ToString("X" + width * 2)}";
            string wide = width > 1 ? $" ({width}-byte{(w.BigEndian ? ", big-endian" : "")})" : "";
            string repeat = w.Repeat > 1 ? $" x{w.Repeat} step 0x{w.RepeatAddress:X}" + (w.RepeatValue != 0 ? $"/+0x{w.RepeatValue:X}" : "") : "";
            return $"{where} {op} {value}{wide}{repeat}";
        }
    }
}
