namespace EmuSen.Crystal.Systems.SegaDisc
{
    // The Saturn's and the Dreamcast's system packs, consoles only hosted cores run: their systems-table entries, the recogniser of their discs' system area the library's; no codec, no medium - see EmuSen_Cores.md §4.9.
    public static class SegaDiscSystems
    {
        public const string SaturnId = "saturn", DreamcastId = "dreamcast";

        public static readonly SystemEntry Saturn = SystemTable.Entry(SaturnId), Dreamcast = SystemTable.Entry(DreamcastId);

        public static readonly SystemPack SaturnPack = new(Saturn, null, null), DreamcastPack = new(Dreamcast, null, null);
    }
}
