using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Debug
{
    internal sealed class Cheat
    {
        public int Id;
        public string SpaceName = "";
        public int Address;
        public byte Value;
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
        public string SpaceName { get; }
        public int Address { get; }
        public byte Value { get; }
        public string Description { get; }
        public bool Enabled { get; }

        public CheatInfo(int id, string spaceName, int address, byte value, string description, bool enabled)
        {
            Id = id;
            SpaceName = spaceName;
            Address = address;
            Value = value;
            Description = description;
            Enabled = enabled;
        }
    }

    // RAM-poke cheat engine - the mechanism behind era devices like Pro
    // Action Replay and Game Wizard, which don't patch ROM or intercept
    // reads the way Game Genie does (see the Game Genie codec/engine once
    // that exists - a separate, later addition). A RAM-poke cheat just
    // rewrites one fixed address to one fixed byte every frame, cheaply
    // overpowering whatever the game itself writes there - a "999 lives"
    // code is nothing more than "make this address always read back as
    // 0x09", reapplied often enough that the game's own decrement never
    // sticks.
    //
    // Core-agnostic on purpose, same reasoning as WatchRegistry: nothing
    // about "poke this named memory space's address to this byte every
    // frame" is SNES-specific. A future core wires this up the same way
    // Venus's SnesDebugTarget does - own an instance, apply it once per
    // frame via its own GetMemorySpaces() lookup.
    public class CheatRegistry
    {
        private readonly List<Cheat> _cheats = new();
        private int _nextId = 1;

        public int AddCheat(string spaceName, int address, byte value, string description, bool enabled = true)
        {
            var c = new Cheat { Id = _nextId++, SpaceName = spaceName, Address = address, Value = value, Description = description, Enabled = enabled };
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
            return _cheats.Select(c => new CheatInfo(c.Id, c.SpaceName, c.Address, c.Value, c.Description, c.Enabled)).ToList();
        }

        // Called once per frame - re-pokes every enabled cheat's value via
        // the supplied write delegate. Deliberately takes a delegate
        // rather than an IDebugTarget/IDebugMemorySpace directly, so this
        // class doesn't need to know how "space name -> writable thing"
        // resolution works for whatever core owns it - SnesDebugTarget
        // resolves through its own GetMemorySpaces(), the same lookup
        // FrameLogRegistry's per-frame callback already uses.
        public void ApplyAll(Action<string, int, byte> write)
        {
            foreach (Cheat c in _cheats)
            {
                if (c.Enabled) write(c.SpaceName, c.Address, c.Value);
            }
        }
    }
}
