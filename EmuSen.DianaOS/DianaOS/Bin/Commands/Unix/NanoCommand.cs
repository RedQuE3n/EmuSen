using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // A small full-screen text editor, branded "nano" for the muscle
    // memory (Ctrl+O save, Ctrl+X exit, arrow keys to move) - not a
    // clone of GNU nano's full feature set (no search, no cut/paste
    // ring, no syntax highlighting, no line-wrap toggle), just enough
    // to create/edit a script file (see `source`) without leaving the
    // shell for an external editor.
    //
    // Genuinely different from every other command here: everything
    // else in this namespace is "take target/args/stdin, return a
    // string" - one synchronous call. This one takes over the real
    // console (raw key reads via Console.ReadKey, full-screen redraws
    // via Console.SetCursorPosition/Console.Clear) for as long as the
    // user is editing, the same way ConsoleLineReader already does for
    // single-line editing, just for a whole buffer instead of one
    // line. That means it only works against a REAL interactive
    // terminal - Console.IsInputRedirected/IsOutputRedirected (a piped
    // script, `source`, a headless test harness, or a GUI-hosted
    // console like EmuSen.Mistress9's that never had a real terminal
    // to begin with) all get a clean "needs an interactive terminal"
    // error instead of trying to draw anywhere.
    public class NanoCommand : IDianaOSCommand
    {
        public string Name => "nano";
        public bool IsReadOnly => false;
        public string Usage => "  nano <path>                   simple full-screen text editor (Ctrl+O save, Ctrl+X exit - interactive terminal only)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaOSResult.Fail("nano: usage: nano <path>");

            if (!DianaOSSandbox.TryResolve(args[1], out string resolved))
            {
                return DianaOSResult.Fail($"nano: '{args[1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
            }

            if (Console.IsInputRedirected || Console.IsOutputRedirected)
            {
                return DianaOSResult.Fail("nano: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)");
            }

            return new Editor(resolved, args[1]).Run();
        }

        // Isolated from Execute() above so all the mutable cursor/
        // scroll/buffer state for one editing session lives on its own
        // short-lived instance instead of static fields or a pile of
        // ref parameters threaded through a static method.
        private sealed class Editor
        {
            private readonly string _path;
            private readonly string _displayPath;
            private readonly List<string> _lines;
            private int _cursorRow;
            private int _cursorCol;
            private int _topRow;
            private bool _modified;

            public Editor(string path, string displayPath)
            {
                _path = path;
                _displayPath = displayPath;
                _lines = File.Exists(path) ? new List<string>(File.ReadAllLines(path)) : new List<string>();
                if (_lines.Count == 0) _lines.Add("");
            }

            public DianaOSResult Run()
            {
                Console.Clear();
                Draw();
                bool savedAnything = false;

                while (true)
                {
                    ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                    bool ctrl = (key.Modifiers & ConsoleModifiers.Control) != 0;

                    if (ctrl && key.Key == ConsoleKey.X)
                    {
                        if (_modified)
                        {
                            string? choice = PromptSaveOnExit();
                            if (choice is null) { Draw(); continue; } // cancelled - stay in the editor
                            if (choice == "y") { Save(); savedAnything = true; }
                        }
                        break;
                    }

                    if (ctrl && key.Key == ConsoleKey.O)
                    {
                        Save();
                        savedAnything = true;
                        Draw();
                        continue;
                    }

                    HandleEditKey(key);
                    Draw();
                }

                Console.Clear();
                Console.SetCursorPosition(0, 0);
                return savedAnything
                    ? DianaOSResult.Ok($"nano: saved {_lines.Count} line(s) to {_displayPath}")
                    : DianaOSResult.Ok($"nano: exited {_displayPath} without saving");
            }

            private void Save()
            {
                File.WriteAllLines(_path, _lines);
                _modified = false;
            }

            private string? PromptSaveOnExit()
            {
                Console.SetCursorPosition(0, SafeWindowHeight() - 1);
                Console.Write(PadOrTruncate("Save modified buffer? (y/n, Esc to cancel)", SafeWindowWidth()));
                while (true)
                {
                    ConsoleKeyInfo key = Console.ReadKey(intercept: true);
                    if (key.Key == ConsoleKey.Y) return "y";
                    if (key.Key == ConsoleKey.N) return "n";
                    if (key.Key == ConsoleKey.Escape) return null;
                }
            }

            private void HandleEditKey(ConsoleKeyInfo key)
            {
                string line = _lines[_cursorRow];
                int bodyHeight = Math.Max(1, SafeWindowHeight() - 2);

                switch (key.Key)
                {
                    case ConsoleKey.UpArrow:
                        if (_cursorRow > 0) { _cursorRow--; _cursorCol = Math.Min(_cursorCol, _lines[_cursorRow].Length); }
                        break;

                    case ConsoleKey.DownArrow:
                        if (_cursorRow < _lines.Count - 1) { _cursorRow++; _cursorCol = Math.Min(_cursorCol, _lines[_cursorRow].Length); }
                        break;

                    case ConsoleKey.LeftArrow:
                        if (_cursorCol > 0) _cursorCol--;
                        else if (_cursorRow > 0) { _cursorRow--; _cursorCol = _lines[_cursorRow].Length; }
                        break;

                    case ConsoleKey.RightArrow:
                        if (_cursorCol < line.Length) _cursorCol++;
                        else if (_cursorRow < _lines.Count - 1) { _cursorRow++; _cursorCol = 0; }
                        break;

                    case ConsoleKey.Home:
                        _cursorCol = 0;
                        break;

                    case ConsoleKey.End:
                        _cursorCol = line.Length;
                        break;

                    case ConsoleKey.PageUp:
                        _cursorRow = Math.Max(0, _cursorRow - bodyHeight);
                        _cursorCol = Math.Min(_cursorCol, _lines[_cursorRow].Length);
                        break;

                    case ConsoleKey.PageDown:
                        _cursorRow = Math.Min(_lines.Count - 1, _cursorRow + bodyHeight);
                        _cursorCol = Math.Min(_cursorCol, _lines[_cursorRow].Length);
                        break;

                    case ConsoleKey.Backspace:
                        if (_cursorCol > 0)
                        {
                            _lines[_cursorRow] = line.Remove(_cursorCol - 1, 1);
                            _cursorCol--;
                            _modified = true;
                        }
                        else if (_cursorRow > 0)
                        {
                            int prevLen = _lines[_cursorRow - 1].Length;
                            _lines[_cursorRow - 1] += line;
                            _lines.RemoveAt(_cursorRow);
                            _cursorRow--;
                            _cursorCol = prevLen;
                            _modified = true;
                        }
                        break;

                    case ConsoleKey.Delete:
                        if (_cursorCol < line.Length)
                        {
                            _lines[_cursorRow] = line.Remove(_cursorCol, 1);
                            _modified = true;
                        }
                        else if (_cursorRow < _lines.Count - 1)
                        {
                            _lines[_cursorRow] = line + _lines[_cursorRow + 1];
                            _lines.RemoveAt(_cursorRow + 1);
                            _modified = true;
                        }
                        break;

                    case ConsoleKey.Enter:
                        _lines[_cursorRow] = line.Substring(0, _cursorCol);
                        _lines.Insert(_cursorRow + 1, line.Substring(_cursorCol));
                        _cursorRow++;
                        _cursorCol = 0;
                        _modified = true;
                        break;

                    default:
                        if (!char.IsControl(key.KeyChar))
                        {
                            _lines[_cursorRow] = line.Insert(_cursorCol, key.KeyChar.ToString());
                            _cursorCol++;
                            _modified = true;
                        }
                        break;
                }
            }

            private void Draw()
            {
                int width = SafeWindowWidth();
                int height = SafeWindowHeight();
                int bodyHeight = Math.Max(1, height - 2);

                if (_cursorRow < _topRow) _topRow = _cursorRow;
                else if (_cursorRow >= _topRow + bodyHeight) _topRow = _cursorRow - bodyHeight + 1;
                if (_topRow < 0) _topRow = 0;

                Console.SetCursorPosition(0, 0);
                Console.Write(PadOrTruncate($" DianaOS nano  {_displayPath}{(_modified ? " *" : "")} ", width));

                for (int i = 0; i < bodyHeight; i++)
                {
                    Console.SetCursorPosition(0, i + 1);
                    int lineIndex = _topRow + i;
                    Console.Write(PadOrTruncate(lineIndex < _lines.Count ? _lines[lineIndex] : "", width));
                }

                Console.SetCursorPosition(0, height - 1);
                Console.Write(PadOrTruncate("^O Save    ^X Exit    Arrows/PgUp/PgDn move    Enter/Backspace/Delete edit", width));

                Console.SetCursorPosition(Math.Min(_cursorCol, width - 1), _cursorRow - _topRow + 1);
            }

            private static string PadOrTruncate(string text, int width) =>
                text.Length >= width ? text.Substring(0, width) : text.PadRight(width);

            // Console.WindowWidth/Height can throw (or return a bogus 0)
            // outside a real attached terminal - shouldn't be reachable
            // given Execute()'s own IsInputRedirected/IsOutputRedirected
            // guard, but a safe fallback costs nothing and avoids turning
            // an edge-case environment quirk into a crash mid-edit.
            private static int SafeWindowWidth()
            {
                try { return Math.Max(20, Console.WindowWidth); } catch { return 80; }
            }

            private static int SafeWindowHeight()
            {
                try { return Math.Max(5, Console.WindowHeight); } catch { return 24; }
            }
        }
    }
}
