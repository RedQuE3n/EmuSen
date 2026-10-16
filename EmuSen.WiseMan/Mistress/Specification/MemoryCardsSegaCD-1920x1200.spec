# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state MemoryCardsSegaCD-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Memory Cards"
case MemoryCardsSegaCD
entered shoulders
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
        9 TabItem at=410,316,96,72 by=clip drawn focus heard=N64 limit=410,316,1101,540 paints=#86868AFF role=tabitem sideways text=N64
          10 TextBlock at=428,325,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=410,316,96,72 owner=9 paints=#86868AFF role=text size=30 text=N64
        11 TabItem at=506,316,165,72 by=clip drawn focus heard=GENESIS limit=410,316,1101,540 paints=#86868AFF role=tabitem sideways text=Genesis
          12 TextBlock at=524,325,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=506,316,165,72 owner=11 paints=#86868AFF role=text size=30 text=GENESIS
        13 TabItem at=671,316,163.5,72 by=clip drawn focus heard="SEGA CD" limit=410,316,1101,540 op paints=#EEEEF0FF role=tabitem sideways text="Sega CD"
          14 TextBlock at=689,325,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=671,316,163.5,72 owner=13 paints=#EEEEF0FF role=text size=30 text="SEGA CD"
        15 TabItem at=834.5,316,234,72 by=clip drawn focus heard=PLAYSTATION limit=410,316,1101,540 paints=#86868AFF role=tabitem sideways text=PlayStation
          16 TextBlock at=852.5,325,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=834.5,316,234,72 owner=15 paints=#86868AFF role=text size=30 text=PLAYSTATION
        17 ScrollViewer at=428,391,1065,465 by=clip fades=b limit=410,391,1101,465 overflow=1545 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=428,391,1065,465
          18 StackPanel at=428,403,1065,1998 by=scroll in=17 limit=428,391,1065,465 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF,#B4B4B8FF,#D2D2D6FF,#FFFFFFFF role=none seen=428,403,1065,453
            19 StackPanel at=428,403,1065,985.5 by=scroll in=17 limit=428,391,1065,465 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF,#D2D2D6FF role=none seen=428,403,1065,453
              20 SectionHeader at=428,403,1065,51 box=710,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="Internal Backup RAM" in=17 layout=216.1,33.6 limit=428,391,1065,465 paints=#B4B4B8FF role=text size=28 text="Internal Backup RAM"
              21 FieldRow at=428,466,1065,199.5 by=scroll heard=Card in=17 limit=428,391,1065,465 paints=#9C9CA0FF role=group
                22 TextBlock at=428,466,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Card in=17 layout=37.68,28.8 limit=428,466,1065,199.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Card
                23 HintText at=428,515.5,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A card in the drawer, or a game's own." in=17 layout=256.56,24 limit=428,466,1065,199.5 paints=#86868AFF part role=text size=20 text="A card in the drawer, or a game's own."
                24 StackPanel at=428,557.5,1065,108 by=clip heard=Card in=17 limit=428,466,1065,199.5 role=none
                  25 Dropdown at=428,557.5,390,66 by=clip drawn focus heard=Card in=17 limit=428,466,1065,199.5 name="Sega CD.mcd.backup.Card" op paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=combobox sideways
                    26 TextBlock at=446,565,324,48 box=216,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Backup RAM 1" in=17 layout=118.66,31.2 limit=446,565,324,48 owner=25 paints=#EEEEF0FF role=text size=26 text="Backup RAM 1"
                    27 ViewboxContainer at=785,585.6,18,9.81 by=clip in=17 limit=785,585.6,18,9.81 nohit role=none
                  28 HintText at=428,629.5,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Shared: Backup RAM 1 in Inside the console. 0 save(s), 125 of 125 blocks free." in=17 layout=514.16,24 limit=428,466,1065,199.5 name="Sega CD.mcd.backup.About" paints=#86868AFF role=text size=20 text="Shared: Backup RAM 1 in Inside the console. 0 save(s), 125 of 125 blocks free."
              29 FieldRow at=428,677.5,1065,127.5 by=scroll heard=Saves in=17 limit=428,391,1065,465 paints=#9C9CA0FF role=group
                30 TextBlock at=428,677.5,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Saves in=17 layout=47.38,28.8 limit=428,677.5,1065,127.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Saves
                31 HintText at=428,727,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Choose a save to copy, move or delete it." in=17 layout=276.04,24 limit=428,677.5,1065,127.5 paints=#86868AFF part role=text size=20 text="Choose a save to copy, move or delete it."
                32 StackPanel at=428,769,1065,36 by=clip heard=Saves in=17 limit=428,677.5,1065,127.5 name="Sega CD.mcd.backup.Saves" role=none
                  33 HintText at=428,769,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="No saves." in=17 layout=64.6,24 limit=428,677.5,1065,127.5 paints=#86868AFF role=text size=20 text="No saves."
              34 FieldRow at=428,817,1065,193.5 by=scroll heard="Copy or move to" in=17 limit=428,391,1065,465 paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#FFFFFFFF role=group seen=428,817,1065,39
                35 TextBlock at=428,817,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Copy or move to" in=17 layout=131.02,28.8 limit=428,817,1065,193.5 name=PART_Label paints=#9C9CA0FF part role=text seen=428,817,1065,39 size=24 text="Copy or move to"
focus 25
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge 6 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1/0/2/0/0
edge 6 left 13
edge 6 right same
edge 13 up same
edge 13 down 25
edge 25 up 13
edge 25 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0 up 25
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
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 right 6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/3/0/2/0/0
press pageup focus="Dropdown Genesis.mcd.backup.Card" tab=Genesis title="Memory Cards" top=MemoryCardsWindow
press pagedown focus="Dropdown Sega CD.mcd.backup.Card" tab="Sega CD" title="Memory Cards" top=MemoryCardsWindow
press first focus="Dropdown Sega CD.mcd.backup.Card" tab="Sega CD" title="Memory Cards" top=MemoryCardsWindow
press last focus="Dropdown Sega CD.mcd.backup.Card" tab="Sega CD" title="Memory Cards" top=MemoryCardsWindow
press back top=none
