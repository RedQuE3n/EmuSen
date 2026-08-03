using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // Pins an address by undoing every write to it - see `man freeze`.
    // Core-agnostic: fed from the same write-observer seam WatchRegistry uses.
    public class FreezeRegistry
    {
        private sealed class FrozenAddress
        {
            public int Id;
            public string Space = "";
            public int Address;
            public byte Value;
            public long Blocked;
        }

        private readonly List<FrozenAddress> _frozen = new();
        private int _nextId = 1;

        // The restore is itself a write, and must not re-enter NoteWrite.
        private bool _restoring;

        public int Count => _frozen.Count;

        public int Add(string space, int address, byte value)
        {
            var existing = _frozen.FirstOrDefault(f => f.Address == address && string.Equals(f.Space, space, StringComparison.OrdinalIgnoreCase));
            if (existing != null)
            {
                existing.Value = value;
                return existing.Id;
            }
            var entry = new FrozenAddress { Id = _nextId++, Space = space, Address = address, Value = value };
            _frozen.Add(entry);
            return entry.Id;
        }

        public bool Remove(int id) => _frozen.RemoveAll(f => f.Id == id) > 0;

        public void Clear() => _frozen.Clear();

        public IReadOnlyList<(int Id, string Space, int Address, byte Value, long Blocked)> All()
            => _frozen.Select(f => (f.Id, f.Space, f.Address, f.Value, f.Blocked)).ToList();

        // The value to restore, or null if unfrozen - the observer does the writing.
        public byte? NoteWrite(string space, int address, byte written)
        {
            if (_frozen.Count == 0 || _restoring) return null;
            foreach (var entry in _frozen)
            {
                if (entry.Address != address) continue;
                if (!string.Equals(entry.Space, space, StringComparison.OrdinalIgnoreCase)) continue;
                if (entry.Value == written) return null;
                entry.Blocked++;
                return entry.Value;
            }
            return null;
        }

        // Wraps the restoring write so the observer's re-entry is ignored.
        public void Restore(Action write)
        {
            _restoring = true;
            try { write(); }
            finally { _restoring = false; }
        }
    }
}
