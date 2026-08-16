using System;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Dev
{
    // A basic readline-alike, kept here so a future console entry point reuses it - see §3.17.
    public static class ConsoleLineReader
    {
        // History is read live; a redirected stdin falls back to Console.ReadLine.
        public static string? ReadLine(System.Collections.Generic.IReadOnlyList<string> history)
        {
            if (Console.IsInputRedirected) return Console.ReadLine();

            // Anchored past the prompt; wrapping and mid-edit scrolling are known gaps - see §3.17.
            int startLeft = Console.CursorLeft;
            int row = Console.CursorTop;

            var buffer = new StringBuilder();
            int cursor = 0;

            // -1 means editing a fresh line; `pending` is what Down past the newest restores.
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
                        // Bash does not clear on Escape, but the muscle memory is common and cheap to serve.
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
