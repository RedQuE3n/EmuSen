namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // Where a console's bare programs leave their results: words from From, valid once DoneWord is at DoneAt, all offsets into the space and Segment the program's view of them - see Sapphire_Native.md §14.3.
    public sealed record RamRecordForm(string Name, string Space, uint DoneAt, uint DoneWord, uint From, uint Segment)
    {
        // Diamond's programs/*.s: 80100000h on, D1A0D00Eh at 800FFFF0h (Diamond_Referees.md §2.4).
        public static readonly RamRecordForm Diamond = new("diamond", "ram", 0x000F_FFF0, 0xD1A0_D00E, 0x0010_0000, 0x8000_0000);

        // Sapphire's programs/*.s: 00200000h on, 5A99D00Eh at 001FFFF0h (Sapphire_Referees.md §4.1).
        public static readonly RamRecordForm Sapphire = new("sapphire", "ram", 0x001F_FFF0, 0x5A99_D00E, 0x0020_0000, 0);

        // The record's words, or null where the space is missing, the done word absent, or the words past its end.
        public uint[]? Read(RomSnapshot snapshot, int count)
        {
            if (!snapshot.Spaces.TryGetValue(Space, out var ram) || Word(ram, DoneAt) != DoneWord) return null;
            if (From + 4L * count > ram.Length) return null;
            return Enumerable.Range(0, count).Select(i => Word(ram, From + 4 * (uint)i)!.Value).ToArray();
        }

        // The run's last snapshot against the words expected: Passed, Failed naming each word that differs, Incomplete with no done word.
        public RomVerdict Grade(RomRun run, IReadOnlyList<uint> expected)
        {
            string name = $"record:{Name}";
            if (run.Snapshots.Count == 0 || !run.Snapshots[^1].Spaces.ContainsKey(Space)) return new RomVerdict(RomOutcome.NoDump, name, $"no {Space} space");
            if (Read(run.Snapshots[^1], expected.Count) is not { } words) return new RomVerdict(RomOutcome.Incomplete, name, $"no done word {DoneWord:X8} at {DoneAt:X8}");
            var off = Enumerable.Range(0, expected.Count).Where(i => words[i] != expected[i]).Select(i => $"word {i}: {words[i]:X8} against {expected[i]:X8}").ToArray();
            return new RomVerdict(off.Length == 0 ? RomOutcome.Passed : RomOutcome.Failed, name, off.Length == 0 ? $"{expected.Count} words" : string.Join("; ", off));
        }

        public RomProtocol Protocol(string pathSuffix, IReadOnlyList<uint> expected) =>
            new($"record:{Name}:{pathSuffix}", path => path.Replace('\\', '/').EndsWith(pathSuffix, StringComparison.Ordinal), run => Grade(run, expected));

        // A bare MIPS I program in this form: each word stored in turn from From, then the done word, then a branch to itself.
        public uint[] Program(IReadOnlyList<uint> words)
        {
            const int a0 = 4, t0 = 8;
            var asm = new MipsAssembler();
            uint from = Segment | From, done = Segment | DoneAt;
            asm.Lui(a0, (ushort)(from >> 16)).Ori(a0, a0, (ushort)from);
            for (int i = 0; i < words.Count; i++) asm.Lui(t0, (ushort)(words[i] >> 16)).Ori(t0, t0, (ushort)words[i]).Sw(t0, a0, (short)(4 * i));
            asm.Lui(a0, (ushort)(done >> 16)).Ori(a0, a0, (ushort)done).Lui(t0, (ushort)(DoneWord >> 16)).Ori(t0, t0, (ushort)DoneWord).Sw(t0, a0, 0);
            return asm.Beq(0, 0, -1).Nop().ToArray();
        }

        private static uint? Word(byte[] ram, uint at) => at + 4L <= ram.Length ? BitConverter.ToUInt32(ram, (int)at) : null;
    }
}
