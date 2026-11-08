using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.DianaSh
{
    // dianash.text on generated text the corpus does not hold, held to the C# classes' recorded answers - see EmuSen_DianaSh.md §13.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class TextModuleParityTests
    {
        private static readonly string[] Tokens =
        {
            "a", "b", "ab", "A", "B", "é", "É", "ß", "İ", "ı", "😀", "\uE000", "x y", "1", "-2", "3.5", "1,000", "1e3", "+4",
            " ", "\t", "", "NaN", "nan", "Infinity", "∞", "-∞", ",5", "1,,0", "-0", "0", "10", "9", ",", ":", "a,b", "Z", "$", "&", "\\", "[", "*",
        };

        // Each a command line as the shell is given it, its arguments in single quotes.
        private static readonly string[][] Lines =
        {
            new[] { "uniq" }, new[] { "uniq", "-c" }, new[] { "uniq", "-D" }, new[] { "uniq", "-D", "-w", "1" }, new[] { "uniq", "-w", "2" }, new[] { "uniq", "-c", "-w1" },
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

        // Recorded from the C# classes on 2026-11-06 as 64-answer digests; sort left on 2026-11-07, when it took code-point order (EmuSen_RustPlatform.md §47) - see EmuSen_Backend.md §40.7.
        [Fact]
        public void Every_text_command_answers_generated_input_as_its_csharp_class_did()
        {
            using var runner = new GoldenRunner();
            runner.Run(new GoldenCase { Title = "home" });
            var rust = new NativeShell(keepsSessions: false);
            var random = new Random(20261015);
            var golden = new EmuSen.WiseMan.Crystal.ParityGolden("TextModuleParity");
            for (int trial = 0; trial < 200; trial++)
            {
                string text = Generate(random);
                File.WriteAllText(Path.Combine(runner.Home, "in.txt"), text);
                foreach (string[] argv in Lines)
                {
                    string line = argv[0] + string.Concat(argv.Skip(1).Select(a => " '" + a + "'")) + " < in.txt";
                    var (_, output, _) = rust.Submit(0, line);
                    string answer = $"{rust.LastStatus(0)} {output}";
                    golden.AddChunked(argv[0], answer, 64);
                }
            }
            golden.Check(100);
        }

        private static string Show(string text) => "\"" + text.Replace("\\", "\\\\").Replace("\n", "\\n").Replace("\r", "\\r").Replace("\t", "\\t") + "\"";
    }
}
