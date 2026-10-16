using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.DianaSh.DianaSh.Bin.Commands.Unix;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.DianaSh
{
    // dianash.text against the C# classes it replaces, on generated text the corpus does not hold - see EmuSen_DianaSh.md §13.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class TextModuleParityTests
    {
        private static readonly string[] Tokens =
        {
            "a", "b", "ab", "A", "B", "é", "É", "ß", "İ", "ı", "😀", "\uE000", "x y", "1", "-2", "3.5", "1,000", "1e3", "+4",
            " ", "\t", "", "NaN", "nan", "Infinity", "∞", "-∞", ",5", "1,,0", "-0", "0", "10", "9", ",", ":", "a,b", "Z", "$", "&", "\\", "[", "*",
        };

        // Each a command line as the shell is given it, its arguments in single quotes; the C# class is the first word's.
        private static readonly string[][] Lines =
        {
            new[] { "sort" }, new[] { "sort", "-n" }, new[] { "sort", "-r" }, new[] { "sort", "-n", "-r" },
            new[] { "uniq" }, new[] { "uniq", "-c" },
            new[] { "wc" }, new[] { "wc", "-l" }, new[] { "wc", "-w" }, new[] { "wc", "-c" },
            new[] { "head" }, new[] { "head", "-n", "3" }, new[] { "tail" }, new[] { "tail", "-n", "2" },
            new[] { "grep", "a" }, new[] { "grep", "-i", "A" }, new[] { "grep", "-v", "a" }, new[] { "grep", "-c", "b" },
            new[] { "grep", "-n", "[0-9]+" }, new[] { "grep", "é" }, new[] { "grep", "^$" }, new[] { "grep", "-i", "é" },
            new[] { "sed", "s/a/X/" }, new[] { "sed", "s/a/X/g" }, new[] { "sed", "s/[0-9]+/#/g" }, new[] { "sed", "s/(a)(b)?/<\\1>/gi" },
            new[] { "sed", "s/b/&&/" }, new[] { "sed", "s/x/$1/" },
            new[] { "awk", "{print $2}" }, new[] { "awk", "{print NF, $NF}" }, new[] { "awk", "-F", ",", "{print $1}" }, new[] { "awk", "NR>1" },
            new[] { "awk", "/a/ {print NR}" }, new[] { "awk", "END {print NR}" },
            new[] { "xxd" }, new[] { "cat" },
        };

        private static IDianaShCommand Command(string name) => name switch
        {
            "sort" => new SortCommand(),
            "uniq" => new UniqCommand(),
            "wc" => new WcCommand(),
            "head" => new HeadCommand(),
            "tail" => new TailCommand(),
            "grep" => new GrepCommand(),
            "sed" => new SedCommand(),
            "awk" => new AwkCommand(),
            "xxd" => new XxdCommand(),
            "cat" => new CatCommand(),
            _ => throw new ArgumentException(name),
        };

        private static string Generate(Random random)
        {
            var text = new StringBuilder();
            int lines = random.Next(0, 7);
            for (int l = 0; l < lines; l++)
            {
                int words = random.Next(0, 5);
                for (int w = 0; w < words; w++)
                {
                    if (w > 0) text.Append(random.Next(4) == 0 ? "," : " ");
                    text.Append(Tokens[random.Next(Tokens.Length)]);
                }
                if (l < lines - 1 || random.Next(2) == 0) text.Append(random.Next(6) == 0 ? "\r\n" : "\n");
            }
            return text.ToString();
        }

        [Fact]
        public void Every_text_command_answers_generated_input_as_its_csharp_class()
        {
            if (!DianaShNative.Ready) return;
            using var runner = new GoldenRunner();
            runner.Run(new GoldenCase { Title = "home" });
            var rust = new NativeShell(keepsSessions: false);
            var random = new Random(20261015);
            var differences = new List<string>();
            int compared = 0;
            for (int trial = 0; trial < 200; trial++)
            {
                string text = Generate(random);
                File.WriteAllText(Path.Combine(runner.Home, "in.txt"), text);
                string stdin = File.ReadAllText(Path.Combine(runner.Home, "in.txt"));
                foreach (string[] argv in Lines)
                {
                    DianaShResult expected;
                    // The C# read the machine's culture where the Rust reads en-US's, the recording machine's (EmuSen_DianaSh.md §13.8).
                    var culture = System.Globalization.CultureInfo.CurrentCulture;
                    System.Globalization.CultureInfo.CurrentCulture = System.Globalization.CultureInfo.GetCultureInfo("en-US");
                    try { expected = Command(argv[0]).Execute(null, argv, stdin); }
                    catch (Exception ex) { expected = DianaShResult.Fail($"Error: {ex.Message}"); }
                    finally { System.Globalization.CultureInfo.CurrentCulture = culture; }
                    // The C#'s xxd ended its lines with the system's line end, the Rust's with \n everywhere (§13.8).
                    if (argv[0] == "xxd" && OperatingSystem.IsWindows()) expected = new DianaShResult(expected.Output.Replace("\r\n", "\n"), expected.ExitCode);
                    string line = argv[0] + string.Concat(argv.Skip(1).Select(a => " '" + a + "'")) + " < in.txt";
                    var (_, output, _) = rust.Submit(0, line);
                    int status = rust.LastStatus(0);
                    compared++;
                    if (output != expected.Output || status != expected.ExitCode)
                        differences.Add($"{line} over {Show(text)}: C# {Show(expected.Output)} ({expected.ExitCode}), Rust {Show(output)} ({status})");
                }
            }
            Assert.True(differences.Count == 0, $"{differences.Count} of {compared} differ:\n" + string.Join("\n", differences.Take(20)));
        }

        private static string Show(string text) => "\"" + text.Replace("\\", "\\\\").Replace("\n", "\\n").Replace("\r", "\\r").Replace("\t", "\\t") + "\"";
    }
}
