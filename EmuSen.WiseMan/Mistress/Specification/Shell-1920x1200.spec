# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Shell-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Shell
case Shell
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
      2 ToolBar at=386,205,1149,79.5 by=panel limit=350.48,187,1219.05,822 name=HotaruToolbar paints=#55555AFF,#9C9CA0FF role=toolbar
        3 ActionButton at=392,211,118.5,66 by=clip drawn focus heard=BACK limit=386,205,1149,79.5 op paints=#55555AFF,#9C9CA0FF role=button text=Back
          4 TextBlock at=417.5,220,67.5,48 box=45,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=BACK layout=44.85,31.2 limit=392,211,118.5,66 owner=3 paints=#EEEEF0FF role=text size=26 text=BACK
        5 ActionButton at=516.5,211,177,66 by=clip drawn focus heard=FORWARD limit=386,205,1149,79.5 op paints=#55555AFF,#9C9CA0FF role=button text=Forward
          6 TextBlock at=542,220,126,48 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FORWARD layout=83.12,31.2 limit=516.5,211,177,66 owner=5 paints=#9C9CA0FF role=text size=26 text=FORWARD
        7 ActionButton at=699.5,211,190.5,66 by=clip drawn focus heard=COMPLETE limit=386,205,1149,79.5 op paints=#55555AFF,#9C9CA0FF role=button text=Complete
          8 TextBlock at=725,220,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COMPLETE layout=92.09,31.2 limit=699.5,211,190.5,66 owner=7 paints=#9C9CA0FF role=text size=26 text=COMPLETE
        9 ActionButton at=896,211,135,66 by=clip drawn focus heard=CLEAR limit=386,205,1149,79.5 op paints=#55555AFF,#9C9CA0FF role=button text=Clear
          10 TextBlock at=921.5,220,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLEAR layout=55.85,31.2 limit=896,211,135,66 owner=9 paints=#9C9CA0FF role=text size=26 text=CLEAR
        11 ActionButton at=1037,211,120,66 by=clip drawn focus heard=COPY limit=386,205,1149,79.5 op paints=#55555AFF,#9C9CA0FF role=button text=Copy
          12 TextBlock at=1062.5,220,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COPY layout=45.73,31.2 limit=1037,211,120,66 owner=11 paints=#9C9CA0FF role=text size=26 text=COPY
        13 ActionButton at=1163,211,199.5,66 by=clip drawn focus heard=INTERRUPT limit=386,205,1149,79.5 op paints=#55555AFF,#9C9CA0FF role=button text=Interrupt
          14 TextBlock at=1188.5,220,148.5,48 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INTERRUPT layout=98.1,31.2 limit=1163,211,199.5,66 owner=13 paints=#9C9CA0FF role=text size=26 text=INTERRUPT
      15 HintText at=386,973,1149,36 box=766,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes" layout=466.26,24 limit=350.48,187,1219.05,822 name=HotaruHint paints=#86868AFF role=text size=20 text="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes"
      16 ConsolePane at=398,296.5,1125,664.5 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=group
        17 TextBlock at=398,898,192,49.5 box=128,33 by=clip caps=17.1387 drawn face="Noto Sans" heard="DianaSh #: " layout=121.66,32.69 limit=398,296.5,1125,664.5 name=PART_Prompt paints=#9C9CA0FF part role=text size=24 text="DianaSh #: "
        18 TextBox at=596,886,927,75 drawn focus heard="Console input" name=PART_Input op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=edit
          19 ScrollViewer allow="a text box scrolls its own text" at=612.5,896.5,894,54 name=PART_ScrollViewer paints=#9C9CA0FF part role=scrollviewer
            20 TextBlock allow="a text box scrolls its own text" at=612.5,896.5,894,54 drawn in=19 name=PART_Placeholder paints=#86868AFF part role=text
        21 ScrollViewer at=398,296.5,1125,577.5 fades=b name=PART_Scroll overflow=1140 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF part role=scrollviewer viewport=398,296.5,1125,577.5
          22 SelectableTextBlock at=398,296.5,1125,1717.5 box=750,1145 by=scroll caps=17.1387 drawn face="Noto Sans" heard="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each." in=21 layout=736.78,1144.08 limit=398,296.5,1125,577.5 name=PART_Output paints=#9C9CA0FF part role=text seen=398,296.5,1125,577.5 size=24 text="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each."
focus 3
edge 3 up same
edge 3 down 18
edge 3 left same
edge 3 right 5
edge 5 up same
edge 5 down 18
edge 5 left 3
edge 5 right 7
edge 7 up same
edge 7 down 18
edge 7 left 5
edge 7 right 9
edge 9 up same
edge 9 down 18
edge 9 left 7
edge 9 right 11
edge 11 up same
edge 11 down 18
edge 11 left 9
edge 11 right 13
edge 13 up same
edge 13 down 18
edge 13 left 11
edge 13 right same
edge 18 up 5
edge 18 down same
edge 18 left 3
edge 18 right same
press pageup focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press pagedown focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press first focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press last focus="ActionButton 'Back'" title=Shell top=HotaruWindow
press back top=none
