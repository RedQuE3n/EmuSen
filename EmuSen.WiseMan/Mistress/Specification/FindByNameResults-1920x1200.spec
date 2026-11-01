# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FindByNameResults-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,148.5,1219.05,864
title "Find by Name"
case FindByNameResults
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,298,1221,684 by=panel limit=350.48,298,1219.05,684 paints=#D4D4D4FF role=none seen=350.48,298,1219.05,684
    1 StackPanel at=410,340,1101,618 by=panel limit=350.48,298,1219.05,684 role=none
      2 StackPanel at=410,340,1101,312 by=panel limit=350.48,298,1219.05,684 role=none
        3 TextBlock at=410,340,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Aurora Drift" layout=74.68,24 limit=350.48,298,1219.05,684 name=FindByNameTitle paints=#D4D4D4FF role=text size=20 text="Aurora Drift"
        4 HintText at=410,385,1101,144 box=734,96 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed." layout=722.1,96 limit=350.48,298,1219.05,684 name=FindByNameHint paints=#8D8D91FF role=text size=20 text="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed."
        5 Grid at=410,538,1101,69 by=panel limit=350.48,298,1219.05,684 role=none
          6 TextBox at=410,538,940.5,69 by=panel drawn focus heard="The game's name" limit=350.48,298,1219.05,684 name=FindByNameText op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Super Mario"
            7 ScrollViewer allow="a text box scrolls its own text" at=426.5,548.5,907.5,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
          8 Button at=1362.5,539.5,148.5,66 by=panel drawn focus heard=SEARCH limit=350.48,298,1219.05,684 name=FindByNameSearch op paints=#707070FF,#D4D4D4FF role=button text=Search
            9 TextBlock at=1388,548.5,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SEARCH layout=64.9,31.2 limit=1362.5,539.5,148.5,66 owner=8 paints=#D4D4D4FF role=text size=26 text=SEARCH
        10 HintText at=410,616,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="2 games, most likely first. Choose the one this file is." layout=339.78,24 limit=350.48,298,1219.05,684 name=FindByNameStatus paints=#8D8D91FF role=text size=20 text="2 games, most likely first. Choose the one this file is."
      11 ScrollViewer at=410,670,1101,204 by=panel limit=350.48,298,1219.05,684 name=FindByNameScroll overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=410,670,1101,204
        12 StackPanel at=410,670,1101,204 by=fixed-scroll in=11 limit=410,670,1101,204 role=none
          13 Button at=410,670,1101,99 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=11 limit=410,670,1101,204 name=FindByNameResult_0 op paints=#707070FF,#D4D4D4FF role=button
            14 StackPanel at=435.5,677.5,343.5,84 by=clip in=11 limit=410,670,1101,99 role=none
              15 TextBlock at=435.5,677.5,343.5,48 box=229,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Super Mario World (US title)" in=11 layout=228.25,31.2 limit=410,670,1101,99 owner=13 paints=#EEEEF0FF role=text size=26 text="Super Mario World (US title)"
              16 HintText at=435.5,725.5,343.5,36 box=229,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="1991 · Synthetic Publisher" in=11 layout=160.46,24 limit=410,670,1101,99 owner=13 paints=#8D8D91FF role=text size=20 text="1991 · Synthetic Publisher"
          17 Button at=410,775,1101,99 by=fixed-scroll drawn focus heard=Avalonia.Controls.StackPanel in=11 limit=410,670,1101,204 name=FindByNameResult_1 op paints=#707070FF,#D4D4D4FF role=button
            18 StackPanel at=435.5,782.5,1042.5,84 by=clip in=11 limit=410,775,1101,99 role=none
              19 TextBlock at=435.5,782.5,1042.5,48 box=695,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Super Mario Kart: A Name Long Enough To Need The Whole Row And More (US title)" in=11 layout=694.25,31.2 limit=410,775,1101,99 owner=17 paints=#D4D4D4FF role=text size=26 text="Super Mario Kart: A Name Long Enough To Need The Whole Row And More (US title)"
              20 HintText at=435.5,830.5,1042.5,36 box=695,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="1991 · Synthetic Publisher" in=11 layout=160.46,24 limit=410,775,1101,99 owner=17 paints=#8D8D91FF role=text size=20 text="1991 · Synthetic Publisher"
      21 ButtonBar at=1362.5,892,148.5,66 by=panel limit=350.48,298,1219.05,684 paints=#D4D4D4FF role=toolbar
        22 Button at=1362.5,892,148.5,66 by=clip drawn focus heard=CANCEL limit=1362.5,892,148.5,66 name=FindByNameCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          23 TextBlock at=1388,901,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1362.5,892,148.5,66 owner=22 paints=#D4D4D4FF role=text size=26 text=CANCEL
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
edge 13 right 8
edge 17 up 13
edge 17 down 22
edge 17 left same
edge 17 right 22
edge 22 up 17
edge 22 down same
edge 22 left 6
edge 22 right same
press pageup focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press pagedown focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press first focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press last focus="Button FindByNameResult_0" title="Find by Name" top=FindByNameWindow
press back top=none
