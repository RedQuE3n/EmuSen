using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // Named addresses - see `man label`. Core-agnostic: a plain name/address map.
    public class LabelRegistry
    {
        private readonly Dictionary<int, string> _byAddress = new();
        private readonly Dictionary<string, int> _byName = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<int, string> _comments = new();

        public int Count => _byAddress.Count;

        // One-to-one in both directions - see `man label`.
        public void Add(int address, string name, string? comment = null)
        {
            if (_byAddress.TryGetValue(address, out string? existing)) _byName.Remove(existing);
            if (_byName.TryGetValue(name, out int previousAddress))
            {
                _byAddress.Remove(previousAddress);
                _comments.Remove(previousAddress);
            }
            _byAddress[address] = name;
            _byName[name] = address;
            if (!string.IsNullOrEmpty(comment)) _comments[address] = comment!;
        }

        public bool RemoveByName(string name)
        {
            if (!_byName.TryGetValue(name, out int address)) return false;
            _byName.Remove(name);
            _byAddress.Remove(address);
            _comments.Remove(address);
            return true;
        }

        public bool RemoveByAddress(int address)
        {
            if (!_byAddress.TryGetValue(address, out string? name)) return false;
            return RemoveByName(name);
        }

        public void Clear()
        {
            _byAddress.Clear();
            _byName.Clear();
            _comments.Clear();
        }

        public bool TryGetName(int address, out string name) => _byAddress.TryGetValue(address, out name!);

        public bool TryGetAddress(string name, out int address) => _byName.TryGetValue(name, out address);

        public string? CommentAt(int address) => _comments.TryGetValue(address, out string? c) ? c : null;

        // "$00A3B2 <NmiHandler>" when known, plain "$00A3B2" otherwise.
        public string Describe(int address, int digits = 6)
            => TryGetName(address, out string name) ? $"${address.ToString($"X{digits}")} <{name}>" : $"${address.ToString($"X{digits}")}";

        public IReadOnlyList<(int Address, string Name, string? Comment)> All()
            => _byAddress.OrderBy(e => e.Key).Select(e => (e.Key, e.Value, CommentAt(e.Key))).ToList();

        // One "ADDRESS NAME [comment...]" per line - see `man label`.
        public (int Added, IReadOnlyList<string> Errors) LoadFromLines(IEnumerable<string> lines)
        {
            int added = 0;
            var errors = new List<string>();
            int lineNumber = 0;
            foreach (string raw in lines)
            {
                lineNumber++;
                string line = raw.Trim();
                if (line.Length == 0 || line.StartsWith("#")) continue;

                string[] parts = line.Split((char[]?)null, 3, StringSplitOptions.RemoveEmptyEntries);
                if (parts.Length < 2)
                {
                    errors.Add($"line {lineNumber}: expected '<address> <name>', got '{line}'");
                    continue;
                }

                string addressText = parts[0].TrimStart('$');
                if (addressText.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) addressText = addressText.Substring(2);
                if (!int.TryParse(addressText, System.Globalization.NumberStyles.HexNumber, null, out int address))
                {
                    errors.Add($"line {lineNumber}: '{parts[0]}' is not a hex address");
                    continue;
                }

                Add(address, parts[1], parts.Length > 2 ? parts[2] : null);
                added++;
            }
            return (added, errors);
        }

        public IEnumerable<string> ToLines()
            => All().Select(e => e.Comment == null ? $"{e.Address:X6} {e.Name}" : $"{e.Address:X6} {e.Name} {e.Comment}");
    }
}
