# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state MemoryCardsPlayStationHeld-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Memory Cards"
case MemoryCardsPlayStationHeld
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 HintText at=410,229,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes." layout=715.24,48 limit=350.48,187,1219.05,822 paints=#86868AFF role=text size=20 text="Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes."
      3 StackPanel at=410,871,1101,114 by=panel limit=350.48,187,1219.05,822 role=none
        4 HintText at=410,871,1101,36 by=panel drawn limit=350.48,187,1219.05,822 name=MemoryCardsStatus paints=#86868AFF role=text
        5 ButtonBar at=1376,919,135,66 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=toolbar
          6 Button at=1376,919,135,66 by=clip drawn focus heard=CLOSE limit=1376,919,135,66 name=MemoryCardsClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            7 TextBlock at=1401.5,928,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1376,919,135,66 owner=6 paints=#9C9CA0FF role=text size=26 text=CLOSE
      8 Tabs at=410,316,1101,540 by=panel limit=350.48,187,1219.05,822 name=MemoryCardTabs paints=#9C9CA0FF role=tab
        9 TabItem at=410,316,234,72 by=clip drawn focus heard=PLAYSTATION limit=410,316,1101,540 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
          10 TextBlock at=428,325,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=410,316,234,72 owner=9 paints=#EEEEF0FF role=text size=30 text=PLAYSTATION
        11 ScrollViewer at=428,391,1065,465 by=clip fades=b limit=410,391,1101,465 overflow=670.5 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=428,391,1065,465
          12 StackPanel at=428,403,1065,1123.5 by=scroll in=11 limit=428,391,1065,465 role=none seen=428,403,1065,453
            13 StackPanel at=428,403,1065,1123.5 by=scroll in=11 limit=428,391,1065,465 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF,#D2D2D6FF role=none seen=428,403,1065,453
              14 SectionHeader at=428,403,1065,51 box=710,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="Memory Card" in=11 layout=132.78,33.6 limit=428,391,1065,465 paints=#B4B4B8FF role=text size=28 text="Memory Card"
              15 FieldRow at=428,466,1065,235.5 by=scroll heard=Card in=11 limit=428,391,1065,465 paints=#9C9CA0FF role=group
                16 TextBlock at=428,466,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Card in=11 layout=37.68,28.8 limit=428,466,1065,235.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Card
                17 HintText at=428,515.5,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A card in the drawer, or a game's own." in=11 layout=256.56,24 limit=428,466,1065,235.5 paints=#86868AFF part role=text size=20 text="A card in the drawer, or a game's own."
                18 StackPanel at=428,557.5,1065,144 by=clip heard=Card in=11 limit=428,466,1065,235.5 role=none
                  19 Dropdown at=428,557.5,390,66 by=clip drawn focus heard=Card in=11 limit=428,466,1065,235.5 name=PlayStation.psx.card.Card op paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=combobox sideways
                    20 TextBlock at=446,565,324,48 box=216,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Card 1" in=11 layout=52.68,31.2 limit=446,565,324,48 owner=19 paints=#EEEEF0FF role=text size=26 text="Card 1"
                    21 ViewboxContainer at=785,585.6,18,9.81 by=clip in=11 limit=785,585.6,18,9.81 nohit role=none
                  22 HintText at=428,629.5,1065,72 box=710,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Shared: Card 1 in Slot 1, Card 2 in Slot 2. 2 save(s), 11 of 15 blocks free. In use by Amber Saga, which is running: its saves can be looked at, and changed when the game is closed." in=11 layout=676.32,48 limit=428,466,1065,235.5 name=PlayStation.psx.card.About paints=#86868AFF role=text size=20 text="Shared: Card 1 in Slot 1, Card 2 in Slot 2. 2 save(s), 11 of 15 blocks free. In use by Amber Saga, which is running: its saves can be looked at, and changed when the game is closed."
              23 FieldRow at=428,713.5,1065,229.5 by=scroll heard=Saves in=11 limit=428,391,1065,465 paints=#9C9CA0FF role=group seen=428,713.5,1065,142.5
                24 TextBlock at=428,713.5,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Saves in=11 layout=47.38,28.8 limit=428,713.5,1065,229.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Saves
                25 HintText at=428,763,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Choose a save to copy, move or delete it." in=11 layout=276.04,24 limit=428,713.5,1065,229.5 paints=#86868AFF part role=text size=20 text="Choose a save to copy, move or delete it."
                26 StackPanel at=428,805,1065,138 by=clip heard=Saves in=11 limit=428,713.5,1065,229.5 name=PlayStation.psx.card.Saves paints=#55555AFF,#9C9CA0FF role=none seen=428,805,1065,51
                  27 Button at=428,805,1065,66 by=clip drawn focus heard="DRACULA X CHRONICLE SAVE  ·  1 BLOCK(S)" in=11 limit=428,713.5,1065,229.5 name=PlayStation.psx.card.Save.0 op paints=#55555AFF,#9C9CA0FF role=button seen=428,805,1065,51 text="DRACULA X CHRONICLE SAVE  ·  1 block(s)"
                    28 TextBlock at=453.5,814,549,48 box=366,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DRACULA X CHRONICLE SAVE  ·  1 BLOCK(S)" in=11 layout=365.79,31.2 limit=428,805,1065,66 owner=27 paints=#9C9CA0FF role=text seen=453.5,814,549,42 size=26 text="DRACULA X CHRONICLE SAVE  ·  1 BLOCK(S)"
focus 19
edge 6 up 27
edge 6 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
edge 6 left 19
edge 6 right same
edge 9 up same
edge 9 down 19
edge 19 up 9
edge 19 down 27
edge 27 up 19
edge 27 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1
edge 27 left same
edge 27 right 6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 up 27
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/2/0/2/0/1 right 6
press pageup focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press pagedown focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press first focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press last focus="Dropdown PlayStation.psx.card.Card" tab=PlayStation title="Memory Cards" top=MemoryCardsWindow
press back top=none
