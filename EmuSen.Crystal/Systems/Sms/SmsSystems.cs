using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.Sms
{
    // The Master System's and the SG-1000's system packs, one extension each, which share the two cheat formats and the debugger's views - see Endou_Plan.md §4, Endou_Native.md §3, §14 and §15.
    public static class SmsSystems
    {
        public const string MasterSystemId = "sms", Sg1000Id = "sg1000";

        // The systems table's rows - see EmuSen_Backend.md §12.
        public static readonly SystemEntry MasterSystem = SystemTable.Entry(MasterSystemId), Sg1000 = SystemTable.Entry(Sg1000Id);

        // The space a Pro Action Replay code writes, as Endou names the work RAM.
        public const string Ram = "RAM";

        // Each codec is emusen-systems' under its id - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec ProActionReplay() => SystemCode.Codec("sms.par");

        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("sms.gg");

        public static readonly SystemPack MasterSystemPack = new(MasterSystem, ProActionReplay, GameGenie);
        public static readonly SystemPack Sg1000Pack = new(Sg1000, ProActionReplay, GameGenie);
    }
}
