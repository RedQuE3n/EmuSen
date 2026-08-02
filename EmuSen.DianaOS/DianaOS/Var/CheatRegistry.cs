using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Galaxia.Models;

namespace EmuSen.DianaOS.DianaOS.Var
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

        // One cheat, one toggle, however many writes - see `man cheat`.
        public List<CheatWrite> Writes = new();

        // RomPatch only, and single-byte only - see CheatRegistry.AddCheat.
        public byte? Compare;

        public string Description = "";
        public bool Enabled = true;
    }

    // One cheat, exposed read-only.
    public readonly struct CheatInfo
    {
        public int Id { get; }
        public CheatKind Kind { get; }
        public IReadOnlyList<CheatWrite> Writes { get; }
        public byte? Compare { get; }
        public string Description { get; }
        public bool Enabled { get; }

        public CheatInfo(int id, CheatKind kind, IReadOnlyList<CheatWrite> writes, byte? compare, string description, bool enabled)
        {
            Id = id;
            Kind = kind;
            Writes = writes;
            Compare = compare;
            Description = description;
            Enabled = enabled;
        }

        // The first write's fields, for the many callers that only ever
        // deal in single-write cheats.
        public string SpaceName => Writes.Count > 0 ? Writes[0].Space : "";
        public int Address => Writes.Count > 0 ? Writes[0].Address : 0;
        public uint Value => Writes.Count > 0 ? Writes[0].Value : 0;
    }

    // Two fundamentally different mechanisms era cheat devices used,
    // unified behind one registry (one ID space, one list/enable/disable/
    // remove UX) since a player thinks of both as just "my cheats":
    //
    // - RamPoke: the Pro Action Replay/Game Wizard mechanism. Rewrites an
    //   address every frame, cheaply overpowering whatever the game itself
    //   writes there. Applied via ApplyAll, called once per frame from
    //   outside this class (see SnesDebugTarget.OnFrame).
    // - RomPatch: the Game Genie mechanism. Substitutes the byte a
    //   specific *cartridge* read returns, optionally gated on the real
    //   byte matching a compare value. Consulted per-read via TryPatchRom
    //   from MemoryBus's IRomReadPatcher hook - the underlying ROM byte is
    //   never actually touched.
    //
    // Core-agnostic on purpose: nothing about either mechanism is
    // SNES-specific, and this class never touches memory itself - it is
    // handed read/write delegates. A future core wires this up the same
    // way Venus's SnesDebugTarget does. See `man cheat` for the write
    // model (widths, byte order, repeat runs, bit positions).
    public class CheatRegistry
    {
        private readonly List<Cheat> _cheats = new();
        private int _nextId = 1;

        // TryPatchRom runs on every cartridge-routed read, so the common
        // "no ROM patches active" case must not walk the list at all.
        private int _enabledRomPatches;

        public int AddRamPoke(string spaceName, int address, byte value, string description, bool enabled = true) =>
            AddCheat(CheatKind.RamPoke, new[] { CheatWrite.Poke(spaceName, address, value) }, null, description, enabled);

        // compare = null means unconditional, matching a 6-character Game
        // Genie code (or a hand-added patch with no compare byte).
        public int AddRomPatch(int address, byte value, byte? compare, string description, bool enabled = true) =>
            AddCheat(CheatKind.RomPatch, new[] { CheatWrite.Patch(address, value) }, compare, description, enabled);

        // The general form. Throws on the two combinations that cannot be
        // honestly implemented rather than silently doing something else:
        //
        // - A ROM patch cannot Increase/Decrease. There is nothing to
        //   accumulate - the substitution happens at read time and the
        //   real byte is never written, so "add 1 each frame" has no
        //   meaning.
        // - A compare byte only works on a single-byte patch. TryPatchRom
        //   is handed one byte at a time, so it cannot check a compare
        //   that spans several addresses; allowing it would produce a torn
        //   patch where the first byte declines and the rest apply.
        public int AddCheat(CheatKind kind, IEnumerable<CheatWrite> writes, byte? compare, string description, bool enabled = true)
        {
            var list = writes.ToList();
            if (list.Count == 0) throw new ArgumentException("A cheat needs at least one write.", nameof(writes));

            if (kind == CheatKind.RomPatch)
            {
                if (list.Any(w => w.Type != CheatWriteType.Set))
                {
                    throw new ArgumentException("A ROM patch can only set a value - increase/decrease needs somewhere to accumulate, and a ROM read substitution has none.");
                }
                if (compare.HasValue && list.Any(w => w.EffectiveWidth != 1))
                {
                    throw new ArgumentException("A compare byte only applies to a single-byte ROM patch.");
                }
            }

            var cheat = new Cheat
            {
                Id = _nextId++,
                Kind = kind,
                Writes = list,
                Compare = compare,
                Description = description,
                Enabled = enabled,
            };
            _cheats.Add(cheat);
            RecountRomPatches();
            return cheat.Id;
        }

        public bool RemoveCheat(int id)
        {
            bool removed = _cheats.RemoveAll(c => c.Id == id) > 0;
            if (removed) RecountRomPatches();
            return removed;
        }

        public bool SetEnabled(int id, bool enabled)
        {
            Cheat? c = _cheats.FirstOrDefault(x => x.Id == id);
            if (c is null) return false;
            c.Enabled = enabled;
            RecountRomPatches();
            return true;
        }

        public void Clear()
        {
            _cheats.Clear();
            RecountRomPatches();
        }

        private void RecountRomPatches() =>
            _enabledRomPatches = _cheats.Count(c => c.Kind == CheatKind.RomPatch && c.Enabled);

        public IReadOnlyList<CheatInfo> GetCheats() =>
            _cheats.Select(c => new CheatInfo(c.Id, c.Kind, c.Writes, c.Compare, c.Description, c.Enabled)).ToList();

        // Snapshot for etc/EmuSen/cheats/<name>.json - see EmuSen_Config_Reference.md §3.4.
        public CheatFile ToCheatFile()
        {
            var file = new CheatFile();
            foreach (Cheat c in _cheats)
            {
                bool isRomPatch = c.Kind == CheatKind.RomPatch;
                file.Cheats.Add(new CheatFileEntry
                {
                    Kind = isRomPatch ? "RomPatch" : "RamPoke",
                    Compare = c.Compare?.ToString("X2"),
                    Description = c.Description,
                    Enabled = c.Enabled,
                    Writes = c.Writes.Select(w => new CheatFileWrite
                    {
                        Space = w.Space,
                        Address = CheatFileWrite.Hex((uint)w.Address, isRomPatch ? 6 : 1),
                        Value = CheatFileWrite.Hex(w.Value, w.EffectiveWidth * 2),
                        Width = w.EffectiveWidth,
                        Type = w.Type.ToString(),
                        BitPosition = w.BitPosition,
                        BigEndian = w.BigEndian,
                        RepeatCount = w.EffectiveRepeatCount,
                        RepeatAddAddress = CheatFileWrite.Hex((uint)w.RepeatAddAddress, 1),
                        RepeatAddValue = CheatFileWrite.Hex(w.RepeatAddValue, 1),
                    }).ToList(),
                });
            }
            return file;
        }

        // Adds every well-formed entry and returns how many, plus how many were
        // skipped as unparseable - a hand-edited file with one bad line still
        // loads the rest, see EmuSen_Config_Reference.md §3.4.
        public (int loaded, int skipped) LoadFrom(CheatFile file)
        {
            int loaded = 0, skipped = 0;
            foreach (CheatFileEntry e in file.Cheats)
            {
                if (!TryReadEntry(e, out List<CheatWrite> writes, out byte? compare))
                {
                    skipped++;
                    continue;
                }

                try
                {
                    AddCheat(e.IsRomPatch ? CheatKind.RomPatch : CheatKind.RamPoke, writes, compare, e.Description, e.Enabled);
                    loaded++;
                }
                catch (ArgumentException)
                {
                    skipped++;
                }
            }
            return (loaded, skipped);
        }

        // Maps the on-disk carrier onto CheatWrite. Lives here rather than on
        // CheatFileEntry because EmuSen.Galaxia cannot see this assembly.
        private static bool TryReadEntry(CheatFileEntry entry, out List<CheatWrite> writes, out byte? compare)
        {
            writes = new List<CheatWrite>();
            if (!entry.TryParseCompare(out compare)) return false;

            IReadOnlyList<CheatFileWrite> source = entry.EffectiveWrites;
            if (source.Count == 0) return false;

            foreach (CheatFileWrite w in source)
            {
                if (!w.TryParseNumbers(out uint address, out uint value, out uint repeatAddAddress, out uint repeatAddValue)) return false;
                if (!TryParseWriteType(w.Type, out CheatWriteType type)) return false;

                writes.Add(new CheatWrite
                {
                    Space = w.Space ?? "",
                    Address = (int)address,
                    Value = value,
                    Width = w.Width,
                    Type = type,
                    BitPosition = w.BitPosition,
                    BigEndian = w.BigEndian,
                    RepeatCount = w.RepeatCount,
                    RepeatAddAddress = (int)repeatAddAddress,
                    RepeatAddValue = repeatAddValue,
                });
            }
            return true;
        }

        public static bool TryParseWriteType(string? text, out CheatWriteType type)
        {
            type = CheatWriteType.Set;
            if (string.IsNullOrWhiteSpace(text)) return true; // absent means Set
            return Enum.TryParse(text.Trim(), ignoreCase: true, out type);
        }

        // Called once per frame. Takes read as well as write because
        // Increase/Decrease and BitPosition writes have to see what is
        // already there - the registry still never touches memory itself.
        public void ApplyAll(Func<string, int, byte> read, Action<string, int, byte> write)
        {
            foreach (Cheat c in _cheats)
            {
                if (c.Kind != CheatKind.RamPoke || !c.Enabled) continue;
                foreach (CheatWrite w in c.Writes) ApplyWrite(w, read, write);
            }
        }

        // Kept so an existing caller that only has a write delegate still
        // compiles; Increase/Decrease and bit writes read back as 0.
        public void ApplyAll(Action<string, int, byte> write) => ApplyAll((_, _) => 0, write);

        private static void ApplyWrite(CheatWrite w, Func<string, int, byte> read, Action<string, int, byte> write)
        {
            int width = w.EffectiveWidth;
            int repeats = w.EffectiveRepeatCount;

            for (int i = 0; i < repeats; i++)
            {
                int address = w.AddressAt(i);
                uint value = w.ValueAt(i);

                if (w.BitPosition is int bit)
                {
                    int mask = 1 << (bit & 7);
                    byte current = read(w.Space, address);
                    byte updated = (value & 1) != 0 ? (byte)(current | mask) : (byte)(current & ~mask);
                    write(w.Space, address, updated);
                    continue;
                }

                uint target = w.Type switch
                {
                    CheatWriteType.Increase => unchecked(ReadWide(read, w, address, width) + value),
                    CheatWriteType.Decrease => unchecked(ReadWide(read, w, address, width) - value),
                    _ => value,
                };

                for (int b = 0; b < width; b++) write(w.Space, address + b, w.ByteAt(target, b));
            }
        }

        private static uint ReadWide(Func<string, int, byte> read, CheatWrite w, int address, int width)
        {
            uint value = 0;
            for (int b = 0; b < width; b++)
            {
                int shift = w.BigEndian ? 8 * (width - 1 - b) : 8 * b;
                value |= (uint)read(w.Space, address + b) << shift;
            }
            return value;
        }

        // Called for every cartridge-routed read - see IRomReadPatcher.
        // First enabled, address-matching, compare-satisfying RomPatch
        // wins (list order = add order); returns false if nothing matches
        // so the caller falls back to the real cartridge byte unchanged.
        //
        // Must stay cheap: this is on the hot path of every ROM read, so
        // the no-patches case exits on one int compare, and a repeat run
        // resolves by division rather than by walking its repetitions.
        public bool TryPatchRom(uint address, byte originalValue, out byte patchedValue)
        {
            patchedValue = 0;
            if (_enabledRomPatches == 0) return false;

            int target = (int)address;

            foreach (Cheat c in _cheats)
            {
                if (c.Kind != CheatKind.RomPatch || !c.Enabled) continue;
                if (c.Compare.HasValue && c.Compare.Value != originalValue) continue;

                foreach (CheatWrite w in c.Writes)
                {
                    if (target < w.FirstAddress || target > w.LastAddress) continue;
                    if (!TryResolveRepetition(w, target, out int repetition, out int offset)) continue;

                    patchedValue = w.ByteAt(w.ValueAt(repetition), offset);
                    return true;
                }
            }

            return false;
        }

        // Which repetition of a repeat run covers <target>, and how far into
        // that repetition's width it sits. Division for the ordinary forward
        // stride; a bounded walk only for the zero/negative strides that
        // cannot be indexed that way.
        private static bool TryResolveRepetition(CheatWrite w, int target, out int repetition, out int offset)
        {
            int width = w.EffectiveWidth;
            int repeats = w.EffectiveRepeatCount;

            if (w.RepeatAddAddress > 0)
            {
                int delta = target - w.Address;
                if (delta < 0) { repetition = offset = 0; return false; }

                repetition = delta / w.RepeatAddAddress;
                if (repetition >= repeats) { repetition = offset = 0; return false; }

                offset = delta - repetition * w.RepeatAddAddress;
                return offset < width;
            }

            for (int i = 0; i < repeats; i++)
            {
                int delta = target - w.AddressAt(i);
                if (delta >= 0 && delta < width)
                {
                    repetition = i;
                    offset = delta;
                    return true;
                }
            }

            repetition = offset = 0;
            return false;
        }
    }
}
