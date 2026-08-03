namespace EmuSen.Cores.Nintendo.Moon.Memory
{
    // The watchpoint seam, deliberately a twin of Venus's rather than shared - see Moon_Memory.md §6.
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);
    }
}
