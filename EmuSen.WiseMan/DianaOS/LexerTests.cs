using System.Linq;
using EmuSen.DianaOS.Ast;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.WiseMan.DianaOS
{
    // Characterization tests pinning Lexer.Tokenize's current behavior - see EmuSen_Debugging_Tools_Reference_v5.md §3.18a.
    public class LexerTests
    {
        // Renders one word's parts as "Kind[q]:text" joined by '+', so a test asserts part kinds/quoting/splitting at once.
        private static string Render(Word w) => string.Join("+", w.Parts.Select(p =>
            (p.Kind switch { PartKind.Literal => "L", PartKind.Variable => "V", _ => "C" })
            + (p.Quoted ? "q" : "") + ":" + p.Text));

        private static LexResult Complete(string src)
        {
            LexResult r = Lexer.Tokenize(src);
            Assert.False(r.NeedsMoreInput, $"'{src}' unexpectedly reported NeedsMoreInput.");
            return r;
        }

        private static string[] Words(string src) =>
            Complete(src).Tokens.Where(t => t.Type == TokenType.Word).Select(t => Render(t.Word!)).ToArray();

        private static TokenType[] Types(string src) => Complete(src).Tokens.Select(t => t.Type).ToArray();

        // --- word splitting ---

        [Fact]
        public void Unquoted_whitespace_splits_words()
        {
            Assert.Equal(new[] { "L:echo", "L:a", "L:b" }, Words("echo a b"));
        }

        [Fact]
        public void Runs_of_spaces_and_tabs_collapse()
        {
            Assert.Equal(new[] { "L:echo", "L:a" }, Words("echo \t  a"));
        }

        [Fact]
        public void Leading_and_trailing_whitespace_produce_no_empty_words()
        {
            Assert.Equal(new[] { "L:echo" }, Words("   echo   "));
        }

        [Fact]
        public void Empty_input_lexes_to_just_EOF()
        {
            Assert.Equal(new[] { TokenType.EOF }, Types(""));
        }

        // --- quoting ---

        [Fact]
        public void Single_quotes_make_one_word_and_mark_it_quoted()
        {
            Assert.Equal(new[] { "L:echo", "Lq:a b" }, Words("echo 'a b'"));
        }

        [Fact]
        public void Single_quotes_suppress_variable_expansion()
        {
            Assert.Equal(new[] { "Lq:$HOME" }, Words("'$HOME'"));
        }

        [Fact]
        public void Double_quotes_keep_one_word_but_still_expand_variables()
        {
            Assert.Equal(new[] { "Lq:a ", "Vq:HOME" }, Words("\"a \" \"$HOME\""));
        }

        [Fact]
        public void Double_quotes_do_not_word_split_on_inner_whitespace()
        {
            Assert.Equal(new[] { "echo", "a b" }, Words("echo \"a b\"").Select(w => w.Split(':')[1]).ToArray());
        }

        [Fact]
        public void An_empty_single_quoted_string_is_still_a_real_word()
        {
            Assert.Equal(new[] { "L:echo", "" }, Words("echo ''"));
        }

        [Fact]
        public void Double_quotes_inside_single_quotes_are_literal()
        {
            Assert.Equal(new[] { "Lq:say \"hi\"" }, Words("'say \"hi\"'"));
        }

        [Fact]
        public void Single_quote_inside_double_quotes_is_literal()
        {
            Assert.Equal(new[] { "Lq:it's" }, Words("\"it's\""));
        }

        [Fact]
        public void Operators_inside_double_quotes_are_literal_text()
        {
            Assert.Equal(new[] { TokenType.Word, TokenType.EOF }, Types("\"a | b ; c\""));
        }

        // --- backslash escaping ---

        [Fact]
        public void Backslash_escapes_a_space_into_the_same_word()
        {
            Assert.Equal(new[] { "L:echo", "L:a+Lq: +L:b" }, Words(@"echo a\ b"));
        }

        // The escaped '$' lands in its own part: AppendLiteralChar flushes whenever the quoted flag flips.
        [Fact]
        public void Backslash_escapes_a_dollar_sign_into_a_separate_literal_part()
        {
            Assert.Equal(new[] { "Lq:$+L:HOME" }, Words(@"\$HOME"));
        }

        [Fact]
        public void Inside_double_quotes_backslash_escapes_only_quote_backslash_and_dollar()
        {
            Assert.Equal(new[] { "Lq:\"" }, Words("\"\\\"\""));
            Assert.Equal(new[] { "Lq:$" }, Words("\"\\$\""));
        }

        [Fact]
        public void Inside_double_quotes_an_unrecognized_backslash_stays_literal()
        {
            Assert.Equal(new[] { @"Lq:a\nb" }, Words("\"a\\nb\""));
        }

        // --- expansion references ---

        [Fact]
        public void Bare_dollar_name_lexes_as_a_variable_part()
        {
            Assert.Equal(new[] { "V:HOME" }, Words("$HOME"));
        }

        [Fact]
        public void Braced_dollar_name_lexes_as_a_variable_part()
        {
            Assert.Equal(new[] { "V:HOME" }, Words("${HOME}"));
        }

        [Fact]
        public void Dollar_question_lexes_as_the_exit_code_variable()
        {
            Assert.Equal(new[] { "V:?" }, Words("$?"));
        }

        [Fact]
        public void A_variable_name_stops_at_the_first_non_identifier_character()
        {
            Assert.Equal(new[] { "V:VAR+L:-x" }, Words("$VAR-x"));
        }

        [Fact]
        public void A_dollar_sign_with_no_valid_name_after_it_is_a_literal_dollar()
        {
            Assert.Equal(new[] { "L:$" }, Words("$"));
            Assert.Equal(new[] { "L:$-" }, Words("$-"));
        }

        [Fact]
        public void Command_substitution_keeps_its_inner_text_unparsed()
        {
            Assert.Equal(new[] { "C:echo hi" }, Words("$(echo hi)"));
        }

        [Fact]
        public void Nested_parentheses_inside_command_substitution_are_depth_tracked()
        {
            Assert.Equal(new[] { "C:echo $(echo inner)" }, Words("$(echo $(echo inner))"));
        }

        [Fact]
        public void A_closing_paren_inside_quotes_does_not_end_a_command_substitution()
        {
            Assert.Equal(new[] { "C:echo ')'" }, Words("$(echo ')')"));
            Assert.Equal(new[] { "C:echo \")\"" }, Words("$(echo \")\")"));
        }

        [Fact]
        public void A_word_can_mix_literal_variable_and_substitution_parts_in_order()
        {
            Assert.Equal(new[] { "L:pre+V:VAR+L:post+Cq:cmd" }, Words("pre${VAR}post\"$(cmd)\""));
        }

        [Fact]
        public void An_assignment_value_containing_a_substitution_keeps_the_name_first()
        {
            Assert.Equal(new[] { "L:Y=+C:echo x" }, Words("Y=$(echo x)"));
        }

        // --- operators ---

        [Theory]
        [InlineData("a | b", TokenType.Pipe)]
        [InlineData("a || b", TokenType.OrOr)]
        [InlineData("a && b", TokenType.AndAnd)]
        [InlineData("a > b", TokenType.Greater)]
        [InlineData("a >> b", TokenType.DGreater)]
        [InlineData("a < b", TokenType.Less)]
        public void Operators_lex_to_their_token_type(string src, TokenType expected)
        {
            Assert.Contains(expected, Types(src));
        }

        [Fact]
        public void Semicolons_and_newlines_are_separate_separator_tokens()
        {
            Assert.Equal(new[] { TokenType.Word, TokenType.Semicolon, TokenType.Word, TokenType.EOF }, Types("a;b"));
            Assert.Equal(new[] { TokenType.Word, TokenType.Newline, TokenType.Word, TokenType.EOF }, Types("a\nb"));
        }

        [Fact]
        public void Operators_need_no_surrounding_whitespace()
        {
            Assert.Equal(new[] { TokenType.Word, TokenType.Pipe, TokenType.Word, TokenType.EOF }, Types("a|b"));
        }

        [Fact]
        public void A_bare_ampersand_is_rejected_as_an_unsupported_background_job()
        {
            Assert.Throws<System.NotSupportedException>(() => Lexer.Tokenize("sleep 1 &"));
        }

        [Fact]
        public void A_heredoc_operator_is_rejected_as_unsupported()
        {
            Assert.Throws<System.NotSupportedException>(() => Lexer.Tokenize("cat << EOF"));
        }

        // --- comments ---

        [Fact]
        public void A_hash_at_word_start_comments_out_the_rest_of_the_line()
        {
            Assert.Equal(new[] { "L:echo", "L:a" }, Words("echo a # trailing note"));
        }

        [Fact]
        public void A_hash_mid_word_is_ordinary_literal_text()
        {
            Assert.Equal(new[] { "L:echo", "L:a#b" }, Words("echo a#b"));
        }

        [Fact]
        public void A_comment_stops_at_the_newline_and_the_next_line_still_lexes()
        {
            Assert.Equal(new[] { "L:a", "L:b" }, Words("a # note\nb"));
        }

        // --- incomplete input ---

        [Theory]
        [InlineData("echo 'unterminated")]
        [InlineData("echo \"unterminated")]
        [InlineData("echo $(unterminated")]
        [InlineData("echo $(echo 'inner")]
        [InlineData("echo ${unterminated")]
        [InlineData(@"echo trailing\")]
        public void Unfinished_input_reports_NeedsMoreInput_rather_than_throwing(string src)
        {
            Assert.True(Lexer.Tokenize(src).NeedsMoreInput, $"'{src}' should report NeedsMoreInput.");
        }

        [Fact]
        public void A_documented_quirk_arithmetic_expansion_lexes_as_a_nested_substitution()
        {
            Assert.Equal(new[] { "C:(1+2)" }, Words("$((1+2))"));
        }
    }
}
