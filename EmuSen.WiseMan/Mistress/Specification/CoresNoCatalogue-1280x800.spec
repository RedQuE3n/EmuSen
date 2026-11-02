# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresNoCatalogue-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresNoCatalogue
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=CORES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=719,47,497,25 box=497,25 by=panel caps=13.999 drawn face=Barlow heard="4 cores · 4 installed · ⚠ 1 to approve · catalogue not read yet" layout=496.85,24.82 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="4 cores · 4 installed · ⚠ 1 to approve · catalogue not read yet"
                8 StackPanel at=64,40,643,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,643,39 box=643,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    20 TextBlock at=154,99,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,91,146,44 owner=19 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  21 ToggleButton at=292,91,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    22 TextBlock at=308,99,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,91,160,44 owner=21 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  23 ToggleButton at=460,91,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=476,99,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,91,79,44 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,147,691,506 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,147,691,506 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,147,691,506 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=64,158,691,495 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,495
                        31 ListBoxItem at=64,158,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,158,691,495 op paints=#D4D4D4FF role=listitem
                          32 Border at=88,168,643,33 by=clip in=30 limit=64,158,691,54 paints=#707070FF role=none
                            33 Grid at=88,168,643,32 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,158,691,54 paints=#4C9AE8FF role=none
                              34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,168,246,32 box=246,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=382,168,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=550,168,157,32 box=157,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=75.79,31.2 owner=31 paints=#8D8D91FF role=edit size=26 text="Built in"
                        37 ListBoxItem at=64,212,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,158,691,495 op paints=#D4D4D4FF role=listitem
                          38 Border at=88,222,643,34 by=clip in=30 limit=64,212,691,54 paints=#707070FF role=none
                            39 Grid at=88,222,643,33 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,212,691,54 paints=#4C9AE8FF role=none
                              40 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,222,246,32 box=246,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=382,222,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=550,222,157,33 box=157,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=37 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        43 ListBoxItem at=64,266,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,158,691,495 op paints=#D4D4D4FF role=listitem
                          44 Border at=88,276,643,34 by=clip in=30 limit=64,266,691,54 paints=#707070FF role=none
                            45 Grid at=88,276,643,33 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,266,691,54 paints=#4C9AE8FF role=none
                              46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,276,246,32 box=246,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=382,276,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=60.79,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=SNES
                              48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=550,276,157,33 box=157,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=43 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        49 ListBoxItem at=64,320,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,158,691,495 op paints=#D4D4D4FF role=listitem
                          50 Border at=88,330,643,34 by=clip in=30 limit=64,320,691,54 paints=#707070FF role=none
                            51 Grid at=88,330,643,33 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,320,691,54 paints=#4C9AE8FF role=none
                              52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,330,246,32 box=246,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                              53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=382,330,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=550,330,157,33 box=157,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=156.22,32.27 owner=49 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                55 Border at=64,653,691,93 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  56 Grid at=68,657,683,85 by=panel limit=0,0,1280,800 role=none
                    57 Grid at=68,657,683,53 by=panel limit=0,0,1280,800 role=none
                      58 TextBlock at=76,657,675,53 box=675,53 by=panel caps=15.3989 drawn face=Barlow heard="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com." layout=623.92,52.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com."
                    59 StackPanel at=352,718,399,24 by=panel limit=0,0,1280,800 role=none
                      60 StackPanel at=352,718,399,24 by=panel limit=0,0,1280,800 role=none
                        61 ProgressBar at=352,723,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                        62 TextBlock at=560,718,191,24 box=191,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 0 B · 410 GB free" layout=190.96,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 0 B · 410 GB free"
              63 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                64 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  65 TextBlock at=803,179,381,535 box=381,535 by=panel caps=15.3989 drawn face=Barlow heard="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com." layout=326.74,79.2 limit=0,0,1280,800 name=CoresOverview paints=#D4D4D4FF role=text size=22 text="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com."
            66 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              67 HintBar at=96,757,1088,29 by=panel heard="Back, Console, List, Apply…, More" limit=0,0,1280,800 role=group
focus 12
edge 12 up same
edge 12 down 31
edge 12 left 23
edge 12 right same
edge 17 up same
edge 17 down 31
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 31
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 31
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 31
edge 23 left 21
edge 23 right 12
edge 31 up 17
edge 31 down 37
edge 31 left same
edge 31 right 12
edge 37 up 31
edge 37 down 43
edge 37 left same
edge 37 right 12
edge 43 up 37
edge 43 down 49
edge 43 left same
edge 43 right 12
edge 49 up 43
edge 49 down same
edge 49 left same
edge 49 right 12
press pageup focus="TextBox Search cores (Search)" top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in)" top=CoresWindow
