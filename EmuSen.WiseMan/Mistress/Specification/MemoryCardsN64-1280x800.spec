# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state MemoryCardsN64-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Memory Cards"
case MemoryCardsN64
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 DockPanel at=274,152,733,504 by=panel limit=234,124,812.35,548 role=none
      2 HintText at=274,152,733,48 box=733,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes." layout=715.24,48 limit=234,124,812.35,548 paints=#86868AFF role=text size=20 text="Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes."
      3 StackPanel at=274,580,733,76 by=panel limit=234,124,812.35,548 role=none
        4 HintText at=274,580,733,24 by=panel drawn limit=234,124,812.35,548 name=MemoryCardsStatus paints=#86868AFF role=text
        5 ButtonBar at=917,612,90,44 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=toolbar
          6 Button at=917,612,90,44 by=clip drawn focus heard=CLOSE limit=917,612,90,44 name=MemoryCardsClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            7 TextBlock at=934,618,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=917,612,90,44 owner=6 paints=#9C9CA0FF role=text size=26 text=CLOSE
      8 Tabs at=274,210,733,360 by=panel limit=234,124,812.35,548 name=MemoryCardTabs paints=#9C9CA0FF role=tab
        9 TabItem at=274,210,64,48 by=clip drawn focus heard=N64 limit=274,210,733,360 op paints=#EEEEF0FF role=tabitem sideways text=N64
          10 TextBlock at=286,216,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=274,210,64,48 owner=9 paints=#EEEEF0FF role=text size=30 text=N64
        11 TabItem at=338,210,110,48 by=clip drawn focus heard=GENESIS limit=274,210,733,360 paints=#86868AFF role=tabitem sideways text=Genesis
          12 TextBlock at=350,216,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=338,210,110,48 owner=11 paints=#86868AFF role=text size=30 text=GENESIS
        13 TabItem at=448,210,109,48 by=clip drawn focus heard="SEGA CD" limit=274,210,733,360 paints=#86868AFF role=tabitem sideways text="Sega CD"
          14 TextBlock at=460,216,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=448,210,109,48 owner=13 paints=#86868AFF role=text size=30 text="SEGA CD"
        15 TabItem at=557,210,156,48 by=clip drawn focus heard=PLAYSTATION limit=274,210,733,360 paints=#86868AFF role=tabitem sideways text=PlayStation
          16 TextBlock at=569,216,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=557,210,156,48 owner=15 paints=#86868AFF role=text size=30 text=PLAYSTATION
        17 ScrollViewer at=286,260,709,310 by=clip fades=b limit=274,260,733,310 overflow=447 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=286,260,709,310
          18 StackPanel at=286,268,709,749 by=scroll in=17 limit=286,260,709,310 role=none seen=286,268,709,302
            19 StackPanel at=286,268,709,749 by=scroll in=17 limit=286,260,709,310 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF,#D2D2D6FF role=none seen=286,268,709,302
              20 SectionHeader at=286,268,709,34 box=709,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="Controller Pak" in=17 layout=164.05,33.6 limit=286,260,709,310 paints=#B4B4B8FF role=text size=28 text="Controller Pak"
              21 FieldRow at=286,310,709,157 by=scroll heard=Card in=17 limit=286,260,709,310 paints=#9C9CA0FF role=group
                22 TextBlock at=286,310,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Card in=17 layout=37.68,28.8 limit=286,310,709,157 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Card
                23 HintText at=286,343,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A card in the drawer, or a game's own." in=17 layout=256.56,24 limit=286,310,709,157 paints=#86868AFF part role=text size=20 text="A card in the drawer, or a game's own."
                24 StackPanel at=286,371,709,96 by=clip heard=Card in=17 limit=286,310,709,157 role=none
                  25 Dropdown at=286,371,260,44 by=clip drawn focus heard=Card in=17 limit=286,310,709,157 name=N64.n64.pak.Card op paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=combobox sideways
                    26 TextBlock at=298,376,216,32 box=216,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Pak 1" in=17 layout=44.62,31.2 limit=298,376,216,32 owner=25 paints=#EEEEF0FF role=text size=26 text="Pak 1"
                    27 ViewboxContainer at=524,389.73,12,6.54 by=clip in=17 limit=524,389.73,12,6.54 nohit role=none
                  28 HintText at=286,419,709,48 box=709,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Shared: Pak 1 in Controller 1, Pak 2 in Controller 2, Pak 3 in Controller 3, Pak 4 in Controller 4. 2 save(s), 118 of 123 pages free." in=17 layout=688.12,48 limit=286,310,709,157 name=N64.n64.pak.About paints=#86868AFF role=text size=20 text="Shared: Pak 1 in Controller 1, Pak 2 in Controller 2, Pak 3 in Controller 3, Pak 4 in Controller 4. 2 save(s), 118 of 123 pages free."
              29 FieldRow at=286,475,709,153 by=scroll heard=Saves in=17 limit=286,260,709,310 paints=#9C9CA0FF role=group seen=286,475,709,95
                30 TextBlock at=286,475,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Saves in=17 layout=47.38,28.8 limit=286,475,709,153 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Saves
                31 HintText at=286,508,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Choose a save to copy, move or delete it." in=17 layout=276.04,24 limit=286,475,709,153 paints=#86868AFF part role=text size=20 text="Choose a save to copy, move or delete it."
                32 StackPanel at=286,536,709,92 by=clip heard=Saves in=17 limit=286,475,709,153 name=N64.n64.pak.Saves paints=#55555AFF,#9C9CA0FF role=none seen=286,536,709,34
                  33 Button at=286,536,709,44 by=clip drawn focus heard="MARIO KART  ·  2 PAGE(S)" in=17 limit=286,475,709,153 name=N64.n64.pak.Save.0 op paints=#55555AFF,#9C9CA0FF role=button seen=286,536,709,34 text="MARIO KART  ·  2 page(s)"
                    34 TextBlock at=303,542,212,32 box=212,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARIO KART  ·  2 PAGE(S)" in=17 layout=211.82,31.2 limit=286,536,709,44 owner=33 paints=#9C9CA0FF role=text seen=303,542,212,28 size=26 text="MARIO KART  ·  2 PAGE(S)"
focus 25
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge 6 down same
edge 6 left 9
edge 6 right same
edge 9 up same
edge 9 down 25
edge 25 up 9
edge 25 down 33
edge 33 up 25
edge 33 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1
edge 33 left same
edge 33 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 up 33
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1 right 6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 down 6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 right 6
press pageup focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press pagedown focus="Dropdown N64.n64.pak.Card" tab=N64 title="Memory Cards" top=MemoryCardsWindow
press first focus="Dropdown N64.n64.pak.Card" tab=N64 title="Memory Cards" top=MemoryCardsWindow
press last focus="Dropdown N64.n64.pak.Card" tab=N64 title="Memory Cards" top=MemoryCardsWindow
press back top=none
