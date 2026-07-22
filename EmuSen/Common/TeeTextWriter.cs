using System;
using System.IO;
using System.Text;

namespace EmuSen.Common
{
    // Forwards everything written to it to two underlying writers - lets console
    // output go to both the terminal (for live viewing) and a file (so nothing is
    // lost if the process gets killed, and output can be reviewed or pasted
    // afterward without scrolling back through terminal history). Console-agnostic:
    // any console's frontend can reuse this as-is.
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
