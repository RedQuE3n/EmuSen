using System.Collections.Generic;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.DianaOS.DianaOS.Bin.Commands.Unix;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;

namespace EmuSen.WiseMan.DianaOS
{
    // IDianaOSCommand.IsReadOnly - every implementing class states its own
    // classification (see that interface member's own comment). This isn't
    // a compile-safety check (the interface already forces every class to
    // define it) - it's a correctness check on the trickier calls: mixed
    // read/write sub-verb commands (watch/bp/framelog/cheat/snapshot/
    // search/history/log/trace) must report false even though they have a
    // read-only sub-verb, since IsReadOnly can't vary per invocation (see
    // that property's own doc comment).
    public class IsReadOnlyClassificationTests
    {
        [Fact]
        public void Plain_read_commands_report_read_only()
        {
            Assert.True(new MemCommand().IsReadOnly);
            Assert.True(new RegsCommand().IsReadOnly);
            Assert.True(new SpritesCommand().IsReadOnly);
            Assert.True(new PalCommand().IsReadOnly);
            Assert.True(new ChannelsCommand().IsReadOnly);
            Assert.True(new SpacesCommand().IsReadOnly);
            Assert.True(new DisasmCommand().IsReadOnly);
            Assert.True(new TileCommand().IsReadOnly);
            Assert.True(new TilemapCommand().IsReadOnly);
            Assert.True(new CallersCommand().IsReadOnly);
            Assert.True(new WritersCommand().IsReadOnly);
            Assert.True(new ReadersCommand().IsReadOnly);
            Assert.True(new DumpCommand().IsReadOnly);
            Assert.True(new ClearCommand().IsReadOnly);
            Assert.True(new EchoCommand().IsReadOnly);
            Assert.True(new TestCommand().IsReadOnly);
            Assert.True(new BracketCommand().IsReadOnly);
        }

        [Fact]
        public void Diff_is_read_only_but_its_sibling_snapshot_command_is_not()
        {
            var store = new SnapshotStore();
            Assert.True(new DiffCommand(store).IsReadOnly);
            Assert.False(new SnapshotCommand(store).IsReadOnly);
        }

        [Fact]
        public void Plain_mutating_commands_report_not_read_only()
        {
            Assert.False(new WriteCommand().IsReadOnly);
            Assert.False(new LoadCommand().IsReadOnly);
            Assert.False(new MuteCommand().IsReadOnly);
            Assert.False(new CdCommand(() => DianaOSInterpreter.CreateDefault(null)).IsReadOnly);
            Assert.False(new MvCommand().IsReadOnly);
            Assert.False(new NanoCommand().IsReadOnly);
            Assert.False(new StepCommand().IsReadOnly);
            Assert.False(new ResumeCommand().IsReadOnly);
            Assert.False(new ShutdownCommand().IsReadOnly);
            Assert.False(new CoreCommand(new Dictionary<string, CoreDescriptor>()).IsReadOnly);
            Assert.False(new StateCommand(_ => { }, _ => { }, () => "").IsReadOnly);
        }

        [Fact]
        public void Mixed_readwrite_subverb_commands_conservatively_report_not_read_only()
        {
            Assert.False(new WatchCommand().IsReadOnly);
            Assert.False(new BreakCommand().IsReadOnly);
            Assert.False(new FrameLogCommand().IsReadOnly);
            Assert.False(new CheatCommand().IsReadOnly);
            Assert.False(new LogCommand().IsReadOnly);
            Assert.False(new TraceCommand().IsReadOnly);
            Assert.False(new SearchCommand().IsReadOnly);
            Assert.False(new HistoryCommand(new CommandHistory()).IsReadOnly);
        }

        [Fact]
        public void Coretop_is_not_fast_path_eligible_despite_never_writing_core_state()
        {
            // Raw-terminal coretop blocks the calling thread indefinitely
            // (its own Console.ReadKey loop until Ctrl+C) - fast-pathing
            // it would let it hold the live terminal's interpreter lock
            // open-ended, which could stall a queued mutating command's
            // turn on the emulation thread. Queued (false) instead, same
            // blocking-while-open category as `nano`.
            Assert.False(new CoretopCommand().IsReadOnly);
        }
    }
}
