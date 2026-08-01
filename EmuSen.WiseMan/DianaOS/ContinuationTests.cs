using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;

namespace EmuSen.WiseMan.DianaOS
{
    // The multi-line buffering the F4 prompt's secondary "> " depends on - see EmuSen_Debugging_Tools_Reference_v5.md §3.17.
    public class ContinuationTests
    {
        private static DianaOSInterpreter NewShell() => DianaOSInterpreter.CreateDefault(null);

        // Feeds lines one at a time the way a real prompt does, returning the final output.
        private static (string Output, bool StillWaiting) Feed(DianaOSInterpreter shell, params string[] lines)
        {
            string output = "";
            bool waiting = false;
            foreach (string line in lines)
            {
                (waiting, output, _) = shell.Submit(line);
            }
            return (output, waiting);
        }

        [Theory]
        [InlineData("echo 'unterminated", "still here'")]
        [InlineData("echo \"unterminated", "still here\"")]
        [InlineData("echo $(echo", "nested)")]
        public void An_open_construct_waits_then_completes_on_the_next_line(string first, string second)
        {
            var shell = NewShell();

            (_, bool waiting) = Feed(shell, first);
            Assert.True(waiting);
            Assert.True(shell.IsAwaitingMoreInput);

            (string output, bool stillWaiting) = Feed(shell, second);
            Assert.False(stillWaiting);
            Assert.False(shell.IsAwaitingMoreInput);
            Assert.NotEmpty(output);
        }

        [Fact]
        public void An_if_block_buffers_until_its_fi_arrives()
        {
            var shell = NewShell();

            Assert.True(Feed(shell, "if true; then").StillWaiting);
            Assert.True(Feed(shell, "echo inside").StillWaiting);
            Assert.True(shell.IsAwaitingMoreInput);

            (string output, bool waiting) = Feed(shell, "fi");
            Assert.False(waiting);
            Assert.Contains("inside", output);
        }

        [Fact]
        public void A_for_block_buffers_until_its_done_arrives()
        {
            var shell = NewShell();

            Assert.True(Feed(shell, "for i in 1 2 3; do").StillWaiting);
            (string output, bool waiting) = Feed(shell, "echo item $i", "done");

            Assert.False(waiting);
            Assert.Contains("item 1", output);
            Assert.Contains("item 3", output);
        }

        [Fact]
        public void A_while_block_buffers_until_its_done_arrives()
        {
            var shell = NewShell();

            Assert.True(Feed(shell, "X=0").StillWaiting == false);
            Assert.True(Feed(shell, "while false; do").StillWaiting);
            (_, bool waiting) = Feed(shell, "echo never", "done");
            Assert.False(waiting);
        }

        [Fact]
        public void Nested_blocks_only_finish_on_the_outermost_terminator()
        {
            var shell = NewShell();

            Assert.True(Feed(shell, "for i in 1; do").StillWaiting);
            Assert.True(Feed(shell, "if true; then").StillWaiting);
            Assert.True(Feed(shell, "echo deep").StillWaiting);
            Assert.True(Feed(shell, "fi").StillWaiting);

            (string output, bool waiting) = Feed(shell, "done");
            Assert.False(waiting);
            Assert.Contains("deep", output);
        }

        // A real syntax error must reset the buffer, not leave the prompt stuck forever.
        [Fact]
        public void A_syntax_error_clears_the_pending_buffer()
        {
            var shell = NewShell();

            Feed(shell, "if true; then");
            (string output, bool waiting) = Feed(shell, "echo a > | b");

            Assert.False(waiting);
            Assert.False(shell.IsAwaitingMoreInput);
            Assert.Contains("rror", output);
        }

        // The wrong closing keyword is an ordinary word, so the block stays open rather than
        // erroring - the prompt keeps asking, and the real 'fi' still finishes it.
        [Fact]
        public void The_wrong_closing_keyword_keeps_waiting_rather_than_erroring()
        {
            var shell = NewShell();

            Feed(shell, "if true; then", "echo a");
            Assert.True(Feed(shell, "done").StillWaiting);

            (string output, bool waiting) = Feed(shell, "fi");
            Assert.False(waiting);
            Assert.Contains("a", output);
        }

        // An unsupported operator throws out of the lexer; that path must clear the buffer too.
        [Fact]
        public void An_unsupported_operator_clears_the_pending_buffer()
        {
            var shell = NewShell();

            (string output, bool waiting) = Feed(shell, "sleep 1 &");

            Assert.False(waiting);
            Assert.False(shell.IsAwaitingMoreInput);
            Assert.Contains("rror", output);
        }

        [Fact]
        public void A_completed_block_is_recorded_in_history_once_as_one_entry()
        {
            var shell = NewShell();
            Feed(shell, "for i in 1; do", "echo x", "done");

            Assert.Single(shell.History.Entries.Where(e => e.Contains("for i in 1")));
            Assert.DoesNotContain(shell.History.Entries, e => e == "echo x");
        }

        [Fact]
        public void A_bare_break_outside_any_loop_is_silently_ignored()
        {
            var shell = NewShell();
            (string output, bool waiting) = Feed(shell, "break");

            Assert.False(waiting);
            Assert.DoesNotContain("Unhandled", output);
        }
    }
}
