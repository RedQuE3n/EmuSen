# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Shell-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Shell
case Shell
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 DockPanel at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
      2 ToolBar at=258,136,765,53 by=panel limit=234,124,812.35,548 name=HotaruToolbar paints=#55555AFF,#9C9CA0FF role=toolbar
        3 ActionButton at=262,140,157,44 by=clip drawn focus heard="HISTORY BACK" limit=258,136,765,53 op paints=#55555AFF,#9C9CA0FF role=button text="History Back"
          4 TextBlock at=279,146,123,32 box=123,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="HISTORY BACK" layout=122.64,31.2 limit=262,140,157,44 owner=3 paints=#EEEEF0FF role=text size=26 text="HISTORY BACK"
        5 ActionButton at=423,140,195,44 by=clip drawn focus heard="HISTORY FORWARD" limit=258,136,765,53 op paints=#55555AFF,#9C9CA0FF role=button text="History Forward"
          6 TextBlock at=440,146,161,32 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="HISTORY FORWARD" layout=160.91,31.2 limit=423,140,195,44 owner=5 paints=#9C9CA0FF role=text size=26 text="HISTORY FORWARD"
        7 ActionButton at=622,140,127,44 by=clip drawn focus heard=COMPLETE limit=258,136,765,53 op paints=#55555AFF,#9C9CA0FF role=button text=Complete
          8 TextBlock at=639,146,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COMPLETE layout=92.09,31.2 limit=622,140,127,44 owner=7 paints=#9C9CA0FF role=text size=26 text=COMPLETE
        9 ActionButton at=753,140,90,44 by=clip drawn focus heard=CLEAR limit=258,136,765,53 op paints=#55555AFF,#9C9CA0FF role=button text=Clear
          10 TextBlock at=770,146,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLEAR layout=55.85,31.2 limit=753,140,90,44 owner=9 paints=#9C9CA0FF role=text size=26 text=CLEAR
        11 ActionButton at=847,140,156,44 by=clip drawn focus heard="COPY OUTPUT" limit=258,136,765,53 op paints=#55555AFF,#9C9CA0FF role=button text="Copy Output"
          12 TextBlock at=864,146,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="COPY OUTPUT" layout=121.03,31.2 limit=847,140,156,44 owner=11 paints=#9C9CA0FF role=text size=26 text="COPY OUTPUT"
      13 HintText at=258,648,765,24 box=765,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes" layout=466.26,24 limit=234,124,812.35,548 name=HotaruHint paints=#86868AFF role=text size=20 text="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes"
      14 ConsolePane at=266,197,749,443 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=group
        15 TextBlock at=266,598,128,33 box=128,33 by=clip caps=17.1387 drawn face="Noto Sans" heard="DianaSh #: " layout=121.66,32.69 limit=266,197,749,443 name=PART_Prompt paints=#9C9CA0FF part role=text size=24 text="DianaSh #: "
        16 TextBox at=398,590,617,50 drawn focus heard="Console input" name=PART_Input op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=edit
          17 ScrollViewer allow="a text box scrolls its own text" at=409,597,595,36 name=PART_ScrollViewer paints=#9C9CA0FF part role=scrollviewer
            18 TextBlock allow="a text box scrolls its own text" at=409,597,595,36 drawn in=17 name=PART_Placeholder paints=#86868AFF part role=text
        19 ScrollViewer at=266,197,749,385 fades=b name=PART_Scroll overflow=760 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF part role=scrollviewer viewport=266,197,749,385
          20 SelectableTextBlock at=266,197,749,1145 box=749,1145 by=scroll caps=17.1387 drawn face="Noto Sans" heard="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each." in=19 layout=736.78,1144.08 limit=266,197,749,385 name=PART_Output paints=#9C9CA0FF part role=text seen=266,197,749,385 size=24 text="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each."
focus 3
edge 3 up same
edge 3 down 16
edge 3 left same
edge 3 right 5
edge 5 up same
edge 5 down 16
edge 5 left 3
edge 5 right 7
edge 7 up same
edge 7 down 16
edge 7 left 5
edge 7 right 9
edge 9 up same
edge 9 down 16
edge 9 left 7
edge 9 right 11
edge 11 up same
edge 11 down 16
edge 11 left 9
edge 11 right same
edge 16 up 5
edge 16 down same
edge 16 left same
edge 16 right same
press pageup focus="ActionButton 'History Back'" title=Shell top=HotaruWindow
press pagedown focus="ActionButton 'History Back'" title=Shell top=HotaruWindow
press first focus="ActionButton 'History Back'" title=Shell top=HotaruWindow
press last focus="ActionButton 'History Back'" title=Shell top=HotaruWindow
press back top=none
