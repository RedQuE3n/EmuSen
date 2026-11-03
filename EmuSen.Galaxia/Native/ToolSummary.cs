using System;
using System.Collections.Generic;
using System.Text.Json;
using System.Threading;

namespace EmuSen.Galaxia.Native
{
    // A tool's one-line state as its own library words it: the line, a shorter one for a 150-unit place, and whether it needs the player - see EmuSen_Interface.md §19.
    public sealed record ToolSummary(string Line, string? Short = null, bool NeedsAttention = false)
    {
        public const string Mark = "⚠ ";

        // The short form where one is given, else the line.
        public string Brief => Short ?? Line;

        // The line with its warning sign where it needs the player, as a row's value or a header's fact shows it.
        public string Marked => NeedsAttention ? Mark + Line : Line;

        public string MarkedBrief => NeedsAttention ? Mark + Brief : Brief;

        public static string Count(int n, string one, string many) => n == 1 ? $"1 {one}" : $"{n:N0} {many}";

        // A job's events taken to its end off the interface thread and read as rows, for a summary read where no window follows it; stopped and null when cancelled.
        public static List<T>? Rows<T>(IToolJob job, Func<JsonElement, T?> row, CancellationToken cancel) where T : class
        {
            var rows = new List<T>();
            using (job)
            {
                while (!job.Ended)
                {
                    if (cancel.IsCancellationRequested)
                    {
                        job.Stop();
                        return null;
                    }
                    if (job.Next(50) is not { } line) continue;
                    using JsonDocument doc = JsonDocument.Parse(line);
                    if (row(doc.RootElement) is { } r) rows.Add(r);
                }
            }
            return rows;
        }
    }
}
