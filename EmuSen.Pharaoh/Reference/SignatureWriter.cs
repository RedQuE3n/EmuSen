using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;

namespace EmuSen.Pharaoh.Reference
{
    // The C# half of the signature stream the probe writes - see §3.48.
    public sealed class SignatureWriter : IDisposable
    {
        private readonly StreamWriter _file;
        private string[] _columns = Array.Empty<string>();

        public SignatureWriter(string path) => _file = new StreamWriter(path);

        public void WriteHeader(string backend, string system, string romPath, string board, string region,
            string headerTrust, long prgBytes, long chrBytes, bool saveLoaded, string screenFormat,
            IEnumerable<(string Name, byte[] Data)> spaces)
        {
            _file.Write("# emusen-probe-signature 1\n");
            _file.Write($"# backend={backend}\n");
            _file.Write($"# system={system}\n");
            _file.Write($"# rom={romPath}\n");
            _file.Write($"# board={board}\n");
            _file.Write($"# region={region}\n");
            _file.Write($"# headerTrust={headerTrust}\n");
            _file.Write($"# prg={prgBytes}\n");
            _file.Write($"# chr={chrBytes}\n");
            _file.Write($"# saveLoaded={(saveLoaded ? 1 : 0)}\n");
            _file.Write($"# screenFormat={screenFormat}\n");

            _columns = spaces.Where(s => s.Data.Length > 0).Select(s => s.Name).ToArray();
            _file.Write("frame," + string.Join(",", _columns) + ",screen\n");
        }

        public void WriteRow(long frame, IEnumerable<(string Name, byte[] Data)> spaces, byte[] screen)
        {
            Dictionary<string, byte[]> byName = spaces.ToDictionary(s => s.Name, s => s.Data);

            var row = new System.Text.StringBuilder();
            row.Append(frame.ToString(CultureInfo.InvariantCulture));
            foreach (string column in _columns)
            {
                uint crc = byName.TryGetValue(column, out byte[]? data) ? Crc32.Of(data) : 0;
                row.Append(',').Append(crc.ToString("x8", CultureInfo.InvariantCulture));
            }
            row.Append(',').Append(Crc32.Of(screen).ToString("x8", CultureInfo.InvariantCulture)).Append('\n');

            _file.Write(row.ToString());
        }

        public void Dispose() => _file.Dispose();
    }

    // The ordinary reflected 0xEDB88320, so the probe's C++ and this agree without being told.
    public static class Crc32
    {
        private static readonly uint[] Table = Build();

        private static uint[] Build()
        {
            var table = new uint[256];
            for (uint i = 0; i < 256; i++)
            {
                uint c = i;
                for (int k = 0; k < 8; k++) c = (c & 1) != 0 ? 0xEDB88320u ^ (c >> 1) : c >> 1;
                table[i] = c;
            }
            return table;
        }

        public static uint Of(ReadOnlySpan<byte> data)
        {
            uint crc = 0xFFFFFFFFu;
            foreach (byte b in data) crc = Table[(crc ^ b) & 0xFF] ^ (crc >> 8);
            return crc ^ 0xFFFFFFFFu;
        }
    }
}
