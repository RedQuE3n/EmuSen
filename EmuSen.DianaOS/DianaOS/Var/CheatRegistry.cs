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
    // Two mechanisms era devices used, unified behind one registry - see EmuSen_Cheats.md.
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

        // The first write's fields, for callers that only deal in single-write cheats.
        public string SpaceName => Writes.Count > 0 ? Writes[0].Space : "";
        public int Address => Writes.Count > 0 ? Writes[0].Address : 0;
        public uint Value => Writes.Count > 0 ? Writes[0].Value : 0;
    }

    // RamPoke rewrites every frame; RomPatch substitutes a read - see EmuSen_Cheats.md and `man cheat`.
    public class CheatRegistry
    {
        // Copy-on-write: mutated under _gate, read without a lock - see EmuSen_Settings_Reference.md §4.15.
        private Cheat[] _cheats = Array.Empty<Cheat>();
        private readonly object _gate = new();
        private int _nextId = 1;

        // On every cartridge read, so the no-patches case must not walk the list.
        private volatile int _enabledRomPatches;

        private volatile bool _masterEnabled = true;

        // One switch over every cheat, runtime-only and never saved - see `man cheat`.
        public bool MasterEnabled
        {
            get => _masterEnabled;
            set
            {
                lock (_gate)
                {
                    _masterEnabled = value;
                    RecountRomPatches();
                }
            }
        }

        // One volatile read, then the array is the reader's for the rest of the call.
        private Cheat[] Snapshot() => System.Threading.Volatile.Read(ref _cheats);

        // Publishes a new snapshot. Callers hold _gate.
        private void Publish(Cheat[] next)
        {
            System.Threading.Volatile.Write(ref _cheats, next);
            RecountRomPatches();
        }

        public int AddRamPoke(string spaceName, int address, byte value, string description, bool enabled = true) =>
            AddCheat(CheatKind.RamPoke, new[] { CheatWrite.Poke(spaceName, address, value) }, null, description, enabled);

        // Null compare means unconditional, matching a 6-character Game Genie code.
        public int AddRomPatch(int address, byte value, byte? compare, string description, bool enabled = true) =>
            AddCheat(CheatKind.RomPatch, new[] { CheatWrite.Patch(address, value) }, compare, description, enabled);

        // Throws on the two combinations that cannot be honestly implemented - see EmuSen_Cheats.md.
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

            lock (_gate)
            {
                var cheat = new Cheat
                {
                    Id = _nextId++,
                    Kind = kind,
                    Writes = list,
                    Compare = compare,
                    Description = description,
                    Enabled = enabled,
                };

                var next = new Cheat[_cheats.Length + 1];
                Array.Copy(_cheats, next, _cheats.Length);
                next[^1] = cheat;
                Publish(next);
                return cheat.Id;
            }
        }

        public bool RemoveCheat(int id)
        {
            lock (_gate)
            {
                Cheat[] next = _cheats.Where(c => c.Id != id).ToArray();
                if (next.Length == _cheats.Length) return false;
                Publish(next);
                return true;
            }
        }

        public bool SetEnabled(int id, bool enabled)
        {
            lock (_gate)
            {
                Cheat? c = _cheats.FirstOrDefault(x => x.Id == id);
                if (c is null) return false;
                // In place, not a new snapshot: either flag value is correct and both are torn-free.
                c.Enabled = enabled;
                RecountRomPatches();
                return true;
            }
        }

        public void Clear()
        {
            lock (_gate) Publish(Array.Empty<Cheat>());
        }

        // Master off counts as zero, so TryPatchRom's hot path stays one int compare.
        private void RecountRomPatches() =>
            _enabledRomPatches = _masterEnabled ? _cheats.Count(c => c.Kind == CheatKind.RomPatch && c.Enabled) : 0;

        public IReadOnlyList<CheatInfo> GetCheats() =>
            Snapshot().Select(c => new CheatInfo(c.Id, c.Kind, c.Writes, c.Compare, c.Description, c.Enabled)).ToList();

        // Snapshot for etc/EmuSen/cheats/<name>.json - see EmuSen_Config_Reference.md §3.4.
        public CheatFile ToCheatFile()
        {
            var file = new CheatFile();
            foreach (Cheat c in Snapshot())
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

        // One bad line still loads the rest - see EmuSen_Config_Reference.md §3.4.
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

        // Here rather than on CheatFileEntry, which cannot see this assembly.
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

        // Takes read too: Increase/Decrease and bit writes must see what is already there.
        public void ApplyAll(Func<string, int, byte> read, Action<string, int, byte> write)
        {
            if (!_masterEnabled) return;

            foreach (Cheat c in Snapshot())
            {
                if (c.Kind != CheatKind.RamPoke || !c.Enabled) continue;
                foreach (CheatWrite w in c.Writes) ApplyWrite(w, read, write);
            }
        }

        // Kept so a write-only caller still compiles; those writes read back as 0.
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

        // First match wins, and the no-patch case exits on one int compare - see EmuSen_Cheats.md.
        public bool TryPatchRom(uint address, byte originalValue, out byte patchedValue)
        {
            patchedValue = 0;
            if (_enabledRomPatches == 0) return false;

            int target = (int)address;

            foreach (Cheat c in Snapshot())
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

        // Division for the ordinary forward stride; a bounded walk only for zero or negative.
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
