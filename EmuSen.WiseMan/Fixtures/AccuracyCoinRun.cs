namespace EmuSen.WiseMan.Fixtures
{
    // One sub-test's result byte as AccuracyCoin encodes it - see Moon_Native.md §3.4.1.
    public readonly record struct AccuracyCoinResult(int Address, byte Raw)
    {
        public int Status => Raw & 0x03;
        public int Code => Raw >> 2;
        public bool Passed => Raw != 0xFF && Status == 1;
        public bool Failed => Raw != 0xFF && Status == 2;
        public bool Skipped => Raw == 0xFF;
    }

    // AccuracyCoin's "run all tests" from power-on, over any NES core's frame, RAM and Start button - see Moon_Native.md §3.4.1.
    public sealed class AccuracyCoinRun
    {
        public const string RomSha256 = "4fe8c2bc9abc6f4d418da47b73f62cba89fcacd950fae763097a1681d650e839";
        public const int FirstResult = 0x0400, LastResult = 0x0495;
        public const int MenuProgress = 0xEC, MenuReady = 0x0A;
        public const int TestTally = 0x37, PassTally = 0x38, SkipTally = 0x3F;

        // The results table's NMI hook, JMP PressStartToContinue, at this build's address of it.
        public static readonly byte[] TableHook = { 0x4C, 0x08, 0x93 };

        public int MenuFrame { get; private set; } = -1;
        public int TableFrame { get; private set; } = -1;
        public int Tested { get; private set; }
        public int PassedTally { get; private set; }
        public int SkippedTally { get; private set; }
        public byte[] Block { get; private set; } = Array.Empty<byte>();

        // The 150 result bytes; $0421 is a gap the ROM keeps zero, and a byte of 0 was never run.
        public IEnumerable<AccuracyCoinResult> Results => Block.Select((b, i) => new AccuracyCoinResult(FirstResult + i, b)).Where(r => r.Address != 0x0421 && r.Raw != 0);

        public static AccuracyCoinRun Run(Action runFrame, Func<int, byte> readRam, Action<bool> setStart, int budget = 8000)
        {
            var run = new AccuracyCoinRun();
            int pressAt = -1;
            for (int frame = 1; frame <= budget; frame++)
            {
                runFrame();
                if (run.MenuFrame < 0 && readRam(MenuProgress) == MenuReady)
                {
                    run.MenuFrame = frame;
                    pressAt = frame + 30;
                }
                if (frame == pressAt) setStart(true);
                if (frame == pressAt + 5) setStart(false);
                if (pressAt > 0 && frame > pressAt + 5 && readRam(0x0700) == TableHook[0] && readRam(0x0701) == TableHook[1] && readRam(0x0702) == TableHook[2])
                {
                    run.TableFrame = frame;
                    break;
                }
            }
            run.Tested = readRam(TestTally);
            run.PassedTally = readRam(PassTally);
            run.SkippedTally = readRam(SkipTally);
            run.Block = Enumerable.Range(FirstResult, LastResult - FirstResult + 1).Select(a => readRam(a)).ToArray();
            return run;
        }
    }
}
