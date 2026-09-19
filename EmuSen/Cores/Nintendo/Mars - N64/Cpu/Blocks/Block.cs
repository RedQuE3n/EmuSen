namespace EmuSen.Cores.Nintendo.Mars.Cpu.Blocks
{
    internal delegate void BlockCode(Core.Cpu cpu);

    // A run of instructions at one address: interpreted until it is hot, then compiled from the bytes it keeps - see Mars_Recompiler.md §2.
    internal sealed class Block
    {
        public readonly uint Physical;
        public readonly int Length;

        // A shape the compiler does not take, which the interpreter runs every time - see Mars_Recompiler.md §2.2.
        public readonly bool Refused;

        public int Runs;
        public byte[]? Image;
        public BlockCode? Code;

        public Block(uint physical, int length, bool refused)
        {
            Physical = physical;
            Length = length;
            Refused = refused;
        }
    }
}
