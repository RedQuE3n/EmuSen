# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Cores-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Cores
case Cores
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 CoresPane at=258,136,765,536 by=panel heard=Cores limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=115.18,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search cores"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,66,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=274,149,66,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=348,149,149,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=365,155,115,32 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=348,149,149,44 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=505,149,164,44 by=panel drawn focus heard="PlayStation 2" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=522,155,130,32 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=129.97,31.2 limit=505,149,164,44 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=677,149,80,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=694,155,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=677,149,80,44 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
          19 Border at=274,206,733,227 by=panel limit=234,124,812.35,548 name=AdaptiveContent role=none
            20 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
              21 LunaTable at=274,206,733,227 by=panel heard=Rows limit=234,124,812.35,548 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                22 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=286,212,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                23 GridSplitter at=320,210,6,32 by=clip focus heard="Resize Mark" limit=274,206,733,43 paints=#9C9CA0FF role=thumb
                24 Button at=326,210,335,32 by=clip drawn focus heard="Core, not sorted" limit=274,206,733,43 op paints=#55555AFF,#9C9CA0FF role=button
                  25 StackPanel at=326,210,41,32 by=clip limit=326,210,335,32 role=none
                    26 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=326,210,41,32 box=41,32 caps=18.1987 drawn face="Barlow Condensed" heard=Core layout=40.14,31.2 owner=24 paints=#9C9CA0FF role=text size=26 text=Core
                27 GridSplitter at=655,210,6,32 by=clip focus heard="Resize Core" limit=274,206,733,43 paints=#9C9CA0FF role=thumb
                28 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,212,334,29 box=334,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=42.48,28.8 paints=#9C9CA0FF role=text size=24 text=State
                29 ScrollViewer at=274,249,733,184 fades=tb name=PART_ScrollViewer overflow=152 paints=#55555AFF,#FFFFFFFF part role=scrollviewer viewport=274,249,733,184
                  30 ListBoxItem at=274,223,733,42 by=scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=274,249,733,184 op paints=#333337FF,#9C9CA0FF role=listitem seen=274,249,733,16
                    31 Grid at=284,228,713,31 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=274,223,733,42 role=none seen=284,249,713,10
                      32 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,228,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=30 paints=#9C9CA0FF role=text seen=284,249,40,10 size=24 text="☐"
                      33 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,229,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=72.29,28.8 owner=30 paints=#9C9CA0FF role=edit seen=324,249,337,9 size=24 text=Sapphire
                      34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,229,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=56.04,28.8 owner=30 paints=#9C9CA0FF role=edit seen=661,249,336,9 size=24 text="built in"
                  35 ListBoxItem at=274,265,733,42 by=scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=274,249,733,184 op paints=#333337FF,#9C9CA0FF role=listitem
                    36 Grid at=284,270,713,31 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=274,265,733,42 role=none
                      37 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,270,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=35 paints=#9C9CA0FF role=text size=24 text="☐"
                      38 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,271,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=71.9,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text=Diamond
                      39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,271,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=56.04,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text="built in"
                  40 ListBoxItem at=274,307,733,42 by=scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=274,249,733,184 op paints=#333337FF,#9C9CA0FF role=listitem
                    41 Grid at=284,312,713,31 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=274,307,733,42 role=none
                      42 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,312,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#9C9CA0FF role=text size=24 text="☐"
                      43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,313,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=69.5,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text=VenusRT
                      44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,313,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=56.04,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text="built in"
                  45 ListBoxItem at=274,349,733,42 by=scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=274,249,733,184 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 Grid at=284,354,713,31 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=274,349,733,42 role=none
                      47 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,354,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=45 paints=#9C9CA0FF role=text size=24 text="☐"
                      48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,355,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=119.5,28.8 owner=45 paints=#9C9CA0FF role=edit size=24 text="Extra PS2 Core"
                      49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,355,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=162.96,28.8 owner=45 paints=#9C9CA0FF role=edit size=24 text="waiting for approval"
                  50 ListBoxItem at=274,391,733,42 by=scroll drawn focus heard="Mark: not marked, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=29 limit=274,249,733,184 op paints=#333337FF,#9C9CA0FF role=listitem
                    51 Grid at=284,396,713,31 by=clip heard="Mark: not marked, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=29 limit=274,391,733,42 role=none
                      52 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,396,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=50 paints=#EEEEF0FF role=text size=24 text="☐"
                      53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,397,337,29 box=337,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Core in=29 layout=333.14,28.8 owner=50 paints=#EEEEF0FF role=edit size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                      54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,397,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=72.14,28.8 owner=50 paints=#EEEEF0FF role=edit size=24 text=available
        55 Border at=274,441,733,96 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          56 WrapPanel at=274,441,733,96 by=panel limit=234,124,812.35,548 role=none
            57 ActionButton at=274,441,105,44 by=panel drawn focus heard=INSTALL limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Install
              58 TextBlock at=291,447,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=70.23,31.2 limit=274,441,105,44 owner=57 paints=#9C9CA0FF role=text size=26 text=INSTALL
            59 ActionButton at=391,441,101,44 by=panel drawn focus heard=UPDATE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Update
              60 TextBlock at=408,447,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=391,441,101,44 owner=59 paints=#9C9CA0FF role=text size=26 text=UPDATE
            61 ActionButton at=504,441,105,44 by=panel drawn focus heard=REMOVE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Remove
              62 TextBlock at=521,447,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=504,441,105,44 owner=61 paints=#9C9CA0FF role=text size=26 text=REMOVE
            63 ActionButton at=621,441,130,44 by=panel drawn heard="APPROVE…" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Approve…"
              64 TextBlock at=638,447,96,32 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=95.76,31.2 limit=621,441,130,44 off owner=63 paints=#55555AFF role=text size=26 text="APPROVE…"
            65 ActionButton at=763,441,162,44 by=panel drawn heard="MAKE DEFAULT" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Make Default"
              66 TextBlock at=780,447,128,32 box=128,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=127.82,31.2 limit=763,441,162,44 off owner=65 paints=#55555AFF role=text size=26 text="MAKE DEFAULT"
            67 ActionButton at=274,493,126,44 by=panel drawn heard="OPTIONS…" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Options…"
              68 TextBlock at=291,499,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=91.68,31.2 limit=274,493,126,44 off owner=67 paints=#55555AFF role=text size=26 text="OPTIONS…"
            69 ActionButton at=412,493,96,44 by=panel drawn focus heard=VERIFY limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Verify
              70 TextBlock at=429,499,62,32 box=62,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=61.2,31.2 limit=412,493,96,44 owner=69 paints=#9C9CA0FF role=text size=26 text=VERIFY
            71 ActionButton at=520,493,214,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text="Check for Updates"
              72 TextBlock at=537,499,180,32 box=180,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=179.58,31.2 limit=520,493,214,44 owner=71 paints=#9C9CA0FF role=text size=26 text="CHECK FOR UPDATES"
            73 ActionButton at=746,493,82,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              74 TextBlock at=763,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=746,493,82,44 owner=73 paints=#9C9CA0FF role=text size=26 text=MARK
        75 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          76 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            77 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              78 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
            79 StackPanel at=491,586,528,44 by=panel limit=234,124,812.35,548 role=none
              80 Button at=491,586,105,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                81 TextBlock at=508,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=491,586,105,44 off owner=80 paints=#55555AFF role=text size=26 text="APPLY…"
              82 StackPanel at=608,586,411,44 by=panel limit=234,124,812.35,548 role=none
                83 ProgressBar at=608,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                84 TextBlock at=816,594,203,29 box=203,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=202.22,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="/Cores 11 MB · 410 GB free"
        85 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=234,124,812.35,548 role=group
focus 23
edge 5 up same
edge 5 down 30
edge 5 left 16
edge 5 right same
edge 10 up same
edge 10 down 23
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 24
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 24
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 30
edge 16 left 14
edge 16 right 5
edge 23 up 10
edge 23 down 30
edge 23 left 30
edge 23 right 24
edge 24 up 12
edge 24 down 30
edge 24 left 23
edge 24 right 30
edge 30 up 23
edge 30 down 35
edge 30 left same
edge 30 right same
edge 35 up 30
edge 35 down 40
edge 35 left same
edge 35 right same
edge 40 up 35
edge 40 down 45
edge 40 left same
edge 40 right same
edge 45 up 40
edge 45 down 50
edge 45 left same
edge 45 right same
edge 50 up 45
edge 50 down "row:ListBox/5/Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑"
edge 50 left same
edge 50 right same
edge 57 up "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported"
edge 57 down 69
edge 57 left same
edge 57 right 59
edge 59 up "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported"
edge 59 down 69
edge 59 left 57
edge 59 right 61
edge 61 up "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported"
edge 61 down 71
edge 61 left 59
edge 61 right 73
edge 69 up 59
edge 69 down same
edge 69 left 57
edge 69 right 71
edge 71 up 61
edge 71 down same
edge 71 left 69
edge 71 right 73
edge 73 up "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported"
edge 73 down same
edge 73 left 71
edge 73 right 5
edge "row:ListBox/5/Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" up 50
edge "row:ListBox/5/Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" down "row:ListBox/6/Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed"
edge "row:ListBox/5/Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" left same
edge "row:ListBox/5/Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" right same
edge "row:ListBox/6/Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" up "row:ListBox/5/Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑"
edge "row:ListBox/6/Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" down "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported"
edge "row:ListBox/6/Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" left same
edge "row:ListBox/6/Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" right same
edge "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported" up "row:ListBox/6/Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed"
edge "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported" down 57
edge "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported" left same
edge "row:ListBox/7/Mark: not marked, Core: Only GL Core, State: not supported" right same
press pageup focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
press pagedown focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
press first focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
press last focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
press back top=none
