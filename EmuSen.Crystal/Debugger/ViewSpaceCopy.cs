using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Crystal.Debugger
{
    // The spaces a view reads, learnt once by watching it decode, copied each frame into a buffer and offered back over the copy - see EmuSen_Backend.md §14.5.
    public sealed class ViewSpaceCopy
    {
        public delegate void Reader(int space, Span<byte> into);

        public static readonly ViewSpaceCopy None = new(Array.Empty<(int, int, int)>());

        private readonly (int Index, int Offset, int Size)[] _copied;

        private ViewSpaceCopy((int Index, int Offset, int Size)[] copied)
        {
            _copied = copied;
            Total = copied.Length == 0 ? 0 : copied[^1].Offset + copied[^1].Size;
        }

        // How many bytes a frame's copy takes.
        public int Total { get; }

        // The spaces without side effects that the view read while decoding them once.
        public static ViewSpaceCopy Learn(IReadOnlyList<IDebugMemorySpace> spaces, Action<IReadOnlyList<IDebugMemorySpace>> view)
        {
            var touched = new bool[spaces.Count];
            view(spaces.Select((s, i) => (IDebugMemorySpace)new DelegateDebugMemorySpace(s.Name, s.Size, a => { touched[i] = true; return s.Read(a); }, null, s.HasSideEffects)).ToArray());
            var copied = new List<(int, int, int)>();
            int offset = 0;
            for (int i = 0; i < spaces.Count; i++)
            {
                if (!touched[i] || spaces[i].HasSideEffects || spaces[i].Size == 0) continue;
                copied.Add((i, offset, spaces[i].Size));
                offset += spaces[i].Size;
            }
            return new ViewSpaceCopy(copied.ToArray());
        }

        // The learnt spaces' bytes into a buffer of Total, allocating nothing.
        public void Fill(Span<byte> into, Reader read)
        {
            foreach (var (index, offset, size) in _copied) read(index, into.Slice(offset, size));
        }

        // The spaces with each learnt one read from the copy; another is the live one, as the view did not read it when learnt.
        public IReadOnlyList<IDebugMemorySpace> Over(IReadOnlyList<IDebugMemorySpace> live, byte[] copy)
        {
            var spaces = live.ToArray();
            foreach (var (index, offset, size) in _copied)
                if (index < spaces.Length && offset + size <= copy.Length)
                    spaces[index] = new DelegateDebugMemorySpace(spaces[index].Name, size, a => copy[offset + a]);
            return spaces;
        }
    }
}
