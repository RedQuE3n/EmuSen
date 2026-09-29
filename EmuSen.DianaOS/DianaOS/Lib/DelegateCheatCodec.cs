using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // One format's decoder as data: its name, kind and space, and the static decoder's functions - see EmuSen_Settings_Reference.md §4.85.7.
    public sealed class DelegateCheatCodec : ICheatCodeCodec
    {
        private readonly Func<string, bool> _canDecode;
        private readonly Func<string, (int Address, byte Value)> _decode;
        private readonly Func<string, byte?>? _decodeCompare;
        private readonly Func<string, IReadOnlyList<CheatWrite>?>? _decodeWrites;

        public DelegateCheatCodec(string name, CheatCodeKind kind, string? spaceName, Func<string, bool> canDecode, Func<string, (int Address, byte Value)> decode,
            Func<string, byte?>? decodeCompare = null, Func<string, IReadOnlyList<CheatWrite>?>? decodeWrites = null)
        {
            Name = name;
            Kind = kind;
            SpaceName = spaceName;
            _canDecode = canDecode;
            _decode = decode;
            _decodeCompare = decodeCompare;
            _decodeWrites = decodeWrites;
        }

        public string Name { get; }
        public CheatCodeKind Kind { get; }
        public string? SpaceName { get; }

        public bool CanDecode(string code) => _canDecode(code);
        public (int Address, byte Value) Decode(string code) => _decode(code);
        public byte? DecodeCompare(string code) => _decodeCompare?.Invoke(code);
        public IReadOnlyList<CheatWrite>? DecodeWrites(string code) => _decodeWrites?.Invoke(code);
    }
}
