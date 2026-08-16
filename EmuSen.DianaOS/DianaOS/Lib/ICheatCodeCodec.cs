using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Lib
{
    // What a decoded code becomes, mapping onto CheatKind - see EmuSen_Cheats.md.
    public enum CheatCodeKind
    {
        RamPoke,
        RomPatch,
    }

    // One format's decoder, injected so CheatCommand references no core - see §3.3b.
    public interface ICheatCodeCodec
    {
        // Human-readable name for `add`'s own "(detected X format)" message.
        string Name { get; }

        // True if <code> looks like this codec's format at all.
        bool CanDecode(string code);

        CheatCodeKind Kind { get; }

        // The space a RamPoke targets; unused and possibly null for a RomPatch.
        string? SpaceName { get; }

        (int Address, byte Value) Decode(string code);

        // Dropping a compare would make an NES code wrong, not lesser - see EmuSen_Cheats.md §2.
        byte? DecodeCompare(string code) => null;
    }
}
