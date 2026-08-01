using System.Linq;
using EmuSen.DianaOS.Ast;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.WiseMan.DianaOS
{
    // Characterization tests pinning Parser.Parse's current behavior - see EmuSen_Debugging_Tools_Reference_v5.md §3.18a.
    public class ParserTests
    {
        private static StatementList Parse(string src)
        {
            LexResult lex = Lexer.Tokenize(src);
            Assert.False(lex.NeedsMoreInput, $"'{src}' unexpectedly reported NeedsMoreInput.");
            return Parser.Parse(lex.Tokens);
        }

        private static Node Single(string src)
        {
            StatementList list = Parse(src);
            Assert.Single(list.Statements);
            return list.Statements[0];
        }

        private static SimpleCommand OnlyCommand(string src)
        {
            var andOr = Assert.IsType<AndOrList>(Single(src));
            Assert.Empty(andOr.Rest);
            return Assert.Single(andOr.First.Stages);
        }

        // Flattens a word to its concatenated part text - only meaningful for all-literal words.
        private static string Text(Word w) => string.Concat(w.Parts.Select(p => p.Text));

        private static string[] WordTexts(SimpleCommand c) => c.Words.Select(Text).ToArray();

        // --- simple commands ---

        [Fact]
        public void A_bare_command_parses_to_one_statement_with_its_arguments()
        {
            Assert.Equal(new[] { "echo", "a", "b" }, WordTexts(OnlyCommand("echo a b")));
        }

        [Fact]
        public void An_empty_script_parses_to_zero_statements()
        {
            Assert.Empty(Parse("").Statements);
            Assert.Empty(Parse("   ").Statements);
            Assert.Empty(Parse(";;\n\n").Statements);
        }

        [Fact]
        public void Statements_separate_on_semicolons_and_newlines()
        {
            Assert.Equal(2, Parse("echo a; echo b").Statements.Count);
            Assert.Equal(2, Parse("echo a\necho b").Statements.Count);
        }

        [Fact]
        public void Trailing_and_repeated_separators_produce_no_empty_statements()
        {
            Assert.Single(Parse("echo a;").Statements);
            Assert.Single(Parse("echo a;;\n\n").Statements);
        }

        // --- pipelines, and-or, negation ---

        [Fact]
        public void A_pipeline_collects_every_stage_in_order()
        {
            var andOr = Assert.IsType<AndOrList>(Single("regs | grep -i vram | wc -l"));
            Assert.Equal(3, andOr.First.Stages.Count);
            Assert.Equal(new[] { "regs" }, WordTexts(andOr.First.Stages[0]));
            Assert.Equal(new[] { "wc", "-l" }, WordTexts(andOr.First.Stages[2]));
        }

        [Fact]
        public void And_or_chains_are_flat_and_left_to_right()
        {
            var andOr = Assert.IsType<AndOrList>(Single("a && b || c"));
            Assert.Equal(new[] { "a" }, WordTexts(andOr.First.Stages[0]));
            Assert.Equal(new[] { AndOrKind.And, AndOrKind.Or }, andOr.Rest.Select(r => r.Kind).ToArray());
            Assert.Equal(new[] { "c" }, WordTexts(andOr.Rest[1].Pipeline.Stages[0]));
        }

        [Fact]
        public void A_leading_bang_sets_Negate_on_the_pipeline_only()
        {
            var andOr = Assert.IsType<AndOrList>(Single("! grep -q x"));
            Assert.True(andOr.First.Negate);
            Assert.Equal(new[] { "grep", "-q", "x" }, WordTexts(andOr.First.Stages[0]));
        }

        [Fact]
        public void Negation_applies_per_pipeline_not_per_chain()
        {
            var andOr = Assert.IsType<AndOrList>(Single("a && ! b"));
            Assert.False(andOr.First.Negate);
            Assert.True(andOr.Rest[0].Pipeline.Negate);
        }

        // --- redirection ---

        [Theory]
        [InlineData("echo a > out.txt", RedirectKind.Truncate)]
        [InlineData("echo a >> out.txt", RedirectKind.Append)]
        [InlineData("wc -l < in.txt", RedirectKind.Input)]
        public void Redirections_record_their_kind_and_target(string src, RedirectKind expected)
        {
            SimpleCommand cmd = OnlyCommand(src);
            Redirection r = Assert.Single(cmd.Redirections);
            Assert.Equal(expected, r.Kind);
            Assert.Contains(Text(r.Target), new[] { "out.txt", "in.txt" });
        }

        [Fact]
        public void A_redirection_attaches_to_the_stage_it_textually_follows()
        {
            var andOr = Assert.IsType<AndOrList>(Single("regs | grep x > out.txt"));
            Assert.Empty(andOr.First.Stages[0].Redirections);
            Assert.Single(andOr.First.Stages[1].Redirections);
        }

        [Fact]
        public void A_redirection_may_appear_before_the_command_name()
        {
            SimpleCommand cmd = OnlyCommand("< in.txt wc -l");
            Assert.Single(cmd.Redirections);
            Assert.Equal(new[] { "wc", "-l" }, WordTexts(cmd));
        }

        // --- assignments ---

        [Fact]
        public void A_leading_name_equals_value_is_an_assignment_not_a_word()
        {
            SimpleCommand cmd = OnlyCommand("X=42");
            Assignment a = Assert.Single(cmd.Assignments);
            Assert.Equal("X", a.Name);
            Assert.Equal("42", Text(a.Value));
            Assert.Empty(cmd.Words);
        }

        [Fact]
        public void Multiple_leading_assignments_all_bind_before_the_command_name()
        {
            SimpleCommand cmd = OnlyCommand("X=1 Y=2 echo hi");
            Assert.Equal(new[] { "X", "Y" }, cmd.Assignments.Select(a => a.Name).ToArray());
            Assert.Equal(new[] { "echo", "hi" }, WordTexts(cmd));
        }

        [Fact]
        public void An_assignment_after_the_command_name_is_a_literal_argument()
        {
            SimpleCommand cmd = OnlyCommand("echo X=42");
            Assert.Empty(cmd.Assignments);
            Assert.Equal(new[] { "echo", "X=42" }, WordTexts(cmd));
        }

        [Fact]
        public void A_quoted_assignment_is_a_literal_argument()
        {
            SimpleCommand cmd = OnlyCommand("'X=42'");
            Assert.Empty(cmd.Assignments);
            Assert.Equal(new[] { "X=42" }, WordTexts(cmd));
        }

        [Fact]
        public void A_non_identifier_name_is_not_an_assignment()
        {
            Assert.Empty(OnlyCommand("1X=42").Assignments);
            Assert.Empty(OnlyCommand("a-b=42").Assignments);
            Assert.Empty(OnlyCommand("=42").Assignments);
        }

        [Fact]
        public void An_assignment_value_may_be_empty()
        {
            SimpleCommand cmd = OnlyCommand("X=");
            Assignment a = Assert.Single(cmd.Assignments);
            Assert.Equal("X", a.Name);
            Assert.Equal("", Text(a.Value));
        }

        [Fact]
        public void An_assignment_value_keeps_its_non_literal_parts()
        {
            Assignment a = Assert.Single(OnlyCommand("Y=$(echo x)").Assignments);
            Assert.Equal("Y", a.Name);
            Assert.Equal(PartKind.CommandSubstitution, Assert.Single(a.Value.Parts).Kind);
        }

        // --- if ---

        [Fact]
        public void An_if_without_else_has_one_branch_and_no_else_body()
        {
            var node = Assert.IsType<IfNode>(Single("if true; then echo a; fi"));
            Assert.Single(node.Branches);
            Assert.Null(node.ElseBody);
        }

        [Fact]
        public void Elif_chains_append_branches_in_order_and_else_is_separate()
        {
            var node = Assert.IsType<IfNode>(Single("if a; then b; elif c; then d; elif e; then f; else g; fi"));
            Assert.Equal(3, node.Branches.Count);
            Assert.NotNull(node.ElseBody);
            Assert.Single(node.ElseBody!.Statements);
        }

        [Fact]
        public void An_if_body_may_contain_another_compound_statement()
        {
            var node = Assert.IsType<IfNode>(Single("if a; then for i in 1; do echo $i; done; fi"));
            Assert.IsType<ForNode>(node.Branches[0].Body.Statements[0]);
        }

        // --- for ---

        [Fact]
        public void A_for_loop_records_its_variable_and_item_list()
        {
            var node = Assert.IsType<ForNode>(Single("for i in 1 2 3; do echo $i; done"));
            Assert.Equal("i", node.VarName);
            Assert.Equal(new[] { "1", "2", "3" }, node.Items.Select(Text).ToArray());
            Assert.Single(node.Body.Statements);
        }

        [Fact]
        public void A_for_loop_may_have_an_empty_item_list()
        {
            var node = Assert.IsType<ForNode>(Single("for i in; do echo x; done"));
            Assert.Empty(node.Items);
        }

        [Fact]
        public void A_for_loop_variable_must_be_a_plain_unquoted_name()
        {
            Assert.Throws<ShellSyntaxException>(() => Parse("for 'i' in 1; do echo x; done"));
            Assert.Throws<ShellSyntaxException>(() => Parse("for 1i in 1; do echo x; done"));
        }

        // --- while / until ---

        [Fact]
        public void While_and_until_differ_only_by_the_Until_flag()
        {
            Assert.False(Assert.IsType<WhileNode>(Single("while a; do b; done")).Until);
            Assert.True(Assert.IsType<WhileNode>(Single("until a; do b; done")).Until);
        }

        [Fact]
        public void A_loop_condition_may_be_a_full_pipeline()
        {
            var node = Assert.IsType<WhileNode>(Single("while regs | grep -q x; do b; done"));
            var cond = Assert.IsType<AndOrList>(Assert.Single(node.Condition.Statements));
            Assert.Equal(2, cond.First.Stages.Count);
        }

        // --- break / continue ---

        [Fact]
        public void Break_and_continue_are_parser_keywords_not_commands()
        {
            Assert.IsType<BreakNode>(Single("break"));
            Assert.IsType<ContinueNode>(Single("continue"));
        }

        [Fact]
        public void Break_parses_inside_a_loop_body()
        {
            var node = Assert.IsType<WhileNode>(Single("while true; do break; done"));
            Assert.IsType<BreakNode>(Assert.Single(node.Body.Statements));
        }

        // --- incomplete vs. wrong ---

        [Theory]
        [InlineData("if true; then echo a")]
        [InlineData("if true")]
        [InlineData("for i in 1; do echo x")]
        [InlineData("while true; do echo x")]
        [InlineData("echo a &&")]
        [InlineData("echo a |")]
        [InlineData("echo a >")]
        public void An_unfinished_construct_reports_incomplete_rather_than_a_syntax_error(string src)
        {
            Assert.Throws<ShellIncompleteException>(() => Parse(src));
        }

        [Theory]
        [InlineData("for i 1 2; do echo x; done")]
        [InlineData("echo a > | b")]
        public void A_genuinely_wrong_token_reports_a_syntax_error(string src)
        {
            Assert.Throws<ShellSyntaxException>(() => Parse(src));
        }

        // A closing keyword is only a keyword where the grammar looks for one; the wrong one reads as an unclosed block.
        [Fact]
        public void A_mismatched_closing_keyword_reports_incomplete_not_a_syntax_error()
        {
            var ex = Assert.Throws<ShellIncompleteException>(() => Parse("if true; then echo a; done"));
            Assert.Contains("elif, else, fi", ex.Message);
        }

        // Outside any block these are ordinary words - the interpreter rejects them later as unknown commands.
        [Theory]
        [InlineData("fi")]
        [InlineData("done")]
        [InlineData("then")]
        public void A_stray_closing_keyword_parses_as_an_ordinary_command_word(string src)
        {
            Assert.Equal(new[] { src }, WordTexts(OnlyCommand(src)));
        }
    }
}
