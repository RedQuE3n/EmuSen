using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems
{
    // A pack's codec answered by emusen-systems under the codec's id; a lone surrogate crosses as U+FFFD - see EmuSen_Backend.md §13, §40.1.
    public sealed class NativeCheatCodec : ICheatCodeCodec
    {
        private sealed record Answer(string Code, bool Can, JsonElement Decode, JsonElement Compare, JsonElement Writes);

        private readonly SystemCode.CodecFacts _facts;
        private Answer? _last;

        public NativeCheatCodec(string id)
        {
            Id = id;
            _facts = SystemCode.Codecs.FirstOrDefault(c => c.Id == id) ?? throw new ArgumentException($"the library has no codec {id}", nameof(id));
        }

        public string Id { get; }

        public string Name => _facts.Name;
        public CheatCodeKind Kind => _facts.Kind;
        public string? SpaceName => _facts.Space;

        // One call answers all four questions; the last code's answers are kept, as a caller asks CanDecode and then decodes the same code.
        private Answer Ask(string code)
        {
            ArgumentNullException.ThrowIfNull(code);
            if (_last is { } last && last.Code == code) return last;
            using JsonDocument document = SystemCode.CheatDecode(Id, code);
            JsonElement r = document.RootElement;
            var answer = new Answer(code, r.GetProperty("can").GetBoolean(), r.GetProperty("decode").Clone(), r.GetProperty("compare").Clone(), r.GetProperty("writes").Clone());
            _last = answer;
            return answer;
        }

        // A refusal as the C# codec threw it: its FormatException's words, or the index it ran past.
        private static void Refused(JsonElement e)
        {
            if (e.ValueKind != JsonValueKind.Object) return;
            if (e.TryGetProperty("format", out JsonElement words)) throw new FormatException(words.GetString());
            if (e.TryGetProperty("range", out _)) throw new ArgumentOutOfRangeException("index", "Index was out of range. Must be non-negative and less than the size of the collection.");
        }

        public bool CanDecode(string code) => Ask(code).Can;

        public (int Address, byte Value) Decode(string code)
        {
            JsonElement d = Ask(code).Decode;
            Refused(d);
            return (d.GetProperty("address").GetInt32(), d.GetProperty("value").GetByte());
        }

        public byte? DecodeCompare(string code)
        {
            JsonElement c = Ask(code).Compare;
            if (c.ValueKind == JsonValueKind.Null) return null;
            Refused(c);
            return c.GetByte();
        }

        public IReadOnlyList<CheatWrite>? DecodeWrites(string code)
        {
            JsonElement w = Ask(code).Writes;
            if (w.ValueKind == JsonValueKind.Null) return null;
            Refused(w);
            return w.EnumerateArray().Select(x => new CheatWrite
            {
                Space = x.GetProperty("space").GetString()!,
                Address = x.GetProperty("address").GetInt32(),
                Value = x.GetProperty("value").GetUInt32(),
                Width = x.GetProperty("width").GetInt32(),
                Type = (CheatWriteType)x.GetProperty("type").GetInt32(),
                BitPosition = x.GetProperty("bit").ValueKind == JsonValueKind.Null ? null : x.GetProperty("bit").GetInt32(),
                BigEndian = x.GetProperty("big_endian").GetBoolean(),
                RepeatCount = x.GetProperty("repeat").GetInt32(),
                RepeatAddAddress = x.GetProperty("repeat_address").GetInt32(),
                RepeatAddValue = x.GetProperty("repeat_value").GetUInt32(),
            }).ToList();
        }
    }
}
