# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state MemoryCardsPlayStationHeld-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Memory Cards"
case MemoryCardsPlayStationHeld
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
        9 TabItem at=274,210,156,48 by=clip drawn focus heard=PLAYSTATION limit=274,210,733,360 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
          10 TextBlock at=286,216,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=274,210,156,48 owner=9 paints=#EEEEF0FF role=text size=30 text=PLAYSTATION
        11 ScrollViewer at=286,260,709,310 by=clip fades=b limit=274,260,733,310 overflow=447 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=286,260,709,310
          12 StackPanel at=286,268,709,749 by=scroll in=11 limit=286,260,709,310 role=none seen=286,268,709,302
            13 StackPanel at=286,268,709,749 by=scroll in=11 limit=286,260,709,310 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF,#D2D2D6FF role=none seen=286,268,709,302
              14 SectionHeader at=286,268,709,34 box=709,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="Memory Card" in=11 layout=132.78,33.6 limit=286,260,709,310 paints=#B4B4B8FF role=text size=28 text="Memory Card"
              15 FieldRow at=286,310,709,157 by=scroll heard=Card in=11 limit=286,260,709,310 paints=#9C9CA0FF role=group
                16 TextBlock at=286,310,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Card in=11 layout=37.68,28.8 limit=286,310,709,157 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Card
                17 HintText at=286,343,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A card in the drawer, or a game's own." in=11 layout=256.56,24 limit=286,310,709,157 paints=#86868AFF part role=text size=20 text="A card in the drawer, or a game's own."
                18 StackPanel at=286,371,709,96 by=clip heard=Card in=11 limit=286,310,709,157 role=none
                  19 Dropdown at=286,371,260,44 by=clip drawn focus heard=Card in=11 limit=286,310,709,157 name=PlayStation.psx.card.Card op paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=combobox sideways
                    20 TextBlock at=298,376,216,32 box=216,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Card 1" in=11 layout=52.68,31.2 limit=298,376,216,32 owner=19 paints=#EEEEF0FF role=text size=26 text="Card 1"
                    21 ViewboxContainer at=524,389.73,12,6.54 by=clip in=11 limit=524,389.73,12,6.54 nohit role=none
                  22 HintText at=286,419,709,48 box=709,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Shared: Card 1 in Slot 1, Card 2 in Slot 2. 2 save(s), 11 of 15 blocks free. In use by Amber Saga, which is running: its saves can be looked at, and changed when the game is closed." in=11 layout=676.32,48 limit=286,310,709,157 name=PlayStation.psx.card.About paints=#86868AFF role=text size=20 text="Shared: Card 1 in Slot 1, Card 2 in Slot 2. 2 save(s), 11 of 15 blocks free. In use by Amber Saga, which is running: its saves can be looked at, and changed when the game is closed."
              23 FieldRow at=286,475,709,153 by=scroll heard=Saves in=11 limit=286,260,709,310 paints=#9C9CA0FF role=group seen=286,475,709,95
                24 TextBlock at=286,475,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Saves in=11 layout=47.38,28.8 limit=286,475,709,153 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Saves
                25 HintText at=286,508,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Choose a save to copy, move or delete it." in=11 layout=276.04,24 limit=286,475,709,153 paints=#86868AFF part role=text size=20 text="Choose a save to copy, move or delete it."
                26 StackPanel at=286,536,709,92 by=clip heard=Saves in=11 limit=286,475,709,153 name=PlayStation.psx.card.Saves paints=#55555AFF,#9C9CA0FF role=none seen=286,536,709,34
                  27 Button at=286,536,709,44 by=clip drawn focus heard="DRACULA X CHRONICLE SAVE  ·  1 BLOCK(S)" in=11 limit=286,475,709,153 name=PlayStation.psx.card.Save.0 op paints=#55555AFF,#9C9CA0FF role=button seen=286,536,709,34 text="DRACULA X CHRONICLE SAVE  ·  1 block(s)"
                    28 TextBlock at=303,542,366,32 box=366,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DRACULA X CHRONICLE SAVE  ·  1 BLOCK(S)" in=11 layout=365.79,31.2 limit=286,536,709,44 owner=27 paints=#9C9CA0FF role=text seen=303,542,366,28 size=26 text="DRACULA X CHRONICLE SAVE  ·  1 BLOCK(S)"
focus 19
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1
edge 6 down same
edge 6 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
edge 6 right same
edge 9 up same
edge 9 down 19
edge 19 up 9
edge 19 down 27
edge 27 up 19
edge 27 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1
edge 27 left same
edge 27 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 up 27
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 down 6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/0 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
press pageup focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press pagedown focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press first focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press last focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press back top=none
