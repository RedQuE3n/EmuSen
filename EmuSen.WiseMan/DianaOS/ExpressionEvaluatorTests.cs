using System.Collections.Generic;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // The expression language behind `eval` and conditional breakpoints - see `man eval`.
    public class ExpressionEvaluatorTests
    {
        // A stand-in core, so the language is tested without a loaded ROM.
        private sealed class FakeContext : IExpressionContext
        {
            public readonly Dictionary<string, long> Symbols = new();
            public readonly Dictionary<(string? Space, int Address), byte> Memory = new();

            public IReadOnlyList<string> SymbolNames => new List<string>(Symbols.Keys);

            public bool TryGetSymbol(string name, out long value) => Symbols.TryGetValue(name, out value);

            public bool TryReadMemory(string? space, int address, int width, out long value)
            {
                value = 0;
                for (int i = 0; i < width; i++)
                {
                    if (!Memory.TryGetValue((space, address + i), out byte b)) return false;
                    value |= (long)b << (8 * i);
                }
                return true;
            }
        }

        private static long Eval(string expression, FakeContext? context = null)
            => new ExpressionEvaluator(context ?? new FakeContext()).Evaluate(expression);

        [Theory]
        [InlineData("1 + 2", 3)]
        [InlineData("2 * 3 + 4", 10)]
        [InlineData("4 + 2 * 3", 10)]
        [InlineData("(4 + 2) * 3", 18)]
        [InlineData("$FF", 255)]
        [InlineData("0x10", 16)]
        [InlineData("%1010", 10)]
        [InlineData("10 - 3 - 2", 5)]
        [InlineData("1 << 8", 256)]
        [InlineData("$FF00 >> 8", 255)]
        [InlineData("$F0 & $3C", 0x30)]
        [InlineData("$F0 | $0F", 0xFF)]
        [InlineData("$FF ^ $0F", 0xF0)]
        [InlineData("-5 + 8", 3)]
        [InlineData("~0 & $FF", 255)]
        [InlineData("!0", 1)]
        [InlineData("!5", 0)]
        [InlineData("7 % 4", 3)]
        public void Arithmetic_and_literals_evaluate(string expression, long expected)
            => Assert.Equal(expected, Eval(expression));

        [Theory]
        [InlineData("1 == 1", 1)]
        [InlineData("1 != 1", 0)]
        [InlineData("2 > 1", 1)]
        [InlineData("2 <= 1", 0)]
        [InlineData("1 && 1", 1)]
        [InlineData("1 && 0", 0)]
        [InlineData("0 || 3", 1)]
        [InlineData("0 || 0", 0)]
        public void Comparisons_produce_one_or_zero(string expression, long expected)
            => Assert.Equal(expected, Eval(expression));

        // Every debugger's condition field treats nonzero as true.
        [Fact]
        public void EvaluateBool_treats_any_nonzero_value_as_true()
        {
            var evaluator = new ExpressionEvaluator(new FakeContext());

            Assert.True(evaluator.EvaluateBool("42"));
            Assert.True(evaluator.EvaluateBool("-1"));
            Assert.False(evaluator.EvaluateBool("0"));
        }

        [Fact]
        public void Symbols_resolve_through_the_context()
        {
            var context = new FakeContext();
            context.Symbols["a"] = 0x1234;
            context.Symbols["flag.c"] = 1;

            Assert.Equal(0x1234, Eval("a", context));
            Assert.Equal(1, Eval("a == $1234 && flag.c", context));
        }

        [Fact]
        public void Square_brackets_read_a_byte_and_braces_read_a_word()
        {
            var context = new FakeContext();
            context.Memory[(null, 0x7E0020)] = 0x34;
            context.Memory[(null, 0x7E0021)] = 0x12;

            Assert.Equal(0x34, Eval("[$7E0020]", context));
            Assert.Equal(0x1234, Eval("{$7E0020}", context));
        }

        [Fact]
        public void A_space_prefix_reads_from_that_space()
        {
            var context = new FakeContext();
            context.Memory[("VRAM", 0x2760)] = 0x5A;

            Assert.Equal(0x5A, Eval("[VRAM:$2760]", context));
        }

        // The guard that makes `[x] != 0 && n / [x]` safe to write.
        [Fact]
        public void And_short_circuits_before_evaluating_the_right_side()
        {
            var context = new FakeContext();
            context.Memory[(null, 0x10)] = 0;

            Assert.Equal(0, Eval("[$10] != 0 && 100 / [$10] > 2", context));
        }

        [Fact]
        public void Or_short_circuits_before_evaluating_the_right_side()
            => Assert.Equal(1, Eval("1 || 100 / 0"));

        [Theory]
        [InlineData("1 +")]
        [InlineData("(1 + 2")]
        [InlineData("1 @ 2")]
        [InlineData("[1")]
        [InlineData("100 / 0")]
        public void A_malformed_expression_throws_ExpressionException(string expression)
            => Assert.Throws<ExpressionException>(() => Eval(expression));

        [Fact]
        public void An_unknown_symbol_names_itself_in_the_error()
        {
            var ex = Assert.Throws<ExpressionException>(() => Eval("nosuchthing + 1"));

            Assert.Contains("nosuchthing", ex.Message);
        }

        // Conditions run on the emulation thread, where a throw is fatal.
        [Fact]
        public void TryEvaluateBool_reports_an_error_instead_of_throwing()
        {
            var evaluator = new ExpressionEvaluator(new FakeContext());

            Assert.False(evaluator.TryEvaluateBool("bogus ==", out bool result, out string error));
            Assert.False(result);
            Assert.NotEmpty(error);
        }

        [Fact]
        public void Format_shows_decimal_hex_and_binary_at_once()
        {
            string formatted = ExpressionEvaluator.Format(0x2A);

            Assert.Contains("42", formatted);
            Assert.Contains("$2A", formatted);
            Assert.Contains("%", formatted);
        }

        // The generic fallback is what makes `eval` work on a core that
        // publishes no context of its own - see `man eval`.
        [Fact]
        public void The_generic_fallback_context_resolves_registers_a_target_already_reports()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var generic = new DebugTargetExpressionContext(target);

            Assert.NotEmpty(generic.SymbolNames);
            Assert.True(generic.TryGetSymbol("frame", out long frame));
            Assert.Equal(target.FrameCount, frame);
        }

        [Fact]
        public void Eval_prints_a_value_three_ways_through_the_shell()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);

            string output = DianaOSInterpreter.CreateDefault(target).Submit("eval $2A").Output;

            Assert.Contains("42", output);
            Assert.Contains("$2A", output);
        }

        [Fact]
        public void Eval_symbols_lists_this_cores_register_names()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);

            string output = DianaOSInterpreter.CreateDefault(target).Submit("eval symbols flag").Output;

            Assert.Contains("flag.c", output);
            Assert.Contains("flag.z", output);
        }

        // The shell splits `a == 5` into three words before this command sees it.
        [Fact]
        public void Eval_rejoins_an_unquoted_multi_word_expression()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);

            Assert.Contains("3", DianaOSInterpreter.CreateDefault(target).Submit("eval 1 + 2").Output);
        }

        [Fact]
        public void Eval_reports_a_bad_expression_with_a_nonzero_exit_code()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var shell = DianaOSInterpreter.CreateDefault(target);

            shell.Submit("eval 1 +");

            Assert.Equal("1", shell.Submit("echo $?").Output.Trim());
        }

        [Fact]
        public void A_label_becomes_usable_as_a_symbol()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            var shell = DianaOSInterpreter.CreateDefault(target);
            shell.Submit("label add 00A3B2 NmiHandler");

            Assert.Contains("$A3B2", shell.Submit("eval NmiHandler").Output);
        }
    }
}
