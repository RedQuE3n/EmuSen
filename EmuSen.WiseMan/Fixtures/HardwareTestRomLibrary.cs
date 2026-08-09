using EmuSen.DianaOS.DianaOS.Etc;

namespace EmuSen.WiseMan.Fixtures
{
    // What a hardware test ROM said down the link port - see Mercury_HardwareTests.md §2.
    public enum TestRomVerdict
    {
        NoVerdict = 0,
        Passed = 1,
        Failed = 2,
    }

    // Homebrew hardware tests in the gitignored TestRoms/hardware, none required - see Mercury_HardwareTests.md §1.
    public static class HardwareTestRomLibrary
    {
        public const string SubdirectoryName = "hardware";

        // Mooneye signals success by writing the Fibonacci run to serial - see Mercury_HardwareTests.md §2.1.
        public static readonly byte[] MooneyeSuccess = { 3, 5, 8, 13, 21, 34 };

        // ...and failure as six copies of $42, which is why "all B" is not a text verdict.
        public const byte MooneyeFailureByte = 0x42;

        public static string Root => Path.Combine(DianaOSSandbox.InstallDirectory, "TestRoms", SubdirectoryName);

        public static IReadOnlyList<string> Find()
        {
            if (!Directory.Exists(Root)) return Array.Empty<string>();

            var found = new List<string>();
            foreach (string extension in new[] { ".gb", ".gbc" })
            {
                found.AddRange(Directory.GetFiles(Root, $"*{extension}", SearchOption.AllDirectories));
            }

            found.Sort(StringComparer.Ordinal);
            return found;
        }

        // xUnit rejects empty TheoryData, so "no ROMs present" has to arrive as one empty case.
        public static TheoryData<string> AsTheoryData()
        {
            var data = new TheoryData<string>();
            foreach (string path in Find()) data.Add(path);

            if (data.Count == 0) data.Add(string.Empty);
            return data;
        }

        // Both corpora's conventions, read off one byte stream - see Mercury_HardwareTests.md §2.
        public static TestRomVerdict ReadVerdict(IReadOnlyList<byte> serial)
        {
            if (serial.Count == 0) return TestRomVerdict.NoVerdict;

            if (EndsWith(serial, MooneyeSuccess)) return TestRomVerdict.Passed;

            // Six identical $42 bytes is mooneye's failure marker; a text run of 'B's never reaches six.
            if (serial.Count >= 6 && AllTrailing(serial, 6, MooneyeFailureByte)) return TestRomVerdict.Failed;

            string text = AsText(serial);

            // Order matters: blargg's summary line says "Failed" after listing each subtest's number.
            if (text.Contains("Failed", StringComparison.Ordinal)) return TestRomVerdict.Failed;
            if (text.Contains("Passed", StringComparison.Ordinal)) return TestRomVerdict.Passed;

            return TestRomVerdict.NoVerdict;
        }

        public static string AsText(IReadOnlyList<byte> serial)
        {
            var chars = new char[serial.Count];
            for (int i = 0; i < serial.Count; i++)
            {
                byte b = serial[i];
                chars[i] = b is >= 0x20 and < 0x7F or (byte)'\n' ? (char)b : '.';
            }

            return new string(chars);
        }

        private static bool EndsWith(IReadOnlyList<byte> serial, byte[] marker)
        {
            if (serial.Count < marker.Length) return false;

            int start = serial.Count - marker.Length;
            for (int i = 0; i < marker.Length; i++)
            {
                if (serial[start + i] != marker[i]) return false;
            }

            return true;
        }

        private static bool AllTrailing(IReadOnlyList<byte> serial, int count, byte value)
        {
            for (int i = serial.Count - count; i < serial.Count; i++)
            {
                if (serial[i] != value) return false;
            }

            return true;
        }
    }
}
