using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Shell
{
    // Which mechanism a cheat uses - the two fundamentally different
    // things era cheat devices actually did, see CheatRegistry's own
    // comment for the full explanation of each.
    public enum CheatKind
    {
        RamPoke,
        RomPatch,
    }

    internal sealed class Cheat
    {
        public int Id;
        public CheatKind Kind;

        // RamPoke only - which named memory space to write into.
        public string SpaceName = "";

        // RamPoke: space-relative address. RomPatch: raw 24-bit CPU-bus
        // address, matched exactly as MemoryBus.RomPatcher receives it.
        public int Address;

        public byte Value;

        // RomPatch only - null means unconditional (a 6-character Game
        // Genie code); set means "only patch while the real cartridge
        // byte there equals this" (an 8-character code's compare byte).
        public byte? Compare;

        public string Description = "";
        public bool Enabled = true;
    }

    // One cheat, exposed read-only - same shape convention as
    // WatchRegistry's tuple-returning GetWatches(), but a named struct
    // since there are more fields here worth naming than a tuple reads
    // comfortably.
    public readonly struct CheatInfo
    {
        public int Id { get; }
        public CheatKind Kind { get; }
        public string SpaceName { get; }
        public int Address { get; }
        public byte Value { get; }
        public byte? Compare { get; }
        public string Description { get; }
        public bool Enabled { get; }

        public CheatInfo(int id, CheatKind kind, string spaceName, int address, byte value, byte? compare, string description, bool enabled)
        {
            Id = id;
            Kind = kind;
            SpaceName = spaceName;
            Address = address;
            Value = value;
            Compare = compare;
            Description = description;
            Enabled = enabled;
        }
    }

    // Two fundamentally different mechanisms era cheat devices used,
    // unified behind one registry (one ID space, one list/enable/disable/
    // remove UX) since a player thinks of both as just "my cheats":
    //
    // - RamPoke: the Pro Action Replay/Game Wizard mechanism. Rewrites one
    //   fixed address to one fixed byte every frame, cheaply overpowering
    //   whatever the game itself writes there - a "999 lives" code is
    //   nothing more than "make this address always read back as 0x09".
    //   No ROM patching, no read interception, just a write that keeps
    //   winning. Applied via ApplyAll, called once per frame from outside
    //   this class (see SnesDebugTarget.OnFrame).
    // - RomPatch: the Game Genie mechanism. Fundamentally different -
    //   substitutes the byte a specific *cartridge* read (ROM, or SRAM if
    //   a code is ever pointed there) returns, optionally gated on the
    //   real byte there matching a compare value. Consulted per-read via
    //   TryPatchRom, called from MemoryBus's IRomReadPatcher hook (see
    //   that interface) - not reapplied like a RamPoke, since there's
    //   nothing to "reapply": the substitution happens at read time, the
    //   underlying ROM byte is never actually touched.
    //
    // Core-agnostic on purpose, same reasoning as WatchRegistry: nothing
    // about either mechanism is SNES-specific. A future core wires this up
    // the same way Venus's SnesDebugTarget does - own an instance, call
    // ApplyAll once per frame and implement IRomReadPatcher forwarding to
    // TryPatchRom.
    public class CheatRegistry
    {
        private readonly List<Cheat> _cheats = new();
        private int _nextId = 1;

        public int AddRamPoke(string spaceName, int address, byte value, string description, bool enabled = true)
        {
            var c = new Cheat { Id = _nextId++, Kind = CheatKind.RamPoke, SpaceName = spaceName, Address = address, Value = value, Description = description, Enabled = enabled };
            _cheats.Add(c);
            return c.Id;
        }

        // compare = null means unconditional, matching a 6-character Game
        // Genie code (or a hand-added patch with no compare byte).
        public int AddRomPatch(int address, byte value, byte? compare, string description, bool enabled = true)
        {
            var c = new Cheat { Id = _nextId++, Kind = CheatKind.RomPatch, Address = address, Value = value, Compare = compare, Description = description, Enabled = enabled };
            _cheats.Add(c);
            return c.Id;
        }

        public bool RemoveCheat(int id) => _cheats.RemoveAll(c => c.Id == id) > 0;

        public bool SetEnabled(int id, bool enabled)
        {
            Cheat? c = _cheats.FirstOrDefault(x => x.Id == id);
            if (c is null) return false;
            c.Enabled = enabled;
            return true;
        }

        public void Clear() => _cheats.Clear();

        public IReadOnlyList<CheatInfo> GetCheats()
        {
            return _cheats.Select(c => new CheatInfo(c.Id, c.Kind, c.SpaceName, c.Address, c.Value, c.Compare, c.Description, c.Enabled)).ToList();
        }

        // Called once per frame - re-pokes every enabled RamPoke cheat's
        // value via the supplied write delegate. Deliberately takes a
        // delegate rather than an IDebugTarget/IDebugMemorySpace directly,
        // so this class doesn't need to know how "space name -> writable
        // thing" resolution works for whatever core owns it -
        // SnesDebugTarget resolves through its own GetMemorySpaces(), the
        // same lookup FrameLogRegistry's per-frame callback already uses.
        public void ApplyAll(Action<string, int, byte> write)
        {
            foreach (Cheat c in _cheats)
            {
                if (c.Kind == CheatKind.RamPoke && c.Enabled) write(c.SpaceName, c.Address, c.Value);
            }
        }

        // Called for every cartridge-routed read - see IRomReadPatcher.
        // First enabled, address-matching, compare-satisfying RomPatch
        // wins (list order = add order); returns false if nothing matches
        // so the caller falls back to the real cartridge byte unchanged.
        public bool TryPatchRom(uint address, byte originalValue, out byte patchedValue)
        {
            foreach (Cheat c in _cheats)
            {
                if (c.Kind != CheatKind.RomPatch || !c.Enabled) continue;
                if (c.Address != (int)address) continue;
                if (c.Compare.HasValue && c.Compare.Value != originalValue) continue;

                patchedValue = c.Value;
                return true;
            }

            patchedValue = 0;
            return false;
        }
    }
}
