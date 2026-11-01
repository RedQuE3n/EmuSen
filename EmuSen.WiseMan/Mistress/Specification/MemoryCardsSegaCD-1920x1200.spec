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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 HintText at=410,229,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes." layout=730.08,48 limit=350.48,187,1219.05,822 paints=#8D8D91FF role=text size=20 text="Each console's memory cards and the like: the cards in its drawer and each game's own, and the saves on them. A game that is running keeps its cards until it closes."
      3 StackPanel at=410,871,1101,114 by=panel limit=350.48,187,1219.05,822 role=none
        4 HintText at=410,871,1101,36 by=panel drawn limit=350.48,187,1219.05,822 name=MemoryCardsStatus paints=#8D8D91FF role=text
        5 ButtonBar at=1380.5,919,130.5,66 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=toolbar
          6 Button at=1380.5,919,130.5,66 by=clip drawn focus heard=CLOSE limit=1380.5,919,130.5,66 name=MemoryCardsClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            7 TextBlock at=1406,928,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1380.5,919,130.5,66 owner=6 paints=#D4D4D4FF role=text size=26 text=CLOSE
      8 Tabs at=410,316,1101,540 by=panel limit=350.48,187,1219.05,822 name=MemoryCardTabs paints=#D4D4D4FF role=tab
        9 TabItem at=410,316,93,72 by=clip drawn focus heard=N64 limit=410,316,1101,540 paints=#8D8D91FF role=tabitem sideways text=N64
          10 TextBlock at=428,325,57,54 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=37.98,36 limit=410,316,93,72 owner=9 paints=#8D8D91FF role=text size=30 text=N64
        11 TabItem at=503,316,156,72 by=clip drawn focus heard=GENESIS limit=410,316,1101,540 paints=#8D8D91FF role=tabitem sideways text=Genesis
          12 TextBlock at=521,325,120,54 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=79.89,36 limit=503,316,156,72 owner=11 paints=#8D8D91FF role=text size=30 text=GENESIS
        13 TabItem at=659,316,157.5,72 by=clip drawn focus heard="SEGA CD" limit=410,316,1101,540 op paints=#EEEEF0FF role=tabitem sideways text="Sega CD"
          14 TextBlock at=677,325,121.5,54 box=81,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=80.43,36 limit=659,316,157.5,72 owner=13 paints=#EEEEF0FF role=text size=30 text="SEGA CD"
        15 TabItem at=816.5,316,220.5,72 by=clip drawn focus heard=PLAYSTATION limit=410,316,1101,540 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          16 TextBlock at=834.5,325,184.5,54 box=123,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=122.67,36 limit=816.5,316,220.5,72 owner=15 paints=#8D8D91FF role=text size=30 text=PLAYSTATION
        17 TabItem at=1037,316,247.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=410,316,1101,540 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          18 TextBlock at=1055,325,211.5,54 box=141,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=140.19,36 limit=1037,316,247.5,72 owner=17 paints=#8D8D91FF role=text size=30 text="PLAYSTATION 2"
        19 ScrollViewer at=428,391,1065,465 by=clip fades=b limit=410,391,1101,465 overflow=1545 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=428,391,1065,465
          20 StackPanel at=428,403,1065,1998 by=scroll in=19 limit=428,391,1065,465 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF,#FFFFFFFF role=none seen=428,403,1065,453
            21 StackPanel at=428,403,1065,985.5 by=scroll in=19 limit=428,391,1065,465 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=428,403,1065,453
              22 SectionHeader at=428,403,1065,51 box=710,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="Internal Backup RAM" in=19 layout=207.68,33.6 limit=428,391,1065,465 paints=#D4D4D4FF role=text size=28 text="Internal Backup RAM"
              23 FieldRow at=428,466,1065,199.5 by=scroll heard=Card in=19 limit=428,391,1065,465 paints=#D4D4D4FF role=group
                24 TextBlock at=428,466,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Card in=19 layout=36.36,28.8 limit=428,466,1065,199.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Card
                25 HintText at=428,515.5,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A card in the drawer, or a game's own." in=19 layout=249.38,24 limit=428,466,1065,199.5 paints=#8D8D91FF part role=text size=20 text="A card in the drawer, or a game's own."
                26 StackPanel at=428,557.5,1065,108 by=clip heard=Card in=19 limit=428,466,1065,199.5 role=none
                  27 Dropdown at=428,557.5,390,66 by=clip drawn focus heard=Card in=19 limit=428,466,1065,199.5 name="Sega CD.mcd.backup.Card" op paints=#0E0E10FF,#707070FF,#D4D4D4FF,#EEEEF0FF role=combobox sideways
                    28 TextBlock at=446,565,324,48 box=216,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Backup RAM 1" in=19 layout=115.13,31.2 limit=446,565,324,48 owner=27 paints=#EEEEF0FF role=text size=26 text="Backup RAM 1"
                    29 ViewboxContainer at=785,585.6,18,9.81 by=clip in=19 limit=785,585.6,18,9.81 nohit role=none
                  30 HintText at=428,629.5,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Shared: Backup RAM 1 in Inside the console. 0 save(s), 125 of 125 blocks free." in=19 layout=492.84,24 limit=428,466,1065,199.5 name="Sega CD.mcd.backup.About" paints=#8D8D91FF role=text size=20 text="Shared: Backup RAM 1 in Inside the console. 0 save(s), 125 of 125 blocks free."
              31 FieldRow at=428,677.5,1065,127.5 by=scroll heard=Saves in=19 limit=428,391,1065,465 paints=#D4D4D4FF role=group
                32 TextBlock at=428,677.5,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Saves in=19 layout=45.31,28.8 limit=428,677.5,1065,127.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Saves
                33 HintText at=428,727,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Choose a save to copy, move or delete it." in=19 layout=264.24,24 limit=428,677.5,1065,127.5 paints=#8D8D91FF part role=text size=20 text="Choose a save to copy, move or delete it."
                34 StackPanel at=428,769,1065,36 by=clip heard=Saves in=19 limit=428,677.5,1065,127.5 name="Sega CD.mcd.backup.Saves" role=none
                  35 HintText at=428,769,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="No saves." in=19 layout=61.96,24 limit=428,677.5,1065,127.5 paints=#8D8D91FF role=text size=20 text="No saves."
              36 FieldRow at=428,817,1065,193.5 by=scroll heard="Copy or move to" in=19 limit=428,391,1065,465 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=group seen=428,817,1065,39
                37 TextBlock at=428,817,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Copy or move to" in=19 layout=126.34,28.8 limit=428,817,1065,193.5 name=PART_Label paints=#D4D4D4FF part role=text seen=428,817,1065,39 size=24 text="Copy or move to"
focus 27
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
edge 6 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge 6 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
edge 6 right same
edge 13 up same
edge 13 down 27
edge 27 up 13
edge 27 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/3/0/2/0/0 up 27
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
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/4/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/3/0/2/0/0
press pageup focus="Dropdown Genesis.mcd.backup.Card" tab=Genesis title="Memory Cards" top=MemoryCardsWindow
press pagedown focus="Dropdown Sega CD.mcd.backup.Card" tab="Sega CD" title="Memory Cards" top=MemoryCardsWindow
press first focus="Dropdown Sega CD.mcd.backup.Card" tab="Sega CD" title="Memory Cards" top=MemoryCardsWindow
press last focus="Dropdown Sega CD.mcd.backup.Card" tab="Sega CD" title="Memory Cards" top=MemoryCardsWindow
press back top=none
