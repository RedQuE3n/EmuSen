using System;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // Every core's debugger memory space: a live size and a read and a write; the edge rule is one for all - see EmuSen_Settings_Reference.md §4.85.7.
    public sealed class DelegateDebugMemorySpace : IDebugMemorySpace
    {
        private readonly Func<int> _size;
        private readonly Func<int, byte> _read;
        private readonly Action<int, byte>? _write;
        private readonly bool _wrap;

        // No write is a read-only space; wrap false passes every address through, for a bus or a processor's view.
        public DelegateDebugMemorySpace(string name, Func<int> size, Func<int, byte> read, Action<int, byte>? write = null, bool hasSideEffects = false, bool wrap = true)
        {
            Name = name;
            _size = size;
            _read = read;
            _write = write;
            HasSideEffects = hasSideEffects;
            _wrap = wrap;
        }

        // A space of fixed size.
        public DelegateDebugMemorySpace(string name, int size, Func<int, byte> read, Action<int, byte>? write = null, bool hasSideEffects = false, bool wrap = true)
            : this(name, () => size, read, write, hasSideEffects, wrap) { }

        // A plain array, and so never a side effect; an empty one reads zero and keeps nothing.
        public static DelegateDebugMemorySpace Over(string name, byte[] data, bool isWritable = true) =>
            new(name, data.Length, a => data.Length == 0 ? (byte)0 : data[a], isWritable ? (a, v) => { if (data.Length > 0) data[a] = v; } : null);

        public string Name { get; }
        public int Size => _size();
        public bool IsWritable => _write != null;
        public bool HasSideEffects { get; }

        // Reads and writes both wrap modulo the size, negatives included; a space of size zero sends address zero.
        public byte Read(int address) => _read(Wrap(address));

        public void Write(int address, byte value) => _write?.Invoke(Wrap(address), value);

        private int Wrap(int address)
        {
            if (!_wrap) return address;
            int size = _size();
            return size <= 0 ? 0 : ((address % size) + size) % size;
        }
    }
}
