using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands
{
    // Long-form manual pages for `man <command>` (DianaOSInterpreter.Dispatch's
    // own special case, alongside `help`/`export`/`unset`/`source` -
    // see that method's own comment for why `man` isn't just another
    // IDianaOSCommand). Deliberately a separate, centralized module
    // rather than a `ManPage` member added to IDianaOSCommand itself:
    // that would force all ~40 existing command classes (most of them
    // one-liners) to carry a paragraph of documentation text alongside
    // their actual logic, for a feature that's purely about the shell's
    // own help system, not about how any individual command works. One
    // file, one place to keep this in sync, no per-command interface
    // churn - the same reasoning DebugCommandHelpers already uses for
    // shared, stateless command-support code that doesn't belong wedged
    // into any one command class. Lives directly under Commands/ rather
    // than Commands/Unix or Commands/EmuSen - it documents commands from
    // both, and isn't an IDianaOSCommand itself, so neither subfolder fits.
    //
    // Each page follows the same loose structure a real Unix man page
    // does (NAME/SYNOPSIS/DESCRIPTION, EXAMPLES where an example
    // actually clarifies something the synopsis alone doesn't) without
    // being slavish about it - this is a debug console's help text, not
    // a formal reference manual. `man <command>` with no matching entry
    // here falls back to that command's own one-line `Usage` in
    // DianaOSInterpreter.Dispatch, so a newly-added command still gets
    // *something* useful before anyone gets around to writing its real
    // page - `Lookup` returning null is an expected, handled case, not
    // a bug to fix by adding a page for absolutely everything the
    // instant a command is created.
    public static class ManPages
    {
        public static string? Lookup(string name) =>
            _pages.TryGetValue(name, out string? page) ? page : null;

        private static readonly Dictionary<string, string> _pages = new(System.StringComparer.OrdinalIgnoreCase)
        {
            ["man"] =
                "NAME\n" +
                "    man - show a command's manual page\n\n" +
                "SYNOPSIS\n" +
                "    man [command]\n\n" +
                "DESCRIPTION\n" +
                "    The detail lookup: with a command name, prints that command's full\n" +
                "    manual page (this text) instead of just its one-line usage. With no\n" +
                "    argument, there's nothing else useful to show, so it falls back to the\n" +
                "    same overview 'help' prints. 'help' and 'man' are two separate commands,\n" +
                "    not aliases of each other - 'help' always lists everything and never\n" +
                "    shows a single command's full page; 'man' is the other way around.\n\n" +
                "EXAMPLES\n" +
                "    man\n" +
                "    man watch",

            ["help"] =
                "NAME\n" +
                "    help - list every command with a brief explanation\n\n" +
                "SYNOPSIS\n" +
                "    help\n\n" +
                "DESCRIPTION\n" +
                "    Prints every command, one line each, with a brief explanation of what it\n" +
                "    does - the quick \"what's available\" overview. Ignores any arguments;\n" +
                "    for a specific command's full manual page, use 'man <command>' instead.",

            ["summary"] =
                "NAME\n" +
                "    summary - free-text state dump\n\n" +
                "SYNOPSIS\n" +
                "    summary\n\n" +
                "DESCRIPTION\n" +
                "    Prints whatever the current IDebugTarget's GetSummaryText() has to say -\n" +
                "    a catch-all, unstructured dump for state that doesn't have its own\n" +
                "    dedicated command yet. 'No target attached.' if no ROM is loaded.",

            ["export"] =
                "NAME\n" +
                "    export - set a shell variable\n\n" +
                "SYNOPSIS\n" +
                "    export NAME[=value]\n\n" +
                "DESCRIPTION\n" +
                "    Real bash exports a variable into child processes' environment; there is\n" +
                "    no such thing here (this shell never spawns a subprocess), so 'export' is\n" +
                "    just a plain variable assignment - or a no-op declaration if no '=value'\n" +
                "    is given and the name is already set. Kept purely so a script written\n" +
                "    with real bash habits ('export FOO=bar') still does something sane\n" +
                "    instead of erroring as an unknown command. Plain 'NAME=value' (no\n" +
                "    'export' keyword) works exactly the same way.\n\n" +
                "EXAMPLES\n" +
                "    export FOO=bar\n" +
                "    echo $FOO",

            ["unset"] =
                "NAME\n" +
                "    unset - remove a shell variable\n\n" +
                "SYNOPSIS\n" +
                "    unset NAME\n\n" +
                "DESCRIPTION\n" +
                "    Removes NAME from this shell's variables. Referencing an unset variable\n" +
                "    afterward ($NAME) expands to an empty string, same as bash.",

            ["source"] =
                "NAME\n" +
                "    source, . - run a script file in this shell's own scope\n\n" +
                "SYNOPSIS\n" +
                "    source <path>\n" +
                "    . <path>\n\n" +
                "DESCRIPTION\n" +
                "    Runs <path>'s lines one at a time, exactly as if each had been typed at\n" +
                "    the prompt - including multi-line if/for/while blocks spanning several\n" +
                "    physical lines in the file. Runs in THIS shell's own variable scope, not\n" +
                "    an isolated subshell: a variable the script sets is still set once\n" +
                "    'source' returns, matching real bash 'source' (and unlike $(...) command\n" +
                "    substitution, which does run in an isolated subshell - see 'man' on\n" +
                "    command substitution... there isn't one; see the shell reference doc).\n" +
                "    Blank lines and lines starting with '#' are skipped. A script's own\n" +
                "    lines are NOT recorded into 'history'/!N recall, and '!'/'!!' expansion\n" +
                "    is disabled for them - only the 'source' invocation itself shows up in\n" +
                "    history, matching real bash. Walled to the project's own directory tree\n" +
                "    (the same sandbox 'cd'/'ls'/'mv' use) - a path outside it is rejected.\n" +
                "    A script ending mid-block, or a self-referential/mutually-recursive\n" +
                "    'source' chain (capped at 20 nested levels), reports a clean error\n" +
                "    instead of hanging the console or crashing it.\n\n" +
                "EXAMPLES\n" +
                "    source setup.txt\n" +
                "    . setup.txt",

            ["if"] =
                "NAME\n" +
                "    if - conditional execution\n\n" +
                "SYNOPSIS\n" +
                "    if LIST; then LIST; [elif LIST; then LIST;] ... [else LIST;] fi\n\n" +
                "DESCRIPTION\n" +
                "    Runs the 'if' condition; if its last pipeline exits 0 (success), runs the\n" +
                "    'then' body and stops. Otherwise tries each 'elif' condition in turn the\n" +
                "    same way, finally falling through to 'else' if none matched. Works\n" +
                "    across multiple physical lines at the interactive prompt (a secondary\n" +
                "    '> ' prompt appears until 'fi' closes it) and inside a 'source'd script\n" +
                "    the same way.\n\n" +
                "EXAMPLES\n" +
                "    if [ -f save.state ]; then echo found; else echo missing; fi",

            ["for"] =
                "NAME\n" +
                "    for - loop over a list of words\n\n" +
                "SYNOPSIS\n" +
                "    for NAME in word...; do LIST; done\n\n" +
                "DESCRIPTION\n" +
                "    Sets variable NAME to each word in turn (already expanded - quoting,\n" +
                "    $VAR, and $(...) all apply to the word list) and runs the body once per\n" +
                "    word. 'break' exits the loop early; 'continue' skips to the next word.\n" +
                "    Every iteration counts against a shared 10-second wall-clock safety\n" +
                "    timeout (see 'while') that aborts a runaway loop with a clear error.\n\n" +
                "EXAMPLES\n" +
                "    for i in 1 2 3; do echo $i; done",

            ["while"] =
                "NAME\n" +
                "    while, until - loop while/until a condition holds\n\n" +
                "SYNOPSIS\n" +
                "    while LIST; do LIST; done\n" +
                "    until LIST; do LIST; done\n\n" +
                "DESCRIPTION\n" +
                "    'while' repeats its body as long as the condition's last pipeline exits\n" +
                "    0; 'until' repeats as long as it does NOT (i.e. until it succeeds). Same\n" +
                "    'break'/'continue' as 'for'. A shared 10-second wall-clock budget (also\n" +
                "    counted against by any nested $(...) command substitution, so it can't\n" +
                "    be bypassed that way) aborts a typo'd infinite loop with a clear error\n" +
                "    instead of hanging the console - a real 'while true; do echo x; done'\n" +
                "    prints for about 10 seconds, then cleanly reports the timeout.\n\n" +
                "EXAMPLES\n" +
                "    while false; do echo never; done\n" +
                "    until [ -f ready.txt ]; do echo waiting; done",

            ["break"] =
                "NAME\n" +
                "    break - exit the nearest enclosing loop\n\n" +
                "SYNOPSIS\n" +
                "    break\n\n" +
                "DESCRIPTION\n" +
                "    A hardcoded, zero-argument parser keyword (not an ordinary dispatched\n" +
                "    command), same as real bash's own break/continue builtins. Ends the\n" +
                "    nearest enclosing 'for'/'while'/'until' loop immediately. Outside any\n" +
                "    loop it's a silent no-op, matching bash. Note: the breakpoint command is\n" +
                "    named 'bp', not 'break' - a command literally named 'break' would be\n" +
                "    unreachable, since this keyword is recognized before any command lookup\n" +
                "    happens at all.",

            ["continue"] =
                "NAME\n" +
                "    continue - skip to the next iteration of the nearest enclosing loop\n\n" +
                "SYNOPSIS\n" +
                "    continue\n\n" +
                "DESCRIPTION\n" +
                "    A hardcoded, zero-argument parser keyword, same as 'break'. Skips the\n" +
                "    rest of the current loop body and re-evaluates the loop's own condition\n" +
                "    (or advances to the next word, for 'for'). Outside any loop it's a\n" +
                "    silent no-op, matching bash.",

            ["spaces"] =
                "NAME\n" +
                "    spaces - list available memory spaces\n\n" +
                "SYNOPSIS\n" +
                "    spaces\n\n" +
                "DESCRIPTION\n" +
                "    Lists every named memory space the current core's IDebugTarget exposes -\n" +
                "    the <space> argument every mem/write/watch/dump/... command takes - along\n" +
                "    with each one's size and whether it's writable. Run this first if a\n" +
                "    space name isn't already known.",

            ["mem"] =
                "NAME\n" +
                "    mem - hexdump a range of memory\n\n" +
                "SYNOPSIS\n" +
                "    mem <space> <addr> [<len>]\n\n" +
                "DESCRIPTION\n" +
                "    Prints <len> bytes (default 16) from <space> starting at <addr> as a\n" +
                "    classic hex + ASCII dump, 16 bytes per row. <space> is one of the names\n" +
                "    'spaces' lists. Addresses/lengths are hex (an optional 0x or $ prefix is\n" +
                "    fine either way).\n\n" +
                "EXAMPLES\n" +
                "    mem WRAM 0\n" +
                "    mem CpuBus 8000 64",

            ["write"] =
                "NAME\n" +
                "    write - write one byte to memory\n\n" +
                "SYNOPSIS\n" +
                "    write <space> <addr> <value>\n\n" +
                "DESCRIPTION\n" +
                "    Writes a single byte to <space> at <addr>. Refuses outright if <space>\n" +
                "    isn't writable (see 'spaces'). No pause/synchronization with the\n" +
                "    emulation thread beyond whatever the current frontend provides (see the\n" +
                "    'pause'/'resume' commands in EmuSen.Mistress's console window).\n\n" +
                "EXAMPLES\n" +
                "    write WRAM 10 ff",

            ["regs"] =
                "NAME\n" +
                "    regs - dump CPU/video/APU registers\n\n" +
                "SYNOPSIS\n" +
                "    regs\n" +
                "    regs <cpu>\n\n" +
                "DESCRIPTION\n" +
                "    Prints the current core's CPU registers, video (PPU) registers, and (if\n" +
                "    the core reports any) APU registers, each in their own section. Field\n" +
                "    widths adapt to each register's real bit width.\n\n" +
                "OTHER PROCESSORS\n" +
                "    'regs <cpu>' prints just one chip's registers - see 'man cpus'. Bare\n" +
                "    'regs' keeps printing every section at once, which stays the right\n" +
                "    default when the question is 'what is the machine doing' rather than\n" +
                "    'what is this one chip doing'.\n\n" +
                "    Every value here is read through a side-effect-free view, never the\n" +
                "    chip's own register window - reading a real coprocessor status register\n" +
                "    can acknowledge an interrupt or advance a transfer handshake, and a\n" +
                "    debugger that did that would change the run it is meant to observe.\n\n" +
                "SEE ALSO\n" +
                "    cpus, eval, cophist, copflow, watch",

            ["cophist"] =
                "NAME\n" +
                "    cophist - coprocessor register history\n\n" +
                "SYNOPSIS\n" +
                "    cophist [<reg>] [<count>]\n\n" +
                "DESCRIPTION\n" +
                "    Prints the retained history of the cartridge coprocessor's registers -\n" +
                "    the SuperFX GSU, the SA-1, or a NEC DSP - one row per refresh, oldest\n" +
                "    first. Where `regs` shows the instant, this shows the run-up to it.\n\n" +
                "    Give <reg> to follow a single register (SFR, PBR, R15, ...) instead of\n" +
                "    the whole file, and <count> to change how many rows are shown from the\n" +
                "    default 16. The header also reports how many refreshes the registers\n" +
                "    have gone unchanged, which is the direct answer to \"when did the chip\n" +
                "    stop\".\n\n" +
                "    Prints nothing useful on a cartridge with no coprocessor, which is most\n" +
                "    of them.\n\n" +
                "EXAMPLES\n" +
                "    cophist\n" +
                "    cophist SFR 64\n" +
                "    cophist R15",

            ["sprites"] =
                "NAME\n" +
                "    sprites - dump the active sprite/OBJ table\n\n" +
                "SYNOPSIS\n" +
                "    sprites\n\n" +
                "DESCRIPTION\n" +
                "    Lists every active sprite: index, position, size, tile index, palette,\n" +
                "    priority, and horizontal/vertical flip state.",

            ["pal"] =
                "NAME\n" +
                "    pal - dump a color palette\n\n" +
                "SYNOPSIS\n" +
                "    pal [<index>]\n\n" +
                "DESCRIPTION\n" +
                "    Prints every palette's colors as hex RRGGBB triples, or just <index>'s\n" +
                "    palette if given.\n\n" +
                "EXAMPLES\n" +
                "    pal\n" +
                "    pal 3",

            ["channels"] =
                "NAME\n" +
                "    channels - list audio channels/voices\n\n" +
                "SYNOPSIS\n" +
                "    channels\n\n" +
                "DESCRIPTION\n" +
                "    Lists every audio channel/voice the current core reports: index, active\n" +
                "    state, envelope level (0-100), muted state, and core-specific detail.\n" +
                "    Pair with 'mute' to isolate one voice for listening/measurement.",

            ["mute"] =
                "NAME\n" +
                "    mute - mute or unmute one audio channel\n\n" +
                "SYNOPSIS\n" +
                "    mute <index> <on|off>\n\n" +
                "DESCRIPTION\n" +
                "    Excludes (or restores) one channel from the final audio mix. The\n" +
                "    channel's own playback/envelope state keeps advancing regardless -\n" +
                "    muting only affects the mix, not the underlying voice. Useful with\n" +
                "    'channels' to find a suspect voice, mute every other one, then listen\n" +
                "    to (or dump) just that voice in isolation.\n\n" +
                "EXAMPLES\n" +
                "    mute 2 on\n" +
                "    mute 2 off",

            ["watch"] =
                "NAME\n" +
                "    watch - track reads/writes to a memory range\n\n" +
                "SYNOPSIS\n" +
                "    watch add <space> <addr> <len> [write|read|both]\n" +
                "    watch list\n" +
                "    watch log <id> [<count>]\n" +
                "    watch summary <id>\n" +
                "    watch clear <id>\n" +
                "    watch remove <id>\n\n" +
                "DESCRIPTION\n" +
                "    'add' registers a watch over <addr>..<addr>+<len>-1 in <space> (default\n" +
                "    kind: write) and starts recording matching accesses as they happen -\n" +
                "    this is a LIVE, triggered mechanism (see 'framelog' for the sampled-\n" +
                "    every-frame alternative, and 'callers'/'writers'/'readers' for static\n" +
                "    analysis that doesn't require the access to actually happen first).\n" +
                "    'log' shows a watch's recorded (sequence, kind, address, value, context)\n" +
                "    events, newest last. 'summary' groups those events by access site\n" +
                "    instead of listing every one - the dynamic, addressing-mode-agnostic\n" +
                "    equivalent of 'readers'/'writers'. 'clear' empties a watch's recorded\n" +
                "    events without removing the watch itself; 'remove' deletes it entirely.\n\n" +
                "EXAMPLES\n" +
                "    watch add WRAM 13c6 1\n" +
                "    watch log 1\n" +
                "    watch summary 1",

            ["bp"] =
                "NAME\n" +
                "    bp - manage execution breakpoints\n\n" +
                "SYNOPSIS\n" +
                "    bp add <addr>[-<end>] [if <expr>] [log <expr>]\n" +
                "    bp write <space> <addr>[-<end>] [<value>] [changed] [if <expr>]\n" +
                "    bp read <space> <addr>[-<end>] [<value>] [if <expr>]\n" +
                "    bp uninit <space>\n" +
                "    bp when [<condition>] [log|off]\n" +
                "    bp depth <n>|off\n" +
                "    bp forbid <addr>-<end>\n" +
                "    bp log [<count>] | bp log clear\n" +
                "    bp list\n" +
                "    bp on|off <id>\n" +
                "    bp remove <id>\n" +
                "    bp <cpu> add|list|remove ...\n\n" +
                "CONDITIONS\n" +
                "    An 'if <expr>' suffix turns a breakpoint into a conditional one: the\n" +
                "    address (or the write) still has to match, and then <expr> has to\n" +
                "    evaluate nonzero before execution actually halts. This is what makes a\n" +
                "    breakpoint usable on a routine that runs thousands of times a frame -\n" +
                "    'bp add 80A31C if x == 7' stops on the one iteration that matters\n" +
                "    instead of the first. See 'man eval' for the expression language; every\n" +
                "    symbol and memory form it documents works here unchanged.\n\n" +
                "    Conditions are evaluated on the emulation thread, once per matching\n" +
                "    hit, so an expensive one costs real frame time. Reading an I/O register\n" +
                "    in a condition reads it for real, with whatever side effect that has on\n" +
                "    hardware - 'man eval' covers which reads are and aren't safe.\n\n" +
                "    A condition that fails to evaluate (a typo, an unknown symbol, an\n" +
                "    unreadable address) is DROPPED rather than left to fire on every\n" +
                "    instruction forever: the breakpoint halts once, keeps its address, and\n" +
                "    loses its condition. The error text is reported once at that halt.\n" +
                "    Re-add the breakpoint once the expression is fixed.\n\n" +
                "    Quote a condition containing '&&', '||', '<', '>' or '|' - those are\n" +
                "    the shell's own operators and get consumed before this command ever\n" +
                "    sees them. Everything after the 'if' word is rejoined with single\n" +
                "    spaces, so an unquoted 'if a == 5' works fine.\n\n" +
                "DESCRIPTION\n" +
                "    Registers/lists/removes a breakpoint at a 24-bit CPU address. This\n" +
                "    command only edits the breakpoint list - it doesn't halt or resume\n" +
                "    execution itself, since that requires re-entering a core's own frame\n" +
                "    loop, which only the frontend's own main loop can do. See the F4\n" +
                "    prompt's own 'step'/'s' and 'continue'/'c' shortcuts (EmuSen.Hotaru) for\n" +
                "    the actual halt/resume side. Named 'bp', not 'break' - a command\n" +
                "    literally named 'break' would be unreachable, shadowed by the shell's\n" +
                "    own hardcoded break/continue loop-control keywords.\n\n" +
                "    'bp write' breaks on data instead of on control flow: it halts when\n" +
                "    anything writes <addr> in <space>, optionally only when the value\n" +
                "    written equals <value>. Where 'watch' records a write and carries on,\n" +
                "    this stops the machine there, so 'regs'/'mem' can read the state that\n" +
                "    produced it - the way to answer 'what was the source pointer' rather\n" +
                "    than just 'which PC stored it'. A write happens part-way through an\n" +
                "    instruction, so the halt lands on the instruction AFTER the store,\n" +
                "    exactly as a real debugger reports a data breakpoint.\n\n" +
                "    An optional 'sa1' (or 'cop') scope word in front of the subcommand\n" +
                "    targets a cartridge coprocessor's own CPU instead. The two live in\n" +
                "    separate lists because they are separate address spaces: an SA-1\n" +
                "    game's $00:82D7 is not the S-CPU's $00:82D7. Errors with 'no\n" +
                "    coprocessor CPU to break on' when the cartridge has none. Pair it\n" +
                "    with 'disasm SA1BUS <addr>', which decodes using the SA-1's own\n" +
                "    M/X/E flags rather than the S-CPU's.\n\n" +
                "    'bp read' is the mirror image, halting just after anything reads the\n" +
                "    address. Where 'readers' scans the ROM for instructions that COULD read\n" +
                "    something, this catches the read that actually happened, including ones\n" +
                "    through a computed pointer that no static scan can resolve - the way to\n" +
                "    find out who consumes a flag rather than who sets it.\n\n" +
                "    'bp on'/'bp off' toggle one breakpoint without losing its address,\n" +
                "    condition or hit count - the difference between silencing a breakpoint\n" +
                "    for one experiment and having to retype it afterwards.\n\n" +
                "    'bp list' resolves each address through the label registry, so a\n" +
                "    breakpoint on a named address reads '$00A3B2 <NmiHandler>'. See\n" +
                "    'man label'. A range prints as a range, since a label names one address.\n\n" +
                "RANGES\n" +
                "    Any address argument accepts '<start>-<end>' instead of one address, and\n" +
                "    the breakpoint then matches anywhere inside it. This is the difference\n" +
                "    between needing to already know the answer and being able to ask the\n" +
                "    question: 'bp write WRAM 0100-01FF' catches whoever is clobbering a\n" +
                "    struct when you do not yet know which byte of it moves first, and\n" +
                "    'bp add 818000-818FFF' catches whatever in a bank runs at all.\n\n" +
                "    The halt message names the exact address that matched as well as the\n" +
                "    range, so a range breakpoint tells you where it landed, not just that\n" +
                "    it landed.\n\n" +
                "CHANGED-ONLY WRITES\n" +
                "    'changed' on a write breakpoint suppresses the halt unless the byte\n" +
                "    written differs from the last one seen at that address. Games rewrite\n" +
                "    the same value constantly - a per-frame state byte re-stored every frame\n" +
                "    will trip a plain write breakpoint sixty times a second while telling\n" +
                "    you nothing. 'bp write WRAM 0DB3 changed' fires on the transition\n" +
                "    instead, which is almost always the moment being looked for.\n\n" +
                "    The first write to an address always counts as a change: there is no\n" +
                "    previous value to compare against, and a value written exactly once\n" +
                "    would otherwise never be reported at all. The comparison is against\n" +
                "    what this breakpoint last SAW, not against memory, so it costs no read\n" +
                "    and cannot disturb a side-effecting address.\n\n" +
                "UNINITIALIZED READS\n" +
                "    'bp uninit <space>' halts the first time the machine reads back a byte\n" +
                "    of <space> that nothing has written since the breakpoint was armed. That\n" +
                "    is a real defect nearly every time it happens: the game is consuming\n" +
                "    whatever the previous contents were, so its behavior depends on power-on\n" +
                "    garbage. It is also the single most useful check for the emulator\n" +
                "    itself, because it catches this core failing to initialize something\n" +
                "    that real hardware would have.\n\n" +
                "    Each address reports once and is then treated as initialized, or a boot\n" +
                "    loop over uninitialized memory would halt on the same byte forever.\n" +
                "    Arming allocates one bool per byte of the space and replaces any\n" +
                "    previous arming - one space is watched at a time. Note that a space\n" +
                "    written before arming counts as initialized; arm it early to catch\n" +
                "    reset-time reads. 'counters' reports the same thing as a whole-run\n" +
                "    tally when a halt is too blunt.\n\n" +
                "HARDWARE CONDITIONS\n" +
                "    'bp when <condition>' halts when the machine does something it should\n" +
                "    not, rather than when it reaches an address. 'bp when' with no argument\n" +
                "    lists what this core can detect, with each one's current state. On the\n" +
                "    SNES that is:\n\n" +
                "        stp         the 65816 executed STP and is stopped until reset\n" +
                "        wdm         the 65816 executed WDM, a reserved opcode\n" +
                "        brk         a BRK was taken\n" +
                "        cop         a COP was taken\n" +
                "        ppuaccess   VRAM/CGRAM/OAM written while the display is rendering\n" +
                "        autojoy     $4218-$421F read while the auto-joypad read is running\n\n" +
                "    These matter because none of them are visible in the output. STP is the\n" +
                "    clearest case: a game that executes it has usually jumped somewhere it\n" +
                "    never meant to, and the symptom is simply that the picture stops - with\n" +
                "    no indication of where. 'bp when stp' halts on the instruction itself,\n" +
                "    while the call stack that got there is still readable with 'bt'.\n\n" +
                "    'ppuaccess' and 'autojoy' are the two where the emulator and the game\n" +
                "    disagree most quietly. Real hardware drops a VRAM write made during\n" +
                "    active display, and returns a half-updated byte from a joypad register\n" +
                "    read mid-refresh; a core that is more permissive than the hardware runs\n" +
                "    such a game correctly and hides a real bug, and a core that is less\n" +
                "    permissive breaks one that worked. Either way the first thing you want\n" +
                "    is to know it happened at all.\n\n" +
                "    A condition can be armed 'log' instead, which records every occurrence\n" +
                "    and carries on. That is the right mode for anything the game does often\n" +
                "    - a title screen may read the joypad early on every single frame, and\n" +
                "    halting on it tells you far less than seeing the pattern across frames.\n" +
                "    Logged conditions land in the same ring as logpoints and are read back\n" +
                "    with 'bp log', where they interleave with everything else in the order\n" +
                "    it really happened. 'bp when <condition> off' disarms one.\n\n" +
                "    Conditions cost nothing until one is armed: each detection site tests a\n" +
                "    single bool before computing anything.\n\n" +
                "DEPTH GUARD\n" +
                "    'bp depth <n>' halts as soon as the call stack gets deeper than <n>.\n" +
                "    Runaway recursion and a stack that never unwinds both present as a game\n" +
                "    that slows down and then behaves bizarrely, long after the actual defect\n" +
                "    - by the time anything visible goes wrong the evidence is gone. This\n" +
                "    stops the machine while the chain that caused it is still on the stack\n" +
                "    and readable with 'bt'. Fires once and disarms, so it will not retrigger\n" +
                "    on every instruction that stays deep. 'bp depth off' cancels it.\n\n" +
                "FORBID RANGES\n" +
                "    'bp forbid <start>-<end>' is a suppression rule, not a breakpoint:\n" +
                "    while the PC is anywhere inside the range, nothing halts - no address\n" +
                "    breakpoint, no data breakpoint, no uninitialized read. It exists for\n" +
                "    the case where the interesting breakpoint is drowned by one noisy\n" +
                "    caller: forbid the NMI handler's range and a watch on a shadow register\n" +
                "    reports only the game logic that writes it, not the per-frame flush.\n\n" +
                "    Steps are deliberately exempt. 'step' and 'step over' still work inside\n" +
                "    a forbidden range, or stepping into one would run away with no way to\n" +
                "    stop. Forbid ranges are listed by 'bp list' and removed by 'bp remove'\n" +
                "    like anything else, and 'bp off <id>' suspends one without losing it.\n\n" +
                "LOGPOINTS\n" +
                "    A 'log <expr>' clause turns a breakpoint into a logpoint: when it matches,\n" +
                "    <expr> is evaluated and recorded, and execution CARRIES ON. Read the\n" +
                "    recording back with 'bp log'.\n\n" +
                "    This is for the case a halting breakpoint cannot answer. A routine that\n" +
                "    runs four hundred times a frame cannot be stepped through four hundred\n" +
                "    times, and halting even once changes the timing of everything after it.\n" +
                "    'bp add 80A31C log \"a, x, [$7E0DB3]\"' records all four hundred and leaves\n" +
                "    the machine running at full speed, so the pattern across calls is visible\n" +
                "    rather than one arbitrary call being inspected in isolation.\n\n" +
                "    <expr> is a comma-separated list, and each part is rendered as 'expr=value'\n" +
                "    so a line names what it is showing. 'uote it - commas are fine unquoted\n" +
                "    but the shell eats the brackets and operators that make it worth logging.\n" +
                "    Every symbol 'man eval' documents works, evaluated against the chip the\n" +
                "    breakpoint belongs to.\n\n" +
                "    A logpoint still honors 'if ', so the two compose: match the address,\n" +
                "    check the condition, then record instead of halting. A logpoint that\n" +
                "    fails to evaluate records the error text in place of the value rather\n" +
                "    than being dropped the way a bad condition is - a logpoint cannot fire\n" +
                "    forever on every instruction, so there is nothing to protect against.\n\n" +
                "    The log keeps the most recent 4096 entries and is shared by every\n" +
                "    logpoint on that processor, so entries interleave in the order they\n" +
                "    actually happened. 'bp log' shows the last 32 by default, 'bp log 100'\n" +
                "    more, and 'bp log clear' empties it. Data logpoints record at the moment\n" +
                "    of the access rather than one instruction later, so a logged write shows\n" +
                "    the value that was written.\n\n" +
                "EXAMPLES\n" +
                "    bp add 8000\n" +
                "    bp add 80A31C if x == 7\n" +
                "    bp add 80A31C log \"a, x\"\n" +
                "    bp write WRAM 0DB3 log \"a, [$7E0100]\" if a > 4\n" +
                "    bp log 100\n" +
                "    bp add 818000-818FFF\n" +
                "    bp add 808000 if \"[$7E0020] != 0 && a > 16\"\n" +
                "    bp write VRAM 2760\n" +
                "    bp write WRAM 0100-01FF\n" +
                "    bp write WRAM 0DB3 changed\n" +
                "    bp read WRAM 13c6\n" +
                "    bp uninit WRAM\n" +
                "    bp depth 40\n" +
                "    bp forbid 0080C4-0080FF\n" +
                "    bp off 1\n" +
                "    bp list\n" +
                "    bp sa1 add 0082D7\n" +
                "    bp gsu add 0A80E9 if r14 > $100\n\n" +
                "OTHER PROCESSORS\n" +
                "    A scope word puts the breakpoint on another chip - see 'cpus' for the\n" +
                "    names. Each chip keeps its own breakpoint list, because each runs\n" +
                "    different code at the same addresses, and a condition attached to one\n" +
                "    is evaluated against THAT chip's registers: 'bp gsu add X if r14 > 0'\n" +
                "    reads the GSU's R14, not anything on the main CPU.\n\n" +
                "    A chip the core cannot halt takes no breakpoints and says so rather\n" +
                "    than accepting one that would never fire.\n\n" +
                "SEE ALSO\n" +
                "    cpus, eval, step, runto, bt, watch, cov, counters, readers, writers",

            ["setreg"] =
                "NAME\n" +
                "    setreg - write a CPU register\n\n" +
                "SYNOPSIS\n" +
                "    setreg <register> <value>\n" +
                "    setreg <cpu> <register> <value>\n\n" +
                "DESCRIPTION\n" +
                "    Sets one of a processor's registers to a value. Until this existed the\n" +
                "    debugger could read every register and change none of them, which meant\n" +
                "    a hypothesis could only ever be tested by editing memory and waiting to\n" +
                "    see whether the machine happened to reload the register from it.\n\n" +
                "    What it is actually for is counterfactuals. Halt on the compare that\n" +
                "    gates a routine, flip the register it compares, resume, and watch what\n" +
                "    the game does down the branch it did not take. That answers 'is this\n" +
                "    check the reason my sprite never appears' in one step, where coverage\n" +
                "    and breakpoints can only establish that the branch was not taken.\n\n" +
                "    It also skips a hang: forcing PC past a spin loop lets a boot sequence\n" +
                "    continue far enough to show what the NEXT problem is, which is often\n" +
                "    worth more than the one you are stopped on.\n\n" +
                "    Values are hex, and are REFUSED rather than truncated if they do not fit\n" +
                "    the register's width - a silently masked value would be a wrong answer\n" +
                "    reported as a success. An unknown register name lists the ones that chip\n" +
                "    actually has.\n\n" +
                "    Kept separate from 'regs' on purpose. 'regs' is read-only, which is what\n" +
                "    lets an always-live shell answer it instantly off the emulation thread;\n" +
                "    folding a write into it would drag every register dump onto the\n" +
                "    emulation thread's next frame, and writing a register from another\n" +
                "    thread would be a race besides.\n\n" +
                "OTHER PROCESSORS\n" +
                "    A scope word writes another chip's registers - 'cpus' lists which ones\n" +
                "    support it at all. Not every chip does: the GSU and the NEC DSP report\n" +
                "    their registers through debug accessors with no write path behind them,\n" +
                "    so they refuse rather than pretending. 'cpus' prints 'setreg' in a\n" +
                "    chip's supported list only when it really is writable.\n\n" +
                "    Take care with a halted coprocessor: only one chip halts at a time, so\n" +
                "    writing a register on a chip that is still running races its own\n" +
                "    execution.\n\n" +
                "EXAMPLES\n" +
                "    setreg a 0007\n" +
                "    setreg pc 8123\n" +
                "    setreg p 34\n" +
                "    setreg spc a 00\n" +
                "    setreg sa1 x 0010\n\n" +
                "SEE ALSO\n" +
                "    regs, cpus, bp, step, eval",

            ["vectors"] =
                "NAME\n" +
                "    vectors - where each interrupt vector points\n\n" +
                "SYNOPSIS\n" +
                "    vectors\n\n" +
                "DESCRIPTION\n" +
                "    Reads the interrupt vector table out of the running machine and shows\n" +
                "    what each entry points at, resolved through the label registry. 'Which\n" +
                "    routine handles NMI' is a question that comes up in almost every\n" +
                "    investigation, and the alternative is remembering that the SNES keeps\n" +
                "    its native NMI vector at $00FFEA and hand-reading two bytes.\n\n" +
                "    Vectors are grouped by CPU mode, because a 65816 has two full sets and\n" +
                "    which one the hardware uses depends on the E flag at the moment the\n" +
                "    interrupt is taken. 'regs' shows E. A game that sets up only the native\n" +
                "    table and then takes an interrupt in emulation mode jumps somewhere it\n" +
                "    never intended, and seeing both tables side by side is what makes that\n" +
                "    visible.\n\n" +
                "    When coverage recording is on ('man cov'), a vector whose target has\n" +
                "    actually executed is marked '(ran)'. A vector that points at plausible\n" +
                "    code which never ran is a much stronger signal than one that merely\n" +
                "    looks wrong - it says the interrupt is not firing at all, rather than\n" +
                "    firing and doing the wrong thing.\n\n" +
                "    The pointers are read live through the CpuBus, so a game that rewrites\n" +
                "    its own vector table (or maps different ROM in) reports what is mapped\n" +
                "    right now, not what was there at reset.\n\n" +
                "EXAMPLES\n" +
                "    vectors\n" +
                "    cov on; vectors\n\n" +
                "SEE ALSO\n" +
                "    runto, bt, cov, label, addr",

            ["addr"] =
                "NAME\n" +
                "    addr - decode a CPU-bus address\n\n" +
                "SYNOPSIS\n" +
                "    addr <addr>\n\n" +
                "DESCRIPTION\n" +
                "    Says what a 24-bit CPU-bus address actually reaches: a ROM file offset,\n" +
                "    a RAM offset, or a hardware register. The CPU's view and the ROM file's\n" +
                "    layout are different coordinate systems, and every task that crosses\n" +
                "    between them - comparing against a ROM in a hex editor, writing a patch,\n" +
                "    reading a disassembly someone else produced - needs the translation.\n" +
                "    Doing it by hand means knowing the mapper, which is exactly the detail\n" +
                "    worth not having to remember.\n\n" +
                "    The decode is the cartridge's own, the same one a real read goes\n" +
                "    through, so it is right for whatever mapper this ROM uses rather than\n" +
                "    assuming LoROM. ROM offsets are reported modulo the ROM's real size,\n" +
                "    matching how an undersized ROM mirrors on hardware.\n\n" +
                "    Addressable results print a ready-made 'mem' command underneath, since\n" +
                "    looking at the bytes is almost always the next thing wanted. A hardware\n" +
                "    register reports as a register rather than being given a fake offset -\n" +
                "    there is nothing there to point 'mem' at.\n\n" +
                "    An address the cartridge does not map and the bus does not claim\n" +
                "    reports as unmapped, which is itself an answer: a pointer that decodes\n" +
                "    to nothing is a pointer that was computed wrong.\n\n" +
                "EXAMPLES\n" +
                "    addr 808000\n" +
                "    addr 7E0DB3\n" +
                "    addr 002100\n\n" +
                "SEE ALSO\n" +
                "    mem, dump, spaces, vectors, cheat",

            ["cov"] =
                "NAME\n" +
                "    cov - record which code actually ran\n\n" +
                "SYNOPSIS\n" +
                "    cov on|off\n" +
                "    cov clear\n" +
                "    cov <addr> [<len>]\n" +
                "    cov mark\n" +
                "    cov new <addr> [<len>]\n" +
                "    cov funcs [<count>]\n" +
                "    cov save|load <file>\n" +
                "    cov <cpu> on|off|clear|<addr> [<len>]\n\n" +
                "DESCRIPTION\n" +
                "    Records every 24-bit address executed between 'cov on' and 'cov off',\n" +
                "    then answers 'did control flow ever reach here' for any range. A\n" +
                "    breakpoint can only say whether execution is at an address right now,\n" +
                "    so a breakpoint that never fires proves nothing on its own; this says\n" +
                "    outright that a routine never ran, which is the answer that retires a\n" +
                "    suspect. Pair it with 'callers': walk up from a routine that never ran\n" +
                "    until you reach a caller that did, and the branch between the two is\n" +
                "    the one that skipped it.\n\n" +
                "    Reports executed addresses, not instruction boundaries you guessed at -\n" +
                "    so it also settles where 'disasm' has mis-sized an immediate, since the\n" +
                "    recorded addresses are the real opcode boundaries.\n\n" +
                "    Recording is off by default and costs one bool test while disarmed;\n" +
                "    armed, it allocates a 2MB bitmap per processor recorded.\n\n" +
                "DIFFERENTIAL COVERAGE\n" +
                "    'cov mark' freezes what has run so far; 'cov new <addr>' then reports\n" +
                "    only what has run SINCE. This is the sharpest tool here, because it\n" +
                "    converts a question about code into a question about the machine: mark,\n" +
                "    press the button (open the door, take the damage, trigger the glitch),\n" +
                "    and every address 'cov new' reports is code that ran because of what\n" +
                "    you just did. Whole-run coverage cannot separate that from the boot\n" +
                "    sequence and the per-frame loop, which together dwarf it.\n\n" +
                "    Marking does not stop or clear recording, so several marks in a session\n" +
                "    just move the baseline forward. 'cov clear' drops the mark along with\n" +
                "    everything else. A mark costs a second 2MB bitmap.\n\n" +
                "DISCOVERED ROUTINES\n" +
                "    'cov funcs' lists every address a call actually landed on while\n" +
                "    recording, with how many times each was entered and whether it was\n" +
                "    reached by a call or by an interrupt vector. These are real entry\n" +
                "    points, observed rather than inferred - a static scan finds routines\n" +
                "    reached by a literal JSR, and misses every one reached through a jump\n" +
                "    table or a computed pointer, which on a SNES game is most of them.\n\n" +
                "    Output is designed to be pasted into 'label': the addresses are exactly\n" +
                "    the ones worth naming, and naming them makes every later 'bt', 'bp\n" +
                "    list' and 'profile' readable. Where 'profile' ranks routines by the\n" +
                "    time they cost, this enumerates them, including the cheap ones.\n\n" +
                "PERSISTENCE\n" +
                "    'cov save <file>' writes the bitmap and the discovered routines to\n" +
                "    Logs/<CoreName>/<file>; 'cov load' MERGES a saved map back in rather\n" +
                "    than replacing what is recorded, so maps from several sessions\n" +
                "    accumulate into one. A map of a whole playthrough is worth far more\n" +
                "    than a map of one sitting, and no single sitting reaches every\n" +
                "    routine.\n\n" +
                "    Name it with a .covmap extension - the repo gitignores that, and a map\n" +
                "    is a recording of one person's playthrough rather than source.\n\n" +
                "    The file is a fixed header, the routine table, then the raw bitmap.\n" +
                "    It is keyed to nothing - loading a map taken from a different ROM will\n" +
                "    quietly merge nonsense, so keep the filename tied to the game.\n\n" +
                "OTHER PROCESSORS\n" +
                "    A scope word records another chip's instruction stream instead - see\n" +
                "    'man cpus' for the names. Each chip gets its own bitmap, because each\n" +
                "    runs its own code: an address covered on the SA-1 says nothing about\n" +
                "    whether the main CPU ever executed the same number. 'cop' still works\n" +
                "    as an alias for whichever cartridge coprocessor is present.\n\n" +
                "EXAMPLES\n" +
                "    cov on\n" +
                "    cov 10F452 10\n" +
                "    cov mark\n" +
                "    cov new 108000 8000\n" +
                "    cov funcs 20\n" +
                "    cov save alttp.covmap\n" +
                "    cov gsu 0A80E9 40\n" +
                "    cov spc on\n\n" +
                "SEE ALSO\n" +
                "    cpus, bp, callers, disasm, label, profile",

            ["framelog"] =
                "NAME\n" +
                "    framelog - sample a memory value once per frame\n\n" +
                "SYNOPSIS\n" +
                "    framelog add <space> <addr> [<width>]\n" +
                "    framelog list\n" +
                "    framelog show <id> [<count>]\n" +
                "    framelog clear <id>\n" +
                "    framelog remove <id>\n\n" +
                "DESCRIPTION\n" +
                "    Unlike 'watch' (triggered by an actual read/write), a frame log samples\n" +
                "    its value once every frame regardless of whether anything touched it -\n" +
                "    useful for 'what does this value do over time' questions a write watch\n" +
                "    can't answer, e.g. a counter written once at level start but read every\n" +
                "    frame afterward. <width> is 1, 2, or 4 bytes (default 1).\n\n" +
                "EXAMPLES\n" +
                "    framelog add WRAM 1fb0 2\n" +
                "    framelog show 1",

            ["cheat"] =
                "NAME\n" +
                "    cheat - manage RAM pokes and ROM patches\n\n" +
                "SYNOPSIS\n" +
                "    cheat add <code> [description]\n" +
                "    cheat poke <space> <addr> <value> [description]\n" +
                "    cheat gg <code> [description]\n" +
                "    cheat rompatch <addr> <value> [<compare>|-] [description]\n" +
                "    cheat list\n" +
                "    cheat master [on|off]\n" +
                "    cheat enable <id>\n" +
                "    cheat disable <id>\n" +
                "    cheat remove <id>\n" +
                "    cheat clear\n" +
                "    cheat save <name>\n" +
                "    cheat load <name>\n" +
                "    cheat files\n\n" +
                "DESCRIPTION\n" +
                "    Unifies two SNES-specific cheat mechanisms under one command: RAM pokes\n" +
                "    (Pro Action Replay/Game Wizard style) and ROM-read patches (Game Genie\n" +
                "    style). 'add' decodes an 8-character code, guessing which format it is\n" +
                "    from its punctuation (best-effort, not a guarantee - use 'gg' or 'poke'\n" +
                "    directly if the guess is wrong). 'poke'/'rompatch' add a cheat directly\n" +
                "    without code decoding, e.g. for an address already found with 'search'.\n" +
                "    'enable'/'disable' toggle a cheat without removing it.\n\n" +
                "THE MASTER SWITCH\n" +
                "    'cheat master off' silences every cheat at once; 'cheat master on'\n" +
                "    brings them back. It is a SECOND AXIS, not a bulk edit: it does not\n" +
                "    touch any individual cheat's own enabled flag, so switching off to\n" +
                "    check whether a cheat is causing a bug and switching back on returns\n" +
                "    exactly the arrangement you had, however many were on. A cheat added\n" +
                "    while it is off is inert until it is switched back on. 'cheat list'\n" +
                "    says so at the top while it is off, because otherwise every [on] in\n" +
                "    that listing would be a lie.\n\n" +
                "    It is runtime state and is NOT saved. 'cheat save' does not write it\n" +
                "    and 'cheat load' does not set it: a set saved months ago with the\n" +
                "    switch off would otherwise silently kill every cheat in a session that\n" +
                "    had it on. It starts on, which costs nothing - everything imports\n" +
                "    disabled anyway, so nothing is applying until you say so.\n\n" +
                "    'save'/'load' persist a named set to /etc/EmuSen/cheats/<name>.json - see\n" +
                "    'man hier'. That file is ordinary JSON with addresses and bytes written\n" +
                "    as hex text, so it can be written or corrected by hand from this shell;\n" +
                "    an entry whose hex doesn't parse is skipped and reported, and the rest of\n" +
                "    the file still loads. 'load' ADDS to whatever is already loaded rather\n" +
                "    than replacing it - run 'cheat clear' first to replace. 'files' lists the\n" +
                "    saved sets. Nothing is loaded automatically: a cheat set applies only\n" +
                "    when you ask for it, so a saved file can't silently alter a later run.\n\n" +
                "THE WRITE MODEL\n" +
                "    A cheat is one description, one enable flag, and a LIST of writes. That\n" +
                "    matters because real cheats are rarely one address: 'max every item' is\n" +
                "    a run of twenty, and they have to arm and disarm together. One cheat,\n" +
                "    one ID, one toggle, however many addresses it drives - 'cheat list'\n" +
                "    prints a header line and then one indented line per write.\n\n" +
                "    Each write carries:\n\n" +
                "        width         1, 2 or 4 bytes. A wide write covers consecutive\n" +
                "                      addresses.\n" +
                "        byte order    little-endian unless the write says otherwise. Only\n" +
                "                      meaningful above one byte.\n" +
                "        type          set, increase or decrease. Increase/decrease read what\n" +
                "                      is there and adjust it, so they accumulate every frame\n" +
                "                      - 'increase by 1' climbs, it does not hold at 1.\n" +
                "        bit position  makes the write touch a single bit instead of whole\n" +
                "                      bytes, leaving the other seven alone. For flag bytes\n" +
                "                      where poking the whole byte would clobber unrelated\n" +
                "                      state.\n" +
                "        repeat        count, address stride and value stride. One write can\n" +
                "                      cover a whole run: 8 repetitions stepping the address\n" +
                "                      by 2 pokes every other byte of an inventory table.\n\n" +
                "    Two combinations are refused rather than half-implemented. A ROM patch\n" +
                "    cannot increase or decrease - the substitution happens at read time and\n" +
                "    the real byte is never written, so there is nothing to accumulate into.\n" +
                "    And a compare byte only works on a single-byte ROM patch, because the\n" +
                "    read hook is handed one byte at a time and cannot check a compare that\n" +
                "    spans several addresses; allowing it would give a torn patch where the\n" +
                "    first byte declines and the rest apply.\n\n" +
                "IMPORTING RETROARCH CHEATS\n" +
                "    'cheat import <path.cht>' reads RetroArch/libretro .cht files - the\n" +
                "    format the libretro cheat database ships in, one file per game. Both\n" +
                "    shapes a .cht can take are handled:\n\n" +
                "        cheatN_code       the core-native code string, hex, with several\n" +
                "                          codes joined by '+' becoming one multi-write cheat.\n" +
                "                          Decoded through the same code decoder 'cheat add'\n" +
                "                          uses, so the per-system layout is never hardcoded.\n" +
                "        cheatN_address    RetroArch's own handler fields - cheat_type,\n" +
                "                          memory_search_size, address_bit_position,\n" +
                "                          big_endian, repeat_count and the two strides. These\n" +
                "                          are DECIMAL in a .cht file, unlike the code string.\n\n" +
                "    memory_search_size 3/4/5 are 1/2/4-byte writes; 0-2 are sub-byte and\n" +
                "    become bit writes at address_bit_position. cheat_type 0 means the entry\n" +
                "    exists but does nothing, and is skipped rather than imported as a no-op.\n" +
                "    An entry that cannot be decoded is skipped and counted; the rest of the\n" +
                "    file still imports.\n\n" +
                "    Everything imports DISABLED. A database file routinely holds dozens of\n" +
                "    cheats and switching them all on at once is never what anyone meant -\n" +
                "    'cheat list' then 'cheat enable <id>' picks the ones you want.\n\n" +
                "    'cheat export <path.cht>' writes RAM pokes back out in RetroArch's\n" +
                "    handler form, which round-trips every field above. ROM patches are\n" +
                "    skipped and counted: RetroArch's model has no ROM-read substitution, so\n" +
                "    there is nothing honest to write for them.\n\n" +
                "THE CHEAT DATABASE\n" +
                "    'cheat db' works over a DIRECTORY TREE of .cht files - one file per\n" +
                "    game, in per-system folders, which is exactly how RetroArch stores its\n" +
                "    cheats. EmuSen ships no cheat data of its own and redistributes none.\n" +
                "    There are two ways to have a database, and neither involves us hosting\n" +
                "    anything:\n\n" +
                "        Use one you already have. Set AppSettings.CheatDatabaseDirectory to\n" +
                "        an existing RetroArch cheats folder and it is indexed as-is - no\n" +
                "        copying, no conversion. Unset, it defaults to home/Cheats (see\n" +
                "        'man hier').\n\n" +
                "        'cheat db update' downloads one. It fetches the same archive\n" +
                "        RetroArch's own Online Updater does, straight from libretro to your\n" +
                "        machine, on your explicit request. The data is licensed CC BY-SA 4.0;\n" +
                "        the codes themselves were aggregated from community sources,\n" +
                "        substantially GameHacking.org, and credit belongs to their original\n" +
                "        authors. That notice is printed with the result rather than buried\n" +
                "        here, because attribution is a condition of the licence.\n\n" +
                "    Only .cht entries are extracted, and an archive entry that tries to\n" +
                "    write outside the target directory is skipped rather than followed.\n\n" +
                "    'cheat db' with no argument reports where the database is and how many\n" +
                "    files per system. 'find' searches it; 'load' imports the best match,\n" +
                "    disabled, the same way 'import' does. Matching tolerates a loaded ROM's\n" +
                "    file name - 'cheat db load Super Mario World (USA).sfc' finds\n" +
                "    'Super Mario World (USA).cht' - and prefers an exact name over a prefix\n" +
                "    over a substring, so a game whose title is a prefix of another still\n" +
                "    wins for its own name.\n\n" +
                "    In EmuSen.Mistress the same thing is on the menu bar at Settings > Cheat\n" +
                "    Database..., which shows the folder, what is installed per system, the\n" +
                "    attribution notice, and a download button. Selecting a system there\n" +
                "    lists that system's games (with a filter box - a real SNES folder holds\n" +
                "    several thousand), and picking one loads its cheats into the active\n" +
                "    list, disabled, the same way 'cheat db load' does. It REPLACES that\n" +
                "    list rather than adding to it, unlike the command: picking the same\n" +
                "    game twice from a list is an ordinary thing to do and must not double\n" +
                "    every cheat, whereas typing the command twice is not. An Active\n" +
                "    Cheats... button sits next to Close, since turning a list on is what\n" +
                "    anyone does next.\n\n" +
                "    Settings > Active Cheats... is the GUI half of 'list', 'enable',\n" +
                "    'disable', 'remove', 'clear' and 'master' - one checkbox per cheat, the\n" +
                "    master switch, and a box to type a code into using the same format\n" +
                "    guess 'add' makes. That list is owned by the frontend rather than by\n" +
                "    the running core, so it survives a Reset and can be built before any\n" +
                "    ROM is loaded; loading a DIFFERENT ROM clears it, since one game's\n" +
                "    addresses mean nothing in another. See EmuSen_Settings_Reference.md\n" +
                "    section 4.14.\n\n" +
                "    Its Apply Cheats button is not an arm/commit step - ticking a box\n" +
                "    already takes effect on the next frame, the same as 'cheat enable'.\n" +
                "    Apply forces one poke immediately (useful while paused, where no next\n" +
                "    frame is coming) and reports how many landed; it also turns the master\n" +
                "    switch back on, since applying cheats that the switch would swallow is\n" +
                "    not what the button says.\n\n" +
                "    Save Cheat List writes the list under the running game's file name,\n" +
                "    the same store 'cheat save' uses - so 'cheat load <game>' finds it\n" +
                "    from the shell. Mistress reloads it by itself the next time that game\n" +
                "    starts, which is the point: a saved list is not re-imported from the\n" +
                "    database every session. It never overwrites a list already in hand, so\n" +
                "    a Reset or a close-and-reopen keeps whatever was edited since. See\n" +
                "    section 4.15.\n\n" +
                "PRUNING THE DATABASE\n" +
                "    The libretro database ships around 44 systems and a build with one\n" +
                "    core can use one of them, so most of a 250MB download is dead weight\n" +
                "    that every database scan still walks. `cheat db prune` deletes the\n" +
                "    system folders no core in this build claims.\n\n" +
                "    It is core-agnostic: each core declares the cheat-database folder\n" +
                "    names it covers (CoreDescriptor.CheatSystems), and the pruner keeps\n" +
                "    those and drops the rest - so a second core is one registry entry\n" +
                "    rather than an edit to the pruner. Venus claims both the main SNES\n" +
                "    folder and Satellaview, which is the same cartridge hardware.\n\n" +
                "    Deleting is irreversible and the only way back is another 250MB\n" +
                "    download, so a bare `cheat db prune` only ever lists what would go;\n" +
                "    `--apply` is what actually deletes. It refuses outright rather than\n" +
                "    emptying the database in the two cases that would: when no core\n" +
                "    claims any system at all, and when nothing on disk matches anything\n" +
                "    claimed (a wrong folder, or a wrong mapping). In Mistress the same\n" +
                "    thing is the Prune Unsupported button, whose first click reports and\n" +
                "    whose second deletes; a finished download offers it, since that is\n" +
                "    when the 250MB actually lands. See section 4.16.\n\n" +
                "EXAMPLES\n" +
                "    cheat poke WRAM 9c 63 infinite lives\n" +
                "    cheat list\n" +
                "    cheat disable 1\n" +
                "    cheat master off\n" +
                "    cheat save zelda\n" +
                "    cheat clear && cheat load zelda\n" +
                "    cheat import \"Super Mario World (USA).cht\"\n" +
                "    cheat export my-cheats.cht\n" +
                "    cheat db update\n" +
                "    cheat db find mario\n" +
                "    cheat db prune\n" +
                "    cheat db prune --apply\n" +
                "    cheat db load Super Mario World (USA)",

            ["search"] =
                "NAME\n" +
                "    search - classic 'first scan, then narrow' memory search\n\n" +
                "SYNOPSIS\n" +
                "    search <space> <value> [<width>]\n" +
                "    search refine <value>\n" +
                "    search changed|unchanged|increased|decreased\n" +
                "    search list [<count>]\n" +
                "    search reset\n\n" +
                "DESCRIPTION\n" +
                "    Finds where a game stores something without already knowing the address\n" +
                "    (a score, lives, a flag). Start with 'search <space> <value>' to find\n" +
                "    every address currently equal to <value> (width 1/2/4 bytes, default 1,\n" +
                "    little-endian); then narrow with 'refine' (equal to a new value) or\n" +
                "    changed/unchanged/increased/decreased (relative to the last scan/refine)\n" +
                "    until only the real address remains. One search session at a time -\n" +
                "    starting a new 'search <space> ...' replaces whatever was active.\n" +
                "    Refuses to scan a live-hardware-routed space (e.g. CpuBus) outright,\n" +
                "    since bulk-reading it can have real side effects (RDNMI clearing the\n" +
                "    pending-NMI flag, etc.) - use a plain space like WRAM instead.\n\n" +
                "EXAMPLES\n" +
                "    search WRAM 64\n" +
                "    search refine 65\n" +
                "    search list",

            ["memfind"] =
                "NAME\n" +
                "    memfind - locate a byte sequence in a memory space\n\n" +
                "SYNOPSIS\n" +
                "    memfind <space> <bytes> [<max>]\n\n" +
                "DESCRIPTION\n" +
                "    Scans <space> for every offset where <bytes> occurs. Where 'search'\n" +
                "    matches one 1/2/4-byte scalar and then narrows it down over time, this\n" +
                "    matches a whole block in a single pass - the tool for asking where a\n" +
                "    tile's 32 bytes, a decompressed buffer, or a string came from.\n\n" +
                "    <bytes> is hex byte pairs; ',', ':', '-' and '_' between them are\n" +
                "    ignored, and so is whitespace, so '00FF11', '00 FF 11' and '00:FF:11'\n" +
                "    are the same pattern. '??' in place of a pair matches any byte, which\n" +
                "    is how you skip over the parts of a block that legitimately differ.\n\n" +
                "    Stops after <max> hits (default 20) and says so, so a pattern that is\n" +
                "    too short to be distinctive reports quickly instead of listing\n" +
                "    thousands of offsets. Same live-hardware-space refusal as 'search' and\n" +
                "    'dump': a byte-by-byte scan of CpuBus could change real emulation\n" +
                "    state, so it is refused outright.\n\n" +
                "EXAMPLES\n" +
                "    memfind WRAM 00FFFFFFFF00FF00\n" +
                "    memfind VRAM 00??FF??11 5\n" +
                "    memfind ROM 4E696E74656E646F",

            ["snapshot"] =
                "NAME\n" +
                "    snapshot - capture a memory space's full contents\n\n" +
                "SYNOPSIS\n" +
                "    snapshot <space> <name>\n" +
                "    snapshot list\n" +
                "    snapshot remove <name>\n\n" +
                "DESCRIPTION\n" +
                "    Saves <space>'s entire current contents under <name> for later 'diff'\n" +
                "    against a further-along state - the general-purpose counterpart to\n" +
                "    'search' when the interesting addresses aren't already narrowed down.\n" +
                "    Same live-hardware-space refusal as 'search'/'dump'.\n\n" +
                "EXAMPLES\n" +
                "    snapshot WRAM before\n" +
                "    diff before",

            ["diff"] =
                "NAME\n" +
                "    diff - compare a snapshot against current memory\n\n" +
                "SYNOPSIS\n" +
                "    diff <name> [<count>]\n\n" +
                "DESCRIPTION\n" +
                "    Compares a snapshot saved with 'snapshot' against that same space's\n" +
                "    CURRENT contents and lists every byte address that's different now\n" +
                "    (default 20 shown). Doesn't touch or replace the saved snapshot, so the\n" +
                "    same baseline can be diffed again later against a further state.\n\n" +
                "EXAMPLES\n" +
                "    diff before\n" +
                "    diff before 50",

            ["dump"] =
                "NAME\n" +
                "    dump - write a raw memory range to disk\n\n" +
                "SYNOPSIS\n" +
                "    dump <space> <addr> <len> <file>\n\n" +
                "DESCRIPTION\n" +
                "    Writes <len> raw bytes from <space> starting at <addr> to\n" +
                "    home/Logs/<CoreName>/<file> - no header or\n" +
                "    metadata, so a hex editor can open the result directly. The write-side\n" +
                "    counterpart is 'load'. Same live-hardware-space refusal as\n" +
                "    'search'/'snapshot'.\n\n" +
                "EXAMPLES\n" +
                "    dump WRAM 0 2000 wram.bin",

            ["load"] =
                "NAME\n" +
                "    load - write a raw file's bytes into memory\n\n" +
                "SYNOPSIS\n" +
                "    load <space> <addr> <file>\n\n" +
                "DESCRIPTION\n" +
                "    Reads home/Logs/<CoreName>/<file> (typically\n" +
                "    one 'dump' produced, or hand-edited afterward) and pokes its raw bytes\n" +
                "    into <space> starting at\n" +
                "    <addr>. Refuses if <space> isn't writable.\n\n" +
                "EXAMPLES\n" +
                "    load WRAM 0 wram.bin",

            ["tile"] =
                "NAME\n" +
                "    tile - ASCII-decode one 8x8 tile\n\n" +
                "SYNOPSIS\n" +
                "    tile <space> <addr> <bpp>\n\n" +
                "DESCRIPTION\n" +
                "    Decodes and prints one 8x8 tile's pixels as an ASCII grid of hex nibbles\n" +
                "    (or '.' for a transparent/zero pixel). <bpp> meaning is core-specific -\n" +
                "    2/4/8 bits-per-pixel on the SNES.\n\n" +
                "EXAMPLES\n" +
                "    tile VRAM 0 4",

            ["tilemap"] =
                "NAME\n" +
                "    tilemap - decode a grid of tilemap entries as text\n\n" +
                "SYNOPSIS\n" +
                "    tilemap <space> <addr> <cols> <rows>\n\n" +
                "DESCRIPTION\n" +
                "    Decodes <cols>x<rows> tilemap/nametable entries starting at <addr> as a\n" +
                "    text grid, one label per cell - complementary to 'tile' (which decodes\n" +
                "    one tile's actual pixel content). The caller supplies the exact base\n" +
                "    address (see BG1SC/etc via 'regs') - this doesn't resolve BG-layer-to-\n" +
                "    tilemap-address mapping itself. Built to confirm a menu cursor's\n" +
                "    position or a HUD change numerically, without eyeballing a screenshot.\n\n" +
                "EXAMPLES\n" +
                "    tilemap VRAM 0 32 32",

            ["disasm"] =
                "NAME\n" +
                "    disasm - disassemble instructions\n\n" +
                "SYNOPSIS\n" +
                "    disasm <space> <addr> [<n>]\n" +
                "    disasm <cpu> [<addr>] [<n>]\n\n" +
                "DESCRIPTION\n" +
                "    Disassembles <n> instructions (default 10) starting at <addr>, printing\n" +
                "    address, raw bytes, mnemonic, and operand for each. A linear\n" +
                "    disassembler has no way to know which bytes are really code vs. data\n" +
                "    mixed into the same range, so a run through embedded data can produce\n" +
                "    garbage until it happens to resync - a known, accepted limitation, not a\n" +
                "    bug.\n\n" +
                "OTHER PROCESSORS\n" +
                "    A chip name in place of a space disassembles that chip's own code, in\n" +
                "    its own instruction set - see 'man cpus'. The SNES needs four different\n" +
                "    disassemblers to cover itself: the 65816 for the main CPU and the SA-1,\n" +
                "    the SPC700 for sound, the GSU's RISC encoding, and the NEC DSP's fixed\n" +
                "    24-bit words. Naming the chip picks the right one; naming a raw space\n" +
                "    picks by space, which is why 'disasm APURAM' decodes SPC700 and not\n" +
                "    65816 - it used to decode 65816, and quietly produced nonsense.\n\n" +
                "    With no address, disassembly starts wherever that chip is executing\n" +
                "    right now, which is usually what you want at a breakpoint.\n\n" +
                "    One asymmetry worth knowing: 'disasm DSPPRG' indexes program WORDS, not\n" +
                "    bytes, because the NEC DSP's program counter is a word index and that\n" +
                "    is the only number you ever have to paste in. Reading DSPPRG through\n" +
                "    'mem' is still byte-addressed, three bytes per word.\n\n" +
                "OPERAND WIDTHS (65816 ONLY)\n" +
                "    On the 65816 an immediate operand is one byte or two depending on the\n" +
                "    M (accumulator) and X (index) flags, which are not in the instruction\n" +
                "    bytes. By default those come from the CPU's flags RIGHT NOW, and\n" +
                "    REP/SEP inside the range are then tracked forward - which is correct\n" +
                "    when disassembling from the current PC, and only a guess anywhere\n" +
                "    else. Guessing wrong shifts every following instruction by a byte, so\n" +
                "    the listing stays syntactically plausible while being entirely wrong:\n" +
                "    a routine that really runs 8-bit-index reads as 'LDY #$C500' where the\n" +
                "    bytes are 'LDY #$00' followed by the start of the next instruction.\n\n" +
                "    'm8'/'m16' and 'x8'/'x16' force the starting widths. They may appear\n" +
                "    in any order after the command name and don't count as <addr>/<n>.\n" +
                "    The chosen widths are echoed above the listing. Asking for a 16-bit\n" +
                "    width also selects native mode, because emulation mode forces both\n" +
                "    widths to 8 regardless of M/X - without that, 'x16' would silently do\n" +
                "    nothing whenever the CPU happens to be paused in emulation mode.\n\n" +
                "    If a disassembly looks like it decodes into nonsense a few\n" +
                "    instructions in - implausible operands, a store to a read-only\n" +
                "    register, addresses that don't line up with a known write site - try\n" +
                "    the other widths before concluding the bytes are data.\n\n" +
                "EXAMPLES\n" +
                "    disasm CpuBus 8000 20\n" +
                "    disasm gsu\n" +
                "    disasm spc 05A5 10\n" +
                "    disasm dsp 0 20\n" +
                "    disasm cpu 04FDD8 12 m16 x8\n\n" +
                "SEE ALSO\n" +
                "    cpus, cov, bt, label",

            ["trace"] =
                "NAME\n" +
                "    trace - arm a live CPU instruction trace\n\n" +
                "SYNOPSIS\n" +
                "    trace <count>\n" +
                "    trace off\n\n" +
                "DESCRIPTION\n" +
                "    Arms this core's own live CPU instruction trace for the next <count>\n" +
                "    instructions, counted from whenever this command runs (not from\n" +
                "    power-on) - so a trace can be aimed at a specific moment in a play\n" +
                "    session instead of burning its whole budget during boot. 'trace off'\n" +
                "    cancels an in-progress trace early. Not available for a core with no\n" +
                "    equivalent trace mechanism, or a standalone launch with no core loaded.\n\n" +
                "EXAMPLES\n" +
                "    trace 500\n" +
                "    trace off",

            ["callers"] =
                "NAME\n" +
                "    callers - find static call/jump references to an address\n\n" +
                "SYNOPSIS\n" +
                "    callers <addr> [<scanstart> <scanlen>]\n\n" +
                "DESCRIPTION\n" +
                "    Scans CpuBus (default: <addr>'s own bank, $8000-$FFFF) for instructions\n" +
                "    statically calling/jumping to <addr> - a static-analysis alternative to\n" +
                "    setting a breakpoint and waiting for it to hit: finds every place that\n" +
                "    COULD jump there, whether or not it was ever actually reached during a\n" +
                "    given play session. See 'writers'/'readers' for the memory-access\n" +
                "    equivalents.\n\n" +
                "EXAMPLES\n" +
                "    callers 8000",

            ["writers"] =
                "NAME\n" +
                "    writers - find static write references to an address\n\n" +
                "SYNOPSIS\n" +
                "    writers <addr> [<scanstart> <scanlen>]\n\n" +
                "DESCRIPTION\n" +
                "    Scans CpuBus (default: <addr>'s own bank, $8000-$FFFF) for instructions\n" +
                "    statically storing to <addr> - answers 'what code exists that's capable\n" +
                "    of writing here', regardless of whether that write path was ever\n" +
                "    actually reached during a traced run (which is all a live 'watch' can\n" +
                "    tell you). Only finds statically-known addressing modes (absolute/\n" +
                "    absolute-long); indexed/indirect writes aren't visible to static\n" +
                "    analysis and won't show up here - use 'watch' for those.\n\n" +
                "EXAMPLES\n" +
                "    writers 13c6",

            ["readers"] =
                "NAME\n" +
                "    readers - find static read references to an address\n\n" +
                "SYNOPSIS\n" +
                "    readers <addr> [<scanstart> <scanlen>]\n\n" +
                "DESCRIPTION\n" +
                "    The read-side complement to 'writers': finds every instruction\n" +
                "    statically reading <addr>, e.g. every branch of game logic that checks\n" +
                "    a flag's value - not just the one spot noticed while manually tracing\n" +
                "    execution. Same addressing-mode limitation as 'writers'.\n\n" +
                "EXAMPLES\n" +
                "    readers 13c6",

            ["log"] =
                "NAME\n" +
                "    log - master logging on/off switch\n\n" +
                "SYNOPSIS\n" +
                "    log off\n" +
                "    log on\n" +
                "    log status\n\n" +
                "DESCRIPTION\n" +
                "    Silences (or restores) every trace/diagnostic logging flag across the\n" +
                "    project at once, without changing any of their individually-set values -\n" +
                "    'log on' afterward brings back exactly whatever was individually enabled\n" +
                "    before. A global settings toggle, not scoped to a particular core instance.",

            ["echo"] =
                "NAME\n" +
                "    echo - print arguments\n\n" +
                "SYNOPSIS\n" +
                "    echo <text...>\n\n" +
                "DESCRIPTION\n" +
                "    Prints its arguments back, joined by single spaces - real argument\n" +
                "    quoting applies ('echo 'a b'' is one argument, not two). Ignores stdin\n" +
                "    entirely, matching real echo. Its real use is scripting: a marker line\n" +
                "    in a source'd script, or the natural pipeline source for a text filter\n" +
                "    ('echo <text> | sed s/.../.../ ').\n\n" +
                "EXAMPLES\n" +
                "    echo hello world\n" +
                "    echo $X",

            ["sed"] =
                "NAME\n" +
                "    sed - stream substitution\n\n" +
                "SYNOPSIS\n" +
                "    sed s/pattern/replacement/[gi]\n" +
                "    sed <expr> <text...>\n\n" +
                "DESCRIPTION\n" +
                "    A basic sed-style filter supporting only the s/// substitution command\n" +
                "    (no line addressing, no d/p/hold-space/etc). The delimiter can be any\n" +
                "    character, matching real sed's own 's#/bin#/usr/bin#' convention, with\n" +
                "    '\\<delim>' escaping a literal delimiter inside the pattern/replacement.\n" +
                "    Pattern is a .NET regex. Without 'g', only the first match on EACH LINE\n" +
                "    is replaced (not just the first match in the whole piped blob) - most\n" +
                "    output this runs against (regs, watch log, ...) is multi-line, so this\n" +
                "    matters. Replacement supports '&' (whole match) and \\1-\\9 (capture\n" +
                "    groups), same as real sed. Real use is as a pipeline stage; standalone\n" +
                "    'sed <expr> <text...>' works too when there's nothing to pipe from.\n\n" +
                "EXAMPLES\n" +
                "    regs | sed s/PC=/pc=/\n" +
                "    echo hello | sed s/hello/hi/",

            ["grep"] =
                "NAME\n" +
                "    grep - filter lines by pattern\n\n" +
                "SYNOPSIS\n" +
                "    grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]\n\n" +
                "DESCRIPTION\n" +
                "    Filters lines matching <pattern> (a .NET regex, same choice sed makes).\n" +
                "    -i case-insensitive, -v invert (show non-matching lines), -n prefix each\n" +
                "    match with its 1-based line number, -c print only the match count, -q\n" +
                "    no output at all (exit code only). Exit code is 0 if anything matched, 1\n" +
                "    otherwise (real grep semantics) - -q exists specifically so grep can\n" +
                "    drive an if/while condition ('if regs | grep -q PC; then ...'), not just\n" +
                "    filter text. Reads piped stdin if present, else operates on trailing\n" +
                "    literal text. No -E/-F/-P mode switches, no -A/-B/-C context lines, no\n" +
                "    multi-file support.\n\n" +
                "EXAMPLES\n" +
                "    watch log 1 | grep -v poll\n" +
                "    if regs | grep -q PC; then echo has-pc; fi",

            ["wc"] =
                "NAME\n" +
                "    wc - count lines, words, bytes\n\n" +
                "SYNOPSIS\n" +
                "    wc [-l] [-w] [-c] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Counts lines/words/bytes of piped stdin, or a real file if <path> is\n" +
                "    given (walled to the project's own directory tree). With no flags,\n" +
                "    prints all three in that fixed order regardless of what order flags are\n" +
                "    given in, matching real wc.\n\n" +
                "EXAMPLES\n" +
                "    history | wc -l\n" +
                "    wc -l notes.txt",

            ["sort"] =
                "NAME\n" +
                "    sort - sort lines\n\n" +
                "SYNOPSIS\n" +
                "    sort [-n] [-r]\n\n" +
                "DESCRIPTION\n" +
                "    Sorts piped stdin (or trailing literal text) by line - ordinal string\n" +
                "    comparison by default, numeric with -n, reversed with -r. No -u (that's\n" +
                "    'uniq's job - pair them, 'sort | uniq', the way real shell usage does\n" +
                "    before reaching for 'sort -u' as a shortcut), no field/key selection, no\n" +
                "    locale-aware collation.\n\n" +
                "EXAMPLES\n" +
                "    watch list | sort -n",

            ["uniq"] =
                "NAME\n" +
                "    uniq - collapse adjacent duplicate lines\n\n" +
                "SYNOPSIS\n" +
                "    uniq [-c]\n\n" +
                "DESCRIPTION\n" +
                "    Collapses ADJACENT duplicate lines only, matching real uniq's own\n" +
                "    semantics - it is NOT a global dedup, so pipe through 'sort' first if\n" +
                "    that's what's actually wanted ('regs | sort | uniq'). -c prefixes each\n" +
                "    remaining line with its consecutive repeat count.\n\n" +
                "EXAMPLES\n" +
                "    watch log 1 | sort | uniq -c",

            ["awk"] =
                "NAME\n" +
                "    awk - a small AWK subset for reshaping text by field\n\n" +
                "SYNOPSIS\n" +
                "    awk [-F sep] 'program' [path]\n\n" +
                "DESCRIPTION\n" +
                "    Real AWK is a whole language; this is deliberately a small subset -\n" +
                "    enough to cover 'filter/reshape another command's output by field',\n" +
                "    without embedding a second full expression language. Supported\n" +
                "    patterns: empty (matches every line), /regex/, NR==N (also !=, <, <=, >,\n" +
                "    >=), BEGIN, END. Supported actions: {print expr[, expr...][; print\n" +
                "    ...]}, where expr is $0, $N, $NF, NR, NF, a \"quoted string\", or a bare\n" +
                "    number - no concatenation, no arithmetic, no user variables. A bare\n" +
                "    pattern with no {...} defaults to {print $0} (real awk's own default\n" +
                "    action); an explicit empty {} does nothing. Field splitting is\n" +
                "    whitespace-runs by default, or a literal separator string via -F (not a\n" +
                "    regex the way real awk's multi-character -F is). Reads piped stdin, or a\n" +
                "    trailing file path (walled to the project's own directory tree).\n\n" +
                "EXAMPLES\n" +
                "    regs | awk '{print $1}'\n" +
                "    watch log 1 | awk -F, 'NR>1{print $2}'\n" +
                "    echo -e \"a\\nb\\nc\" | awk 'NR==2'",

            ["ls"] =
                "NAME\n" +
                "    ls - list directory entries\n\n" +
                "SYNOPSIS\n" +
                "    ls [-a] [-l] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Lists a real directory's entries (default: current directory), or a\n" +
                "    single file listed as itself. One entry per line, no multi-column\n" +
                "    terminal layout. -a includes dotfiles; -l shows a simplified type/size/\n" +
                "    modified-time/name line (no permission bits/owner/group - this project\n" +
                "    has no such concept to show). Walled to the project's own directory\n" +
                "    tree - see 'cd'.\n\n" +
                "EXAMPLES\n" +
                "    ls\n" +
                "    ls -la EmuSen",

            ["cd"] =
                "NAME\n" +
                "    cd - change the current directory\n\n" +
                "SYNOPSIS\n" +
                "    cd [dir]\n\n" +
                "DESCRIPTION\n" +
                "    Changes the process-wide current directory - the same one 'pwd' reads\n" +
                "    and every relative path elsewhere in this shell (redirection, 'ls', 'mv',\n" +
                "    'rm', 'source', 'wc <path>', ...) resolves against. No argument goes to\n" +
                "    the current account's own home directory (see 'whoami'/'hier') - real\n" +
                "    $HOME semantics, scoped to this shell's own tree. A path starting with\n" +
                "    '/' means THIS shell's own root, not the real OS filesystem root -\n" +
                "    'cd /SourceLogs' works from anywhere, the same way a real chroot makes '/'\n" +
                "    mean the chroot directory (see 'hier'). Walled to the project root no\n" +
                "    matter how it's spelled - no amount of 'cd ..', a leading '/', or a long\n" +
                "    '../../..' chain can leave it; this is a deliberate walled garden against\n" +
                "    accidents, not a security boundary. A directory name with spaces doesn't\n" +
                "    need quoting ('cd My Folder' works) - cd only ever takes one path, so\n" +
                "    everything after it is treated as that path literally; quoting\n" +
                "    ('cd \"My Folder\"') also works if you prefer it.\n\n" +
                "EXAMPLES\n" +
                "    cd EmuSen.DianaOS\n" +
                "    cd /SourceLogs\n" +
                "    cd My Folder\n" +
                "    cd",

            ["hier"] =
                "NAME\n" +
                "    hier - this shell's own filesystem layout\n\n" +
                "SYNOPSIS\n" +
                "    man hier\n\n" +
                "DESCRIPTION\n" +
                "    A real Unix precedent for this exact page: 'man hier' documents a\n" +
                "    filesystem's layout without 'hier' being a runnable command - same here.\n\n" +
                "    /                 your home, and the whole of what this shell can see.\n" +
                "                      'cd' with no argument comes back here. The install\n" +
                "                      itself - the binaries, the repo when running from\n" +
                "                      source - sits ABOVE this root and is unreachable.\n\n" +
                "    /Games            your ROM library\n" +
                "    /Saves            battery-backed cartridge SRAM ('.srm')\n" +
                "    /Saves/Save States   'state save'/'state load' snapshots\n" +
                "    /Firmware         coprocessor dumps you supply\n" +
                "    /Cheats           your own .cht tree - see 'man cheat'\n" +
                "    /Logs             'dump'/'load'/screenshot/recording output\n" +
                "    /etc/EmuSen       every config file the emulator keeps: appsettings.json,\n" +
                "                      keybindings.json, gamepadbindings.json,\n" +
                "                      hotkeybindings.json, audio.json, graphics.json, and\n" +
                "                      cheats/<name>.json, and themes/<name>.axaml or\n" +
                "                      themes/<name>.css (either a ResourceDictionary or a\n" +
                "                      restricted CSS, both overriding EmuSen.LunaP's Luna*\n" +
                "                      keys - see EmuSen_LunaP.md). Plain JSON, meant to be read and\n" +
                "                      edited from this shell - comments and trailing commas\n" +
                "                      are tolerated. Written by EmuSen.Galaxia; these used to\n" +
                "                      live outside the sandbox under the OS's own per-user\n" +
                "                      config directory, and are copied in the first time each\n" +
                "                      file is read (the originals are left where they were)\n" +
                "    /bin              published builds only: one launcher per frontend, each\n" +
                "                      a two-line shim onto the real binary in /lib/EmuSen\n" +
                "    /lib/EmuSen       published builds only: the whole .NET application\n" +
                "                      directory - every shipped .dll, the apphosts,\n" +
                "                      deps.json/runtimeconfig.json, runtimes/. The Unix\n" +
                "                      '/usr/lib/<app>' shape: private libraries of one app,\n" +
                "                      not a shared library dir. Absent from source\n" +

                "    /tmp              scratch space, nothing here is ever auto-deleted.\n" +
                "                      '/tmp/WiseMan' is the test suite's own scratch\n\n" +
                "    /Documents        published builds only: the EmuSen Manual and the\n" +
                "                      per-console hardware notes under 'Man pages/', staged\n" +
                "                      in at publish time. Running from source they stay in\n" +
                "                      the repo where they are edited, which is above this\n" +
                "                      root - use 'man' rather than 'cat' from source\n\n" +
                "    Every directory above is located by EmuSen.Galaxia, not by this shell -\n" +
                "    the sandbox forwards to it, so a core never asks a debugger where a save\n" +
                "    goes. See EmuSen_Galaxia.md.\n\n" +
                "    '/home' used to be five levels down, at 'EmuSen.DianaOS/DianaOS/Usr/\n" +
                "    Home' - a published user's save folder named after a C# project. If you\n" +
                "    have an install from before the move, everything the emulator wrote\n" +
                "    (saves, states, firmware, cheats, logs) was COPIED to '/home' on first\n" +
                "    run and the originals left untouched. Your ROMs were NOT copied - a\n" +
                "    library is too big to duplicate behind your back. They are still in\n" +
                "    'EmuSen.DianaOS/DianaOS/Usr/Home/{Games,Roms}'; move them into\n" +
                "    '/home/Games' when convenient, or just point RomDirectory at them in\n" +
                "    Preferences, which is where the library is configured anyway.\n\n" +
                "    A leading '/' means THIS root - your home - not the real OS filesystem\n" +
                "    root, and not the install directory. Every real-file command ('cat',\n" +
                "    'ls', 'find', 'awk', 'nano', redirection, 'source') resolves through the\n" +
                "    same check and refuses anything above it, so no amount of '../..' walks\n" +
                "    out. This shell used to be rooted at the project directory, which meant\n" +
                "    'ls /' listed '.git', 'EmuSen.sln' and every C# project beside your\n" +
                "    saves; rooting at home is what stopped that. It is a guardrail against\n" +
                "    accidents, not a security boundary.\n\n" +
                "WHERE THE ROOT ACTUALLY IS\n" +
                "    Decided once at startup by walking up from the running assembly's own\n" +
                "    folder (not the current directory, which a launcher or shortcut gets\n" +
                "    to choose and so can't be trusted), stopping at the first parent that\n" +
                "    holds either marker:\n\n" +
                "    'EmuSen.sln'          running from source: the project root, as\n" +
                "                          described above. A build output dir\n" +
                "                          ('bin/Release/net10.0') is three levels down, so\n" +
                "                          Debug and Release both land on the same root and\n" +
                "                          share one set of saves and logs.\n\n" +
                "    '.dianaosroot'        a published build: the publish directory itself,\n" +
                "                          written there at publish time. No 'EmuSen.sln'\n" +
                "                          ships with a published app, so this is what the\n" +
                "                          walk finds instead - two levels up from\n" +
                "                          '/lib/EmuSen', where the binaries live.\n\n" +
                "    If neither turns up, the fallback is 'DianaOSRoot/' beside the binary.\n" +
                "    That is what a publish looks like WITHOUT the layout below - the doc\n" +
                "    trees stage into 'DianaOSRoot/' either way, so an app dir copied out by\n" +
                "    hand still finds its own manual instead of coming up rootless.\n\n" +
                "    A published tree puts the binaries in 'lib/EmuSen', the launchers in\n" +
                "    'bin', and the shell's root at 'home' BESIDE them - so every shipped\n" +
                "    .dll and the running executable are now ABOVE this shell's root and\n" +
                "    cannot be reached from it at all.\n\n" +
                "    That was not always true. When the shell was rooted at the install\n" +
                "    directory, real '/' contained real '/lib', and 'rm' and 'mv' were\n" +
                "    suspended precisely because nothing here distinguished '/lib/EmuSen'\n" +
                "    from your own files. Rooting at home removed the exposure those two\n" +
                "    suspensions existed for; they are still suspended, which is now a\n" +
                "    stricter setting than the layout requires rather than a necessity.\n\n" +
                "    A published root is NOT empty. Both read-only doc trees stage into\n" +
                "    '/Documents' at publish time, so 'man', 'cat', 'find' and 'grep' have\n" +
                "    the same reference material there as they do from source. Everything\n" +
                "    else - the six directories under '/home' - is created empty on first\n" +
                "    run.\n\n" +
                "    The layout is MSBuild's doing, not this shell's: see\n" +
                "    EmuSen.DianaOS/Publish/DianaOSPublishLayout.targets, imported by each\n" +
                "    publishable frontend. It only rewrites where files are copied - the\n" +
                "    .NET application directory is relocated whole, never taken apart, so\n" +
                "    nothing about how the app resolves its assemblies changes. Publish\n" +
                "    with -p:DianaOSPublishLayout=false for a plain flat app dir.\n\n" +
                "EXAMPLES\n" +
                "    man hier\n" +
                "    ls /\n" +
                "    cd /home/Logs && ls",

            // Not a command - a topic page, like 'hier'. ThemeVocabularyTests pins every name below against EmuSen.LunaP's real allow-lists.
            ["theme"] =
                "NAME\n" +
                "    theme - the colours and fonts every EmuSen window is drawn with\n\n" +
                "SYNOPSIS\n" +
                "    /etc/EmuSen/themes/<name>.css\n" +
                "    /etc/EmuSen/themes/<name>.axaml\n\n" +
                "DESCRIPTION\n" +
                "    A theme overrides whichever of the Luna* palette keys it cares about\n" +
                "    and leaves the rest alone, so a two-line theme is a legitimate theme.\n" +
                "    Drop a file in the directory above and pick it in Preferences; there is\n" +
                "    no registration step and no restart. One name is one theme: if both\n" +
                "    formats spell it, the .axaml is the one that loads.\n\n" +
                "    The .css form is a restricted CSS - a format, not an engine. There is\n" +
                "    no cascade, no specificity, no inheritance and no box model, and the\n" +
                "    only things it can produce are colours, numbers, font families and the\n" +
                "    property setters listed below. The .axaml form is an Avalonia\n" +
                "    ResourceDictionary and is documented in EmuSen_LunaP.md section 12.\n\n" +
                "THE PALETTE\n" +
                "    Written inside ':root'. Each token is a Luna* resource key in kebab\n" +
                "    case, and setting a colour sets both halves the palette spells (the\n" +
                "    Color and the brush), so nothing half-applies.\n\n" +
                "    --luna-surface           tool-window background\n" +
                "    --luna-input-surface     text-input background\n" +
                "    --luna-void              letterbox area behind a game frame\n" +
                "    --luna-text              body and monospace text\n" +
                "    --luna-meter-text        meter-row labels and values\n" +
                "    --luna-muted             hints, group headers, disabled captions\n" +
                "    --luna-section-header    section headings\n" +
                "    --luna-warning           inline caution text\n" +
                "    --luna-nominal           load ramp, below 60%\n" +
                "    --luna-busy              load ramp, 60% and above\n" +
                "    --luna-hot               load ramp, 85% and above\n" +
                "    --luna-mono-font         the monospace family list\n" +
                "    --luna-hint-font-size    hint text size\n" +
                "    --luna-header-font-size  section heading size\n\n" +
                "    The key's SUFFIX decides how its value is read: '...-size' is a number,\n" +
                "    '...-font' a font family, everything else a colour. A font family and a\n" +
                "    named colour are written identically ('monospace', 'gainsboro'), so the\n" +
                "    value alone could not settle it.\n\n" +
                "    Colours take any CSS spelling - #RGB, #RRGGBB, #RGBA, #RRGGBBAA,\n" +
                "    rgb(), rgba() with alpha 0-1, and the named colours. In the eight-digit\n" +
                "    form ALPHA IS LAST, the CSS order; Avalonia's own #AARRGGBB reads the\n" +
                "    same digits backwards, and inside a .css file CSS wins.\n\n" +
                "RULES\n" +
                "    Outside ':root', a block styles one kind of control. The selector is\n" +
                "    'element', 'element.state' or 'element part', and the element is the\n" +
                "    control's own name in kebab case:\n\n" +
                "    button-bar        field-row         path-picker-row   status-bar\n" +
                "    console-pane      filter-bar        rgba-image-view   tabs\n" +
                "    dropdown          hint-text         section-header\n" +
                "    luna-switch       meter-list        meter-row\n" +
                "    mono-text\n\n" +
                "    States: meter-row.nominal, meter-row.busy, meter-row.hot.\n" +
                "    Parts:  meter-row .bar; filter-bar .search, .facet;\n" +
                "            console-pane .output, .input, .prompt.\n\n" +
                "    Properties: background, background-color, color, font-family,\n" +
                "    font-size, font-weight. A value may be a token - 'var(--luna-hot)' -\n" +
                "    which FOLLOWS that token rather than copying it; a rule that restates a\n" +
                "    colour stops tracking the palette.\n\n" +
                "WHEN IT DOES NOT LOAD\n" +
                "    Two tiers, on purpose. A SYNTAX error refuses the whole file and leaves\n" +
                "    the previous theme in force: an unbalanced brace, a declaration with no\n" +
                "    colon, an unterminated comment, an at-rule, a nested rule. An UNKNOWN\n" +
                "    selector, state, part, property or unreadable value is reported and\n" +
                "    skipped, and the rest of the theme still applies - so a theme written\n" +
                "    for a later EmuSen keeps working here.\n\n" +
                "    Either way the reason is printed the same way a config file's is, with\n" +
                "    the line number from your file: comments are not stripped before\n" +
                "    counting, so the number matches what your editor shows.\n\n" +
                "    A theme that is deleted or broken between runs falls back to built-in\n" +
                "    WITHOUT forgetting the choice. Fix the file, restart, and it returns.\n\n" +
                "EXAMPLES\n" +
                "    cat > /etc/EmuSen/themes/Nocturne.css\n" +
                "    :root {\n" +
                "      --luna-surface:        #12131A;\n" +
                "      --luna-section-header: #7AA2F7;\n" +
                "      --luna-mono-font:      \"Fira Code\", monospace;\n" +
                "    }\n" +
                "    meter-row.hot .bar { color: var(--luna-hot); }\n\n" +
                "    ls /etc/EmuSen/themes",

            ["pwd"] =
                "NAME\n" +
                "    pwd - print the current directory\n\n" +
                "SYNOPSIS\n" +
                "    pwd\n\n" +
                "DESCRIPTION\n" +
                "    Prints the process's current working directory - see 'cd'.",

            ["cat"] =
                "NAME\n" +
                "    cat - print a file's contents\n\n" +
                "SYNOPSIS\n" +
                "    cat <path>...\n\n" +
                "DESCRIPTION\n" +
                "    Prints one or more real files' contents, concatenated in the order\n" +
                "    given, walled to the project's own directory tree (see 'cd'). With no\n" +
                "    <path> at all, passes piped stdin straight through unchanged instead -\n" +
                "    the classic 'cat' idiom for previewing a pipeline mid-stage - rather\n" +
                "    than erroring for a missing argument.\n\n" +
                "EXAMPLES\n" +
                "    cat notes.txt\n" +
                "    watch log 1 | cat",

            ["head"] =
                "NAME\n" +
                "    head - print the first lines of a file or stdin\n\n" +
                "SYNOPSIS\n" +
                "    head [-n N] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Prints the first N lines (default 10) of piped stdin, or a real file if\n" +
                "    <path> is given (walled to the project's own directory tree - see 'cd').\n\n" +
                "EXAMPLES\n" +
                "    history | head -n 5\n" +
                "    head notes.txt",

            ["tail"] =
                "NAME\n" +
                "    tail - print the last lines of a file or stdin\n\n" +
                "SYNOPSIS\n" +
                "    tail [-n N] [path]\n\n" +
                "DESCRIPTION\n" +
                "    Prints the last N lines (default 10) of piped stdin, or a real file if\n" +
                "    <path> is given (walled to the project's own directory tree - see 'cd').\n" +
                "    No '-f' follow mode - this shell has nothing that appends to a file\n" +
                "    live while 'tail' is running to follow.\n\n" +
                "EXAMPLES\n" +
                "    history | tail -n 5\n" +
                "    tail notes.txt",

            ["touch"] =
                "NAME\n" +
                "    touch - create an empty file, or update its modified time\n\n" +
                "SYNOPSIS\n" +
                "    touch <path>\n\n" +
                "DESCRIPTION\n" +
                "    Creates <path> as an empty file if it doesn't exist yet, or just updates\n" +
                "    an existing file's last-modified time otherwise - walled to the\n" +
                "    project's own directory tree (see 'cd'). Refuses a path that's already\n" +
                "    a directory.\n\n" +
                "EXAMPLES\n" +
                "    touch scratch.txt",

            ["find"] =
                "NAME\n" +
                "    find - recursively list files and directories\n\n" +
                "SYNOPSIS\n" +
                "    find [path] [-name pattern]\n\n" +
                "DESCRIPTION\n" +
                "    Walks <path> (default: current directory) and everything under it,\n" +
                "    printing one entry per line, each path shown relative the same way it\n" +
                "    was given (e.g. 'find EmuSen' prints 'EmuSen/...' paths). -name filters\n" +
                "    to entries whose own name matches a glob ('*'/'?' wildcards only, no\n" +
                "    regex) - a directory that doesn't match is still walked into, only\n" +
                "    excluded from the printed list itself, matching real find's own\n" +
                "    behavior. Walled to the project's own directory tree (see 'cd').\n\n" +
                "EXAMPLES\n" +
                "    find\n" +
                "    find EmuSen.DianaOS -name '*Command.cs'",

            ["xxd"] =
                "NAME\n" +
                "    xxd, hexdump - hexdump a real file's raw bytes\n\n" +
                "SYNOPSIS\n" +
                "    xxd <path>\n" +
                "    hexdump <path>\n\n" +
                "DESCRIPTION\n" +
                "    Reads <path>'s REAL bytes off disk (not decoded as text the way\n" +
                "    'cat'/'head'/'tail' do, which would corrupt anything that isn't valid\n" +
                "    UTF8) and prints them in the same address/hex/ASCII layout 'mem' uses\n" +
                "    for a live memory space - the on-disk counterpart to 'mem', purpose-\n" +
                "    built so a 'dump'd capture can be inspected without leaving the shell\n" +
                "    for an external hex editor. Walled to the project's own directory tree\n" +
                "    (see 'cd'). With no <path>, hexdumps piped stdin instead - UTF8-encoded\n" +
                "    first, since a pipeline stage's own output is always already-decoded\n" +
                "    text here, never raw bytes (this shell's pipes have no byte-stream\n" +
                "    concept the way a real Unix pipe does). 'hexdump' is a plain alias, not\n" +
                "    a separate command - and NOT real hexdump's own multi-format output,\n" +
                "    just this same one layout under a second, equally-reached-for name.\n\n" +
                "EXAMPLES\n" +
                "    dump WRAM 0 256 wram.bin\n" +
                "    xxd home/Logs/SNES/wram.bin\n" +
                "    echo hi | xxd",

            ["nano"] =
                "NAME\n" +
                "    nano - simple full-screen text editor\n\n" +
                "SYNOPSIS\n" +
                "    nano <path>\n\n" +
                "DESCRIPTION\n" +
                "    A small full-screen editor for creating/editing a real file (a 'source'\n" +
                "    script, say) without leaving the shell for an external editor. NOT a\n" +
                "    full GNU nano clone - no search, no cut/paste ring, no syntax\n" +
                "    highlighting - just arrow-key movement, insert/Backspace/Delete, and\n" +
                "    Ctrl+O to save / Ctrl+X to exit (prompting to save first if there are\n" +
                "    unsaved changes). <path> is created if it doesn't exist yet, and is\n" +
                "    walled to the project's own directory tree like every other real-file\n" +
                "    command here (see 'cd'). Needs a REAL interactive terminal - it takes\n" +
                "    over the console with raw key reads and full-screen redraws the same\n" +
                "    way this shell's own line editor does for a single line, just for a\n" +
                "    whole buffer - so it refuses cleanly (rather than failing strangely) when\n" +
                "    stdin/stdout is redirected (a piped script, a 'source'd file, a headless\n" +
                "    test) or when there's no real terminal at all (EmuSen.Mistress's GUI\n" +
                "    console window).\n\n" +
                "EXAMPLES\n" +
                "    nano setup.txt",

            ["clear"] =
                "NAME\n" +
                "    clear - clear the terminal screen\n\n" +
                "SYNOPSIS\n" +
                "    clear\n\n" +
                "DESCRIPTION\n" +
                "    Wipes whatever's currently on screen (a real Console.Clear(), not just\n" +
                "    printing blank lines) so the next prompt starts at the top instead of at\n" +
                "    the bottom of a long scrollback. Only does anything against a real\n" +
                "    interactive console; a redirected/nonexistent one (a piped script, a\n" +
                "    'source'd file, a headless test) is silently a no-op rather than an\n" +
                "    error, since there was never a real screen to clear in the first place.\n\n" +
                "    In EmuSen.Mistress's own console window (a TextBox, not a real\n" +
                "    terminal), 'clear' is replaced with a windowed equivalent instead: it\n" +
                "    empties that window's own output text rather than touching a system\n" +
                "    console that doesn't exist there.\n\n" +
                "EXAMPLES\n" +
                "    clear",

            ["coretop"] =
                "NAME\n" +
                "    coretop - live htop-style dashboard of the loaded core's hardware\n\n" +
                "SYNOPSIS\n" +
                "    coretop [-w]\n\n" +
                "DESCRIPTION\n" +
                "    A live, auto-refreshing (4x/second) dashboard, styled after htop: per-\n" +
                "    subsystem hardware load bars, CPU registers, a sprite-capacity gauge,\n" +
                "    per-voice audio channel meters, a live color-RAM palette swatch, and (if\n" +
                "    this core has a tilemap concept) a downsampled live preview of VRAM's\n" +
                "    tile sheet - all in one screen, refreshing on its own clock rather than\n" +
                "    waiting for input. Every number comes from IDebugTarget, so a core with\n" +
                "    nothing to report for a given section (no hardware-load breakdown wired\n" +
                "    up, no fixed sprite capacity, no palettes) just makes that section\n" +
                "    shrink or disappear rather than showing something fake.\n\n" +
                "    Load bars are colored green/yellow/red under 60% / 60-85% / over 85%,\n" +
                "    the same rough 'getting busy' convention real htop's CPU bars use.\n\n" +
                "    Press Ctrl+C to exit - unlike everywhere else in this shell, Ctrl+C is\n" +
                "    read as a key here (via Console.TreatControlCAsInput) rather than\n" +
                "    raising a process-level interrupt, restored to normal the moment\n" +
                "    coretop exits. Needs a REAL interactive terminal, same as 'nano' and\n" +
                "    for the same reason - refuses cleanly rather than trying to draw\n" +
                "    anywhere when stdin/stdout is redirected or there's no real terminal at\n" +
                "    all (a piped script, a 'source'd file, a headless test).\n\n" +
                "    In EmuSen.Mistress's own console window (a TextBox, not a real\n" +
                "    terminal - the above wouldn't work there at all), 'coretop' is replaced\n" +
                "    with a windowed equivalent instead: a real, non-blocking Avalonia window\n" +
                "    showing the same data as actual widgets and live images (the palette and\n" +
                "    VRAM tile sheet render as real pictures there, not ANSI blocks) that\n" +
                "    refreshes on its own timer while gameplay keeps running - Ctrl+C doesn't\n" +
                "    apply there, just close the window.\n\n" +
                "    -w  Open the same dashboard in a separate window instead of taking over\n" +
                "        the terminal, so you can dismiss it and keep playing. In\n" +
                "        EmuSen.Mistress, coretop is already windowed by default, so -w is\n" +
                "        accepted and simply has no additional effect there. In EmuSen.Hotaru\n" +
                "        (a Raylib console build with no window of its own to reuse), -w spins\n" +
                "        up a small dedicated Avalonia UI thread the first time it's used and\n" +
                "        opens a real second OS window from it, entirely separate from the\n" +
                "        Raylib game window and its main loop - gameplay keeps running behind\n" +
                "        it. Closing that window is -w's equivalent of Ctrl+C: it's the whole\n" +
                "        'stop watching coretop' action, nothing else to press. If the current\n" +
                "        frontend has no window to open at all, -w fails cleanly with\n" +
                "        \"not supported by this frontend\" rather than falling back to the\n" +
                "        terminal dashboard.\n\n" +
                "EXAMPLES\n" +
                "    coretop\n" +
                "    coretop -w",

            ["vstop"] =
                "NAME\n" +
                "    vstop - live htop-style dashboard of the .NET runtime this process runs on\n\n" +
                "SYNOPSIS\n" +
                "    vstop [-w]\n\n" +
                "DESCRIPTION\n" +
                "    The counterpart to 'coretop'. Where that one watches the emulated\n" +
                "    machine, this one watches the real one underneath it: the .NET virtual\n" +
                "    machine that DianaOS, the loaded core, and the frontend are all running\n" +
                "    on. Managed heap, garbage collector, thread pool, process memory and\n" +
                "    CPU, refreshing twice a second until you quit.\n\n" +
                "    It is the one dashboard that still has something to show with NO ROM\n" +
                "    loaded, because it reports on the host VM rather than a guest - it\n" +
                "    takes no IDebugTarget at all and is completely core-agnostic.\n\n" +
                "    Four bars across the top, colored green/yellow/red under 60% / 60-85% /\n" +
                "    over 85% the same way coretop's are:\n\n" +
                "        CPU        this process's processor time, divided by core count, so\n" +
                "                   100% means every core busy - htop's aggregate scale, not\n" +
                "                   the per-core one where an 8-core box can read 800%.\n" +
                "        Machine    system-wide memory in use. This is the figure the GC\n" +
                "                   compares against its own high-load threshold when it\n" +
                "                   decides to collect more aggressively, so it explains GC\n" +
                "                   behaviour that the heap numbers alone do not.\n" +
                "        Heap frag  fragmented bytes over committed bytes - dead space inside\n" +
                "                   the managed heap that the GC has not handed back. A high\n" +
                "                   reading next to a small live heap is the signature of a\n" +
                "                   pinning or large-object problem.\n" +
                "        Pool load  thread-pool threads against the pool maximum. Climbing\n" +
                "                   toward the ceiling with a non-zero queued-work count is\n" +
                "                   thread-pool starvation.\n\n" +
                "    Below them: process working set and private bytes; managed heap, GC-\n" +
                "    committed bytes, cumulative allocations and the live allocation rate;\n" +
                "    per-generation sizes (gen0/gen1/gen2, then LOH and POH where the\n" +
                "    runtime reports them); GC mode, latency mode, pause-time percentage,\n" +
                "    per-generation collection counts with a collections-per-minute rate and\n" +
                "    total time paused; OS thread and handle counts, pool thread counts and\n" +
                "    limits, queued and completed work items with a throughput rate; and the\n" +
                "    number of loaded assemblies.\n\n" +
                "    Allocation rate, CPU percent, collections per minute and work-item\n" +
                "    throughput are all rates, so they need two readings to exist. The first\n" +
                "    reading is taken before the first frame is drawn - you never see a\n" +
                "    screen of zeroes - but they stay zero for a single refresh if the\n" +
                "    dashboard is somehow drawn without that priming sample.\n\n" +
                "    A subtler point, and the reason several fields can read '-' instead of\n" +
                "    a number: GC.GetGCMemoryInfo() does not describe the heap right now, it\n" +
                "    describes the heap AS OF THE LAST COLLECTION. In a process that has not\n" +
                "    collected yet - which a freshly launched shell has not - every figure\n" +
                "    derived from it is legitimately zero: committed bytes, fragmented bytes,\n" +
                "    machine memory load, per-generation sizes, pause-time percentage. Those\n" +
                "    are exactly the readings someone opening this dashboard is most likely\n" +
                "    to act on, and a bar sitting confidently at 0.0% is worse than no bar,\n" +
                "    because it looks like an answer. So the Machine and Heap frag bars draw\n" +
                "    empty with '(no GC yet - press G)' until a collection has actually\n" +
                "    happened, and the committed/per-generation/pause-time fields show '-'.\n" +
                "    Press G once and the whole screen fills in. Working set, private bytes,\n" +
                "    managed heap, cumulative allocations, thread counts and CPU are all\n" +
                "    live readings and never do this.\n\n" +
                "    Nothing this command reads is allowed to throw. Every counter is read\n" +
                "    behind a guard that falls back to zero, because a runtime or platform\n" +
                "    that does not expose one of them (handle counts are not meaningful\n" +
                "    everywhere) must degrade to a missing number, not take the shell down.\n\n" +
                "    KEYS\n" +
                "        Ctrl+C, Q   exit. As in coretop, Ctrl+C is read as a key here (via\n" +
                "                    Console.TreatControlCAsInput) rather than raising a\n" +
                "                    process-level interrupt, and is restored the moment\n" +
                "                    vstop exits.\n" +
                "        G           force a full blocking GC.Collect(). Deliberately\n" +
                "                    included: the most common question this dashboard gets\n" +
                "                    opened to answer is 'is that a leak or just uncollected\n" +
                "                    garbage', and collecting on demand answers it in one\n" +
                "                    keystroke. It is a diagnostic, not something to lean on.\n\n" +
                "    Needs a REAL interactive terminal, same as 'nano' and 'coretop' and for\n" +
                "    the same reason - refuses cleanly rather than drawing anywhere when\n" +
                "    stdin/stdout is redirected or there is no terminal at all (a piped\n" +
                "    script, a 'source'd file, a headless test).\n\n" +
                "    In EmuSen.Mistress's own console window (a TextBox, not a real\n" +
                "    terminal - the above wouldn't work there at all), 'vstop' is replaced\n" +
                "    with a windowed equivalent, exactly as 'coretop' is: a real, non-\n" +
                "    blocking Avalonia window showing the same figures as widgets, with the\n" +
                "    four meters as real progress bars on the same green/yellow/red\n" +
                "    thresholds, refreshing on its own timer while gameplay keeps running.\n" +
                "    Ctrl+C, Q and G don't apply there - close the window to dismiss it, and\n" +
                "    use its 'Collect now' button for what G does here. The same window is\n" +
                "    on the menu bar at Settings > Runtime Dashboard..., which unlike\n" +
                "    Hardware Dashboard beside it is never greyed out, because this one has\n" +
                "    something to show with no ROM loaded.\n\n" +
                "    -w  Open the same dashboard in a separate window instead of taking over\n" +
                "        the terminal. Wired per frontend; where nothing is wired up it fails\n" +
                "        cleanly with \"not supported by this frontend\" rather than falling\n" +
                "        back to the terminal dashboard. Unlike coretop's -w, this one passes\n" +
                "        no debug target, because there is nothing here that depends on a\n" +
                "        loaded core. In EmuSen.Mistress vstop is already windowed, so -w is\n" +
                "        accepted and simply has no additional effect.\n\n" +
                "EXAMPLES\n" +
                "    vstop\n" +
                "    vstop -w",

            ["mv"] =
                "NAME\n" +
                "    mv - move or rename a file or directory\n\n" +
                "SYNOPSIS\n" +
                "    mv <src> <dst>\n\n" +
                "SUSPENDED\n" +
                "    This command currently refuses to do anything. A published build puts\n" +
                "    its own binaries in '/lib/EmuSen', inside this root - see 'man hier' -\n" +
                "    and nothing in this shell tells the install apart from your data, so\n" +
                "    'mv' could quietly relocate the running application out from under\n" +
                "    itself. Parked until the tree distinguishes the two. Everything below\n" +
                "    describes what it does when re-enabled.\n\n" +
                "DESCRIPTION\n" +
                "    Moves/renames a real file or directory. If <dst> is an existing\n" +
                "    directory, <src> lands inside it ('mv foo.txt logs/' -> 'logs/foo.txt'),\n" +
                "    matching real mv. An existing destination FILE is silently overwritten\n" +
                "    (no -i prompt anywhere in this shell, by design). Both <src> and <dst>\n" +
                "    must resolve inside the project's own directory tree - see 'cd'.\n\n" +
                "EXAMPLES\n" +
                "    mv scratch.txt home/Logs/",

            ["cp"] =
                "NAME\n" +
                "    cp - copy a file or directory\n\n" +
                "SYNOPSIS\n" +
                "    cp <src> <dst>\n" +
                "    cp -r <src> <dst>\n\n" +
                "DESCRIPTION\n" +
                "    Copies a real file. If <dst> is an existing directory, <src> lands\n" +
                "    inside it ('cp foo.txt logs/' -> 'logs/foo.txt'), matching real cp; an\n" +
                "    existing destination FILE is silently overwritten (no -i prompt anywhere\n" +
                "    in this shell, same as 'mv'). A directory requires -r, which copies it\n" +
                "    and everything inside it. Both <src> and <dst> must resolve inside the\n" +
                "    project's own directory tree - see 'cd'.\n\n" +
                "EXAMPLES\n" +
                "    cp notes.txt notes.bak.txt\n" +
                "    cp -r home/Logs/Run1 home/Logs/Run1Backup",

            ["rm"] =
                "NAME\n" +
                "    rm - delete a file or directory\n\n" +
                "SYNOPSIS\n" +
                "    rm <path>\n" +
                "    rm -r <path>\n\n" +
                "SUSPENDED\n" +
                "    This command currently refuses to do anything. A published build puts\n" +
                "    its own binaries in '/lib/EmuSen', inside this root - see 'man hier' -\n" +
                "    and nothing in this shell tells the install apart from your data, so\n" +
                "    'rm -r /lib' would delete the running application. Parked until the\n" +
                "    tree distinguishes the two. Everything below describes what it does\n" +
                "    when re-enabled.\n\n" +
                "DESCRIPTION\n" +
                "    Deletes a real file. A directory requires -r (matching real rm's own\n" +
                "    refusal to remove a directory without it); with -r, the directory and\n" +
                "    everything inside it is deleted, no further confirmation (no -i prompt\n" +
                "    anywhere in this shell, by design, same as 'mv'). Must resolve inside\n" +
                "    the project's own directory tree - see 'cd' - and refuses to remove that\n" +
                "    root directory itself even with -r, so 'rm -r .' from the project root\n" +
                "    (or 'rm -r' on the root's own path) can't wipe out the whole project.\n" +
                "    Like 'cd', rm only ever takes one path, so a directory name with spaces\n" +
                "    doesn't need quoting ('rm -r My Folder' works) - everything after the\n" +
                "    optional -r is treated as the literal path.\n\n" +
                "EXAMPLES\n" +
                "    rm scratch.txt\n" +
                "    rm -r home/Logs/OldRun\n" +
                "    rm -r My Folder",

            ["true"] =
                "NAME\n" +
                "    true - always succeed\n\n" +
                "SYNOPSIS\n" +
                "    true\n\n" +
                "DESCRIPTION\n" +
                "    Does nothing, always exits 0. Its only real use is in control flow -\n" +
                "    'while true; do ...; done', or as a harmless placeholder branch.",

            ["false"] =
                "NAME\n" +
                "    false - always fail\n\n" +
                "SYNOPSIS\n" +
                "    false\n\n" +
                "DESCRIPTION\n" +
                "    Does nothing, always exits 1. Same real use as 'true', just the other\n" +
                "    branch - 'until false; do ...; done' is an infinite loop, same as\n" +
                "    'while true'.",

            ["test"] =
                "NAME\n" +
                "    test, [ - evaluate a condition\n\n" +
                "SYNOPSIS\n" +
                "    test EXPR\n" +
                "    [ EXPR ]\n\n" +
                "DESCRIPTION\n" +
                "    The main way an if/while condition does anything besides check another\n" +
                "    command's own exit code. A deliberately small subset of real test(1):\n" +
                "    a single VALUE (true if non-empty); -z/-n VALUE (string emptiness); -f/-d\n" +
                "    PATH (real file/directory existence - meaningful here since redirection\n" +
                "    already touches real host files); VALUE = / != VALUE (string comparison);\n" +
                "    VALUE -eq/-ne/-lt/-le/-gt/-ge VALUE (numeric comparison); a leading '!'\n" +
                "    negates any of the above. No -a/-o/parenthesized compound expressions -\n" +
                "    combine conditions with the shell's own && / || instead ('[ -f a ] && [\n" +
                "    -f b ]'). '[' is the same evaluator under a different name - the\n" +
                "    trailing ']' is required, matching real bash.\n\n" +
                "EXAMPLES\n" +
                "    if [ -f save.state ]; then echo found; fi\n" +
                "    if test $X -eq 5; then echo five; fi",

            ["["] =
                "NAME\n" +
                "    [ - alias for 'test'\n\n" +
                "SYNOPSIS\n" +
                "    [ EXPR ]\n\n" +
                "DESCRIPTION\n" +
                "    See 'man test' - '[' is the same evaluator under a different name. The\n" +
                "    trailing ']' is required, matching real bash's own '[' builtin (which\n" +
                "    really is just a differently-named 'test').",

            ["history"] =
                "NAME\n" +
                "    history - list previously executed commands\n\n" +
                "SYNOPSIS\n" +
                "    history\n" +
                "    history clear\n" +
                "    !N\n" +
                "    !!\n\n" +
                "DESCRIPTION\n" +
                "    Lists every completed (non-buffered) command line typed at the prompt,\n" +
                "    numbered from 1 like bash's own history builtin. 'history clear' forgets\n" +
                "    all of it. '!N' re-runs history entry N; '!!' re-runs the last command -\n" +
                "    both are expanded before lexing even happens, the same point a real\n" +
                "    shell expands '!'-references, and only at the start of a brand-new\n" +
                "    (not mid-block) line. A script run via 'source' does NOT add its own\n" +
                "    lines to history - only the 'source' invocation itself does.\n\n" +
                "EXAMPLES\n" +
                "    history\n" +
                "    !!\n" +
                "    !12",

            ["state"] =
                "NAME\n" +
                "    state - save or load emulator state\n\n" +
                "SYNOPSIS\n" +
                "    state save [path]\n" +
                "    state load [path]\n\n" +
                "DESCRIPTION\n" +
                "    Saves or loads a full snapshot of emulator state (not battery-backed\n" +
                "    cartridge SRAM, which persists separately) - the same operation each\n" +
                "    frontend's own Save State/Load State hotkey or menu item performs.\n" +
                "    'path' defaults to this frontend's own save-state path (one slot per\n" +
                "    ROM) if omitted.\n\n" +
                "EXAMPLES\n" +
                "    state save\n" +
                "    state load\n" +
                "    state save home/Saves/Save States/before-boss.state",

            ["resume"] =
                "NAME\n" +
                "    resume - resume emulation after an interactive halt\n\n" +
                "SYNOPSIS\n" +
                "    resume\n" +
                "    continue\n" +
                "    c\n\n" +
                "DESCRIPTION\n" +
                "    Resumes emulation after a halt (an interactive debug prompt, a\n" +
                "    breakpoint, a single-step) - 'continue' and 'c' are recognized as plain\n" +
                "    aliases, not separate commands. EmuSen.Mistress registers its own\n" +
                "    pause/resume-aware 'resume' instead (its console window runs on a\n" +
                "    separate thread from emulation, unlike EmuSen.Hotaru's console, which\n" +
                "    shares the emulation thread and needs no pause/resume signal at all).\n\n" +
                "EXAMPLES\n" +
                "    resume\n" +
                "    c",

            ["shutdown"] =
                "NAME\n" +
                "    shutdown - terminate the process\n\n" +
                "SYNOPSIS\n" +
                "    shutdown\n" +
                "    quit\n\n" +
                "DESCRIPTION\n" +
                "    Terminates the whole process - 'quit' is recognized as a plain alias,\n" +
                "    not a separate command. Works the same whether typed at the initial\n" +
                "    no-ROM shell or the in-game debug prompt.\n\n" +
                "EXAMPLES\n" +
                "    shutdown\n" +
                "    quit",

            ["core"] =
                "NAME\n" +
                "    core - load or swap the running ROM\n\n" +
                "SYNOPSIS\n" +
                "    core <corename> <path>\n\n" +
                "DESCRIPTION\n" +
                "    Validates <corename> against the known core registry and <path> against\n" +
                "    that core's supported file extensions, then loads it - reloading\n" +
                "    VenusCore in place if a ROM is already running (a fresh SnesDebugTarget/\n" +
                "    DianaOSInterpreter/FrameRecorder get rebuilt afterward; watches/\n" +
                "    breakpoints/cheats registered against the OLD ROM don't survive a swap,\n" +
                "    matching EmuSen.Mistress's own already-accepted 'shell-level state\n" +
                "    resets on reload' convention). Refuses to swap while a frame recording\n" +
                "    is in progress - stop it first (F6). Only registered where a session\n" +
                "    actually exists to reload - EmuSen.Hotaru's own pre-window launch shell\n" +
                "    has a separate, simpler 'core' handler of its own for resolving the\n" +
                "    FIRST ROM (see EmuSen_Frontend_Driver.md §3), since there's nothing to\n" +
                "    reload yet at that point.\n\n" +
                "EXAMPLES\n" +
                "    core venus game2.smc\n" +
                "    core snes /path/to/other-game.sfc",

            ["tmux"] =
                "NAME\n" +
                "    tmux - multiple independent shell sessions\n\n" +
                "SYNOPSIS\n" +
                "    tmux new [name]\n" +
                "    tmux list\n" +
                "    tmux switch <name>\n" +
                "    tmux kill <name>\n\n" +
                "DESCRIPTION\n" +
                "    Each session is a fully independent DianaOSInterpreter - its own\n" +
                "    variables, command history, and $?/pending-multiline-input state -\n" +
                "    that you can create and switch between. 'new' with no name auto-numbers\n" +
                "    it (session-1, session-2, ...). All sessions observe the same one live\n" +
                "    core/ROM, the same way several real tmux windows attached to one real\n" +
                "    machine each get their own independent shell but see the same one\n" +
                "    machine - this is NOT multiple games running at once, just multiple\n" +
                "    independent shells watching the one that is.\n\n" +
                "    Because a DianaOSInterpreter can't be repointed at a new target once\n" +
                "    built (see 'core'), swapping the ROM rebuilds every session's\n" +
                "    interpreter from scratch, the same 'shell-level state resets on reload'\n" +
                "    convention 'core' already documents - it's not just the current session\n" +
                "    that loses its variables/history on a ROM swap, all of them do.\n\n" +
                "    Not available everywhere: a host needs to opt in by handing\n" +
                "    DianaOSInterpreter.CreateDefault a DianaOSSessionManager. Where it IS\n" +
                "    available, whether switching sessions is itself safe against a\n" +
                "    concurrently-running emulation thread is that host's own concern, not\n" +
                "    this command's - EmuSen.Hotaru/EmuSen.Mistress each keep a separate\n" +
                "    DianaOSInterpreterScheduler per session for exactly that reason; the\n" +
                "    standalone DianaOS shell (no core, no concurrent thread) needs none at\n" +
                "    all.\n\n" +
                "EXAMPLES\n" +
                "    tmux new\n" +
                "    tmux new investigation\n" +
                "    tmux list\n" +
                "    tmux switch investigation\n" +
                "    tmux kill session-1",

            ["ps"] =
                "NAME\n" +
                "    ps, jobs - list active breakpoints, watches, and sessions\n\n" +
                "SYNOPSIS\n" +
                "    ps\n" +
                "    jobs\n\n" +
                "DESCRIPTION\n" +
                "    Lists every active breakpoint (see 'bp'), watch (see 'watch'), and tmux\n" +
                "    session (see 'tmux', if a session manager is registered for this host)\n" +
                "    as one unified table - the read-only counterpart to 'kill'. 'jobs' is a\n" +
                "    plain alias, not a separate command. Each row's id column is exactly\n" +
                "    what 'kill' expects: 'bp<N>' for a breakpoint, 'watch<N>' for a watch, or\n" +
                "    a session's own name - copy a row's id straight into 'kill' unchanged.\n" +
                "    Prints nothing wrong, just an empty-looking table, with no ROM loaded and\n" +
                "    no session manager registered (there's nothing to list, not an error).\n\n" +
                "EXAMPLES\n" +
                "    ps\n" +
                "    jobs",

            ["kill"] =
                "NAME\n" +
                "    kill - remove a breakpoint, watch, or session by its 'ps' id\n\n" +
                "SYNOPSIS\n" +
                "    kill <id>\n\n" +
                "DESCRIPTION\n" +
                "    One verb for tearing down whatever 'ps' just showed, instead of needing\n" +
                "    to know which of 'bp remove'/'watch remove'/'tmux kill' owns a given id -\n" +
                "    all three of those still work exactly as before, this just routes to\n" +
                "    whichever one already owns <id>'s kind of thing, based on its shape:\n" +
                "    'bp<N>' removes breakpoint N, 'watch<N>' removes watch N, and anything\n" +
                "    else is looked up as a session name (same rules as 'tmux kill' - can't\n" +
                "    kill the only remaining session; killing the CURRENT session switches you\n" +
                "    to whichever one is left, the same hand-off 'tmux kill' already does).\n" +
                "    A session literally NAMED like 'bp3' or 'watch1' is only reachable this\n" +
                "    way by accident of that ambiguity - 'kill' always tries the breakpoint/\n" +
                "    watch reading of an id shaped like one first; use 'tmux kill <name>'\n" +
                "    directly for a session whose name collides with that shape.\n\n" +
                "EXAMPLES\n" +
                "    kill bp1\n" +
                "    kill watch3\n" +
                "    kill investigation",

            ["whoami"] =
                "NAME\n" +
                "    whoami - print the account this shell is running as\n\n" +
                "SYNOPSIS\n" +
                "    whoami\n\n" +
                "DESCRIPTION\n" +
                "    Prints THIS shell's own current account (see 'su') - every shell starts\n" +
                "    as 'root'. Unix flavor, not real access control - see 'su's own man page\n" +
                "    for what that means today and what's still future work.\n\n" +
                "EXAMPLES\n" +
                "    whoami",

            ["who"] =
                "NAME\n" +
                "    who - list every session and which account it's running as\n\n" +
                "SYNOPSIS\n" +
                "    who\n\n" +
                "DESCRIPTION\n" +
                "    A different axis from 'ps': 'ps' lists killable THINGS (breakpoints/\n" +
                "    watches/sessions); this lists WHO is running each session, if a session\n" +
                "    manager is registered for this host (see 'tmux'). With none registered,\n" +
                "    just reports this one shell's own account instead of failing.\n\n" +
                "EXAMPLES\n" +
                "    who",

            ["su"] =
                "NAME\n" +
                "    su - switch this shell's active account\n\n" +
                "SYNOPSIS\n" +
                "    su [name]\n\n" +
                "DESCRIPTION\n" +
                "    Switches THIS shell (this tmux session, if any - see 'tmux') to <name>,\n" +
                "    or back to 'root' with no argument, matching real su's own default\n" +
                "    target. <name> must already exist (see 'useradd'). Does NOT check\n" +
                "    'passwd's stored password hash - nothing in this shell enforces\n" +
                "    anything based on which account is active yet, so su succeeds\n" +
                "    unconditionally for any existing account. That enforcement (e.g. a\n" +
                "    restricted account that can't run mutating commands) is real future\n" +
                "    work, not yet built - this pass is the identity/account plumbing it\n" +
                "    will eventually sit on top of, nothing more.\n\n" +
                "    A new session created with 'tmux new' inherits whoever created it\n" +
                "    (not always 'root') and a ROM swap preserves each session's own current\n" +
                "    account across the rebuild - see 'tmux'/'core'.\n\n" +
                "EXAMPLES\n" +
                "    su\n" +
                "    su parent",

            ["useradd"] =
                "NAME\n" +
                "    useradd - add a new account\n\n" +
                "SYNOPSIS\n" +
                "    useradd <name>\n\n" +
                "DESCRIPTION\n" +
                "    Adds <name> to the process-wide account directory shared by every tmux\n" +
                "    session (like a real /etc/passwd) - root only. In-memory only, like\n" +
                "    every other piece of DianaOS state; accounts don't survive a process\n" +
                "    restart yet. See 'su' to switch to the new account.\n\n" +
                "EXAMPLES\n" +
                "    useradd kid",

            ["userdel"] =
                "NAME\n" +
                "    userdel - remove an account\n\n" +
                "SYNOPSIS\n" +
                "    userdel <name>\n\n" +
                "DESCRIPTION\n" +
                "    Removes <name> from the account directory - root only. Refuses to\n" +
                "    remove 'root' itself (the one account that always exists), and refuses\n" +
                "    to remove an account that's currently active in any live session (or\n" +
                "    this shell itself, with no session manager registered) - 'su' that\n" +
                "    session to another account first.\n\n" +
                "EXAMPLES\n" +
                "    userdel kid",

            ["passwd"] =
                "NAME\n" +
                "    passwd - set an account's password\n\n" +
                "SYNOPSIS\n" +
                "    passwd <newpassword>\n" +
                "    passwd <name> <newpassword>\n\n" +
                "DESCRIPTION\n" +
                "    Stores a hash of <newpassword> for your OWN account (no other arguments)\n" +
                "    or, root only, for <name>'s account. NOT checked by 'su' or anything\n" +
                "    else today - see that command's own man page for why. No masked/\n" +
                "    interactive prompt either: there's no reliable blocking-input read\n" +
                "    across every host this shell runs under (a real terminal, a GUI\n" +
                "    TextBox-driven console, a headless test), so the new password is just a\n" +
                "    plain trailing argument, the same way 'bp add'/'watch add' take theirs -\n" +
                "    it will show up in this shell's own 'history' in plain text.\n\n" +
                "EXAMPLES\n" +
                "    passwd hunter2\n" +
                "    passwd kid hunter2",

            ["step"] =
                "NAME\n" +
                "    step - single-step, step over a call, or step out of a routine\n\n" +
                "SYNOPSIS\n" +
                "    step [<count>]\n" +
                "    s [<count>]\n" +
                "    step over\n" +
                "    step out\n" +
                "    step <cpu> [<count>|over|out]\n\n" +
                "DESCRIPTION\n" +
                "    Arms a halt and lets emulation run just far enough to reach it, then\n" +
                "    halts again at the same interactive prompt - 's' is recognized as a\n" +
                "    plain alias, not a separate command. Needs a ROM loaded.\n\n" +
                "    Bare 'step' executes exactly one instruction. 'step <count>' executes\n" +
                "    <count> of them (hex, like every other address/count in this shell), for\n" +
                "    walking past a delay loop without holding down 's'. A real breakpoint\n" +
                "    that fires part-way through still wins and halts early - a multi-step is\n" +
                "    a convenience, not a way to suppress breakpoints.\n\n" +
                "    'step over' runs the call at the current instruction to completion and\n" +
                "    halts on the instruction after it. It works by depth, not by address:\n" +
                "    it records the call stack's current depth and halts at the first\n" +
                "    instruction executed once the stack is back down to it. That makes it\n" +
                "    correct for recursion (an inner call at the same address doesn't stop\n" +
                "    it early) and it degrades gracefully on a non-call instruction, where\n" +
                "    the depth never rises and it behaves exactly like a plain 'step'.\n\n" +
                "    'step out' is the same mechanism with a target of depth - 1: run until\n" +
                "    the routine currently executing returns to its caller. It errors rather\n" +
                "    than running away when the call stack is already empty.\n\n" +
                "    Both depend on the call stack, so both need a core that reports one -\n" +
                "    see 'man bt', which also covers what makes a depth reading go wrong\n" +
                "    (a game that manipulates its own stack) and how to resync it.\n\n" +
                "EXAMPLES\n" +
                "    step\n" +
                "    s 20\n" +
                "    step over\n" +
                "    step out\n" +
                "    step sa1 out\n\n" +
                "OTHER PROCESSORS\n" +
                "    A scope word steps another chip - see 'man cpus'. 'over' and 'out'\n" +
                "    additionally need that chip to report a call stack, so on an SNES they\n" +
                "    work for the main CPU and the SA-1 but not the GSU or SPC700, which\n" +
                "    still accept a plain instruction step.\n\n" +
                "    Stepping one chip lets the others keep running - they share a timebase,\n" +
                "    and freezing everything but one processor would change the behaviour\n" +
                "    being investigated, which on a coprocessor handshake is usually the\n" +
                "    entire bug.\n\n" +
                "SEE ALSO\n" +
                "    cpus, bt, runto, bp, resume",

            ["eval"] =
                "NAME\n" +
                "    eval - evaluate an expression against live core state\n\n" +
                "SYNOPSIS\n" +
                "    eval <expr>\n" +
                "    eval symbols [<filter>]\n" +
                "    eval <cpu> <expr>\n\n" +
                "DESCRIPTION\n" +
                "    Evaluates one integer expression against the machine as it is right\n" +
                "    now and prints the result three ways at once - decimal, hex, binary -\n" +
                "    so a flag word or a packed register never needs converting by hand.\n\n" +
                "    The same language backs conditional breakpoints ('bp add <addr> if\n" +
                "    <expr>'), which is the point of having it: 'eval' is where an expression\n" +
                "    gets checked before being trusted to gate a halt.\n\n" +
                "SYMBOLS\n" +
                "    'eval symbols' lists everything nameable, which is the authoritative\n" +
                "    answer for whatever core is loaded - the list below is the SNES core's.\n\n" +
                "    Registers   a x y s (sp) d (dp) pc pb (pbr) db (dbr) p e\n" +
                "    Flags       flag.c flag.z flag.i flag.d flag.x flag.m flag.v flag.n\n" +
                "                (each 0 or 1)\n" +
                "    Position    frame, scanline, cycle\n" +
                "    Context     opaddr (the 24-bit address of the instruction a halt is\n" +
                "                sitting in front of), stackdepth\n" +
                "    Labels      every name defined via 'label add' - see 'man label'\n\n" +
                "    A core that publishes none of its own still gets symbols for free: the\n" +
                "    generic fallback names every register the core already reports through\n" +
                "    'regs', plus 'frame' and any labels. Nothing has to be written per-core\n" +
                "    for 'eval' to work on a new console.\n\n" +
                "NUMBERS AND MEMORY\n" +
                "    Numbers     123 decimal, $1F or 0x1F hex, %1010 binary\n" +
                "    [addr]      one byte at <addr>\n" +
                "    {addr}      one little-endian word at <addr>\n" +
                "    [SPACE:addr]  read from a named space instead, e.g. [VRAM:$2760]\n\n" +
                "    Without a space name, a read goes to the CPU bus. On the SNES core an\n" +
                "    address that lands in WRAM is served straight from the RAM array\n" +
                "    instead, which is both faster and free of side effects - so the common\n" +
                "    case ('[$7E0020]') is always safe to put in a breakpoint condition.\n" +
                "    An address that does NOT resolve to WRAM goes through the live bus, and\n" +
                "    reading a hardware register there has whatever side effect the real\n" +
                "    register has (RDNMI clears the pending-NMI flag, OPHCT/OPVCT toggle a\n" +
                "    byte-order latch). Name a plain space explicitly when that matters.\n\n" +
                "OPERATORS\n" +
                "    Lowest to highest precedence:\n" +
                "        ||  &&  |  ^  &  == !=  < > <= >=  << >>  + -  * / %\n" +
                "    Unary: - ! ~ . Parentheses group. '&&' and '||' short-circuit, so\n" +
                "    '[$7E0000] != 0 && 100 / [$7E0000] > 2' is safe. Any nonzero value is\n" +
                "    true; comparisons produce 1 or 0.\n\n" +
                "QUOTING\n" +
                "    '&&', '||', '<', '>' and '|' are the shell's own operators and are\n" +
                "    consumed before this command sees them - quote any expression using\n" +
                "    them. Everything after 'eval' is rejoined with single spaces, so simple\n" +
                "    unquoted forms ('eval a + 1') work.\n\n" +
                "EXAMPLES\n" +
                "    eval a\n" +
                "    eval [$7E0020]\n" +
                "    eval {$7E13C6} + 4\n" +
                "    eval \"flag.m == 0 && a > $100\"\n" +
                "    eval [VRAM:$2760]\n" +
                "    eval symbols flag\n" +
                "    eval gsu r14\n" +
                "    eval spc \"a == $F0\"\n\n" +
                "OTHER PROCESSORS\n" +
                "    'eval <cpu> <expr>' evaluates against another chip's registers - see\n" +
                "    'man cpus'. Its symbols are that chip's own, so 'eval gsu r14' reads\n" +
                "    the GSU's R14 while 'eval a' still reads the main CPU's accumulator,\n" +
                "    and an unnamed memory read like '[$8000]' resolves in that chip's own\n" +
                "    code space rather than the main CPU bus.\n\n" +
                "    A chip name alone is an EXPRESSION, not a scope: 'eval a' evaluates the\n" +
                "    symbol 'a'. The scope reading only applies when something follows it.\n\n" +
                "    Symbols for a chip come from whatever registers it already reports, so\n" +
                "    'eval <cpu> symbols' is the reliable way to see what is nameable rather\n" +
                "    than guessing at a register's spelling.\n\n" +
                "SEE ALSO\n" +
                "    cpus, bp, regs, mem, label",

            ["bt"] =
                "NAME\n" +
                "    bt - backtrace the live call chain\n\n" +
                "SYNOPSIS\n" +
                "    bt [<count>]\n" +
                "    bt reset\n" +
                "    bt <cpu> [<count>|reset]\n\n" +
                "DESCRIPTION\n" +
                "    Prints the calls currently on the stack, innermost frame first: what was\n" +
                "    called, where it was called from, and which frame number the call\n" +
                "    happened on. This is the dynamic counterpart to 'callers' (see 'man\n" +
                "    callers'), and the two answer genuinely different questions. 'callers'\n" +
                "    scans code and reports every instruction that COULD reach an address;\n" +
                "    'bt' reports the one path that actually did, this time. When a routine\n" +
                "    has six static callers, 'callers' gives you six suspects and 'bt' gives\n" +
                "    you the answer.\n\n" +
                "    Frames are recorded from the CPU's own call and return opcodes plus its\n" +
                "    interrupt entry points, so an NMI or IRQ frame is labelled as such\n" +
                "    rather than looking like an ordinary call. Addresses are resolved\n" +
                "    through the label registry - see 'man label'.\n\n" +
                "ACCURACY\n" +
                "    A call stack tracked this way is an inference, not machine state, and\n" +
                "    it can drift on code that manipulates its own stack directly: a routine\n" +
                "    that pushes a return address and jumps, that pulls a return address it\n" +
                "    never intends to return through, or that unwinds several frames with a\n" +
                "    stack-pointer write rather than matched returns. Two symptoms give this\n" +
                "    away - a depth that only ever grows, and a rising 'unmatched returns'\n" +
                "    count, which 'bt' reports whenever it is nonzero.\n\n" +
                "    'bt reset' forgets the current chain and starts counting again from the\n" +
                "    current instruction. Do that after stepping through a routine known to\n" +
                "    play with its own stack, and before trusting 'step out' or a 'profile'\n" +
                "    run that has to survive it. Depth is also capped, so a runaway chain\n" +
                "    stops recording rather than growing without bound.\n\n" +
                "    Frames opened before the debug target attached are not on the stack -\n" +
                "    code already running at that moment shows up in 'profile' under\n" +
                "    '(outside any recorded call)' rather than under its real caller.\n\n" +
                "EXAMPLES\n" +
                "    bt\n" +
                "    bt 8\n" +
                "    bt reset\n\n" +
                "OTHER PROCESSORS\n" +
                "    'bt <cpu>' backtraces another chip - see 'man cpus'. Only a chip with a\n" +
                "    real call/return seam has a stack to report: on an SNES that is the\n" +
                "    main CPU and the SA-1, both 65816s. The GSU and the SPC700 report none,\n" +
                "    and say so rather than inventing frames.\n\n" +
                "SEE ALSO\n" +
                "    cpus, callers, step, profile, bp, label",

            ["cpus"] =
                "NAME\n" +
                "    cpus - list the processors a debug command can be aimed at\n\n" +
                "SYNOPSIS\n" +
                "    cpus\n\n" +
                "DESCRIPTION\n" +
                "    A console is rarely one processor. An SNES is always at least two - the\n" +
                "    65816 main CPU and the SPC700 driving sound - and a cartridge can add a\n" +
                "    third: an SA-1, a SuperFX GSU, a NEC DSP. Each runs its own code, in its\n" +
                "    own address space, from its own program counter. An SA-1 game's $00:82D7\n" +
                "    is simply not the same instruction as the main CPU's $00:82D7.\n\n" +
                "    'cpus' lists the ones this core publishes, with what each supports, and\n" +
                "    every scoped command takes one of those names as an optional first word:\n\n" +
                "        bp sa1 add 82D7            breakpoint on the coprocessor's PC\n" +
                "        cov gsu on                 coverage of the GSU's instruction stream\n" +
                "        bt sa1                     the coprocessor's own call chain\n" +
                "        step spc over              step the sound CPU over a call\n" +
                "        eval gsu r14 > 100         evaluate against the GSU's registers\n" +
                "        regs spc                   just that chip's registers\n" +
                "        disasm gsu                 disassemble from where the GSU is now\n\n" +
                "    With no scope word every command means the main CPU, so nothing that\n" +
                "    worked before needs rewriting. 'cop' is still accepted as an alias for\n" +
                "    whichever cartridge coprocessor is present, from before chips had names.\n\n" +
                "WHAT EACH CHIP SUPPORTS\n" +
                "    Support is not uniform, because the hardware is not uniform, and 'cpus'\n" +
                "    prints per chip what is actually wired rather than letting a command\n" +
                "    fail obscurely later:\n\n" +
                "        bp      the core can halt this chip mid-instruction\n" +
                "        cov     its executed addresses can be recorded\n" +
                "        bt      it has a call/return seam to infer a stack from\n" +
                "        regs    it reports named registers\n" +
                "        disasm  it has a code space and a disassembler for its ISA\n\n" +
                "    One absence is worth knowing about. The GSU and the SPC700 have no\n" +
                "    call stack: the GSU's LINK/subroutine convention is register-based\n" +
                "    rather than a stack the way the 65816's JSR/RTS is, so there is no\n" +
                "    honest seam to infer frames from, and 'bt gsu' says so instead of\n" +
                "    inventing them.\n\n" +
                "    The SA-1 gets everything, because it IS a 65816: the same call/return\n" +
                "    and interrupt seams the main CPU uses apply to it unchanged, so\n" +
                "    'bt sa1', 'step sa1 out' and 'profile sa1' all work.\n\n" +
                "    The NEC DSP gets everything too, by a different route. Its firmware\n" +
                "    runs from a mask ROM the cartridge never exposes, so there is no code\n" +
                "    to read - but the chip is still stepped one instruction at a time by\n" +
                "    this core, and that is all a breakpoint needs. Its addresses are word\n" +
                "    indices, not bytes, because that is what the DSP's own PC is: 'bp dsp\n" +
                "    add 100' means the 257th instruction, and 'disasm DSPPRG' indexes the\n" +
                "    same way. Its CALL/RET pair drives a real call stack, so 'bt dsp' and\n" +
                "    'cov dsp funcs' map firmware nobody has source for.\n\n" +
                "HALTING, AND WHICH CHIP RESUMES\n" +
                "    Only one chip halts at a time. Whichever one hit its breakpoint is the\n" +
                "    one that skips a check on resume, so 'continue' leaves the breakpoint it\n" +
                "    is sitting on instead of instantly re-halting - and, just as important,\n" +
                "    the OTHER chips do not skip theirs. Arming the main CPU's resume flag\n" +
                "    for a coprocessor halt would let the coprocessor re-break immediately\n" +
                "    on the same PC, forever.\n\n" +
                "    A halt mid-frame keeps the unspent clock budget of whichever chip was\n" +
                "    running, so resuming continues the frame rather than restarting it.\n\n" +
                "EXAMPLES\n" +
                "    cpus\n" +
                "    bp gsu add 008010\n" +
                "    cov sa1 on\n" +
                "    regs dsp\n\n" +
                "SEE ALSO\n" +
                "    bp, cov, bt, step, profile, eval, regs, disasm, copflow",

            ["copflow"] =
                "NAME\n" +
                "    copflow - watch the CPU and a coprocessor talk\n\n" +
                "SYNOPSIS\n" +
                "    copflow on [<size>]\n" +
                "    copflow off\n" +
                "    copflow clear\n" +
                "    copflow tail [<n>]\n" +
                "    copflow stats\n" +
                "    copflow poll\n\n" +
                "DESCRIPTION\n" +
                "    Every cartridge coprocessor talks to the main CPU through one narrow\n" +
                "    register window, and that window is where coprocessor bugs actually\n" +
                "    live. The chip itself is usually fine. What breaks is the conversation:\n" +
                "    the CPU writes a parameter block and kicks the chip, the chip works and\n" +
                "    sets a status bit, the CPU polls that bit and moves on. Any one of those\n" +
                "    four steps can fail silently, and none of them are visible in a register\n" +
                "    dump - by the time you look, the moment has passed.\n\n" +
                "    'copflow' logs that window: every read and write, with the value, the\n" +
                "    frame it happened on, and which side did it. 'copflow tail' replays the\n" +
                "    recent conversation in order, which is usually enough to see which of\n" +
                "    the four steps never happened.\n\n" +
                "POLL RUNS\n" +
                "    'copflow poll' exists for the most common coprocessor failure by a wide\n" +
                "    margin: the CPU asks a question the chip never answers. It tracks the\n" +
                "    longest run of consecutive reads of one register where the value never\n" +
                "    changed, and reports the run in progress right now.\n\n" +
                "    A long run is unambiguous. If the game read $3030 ninety thousand times\n" +
                "    in a row and always got the same byte back, the game is not slow and the\n" +
                "    plot is not wrong - it is spinning on a status bit that this core never\n" +
                "    updates. That points at the register model, not the chip's arithmetic,\n" +
                "    and it is a very different bug from one 'regs' or 'cophist' would find.\n\n" +
                "    A write by either side always breaks the run, because a write is\n" +
                "    progress: it means someone learned something and acted. Only unbroken\n" +
                "    reads of an unchanging value count.\n\n" +
                "COST\n" +
                "    Disarmed, this costs one bool test per coprocessor register access -\n" +
                "    nothing measurable, and nothing at all on a cartridge with no\n" +
                "    coprocessor, which never reaches the seam. Armed, it keeps whole-run\n" +
                "    tallies per register plus a ring of the most recent accesses, so a long\n" +
                "    session cannot grow memory without bound; 'copflow tail' can only show\n" +
                "    what is still in the ring, while 'copflow stats' and 'copflow poll'\n" +
                "    cover the entire armed run.\n\n" +
                "EXAMPLES\n" +
                "    copflow on\n" +
                "    copflow tail 40\n" +
                "    copflow poll\n" +
                "    copflow stats\n\n" +
                "SEE ALSO\n" +
                "    cpus, cophist, regs, watch, bp",

            ["dma"] =
                "NAME\n" +
                "    dma - what the block-transfer engine is set up to do, and what it did\n\n" +
                "SYNOPSIS\n" +
                "    dma\n" +
                "    dma log on [<n>]\n" +
                "    dma log off\n" +
                "    dma log [<n>]\n" +
                "    dma log clear\n" +
                "    dma stats\n\n" +
                "DESCRIPTION\n" +
                "    On a SNES nearly everything the player sees arrives by DMA. Tiles,\n" +
                "    palettes, sprite tables and the sound driver's samples are all block\n" +
                "    transfers, and per-scanline effects - a status-bar split, a colour\n" +
                "    gradient, a growing window wipe - are HDMA. So a large class of visual\n" +
                "    bug is not a rendering bug at all: the data never arrived, or arrived\n" +
                "    somewhere else, and the renderer faithfully drew what it was given.\n\n" +
                "    'dma' alone prints the channel table: for each of the eight channels,\n" +
                "    its direction, its B-bus write pattern, which register it targets (by\n" +
                "    name, not just an address), where it reads from, how long the transfer\n" +
                "    is, and for HDMA the table pointer and line counter it has reached. An\n" +
                "    idle channel still shows its last configuration, which is usually the\n" +
                "    thing you wanted to see.\n\n" +
                "TRANSFER LOG\n" +
                "    'dma log on' records every transfer as it happens - channel, kind,\n" +
                "    destination, source, length, and the frame and scanline it ran on.\n" +
                "    This is the part the channel table cannot give you, because a general\n" +
                "    DMA reconfigures and fires many times a frame and the table only ever\n" +
                "    shows the last one.\n\n" +
                "    The scanline column is what makes it worth reading. A VRAM transfer\n" +
                "    during vblank is normal; the same transfer at scanline 100 is a bug\n" +
                "    that will show as corrupt graphics, and 'dma log' names the frame it\n" +
                "    first happened on. Pair it with 'bp when ppuaccess' to halt on one.\n\n" +
                "BANDWIDTH\n" +
                "    'dma stats' totals the whole armed run two ways: per channel, split\n" +
                "    into general and HDMA because those cost very differently, and per\n" +
                "    destination register, which answers 'where is the bandwidth going'.\n" +
                "    A game that is slow during one scene and not another is usually\n" +
                "    uploading something enormous, and the destination table names it.\n\n" +
                "COST\n" +
                "    Disarmed the log is a null field test at the end of each transfer, so\n" +
                "    a normal run pays nothing. Armed, it is a fixed-size ring plus a set\n" +
                "    of counters, so a long session cannot grow memory without bound -\n" +
                "    'dma log' shows only what is still in the ring, while 'dma stats'\n" +
                "    covers the whole armed run. The channel table is read on demand and\n" +
                "    costs nothing at all when not being run.\n\n" +
                "EXAMPLES\n" +
                "    dma\n" +
                "    dma log on\n" +
                "    dma log 40\n" +
                "    dma stats\n\n" +
                "SEE ALSO\n" +
                "    bp, copflow, mem, watch, tilemap",

            ["label"] =
                "NAME\n" +
                "    label - name addresses, so an investigation's findings survive it\n\n" +
                "SYNOPSIS\n" +
                "    label add <addr> <name> [<comment...>]\n" +
                "    label list [<filter>]\n" +
                "    label at <addr>\n" +
                "    label remove <name|addr>\n" +
                "    label clear\n" +
                "    label load <path>\n" +
                "    label save <path>\n\n" +
                "DESCRIPTION\n" +
                "    Attaches a name (and optionally a comment) to an address. Once named,\n" +
                "    that address reads as '$00A3B2 <NmiHandler>' everywhere it appears -\n" +
                "    'disasm', 'bt', 'bp list', 'counters top', 'profile top' - and the name\n" +
                "    becomes usable as a symbol in any expression, so 'bp add NmiHandler'\n" +
                "    and 'eval [PlayerX]' work.\n\n" +
                "    This is the piece that makes a long investigation compound instead of\n" +
                "    restarting. Without it, every session re-derives what $7E13C6 was, from\n" +
                "    notes kept somewhere outside the tool. 'label save' writes the set out\n" +
                "    and 'label load' merges one back in, so the naming survives a restart\n" +
                "    and can be committed alongside whatever else an investigation produced.\n\n" +
                "    'disasm' prints a label on its own line above the instruction it names,\n" +
                "    the way a disassembly listing does, and annotates any operand whose\n" +
                "    statically-resolvable target is itself labelled.\n\n" +
                "    'label at <addr>' answers the reverse question - given an address in the\n" +
                "    middle of a routine, which routine is it in? An exact hit prints the\n" +
                "    name; otherwise it reports the nearest label at or below the address\n" +
                "    with the offset ('MainLoop+12').\n\n" +
                "    The map is one-to-one in both directions: renaming an address drops its\n" +
                "    old name, and pointing an existing name at a new address moves it\n" +
                "    rather than leaving two entries.\n\n" +
                "FILE FORMAT\n" +
                "    One entry per line, '<hex address> <name> [comment...]'. Blank lines and\n" +
                "    lines starting with '#' are ignored. '$' and '0x' prefixes on the\n" +
                "    address are accepted. Deliberately its own trivial format rather than\n" +
                "    any one assembler's symbol file, since nothing here knows which\n" +
                "    assembler a given ROM was built with; a real .sym/.mlb file converts\n" +
                "    with one 'awk' line. Paths resolve inside this shell's sandbox - see\n" +
                "    'man hier'.\n\n" +
                "EXAMPLES\n" +
                "    label add 00A3B2 NmiHandler main vblank entry\n" +
                "    label add 7E13C6 CoinCount\n" +
                "    label at 00A3C0\n" +
                "    label list Nmi\n" +
                "    label save /Usr/Home/Documents/smw.labels\n\n" +
                "SEE ALSO\n" +
                "    disasm, bt, bp, eval, callers",

            ["runto"] =
                "NAME\n" +
                "    runto - resume until a named event instead of an address\n\n" +
                "SYNOPSIS\n" +
                "    runto nmi | irq | brk | cop\n" +
                "    runto scanline <n>\n" +
                "    runto frame [<n>]\n" +
                "    runto <addr>\n\n" +
                "DESCRIPTION\n" +
                "    Resumes emulation and halts at the next occurrence of an event, rather\n" +
                "    than requiring an address to put a breakpoint on. Half of debugging a\n" +
                "    console is 'get me to the interesting moment', and for a whole class of\n" +
                "    moments there is no single address to break on - the vblank handler's\n" +
                "    entry address is not knowable before you've found it, and 'the state at\n" +
                "    scanline 100' is not an address at all.\n\n" +
                "    'runto nmi|irq|brk|cop' halts as soon as the CPU takes an interrupt of\n" +
                "    that kind, with the halt landing inside the handler. Pair it with 'bt',\n" +
                "    which then shows what the interrupt interrupted, and 'label add opaddr\n" +
                "    NmiHandler' to keep the address you just found.\n\n" +
                "    'runto scanline <n>' halts at the start of scanline <n> of whichever\n" +
                "    frame is in progress, which is how you catch a mid-frame register write\n" +
                "    (a status-bar split, a mid-screen mode change) in the act. Scanline\n" +
                "    numbers here are decimal.\n\n" +
                "    'runto frame [<n>]' halts once frame <n> has completed; with no argument\n" +
                "    it means the next frame, which is the fastest way to advance exactly one\n" +
                "    frame from a halt.\n\n" +
                "    Events are noticed where they happen but the halt lands at the next\n" +
                "    instruction boundary, because that is the only place a core can safely\n" +
                "    stop - the same one-instruction lag 'bp write' has, for the same reason.\n\n" +
                "    'runto <addr>' is a convenience for a plain address: it adds an ordinary\n" +
                "    breakpoint and resumes. That breakpoint is NOT removed when it fires -\n" +
                "    the command reports its id so 'bp remove <id>' can clear it. A one-shot\n" +
                "    breakpoint would need a concept the registry doesn't have, and silently\n" +
                "    leaving a permanent breakpoint behind would be worse than saying so.\n\n" +
                "    An armed run-to survives a plain 'continue': it fires whenever the event\n" +
                "    next happens. Only one run-to of each kind can be armed at a time -\n" +
                "    arming a second replaces the first.\n\n" +
                "EXAMPLES\n" +
                "    runto nmi\n" +
                "    runto scanline 100\n" +
                "    runto frame\n" +
                "    runto 808000\n\n" +
                "SEE ALSO\n" +
                "    step, bp, bt, resume",

            ["counters"] =
                "NAME\n" +
                "    counters - per-address read/write/execute tallies\n\n" +
                "SYNOPSIS\n" +
                "    counters on <space>\n" +
                "    counters off\n" +
                "    counters clear\n" +
                "    counters <addr> [<len>]\n" +
                "    counters top [r|w|x|u] [<n>]\n" +
                "    counters cold <addr> <len>\n\n" +
                "DESCRIPTION\n" +
                "    Counts every read, write and execute per address in one memory space.\n" +
                "    Where 'watch' records individual events with context and 'cov' records\n" +
                "    only whether an address ever executed, this records how MANY times -\n" +
                "    which is the question behind 'is this table live or dead', 'which byte\n" +
                "    of this struct does the game actually touch', and 'which half of this\n" +
                "    buffer is the one being updated'.\n\n" +
                "    Armed against one space at a time, because the tallies cost four arrays\n" +
                "    the size of that space - worth paying where a question is being asked,\n" +
                "    not worth paying everywhere by default. 'counters off' stops counting\n" +
                "    but leaves what was collected readable; 'counters clear' zeroes the\n" +
                "    counts and stays armed.\n\n" +
                "    A bare address (with an optional length) reports totals over that range,\n" +
                "    including how many of its bytes were touched at all. 'counters top'\n" +
                "    ranks the busiest addresses - 'r' reads (default), 'w' writes, 'x'\n" +
                "    executes, 'u' uninitialized reads. 'counters cold' is the inverse:\n" +
                "    addresses in a range that nothing ever touched, which is what proves a\n" +
                "    table is unused rather than merely quiet.\n\n" +
                "UNINITIALIZED READS\n" +
                "    An address read before anything wrote it (since counting started) is\n" +
                "    counted separately. On real hardware that read returns whatever the RAM\n" +
                "    powered up holding, so a game doing it is either relying on\n" +
                "    power-on state or has a genuine bug - and an emulator whose RAM fill\n" +
                "    differs from hardware will diverge exactly there. 'counters top u' is\n" +
                "    the fastest way to find those addresses.\n\n" +
                "    'Since counting started' is doing real work in that sentence: arm before\n" +
                "    the moment being studied, not after, or an address the game initialized\n" +
                "    earlier will look uninitialized.\n\n" +
                "    Execution is counted against whatever space the core reports its program\n" +
                "    counter in (the CPU bus), so counting a RAM space leaves the execute\n" +
                "    column at zero unless that RAM is genuinely executed from.\n\n" +
                "EXAMPLES\n" +
                "    counters on WRAM\n" +
                "    counters 13c6 8\n" +
                "    counters top w 10\n" +
                "    counters top u\n" +
                "    counters cold 1000 200\n\n" +
                "SEE ALSO\n" +
                "    watch, cov, search, memfind, profile",

            ["freeze"] =
                "NAME\n" +
                "    freeze - pin an address by undoing every write to it\n\n" +
                "SYNOPSIS\n" +
                "    freeze add <space> <addr> [<value>]\n" +
                "    freeze list\n" +
                "    freeze remove <id>\n" +
                "    freeze clear\n\n" +
                "DESCRIPTION\n" +
                "    Holds an address at a value by writing that value straight back the\n" +
                "    instant anything writes something else. Without a <value> it pins\n" +
                "    whatever the address holds right now, which is the common case: find an\n" +
                "    address with 'search', freeze it, watch what stops moving.\n\n" +
                "    This is a debugging instrument, not a cheat. 'cheat poke' re-applies\n" +
                "    once per frame, so the game's own value is live for most of that frame\n" +
                "    and every read in between sees it; a freeze undoes the write\n" +
                "    immediately, so nothing ever observes the value the game tried to store.\n" +
                "    That difference is the whole point when the question is 'what breaks if\n" +
                "    this counter never changes' or 'is this the variable driving that\n" +
                "    animation' - a per-frame poke answers neither cleanly.\n\n" +
                "    The restore is itself a write, and is suppressed from re-entering the\n" +
                "    freeze check, so it cannot recurse. Writes matching the frozen value are\n" +
                "    left alone and not counted. 'freeze list' reports how many writes each\n" +
                "    entry has undone, which doubles as a cheap 'is anything even writing\n" +
                "    here' signal - a freeze with a blocked count of zero means the address\n" +
                "    you suspected is not the one being written.\n\n" +
                "    Only spaces the core reports writes for can be frozen; freezing an\n" +
                "    address in a read-only space is refused rather than silently ignored.\n" +
                "    Freezing an address whose writes the core does not observe will also\n" +
                "    show a blocked count of zero - see 'man watch' for which paths report.\n\n" +
                "EXAMPLES\n" +
                "    freeze add WRAM 13c6\n" +
                "    freeze add WRAM 0dbf 63\n" +
                "    freeze list\n" +
                "    freeze clear\n\n" +
                "SEE ALSO\n" +
                "    cheat, watch, search, counters",

            ["profile"] =
                "NAME\n" +
                "    profile - which routines the instruction budget actually goes to\n\n" +
                "SYNOPSIS\n" +
                "    profile on\n" +
                "    profile off\n" +
                "    profile clear\n" +
                "    profile top [<n>]\n" +
                "    profile <cpu> on|off|clear|top [<n>]\n\n" +
                "DESCRIPTION\n" +
                "    Charges every executed instruction to whichever routine is innermost on\n" +
                "    the call stack at the time, then ranks routines by the count. That makes\n" +
                "    the figure EXCLUSIVE: a routine that spends all its time inside a call\n" +
                "    it made scores low, and the callee scores high. Inclusive ('this\n" +
                "    routine and everything under it') is deliberately not reported, because\n" +
                "    on a stack that can drift (see 'man bt') an inclusive number compounds\n" +
                "    the drift while an exclusive one localizes it.\n\n" +
                "    The unit is instructions, not cycles. Instructions are what the\n" +
                "    per-instruction seam this rides on can count exactly; a cycle figure\n" +
                "    would have to be attributed across a boundary the seam doesn't see. For\n" +
                "    'which routine is hot', the two rank almost identically.\n\n" +
                "    Counting only happens while armed, so 'profile on' ... 'profile off'\n" +
                "    brackets the window being studied. Both the call count and the\n" +
                "    instruction count are reported per routine, so a routine that is hot\n" +
                "    because it is slow is distinguishable from one that is hot because it is\n" +
                "    called constantly.\n\n" +
                "    Instructions executed while nothing is on the call stack are charged to\n" +
                "    '(outside any recorded call)'. A large share there is normal and means\n" +
                "    the code was already running when profiling started - it is not a bug,\n" +
                "    but it is a reason to arm profiling from a known point and to run 'bt\n" +
                "    reset' first if the stack looks wrong.\n\n" +
                "    This profiles the GAME, not the emulator. For where EmuSen's own frame\n" +
                "    time goes, see 'perf' and 'coretop'.\n\n" +
                "EXAMPLES\n" +
                "    profile on\n" +
                "    profile top 10\n" +
                "    profile off\n" +
                "    profile sa1 top 10\n\n" +
                "OTHER PROCESSORS\n" +
                "    A scope word profiles another chip - see 'man cpus'. Since this is\n" +
                "    built on the call stack, it only works for a chip that reports one: on\n" +
                "    an SNES, the main CPU and the SA-1.\n\n" +
                "SEE ALSO\n" +
                "    cpus, bt, counters, cov, coretop",
        };
    }
}
