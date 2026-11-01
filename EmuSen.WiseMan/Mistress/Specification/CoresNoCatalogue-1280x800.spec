# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresNoCatalogue-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,40,812.7,652
title Cores
case CoresNoCatalogue
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,140,813,532 by=panel limit=234,140,812.35,532 paints=#9C9CA0FF role=none seen=234,140,812.35,532
    1 CoresPane at=258,152,765,520 by=panel heard=Cores limit=234,140,812.35,532 role=pane
      2 Grid at=258,152,765,520 by=panel limit=234,140,812.35,532 role=none
        3 DockPanel at=274,164,733,46 by=panel limit=234,140,812.35,532 role=none
          4 Border at=827,164,180,46 by=panel limit=234,140,812.35,532 name=AdaptiveSearch role=none
            5 TextBox at=827,164,180,46 by=panel drawn focus heard=Search limit=234,140,812.35,532 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,171,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,171,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=115.18,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search cores"
          8 Border at=274,164,553,46 by=panel limit=234,140,812.35,532 name=AdaptiveNavigation role=none
            9 WholeRow at=274,164,553,46 by=panel limit=234,140,812.35,532 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,165,66,44 by=panel drawn focus heard=All limit=234,140,812.35,532 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,171,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=274,165,66,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=348,165,149,44 by=panel drawn focus heard=PlayStation limit=234,140,812.35,532 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=365,171,115,32 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=348,165,149,44 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=505,165,164,44 by=panel drawn focus heard="PlayStation 2" limit=234,140,812.35,532 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=522,171,130,32 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=129.97,31.2 limit=505,165,164,44 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=677,165,80,44 by=panel drawn focus heard=SNES limit=234,140,812.35,532 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=694,171,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=677,165,80,44 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=274,222,733,211 by=panel limit=234,140,812.35,532 role=none
          19 Border at=274,222,733,211 by=panel limit=234,140,812.35,532 name=AdaptiveContent role=none
            20 Grid at=274,222,733,211 by=panel limit=234,140,812.35,532 role=none
              21 LunaTable at=274,222,733,211 by=panel heard=Rows limit=234,140,812.35,532 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                22 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=286,228,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                23 GridSplitter at=320,226,6,32 by=clip focus heard="Resize Mark" limit=274,222,733,43 paints=#9C9CA0FF role=thumb
                24 Button at=326,226,335,32 by=clip drawn focus heard="Core, not sorted" limit=274,222,733,43 op paints=#55555AFF,#9C9CA0FF role=button
                  25 StackPanel at=326,226,41,32 by=clip limit=326,226,335,32 role=none
                    26 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=326,226,41,32 box=41,32 caps=18.1987 drawn face="Barlow Condensed" heard=Core layout=40.14,31.2 owner=24 paints=#9C9CA0FF role=text size=26 text=Core
                27 GridSplitter at=655,226,6,32 by=clip focus heard="Resize Core" limit=274,222,733,43 paints=#9C9CA0FF role=thumb
                28 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,228,334,29 box=334,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=42.48,28.8 paints=#9C9CA0FF role=text size=24 text=State
                29 ScrollViewer at=274,265,733,168 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=274,265,733,168
                  30 ListBoxItem at=274,265,733,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=274,265,733,168 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 Grid at=284,270,713,31 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=274,265,733,42 role=none
                      32 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,270,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=30 paints=#9C9CA0FF role=text size=24 text="☐"
                      33 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,271,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=72.29,28.8 owner=30 paints=#9C9CA0FF role=edit size=24 text=Sapphire
                      34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,271,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=56.04,28.8 owner=30 paints=#9C9CA0FF role=edit size=24 text="built in"
                  35 ListBoxItem at=274,307,733,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=274,265,733,168 op paints=#333337FF,#9C9CA0FF role=listitem
                    36 Grid at=284,312,713,31 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=274,307,733,42 role=none
                      37 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,312,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=35 paints=#9C9CA0FF role=text size=24 text="☐"
                      38 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,313,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=71.9,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text=Diamond
                      39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,313,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=56.04,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text="built in"
                  40 ListBoxItem at=274,349,733,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=274,265,733,168 op paints=#333337FF,#9C9CA0FF role=listitem
                    41 Grid at=284,354,713,31 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=274,349,733,42 role=none
                      42 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,354,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#9C9CA0FF role=text size=24 text="☐"
                      43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,355,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=69.5,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text=VenusRT
                      44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,355,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=56.04,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text="built in"
                  45 ListBoxItem at=274,391,733,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=274,265,733,168 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 Grid at=284,396,713,31 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=274,391,733,42 role=none
                      47 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=284,396,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=45 paints=#9C9CA0FF role=text size=24 text="☐"
                      48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=324,397,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=119.5,28.8 owner=45 paints=#9C9CA0FF role=edit size=24 text="Extra PS2 Core"
                      49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=661,397,336,29 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=162.96,28.8 owner=45 paints=#9C9CA0FF role=edit size=24 text="waiting for approval"
        50 Border at=274,441,733,96 by=panel limit=234,140,812.35,532 name=AdaptiveActions role=none
          51 WrapPanel at=274,441,733,96 by=panel limit=234,140,812.35,532 role=none
            52 ActionButton at=274,441,105,44 by=panel drawn focus heard=INSTALL limit=234,140,812.35,532 op paints=#55555AFF,#9C9CA0FF role=button text=Install
              53 TextBlock at=291,447,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=70.23,31.2 limit=274,441,105,44 owner=52 paints=#9C9CA0FF role=text size=26 text=INSTALL
            54 ActionButton at=391,441,101,44 by=panel drawn focus heard=UPDATE limit=234,140,812.35,532 op paints=#55555AFF,#9C9CA0FF role=button text=Update
              55 TextBlock at=408,447,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=391,441,101,44 owner=54 paints=#9C9CA0FF role=text size=26 text=UPDATE
            56 ActionButton at=504,441,105,44 by=panel drawn focus heard=REMOVE limit=234,140,812.35,532 op paints=#55555AFF,#9C9CA0FF role=button text=Remove
              57 TextBlock at=521,447,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=504,441,105,44 owner=56 paints=#9C9CA0FF role=text size=26 text=REMOVE
            58 ActionButton at=621,441,130,44 by=panel drawn heard="APPROVE…" limit=234,140,812.35,532 off paints=#55555AFF,#9C9CA0FF role=button text="Approve…"
              59 TextBlock at=638,447,96,32 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=95.76,31.2 limit=621,441,130,44 off owner=58 paints=#55555AFF role=text size=26 text="APPROVE…"
            60 ActionButton at=763,441,162,44 by=panel drawn heard="MAKE DEFAULT" limit=234,140,812.35,532 off paints=#55555AFF,#9C9CA0FF role=button text="Make Default"
              61 TextBlock at=780,447,128,32 box=128,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=127.82,31.2 limit=763,441,162,44 off owner=60 paints=#55555AFF role=text size=26 text="MAKE DEFAULT"
            62 ActionButton at=274,493,126,44 by=panel drawn heard="OPTIONS…" limit=234,140,812.35,532 off paints=#55555AFF,#9C9CA0FF role=button text="Options…"
              63 TextBlock at=291,499,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=91.68,31.2 limit=274,493,126,44 off owner=62 paints=#55555AFF role=text size=26 text="OPTIONS…"
            64 ActionButton at=412,493,96,44 by=panel drawn focus heard=VERIFY limit=234,140,812.35,532 op paints=#55555AFF,#9C9CA0FF role=button text=Verify
              65 TextBlock at=429,499,62,32 box=62,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=61.2,31.2 limit=412,493,96,44 owner=64 paints=#9C9CA0FF role=text size=26 text=VERIFY
            66 ActionButton at=520,493,214,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=234,140,812.35,532 op paints=#55555AFF,#9C9CA0FF role=button text="Check for Updates"
              67 TextBlock at=537,499,180,32 box=180,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=179.58,31.2 limit=520,493,214,44 owner=66 paints=#9C9CA0FF role=text size=26 text="CHECK FOR UPDATES"
            68 ActionButton at=746,493,82,44 by=panel drawn focus heard=MARK limit=234,140,812.35,532 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              69 TextBlock at=763,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=746,493,82,44 owner=68 paints=#9C9CA0FF role=text size=26 text=MARK
        70 Border at=258,545,765,89 by=panel limit=234,140,812.35,532 name=AdaptiveStatus role=none
          71 Grid at=262,549,757,81 by=panel limit=234,140,812.35,532 role=none
            72 Grid at=262,549,757,29 by=panel limit=234,140,812.35,532 role=none
              73 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com." layout=732.14,28.8 limit=234,140,812.35,532 paints=#9C9CA0FF role=text size=24 text="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com."
            74 StackPanel at=505,586,514,44 by=panel limit=234,140,812.35,532 role=none
              75 Button at=505,586,105,44 by=panel drawn heard="Apply the marked changes" limit=234,140,812.35,532 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                76 TextBlock at=522,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=505,586,105,44 off owner=75 paints=#55555AFF role=text size=26 text="APPLY…"
              77 StackPanel at=622,586,397,44 by=panel limit=234,140,812.35,532 role=none
                78 ProgressBar at=622,601,200,14 by=panel drawn heard=Space limit=234,140,812.35,532 paints=#333337FF,#4C9AE8FF role=progressbar
                79 TextBlock at=830,594,189,29 box=189,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 0 B · 410 GB free" layout=188.59,28.8 limit=234,140,812.35,532 paints=#9C9CA0FF role=text size=24 text="/Cores 0 B · 410 GB free"
        80 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=234,140,812.35,532 role=group
focus 5
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
edge 23 left same
edge 23 right 24
edge 24 up 12
edge 24 down 30
edge 24 left 23
edge 24 right 16
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
edge 45 down 52
edge 45 left same
edge 45 right same
edge 52 up 45
edge 52 down 62
edge 52 left same
edge 52 right 54
edge 54 up 45
edge 54 down 64
edge 54 left 52
edge 54 right 56
edge 56 up 45
edge 56 down 66
edge 56 left 54
edge 56 right 58
edge 58 up 45
edge 58 down 66
edge 58 left 56
edge 58 right 60
edge 62 up 52
edge 62 down same
edge 62 left same
edge 62 right 64
edge 64 up 54
edge 64 down same
edge 64 left 62
edge 64 right 66
edge 66 up 56
edge 66 down same
edge 66 left 64
edge 66 right 68
edge 68 up 60
edge 68 down same
edge 68 left 66
edge 68 right 5
press pageup focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press first focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press last focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
