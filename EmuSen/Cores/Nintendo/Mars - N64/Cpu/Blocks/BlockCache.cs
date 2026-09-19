using System.Threading;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Blocks
{
    // The blocks by physical word address, a page of entries at a time - see Mars_Recompiler.md §2.
    public sealed class BlockCache
    {
        // Interpreted runs before a block is compiled, chosen from the sweep in Mars_Recompiler.md §7.
        public static int Threshold = 64;

        public const int MaxLength = 64;

        private const int PageShift = 12;
        private const int PageWords = 1 << (PageShift - 2);

        private readonly Block?[]?[] _pages;

        // Written by the compiler's thread as well as the processor's, so read through the properties - see Mars_Recompiler.md §2.4.
        private long _compiled;
        private long _compileTicks;

        public long Discarded;

        // Cold entries whose words changed shape before they were hot - see Mars_Recompiler.md §2.3.
        public long Reshaped;

        public BlockCache(int rdramLength)
        {
            _pages = new Block?[]?[(rdramLength + (1 << PageShift) - 1) >> PageShift];
        }

        public long Compiled => Interlocked.Read(ref _compiled);

        public long CompileTicks => Interlocked.Read(ref _compileTicks);

        internal void NoteCompiled(long ticks)
        {
            Interlocked.Increment(ref _compiled);
            Interlocked.Add(ref _compileTicks, ticks);
        }

        internal Block? Find(uint physical)
        {
            Block?[]? page = _pages[physical >> PageShift];
            return page?[(physical >> 2) & (PageWords - 1)];
        }

        internal void Place(Block block)
        {
            ref Block?[]? page = ref _pages[block.Physical >> PageShift];
            page ??= new Block?[PageWords];
            page[(block.Physical >> 2) & (PageWords - 1)] = block;
        }
    }
}
