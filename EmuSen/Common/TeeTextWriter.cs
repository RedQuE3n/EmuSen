using System;
using System.IO;
using System.Text;

namespace EmuSen.Common
{
    // Forwards to two writers, so output is live on the terminal and kept in a file.
    public class TeeTextWriter : TextWriter
    {
        private readonly TextWriter _console;
        private readonly TextWriter _file;

        public TeeTextWriter(TextWriter console, TextWriter file)
        {
            _console = console;
            _file = file;
        }

        public override Encoding Encoding => _console.Encoding;
        public override void Write(char value) { _console.Write(value); _file.Write(value); }
        public override void Write(string? value) { _console.Write(value); _file.Write(value); }
        public override void WriteLine(string? value) { _console.WriteLine(value); _file.WriteLine(value); }
        public override void WriteLine() { _console.WriteLine(); _file.WriteLine(); }
        public override void Flush() { _console.Flush(); _file.Flush(); }
    }
}
