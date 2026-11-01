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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
      2 ToolBar at=386,205,1149,79.5 by=panel limit=350.48,187,1219.05,822 name=HotaruToolbar paints=#707070FF,#D4D4D4FF role=toolbar
        3 ActionButton at=392,211,117,66 by=clip drawn focus heard=BACK limit=386,205,1149,79.5 op paints=#707070FF,#D4D4D4FF role=button text=Back
          4 TextBlock at=417.5,220,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=BACK layout=43.89,31.2 limit=392,211,117,66 owner=3 paints=#EEEEF0FF role=text size=26 text=BACK
        5 ActionButton at=515,211,172.5,66 by=clip drawn focus heard=FORWARD limit=386,205,1149,79.5 op paints=#707070FF,#D4D4D4FF role=button text=Forward
          6 TextBlock at=540.5,220,121.5,48 box=81,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FORWARD layout=80.99,31.2 limit=515,211,172.5,66 owner=5 paints=#D4D4D4FF role=text size=26 text=FORWARD
        7 ActionButton at=693.5,211,181.5,66 by=clip drawn focus heard=COMPLETE limit=386,205,1149,79.5 op paints=#707070FF,#D4D4D4FF role=button text=Complete
          8 TextBlock at=719,220,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COMPLETE layout=86.61,31.2 limit=693.5,211,181.5,66 owner=7 paints=#D4D4D4FF role=text size=26 text=COMPLETE
        9 ActionButton at=881,211,132,66 by=clip drawn focus heard=CLEAR limit=386,205,1149,79.5 op paints=#707070FF,#D4D4D4FF role=button text=Clear
          10 TextBlock at=906.5,220,81,48 box=54,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLEAR layout=53.17,31.2 limit=881,211,132,66 owner=9 paints=#D4D4D4FF role=text size=26 text=CLEAR
        11 ActionButton at=1019,211,117,66 by=clip drawn focus heard=COPY limit=386,205,1149,79.5 op paints=#707070FF,#D4D4D4FF role=button text=Copy
          12 TextBlock at=1044.5,220,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=COPY layout=43.03,31.2 limit=1019,211,117,66 owner=11 paints=#D4D4D4FF role=text size=26 text=COPY
        13 ActionButton at=1142,211,189,66 by=clip drawn focus heard=INTERRUPT limit=386,205,1149,79.5 op paints=#707070FF,#D4D4D4FF role=button text=Interrupt
          14 TextBlock at=1167.5,220,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INTERRUPT layout=91.81,31.2 limit=1142,211,189,66 owner=13 paints=#D4D4D4FF role=text size=26 text=INTERRUPT
      15 HintText at=386,973,1149,36 box=766,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes" layout=443.3,24 limit=350.48,187,1219.05,822 name=HotaruHint paints=#8D8D91FF role=text size=20 text="L1 / R1 recall history · L2 / R2 complete · B clears the line, then closes"
      16 ConsolePane at=398,296.5,1125,664.5 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=group
        17 TextBlock at=398,898,192,49.5 box=128,33 by=clip caps=17.1387 drawn face="Noto Sans" heard="DianaSh #: " layout=121.66,32.69 limit=398,296.5,1125,664.5 name=PART_Prompt paints=#D4D4D4FF part role=text size=24 text="DianaSh #: "
        18 TextBox at=596,886,927,75 drawn focus heard="Console input" name=PART_Input op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
          19 ScrollViewer allow="a text box scrolls its own text" at=612.5,896.5,894,54 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
            20 TextBlock allow="a text box scrolls its own text" at=612.5,896.5,894,54 drawn in=19 name=PART_Placeholder paints=#8D8D91FF part role=text
        21 ScrollViewer at=398,296.5,1125,577.5 fades=b name=PART_Scroll overflow=1140 paints=#707070FF,#D4D4D4FF,#FFFFFFFF part role=scrollviewer viewport=398,296.5,1125,577.5
          22 SelectableTextBlock at=398,296.5,1125,1717.5 box=750,1145 by=scroll caps=17.1387 drawn face="Noto Sans" heard="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each." in=21 layout=736.78,1144.08 limit=398,296.5,1125,577.5 name=PART_Output paints=#D4D4D4FF part role=text seen=398,296.5,1125,577.5 size=24 text="+------------------------------------------------------------------------+\n|                        Welcome to DianaSh v0.1a                        |\n|                A Unix shell centered around emulation.                 |\n|      DianaSh is also a functional cross-platform Bash-like shell       |\n|                        that accepts scripting.                         |\n+------------------------------------------------------------------------+\n\nSupported cores:\n  - NES (Moon)\n  - Game Boy (Mercury)\n  - SNES (Venus)\n  - Nintendo 64 (Mars)\n  - SG-1000 (Endou)\n  - Master System (Endou)\n  - Genesis (Nephrite)\n  - Sega CD (Nephrite)\n  - PlayStation (Diamond)\n  - PlayStation 2 (Sapphire)\n\nFeatures:\n  - Pipes, redirection, $(...) command substitution, shell variables\n  - Control flow: if/for/while, break/continue\n  - Scripting: source <path> to run scripts, nano <path> to edit them\n  - Live hardware inspection: regs, mem, watch, bp, disasm, trace\n  - Cheats (Action Replay / Game Genie), snapshot/diff, dump/load\n  - coretop: live htop-style hardware dashboard (-w opens a window)\n  - vstop: the same, for the .NET runtime underneath (works with no core loaded)\n  - feed / feed -w: watch gameplay without losing the shell\n  - clear, nano, history, and the rest of a real shell's toolkit\n\nType \"help\" for a list of commands. Man is supported for each."
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
