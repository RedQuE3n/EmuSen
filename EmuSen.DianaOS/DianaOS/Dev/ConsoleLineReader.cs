using System;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Dev
{
    // A small, "basic" readline-alike for the F4 console debug prompt -
    // Console.ReadLine() on its own gives you no line editing beyond
    // whatever the terminal itself does, and definitely no up/down-arrow
    // history recall (that's readline's job in bash, and .NET's console
    // doesn't ship an equivalent). Reads one raw key at a time via
    // Console.ReadKey(intercept: true) and manually echoes/redraws, the
    // same approach every from-scratch line editor uses.
    //
    // Deliberately not a general-purpose text widget: no undo, no
    // multi-line, no kill-ring/yank, no word-wise motion - "basic," per
    // its own ask, covers cursor movement, insert/delete, and history
    // recall, which is what actually makes the F4 prompt feel less
    // painful day to day.
    //
    // Lives in EmuSen.DianaOS (not EmuSen.Hotaru) even though only
    // the Raylib console-window prompt uses it today, specifically so a
    // future console-based entry point doesn't have to duplicate it - the
    // same reasoning DianaOSInterpreter's own header comment gives for
    // staying frontend-agnostic. A GUI textbox (the eventual Avalonia
    // debug window) wouldn't use this at all - it would read
    // DianaOSInterpreter.History directly from its own KeyDown handler
    // instead, since a real widget already owns its text editing.
    public static class ConsoleLineReader
    {
        // `history` is read live (Count/indexer only) - a line just typed
        // and submitted becomes recallable on the very next call, since
        // DianaOSInterpreter.Execute appends to the same list before
        // this method is ever called again.
        //
        // Falls back to plain Console.ReadLine() when stdin is redirected
        // (a script/CI feeding input via a pipe, or a redirected test
        // harness) - Console.ReadKey throws in that mode, and there's no
        // "up arrow" to recall from a non-interactive stream anyway.
        public static string? ReadLine(System.Collections.Generic.IReadOnlyList<string> history)
        {
            if (Console.IsInputRedirected) return Console.ReadLine();

            // Anchor to wherever the cursor already was (right after
            // whatever prompt text - "DianaOS #: " - the caller already
            // printed) rather than column 0, or every redraw below would
            // overwrite the prompt itself. Captured once: this reader
            // doesn't handle a line long enough to wrap past the terminal
            // width, or the terminal scrolling mid-edit - an accepted gap
            // for a "basic" implementation, not something real bash-level
            // readline usage at a debug prompt is likely to hit.
            int startLeft = Console.CursorLeft;
            int row = Console.CursorTop;

            var buffer = new StringBuilder();
            int cursor = 0;

            // -1 = "not currently recalling, editing a fresh line."
            // Anything >= 0 indexes into `history` (0 = oldest kept).
            // `pending` is what the user had typed before the first Up
            // press, restored on pressing Down past the newest entry -
            // same behavior bash's readline gives you.
            int historyIndex = -1;
            string pending = "";

            void Redraw(int previousLength)
            {
                Console.SetCursorPosition(startLeft, row);
                Console.Write(buffer.ToString());
                if (buffer.Length < previousLength) Console.Write(new string(' ', previousLength - buffer.Length));
                Console.SetCursorPosition(startLeft + cursor, row);
            }

            void SetBuffer(string text)
            {
                int previousLength = buffer.Length;
                buffer.Clear();
                buffer.Append(text);
                cursor = buffer.Length;
                Redraw(previousLength);
            }

            while (true)
            {
                ConsoleKeyInfo key = Console.ReadKey(intercept: true);

                switch (key.Key)
                {
                    case ConsoleKey.Enter:
                        Console.WriteLine();
                        return buffer.ToString();

                    case ConsoleKey.Backspace:
                        if (cursor > 0)
                        {
                            int prevLen = buffer.Length;
                            buffer.Remove(cursor - 1, 1);
                            cursor--;
                            Redraw(prevLen);
                        }
                        break;

                    case ConsoleKey.Delete:
                        if (cursor < buffer.Length)
                        {
                            int prevLen = buffer.Length;
                            buffer.Remove(cursor, 1);
                            Redraw(prevLen);
                        }
                        break;

                    case ConsoleKey.LeftArrow:
                        if (cursor > 0) { cursor--; Console.SetCursorPosition(startLeft + cursor, row); }
                        break;

                    case ConsoleKey.RightArrow:
                        if (cursor < buffer.Length) { cursor++; Console.SetCursorPosition(startLeft + cursor, row); }
                        break;

                    case ConsoleKey.Home:
                        cursor = 0;
                        Console.SetCursorPosition(startLeft + cursor, row);
                        break;

                    case ConsoleKey.End:
                        cursor = buffer.Length;
                        Console.SetCursorPosition(startLeft + cursor, row);
                        break;

                    case ConsoleKey.UpArrow:
                        if (history.Count == 0) break;
                        if (historyIndex == -1)
                        {
                            pending = buffer.ToString();
                            historyIndex = history.Count - 1;
                        }
                        else if (historyIndex > 0)
                        {
                            historyIndex--;
                        }
                        SetBuffer(history[historyIndex]);
                        break;

                    case ConsoleKey.DownArrow:
                        if (historyIndex == -1) break; // already on the fresh line - nothing newer to go to
                        if (historyIndex < history.Count - 1)
                        {
                            historyIndex++;
                            SetBuffer(history[historyIndex]);
                        }
                        else
                        {
                            historyIndex = -1;
                            SetBuffer(pending);
                        }
                        break;

                    case ConsoleKey.Escape:
                        // Bash doesn't clear the line on Escape by default,
                        // but it's a common enough "start over" muscle
                        // memory from other tools to support cheaply here.
                        historyIndex = -1;
                        SetBuffer("");
                        break;

                    default:
                        if (!char.IsControl(key.KeyChar))
                        {
                            int prevLen = buffer.Length;
                            buffer.Insert(cursor, key.KeyChar);
                            cursor++;
                            Redraw(prevLen);
                        }
                        break;
                }
            }
        }
    }
}
