# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardSave-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case PauseCardSave
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Pause limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1920,1161 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1161 name=AdaptiveCard role=menu
          4 HintBar at=476,1075,967,71 by=panel heard="A Select, B Back, Menu Close Menu, X Screenshot, D-pad left and right Change" limit=0,0,1920,1161 role=group
          5 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
            6 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
              7 LunaList at=36,357,768,567 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=36,357,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,357,768,567
                  9 ListBoxItem at=36,357,768,81 by=fixed-scroll drawn focus heard=Resume in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                  10 ListBoxItem at=36,438,768,81 by=fixed-scroll drawn focus heard=Rewind in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                  11 ListBoxItem at=36,519,768,81 by=fixed-scroll drawn focus heard=Save in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                  12 ListBoxItem at=36,600,768,81 by=fixed-scroll drawn focus heard=Load in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                  13 ListBoxItem at=36,681,768,81 by=fixed-scroll drawn focus heard=Speed in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                  14 ListBoxItem at=36,762,768,81 by=fixed-scroll drawn focus heard="Game Options" in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                  15 ListBoxItem at=36,843,768,81 by=fixed-scroll drawn focus heard=Leave in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
          16 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
            17 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
              18 PauseCardPane at=840,250,1044,576 by=panel limit=0,0,1920,1161 name=PauseCardPane role=none
                19 UniformGrid at=840,250,1044,576 by=panel limit=0,0,1920,1161 name=PaneSlots role=none
                  20 StackPanel at=852,262,237,264 by=panel limit=0,0,1920,1161 role=none
                    21 Border at=850,262,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot1 paints=#0E0E10FF,#4C9AE8FF role=none
                    22 TextBlock at=852,448,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 1" layout=64.02,36 limit=0,0,1920,1161 paints=#EEEEF0FF role=text size=20 text="Slot 1"
                    23 TextBlock at=852,490,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  24 StackPanel at=1113,262,237,264 by=panel limit=0,0,1920,1161 role=none
                    25 Border at=1111,262,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot2 paints=#0E0E10FF,#707070FF role=none
                    26 TextBlock at=1113,448,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 2" layout=69.78,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 2"
                    27 TextBlock at=1113,490,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  28 StackPanel at=1374,262,237,264 by=panel limit=0,0,1920,1161 role=none
                    29 Border at=1372,262,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot3 paints=#0E0E10FF,#707070FF role=none
                    30 TextBlock at=1374,448,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 3" layout=69.45,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 3"
                    31 TextBlock at=1374,490,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  32 StackPanel at=1635,262,237,264 by=panel limit=0,0,1920,1161 role=none
                    33 Border at=1633,262,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot4 paints=#0E0E10FF,#707070FF role=none
                    34 TextBlock at=1635,448,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 4" layout=69.72,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 4"
                    35 TextBlock at=1635,490,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  36 StackPanel at=852,550,237,264 by=panel limit=0,0,1920,1161 role=none
                    37 Border at=850,550,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot5 paints=#0E0E10FF,#707070FF role=none
                    38 TextBlock at=852,736,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 5" layout=69.81,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 5"
                    39 TextBlock at=852,778,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  40 StackPanel at=1113,550,237,264 by=panel limit=0,0,1920,1161 role=none
                    41 Border at=1111,550,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot6 paints=#0E0E10FF,#707070FF role=none
                    42 TextBlock at=1113,736,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 6" layout=69.51,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 6"
                    43 TextBlock at=1113,778,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  44 StackPanel at=1374,550,237,264 by=panel limit=0,0,1920,1161 role=none
                    45 Border at=1372,550,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot7 paints=#0E0E10FF,#707070FF role=none
                    46 TextBlock at=1374,736,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 7" layout=68.64,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 7"
                    47 TextBlock at=1374,778,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
                  48 StackPanel at=1635,550,237,264 by=panel limit=0,0,1920,1161 role=none
                    49 Border at=1633,550,240,180 by=panel limit=0,0,1920,1161 name=PaneSlot8 paints=#0E0E10FF,#707070FF role=none
                    50 TextBlock at=1635,736,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard="Slot 8" layout=70.05,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Slot 8"
                    51 TextBlock at=1635,778,237,36 box=237,36 by=panel caps=13.999 drawn face=Barlow heard=Empty layout=82.47,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text=Empty
focus none
press down menu="open at 3: Load      <  Slot 1 (Empty)  >" top=none
press up menu="open at 2: Save      <  Slot 1 (Empty)  >" top=none
press back top=none
