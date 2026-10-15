# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FindByNameResults-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,83.5,812.7,607
title "Find by Name"
case FindByNameResults
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,184,813,487 by=panel limit=234,184,812.35,487 paints=#9C9CA0FF role=none seen=234,184,812.35,487
    1 StackPanel at=274,212,733,443 by=panel limit=234,184,812.35,487 role=none
      2 StackPanel at=274,212,733,208 by=panel limit=234,184,812.35,487 role=none
        3 TextBlock at=274,212,733,24 box=733,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Aurora Drift" layout=78.86,24 limit=234,184,812.35,487 name=FindByNameTitle paints=#9C9CA0FF role=text size=20 text="Aurora Drift"
        4 HintText at=274,242,733,96 box=733,96 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed." layout=718.32,96 limit=234,184,812.35,487 name=FindByNameHint paints=#86868AFF role=text size=20 text="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed."
        5 Grid at=274,344,733,46 by=panel limit=234,184,812.35,487 role=none
          6 TextBox at=274,344,622,46 by=panel drawn focus heard="The game's name" limit=234,184,812.35,487 name=FindByNameText op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text="Super Mario"
            7 ScrollViewer allow="a text box scrolls its own text" at=285,351,600,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          8 Button at=904,345,103,44 by=panel drawn focus heard=SEARCH limit=234,184,812.35,487 name=FindByNameSearch op paints=#55555AFF,#9C9CA0FF role=button text=Search
            9 TextBlock at=921,351,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SEARCH layout=68.46,31.2 limit=904,345,103,44 owner=8 paints=#9C9CA0FF role=text size=26 text=SEARCH
        10 HintText at=274,396,733,24 box=733,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="2 games, most likely first. Choose the one this file is." layout=356.5,24 limit=234,184,812.35,487 name=FindByNameStatus paints=#86868AFF role=text size=20 text="2 games, most likely first. Choose the one this file is."
      11 ScrollViewer at=274,432,733,167 by=panel limit=234,184,812.35,487 name=FindByNameScroll overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=274,432,733,167
        12 StackPanel at=274,432,733,167 by=fixed-scroll in=11 limit=274,432,733,167 role=none
          13 Button at=274,432,733,66 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=11 limit=274,432,733,167 name=FindByNameResult_0 op paints=#55555AFF,#9C9CA0FF role=button
            14 StackPanel at=291,437,239,56 by=clip in=11 limit=274,432,733,66 role=none
              15 TextBlock at=291,437,239,32 box=239,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Super Mario World (US title)" in=11 layout=238.63,31.2 limit=274,432,733,66 owner=13 paints=#EEEEF0FF role=text size=26 text="Super Mario World (US title)"
              16 HintText at=291,469,239,24 box=239,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="1991 · Synthetic Publisher" in=11 layout=168.6,24 limit=274,432,733,66 owner=13 paints=#86868AFF role=text size=20 text="1991 · Synthetic Publisher"
          17 Button at=274,502,733,97 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=11 limit=274,432,733,167 name=FindByNameResult_1 op paints=#55555AFF,#9C9CA0FF role=button
            18 StackPanel at=291,507,682,87 by=clip in=11 limit=274,502,733,97 role=none
              19 TextBlock at=291,507,682,63 box=682,63 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Super Mario Kart: A Name Long Enough To Need The Whole Row And More (US title)" in=11 layout=676.49,62.4 limit=274,502,733,97 owner=17 paints=#9C9CA0FF role=text size=26 text="Super Mario Kart: A Name Long Enough To Need The Whole Row And More (US title)"
              20 HintText at=291,570,682,24 box=682,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="1991 · Synthetic Publisher" in=11 layout=168.6,24 limit=274,502,733,97 owner=17 paints=#86868AFF role=text size=20 text="1991 · Synthetic Publisher"
      21 ButtonBar at=904,611,103,44 by=panel limit=234,184,812.35,487 paints=#9C9CA0FF role=toolbar
        22 Button at=904,611,103,44 by=clip drawn focus heard=CANCEL limit=904,611,103,44 name=FindByNameCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          23 TextBlock at=921,617,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=904,611,103,44 owner=22 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 13
edge 6 up same
edge 6 down 13
edge 6 left same
edge 6 right 8
edge 8 up same
edge 8 down 13
edge 8 left 6
edge 8 right same
edge 13 up 6
edge 13 down 17
edge 13 left same
edge 13 right same
edge 17 up 13
edge 17 down 22
edge 17 left same
edge 17 right same
edge 22 up 17
edge 22 down same
edge 22 left 6
edge 22 right same
press pageup focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press pagedown focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press first focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press last focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press back top=none
