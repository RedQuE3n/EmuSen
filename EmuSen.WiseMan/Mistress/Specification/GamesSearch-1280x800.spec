# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesSearch-1280x800
size 1280 800
kind menu
root GalleryLayout#GamesScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case GamesSearch
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
          4 TextBlock at=1114,50,102,24 box=102,24 by=panel caps=13.999 drawn face=Barlow heard="1 of 8 games" layout=101.62,24 limit=0,0,1280,800 name=GalleryCount paints=#8D8D91FF role=text size=20 text="1 of 8 games"
          5 WholeRow at=64,40,1026,44 by=panel heard=Sections limit=0,0,1280,800 name=GallerySections role=tab
            6 SectionTab at=64,40,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=ALL
              7 AccessText at=80,48,34,25 box=34,25 by=clip caps=13.999 drawn face=Barlow layout=33.56,24 limit=64,40,66,44 owner=6 paints=#EEEEF0FF role=none size=20 text=ALL
            8 SectionTab at=138,40,138,44 by=panel drawn focus heard=Favourites limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=FAVOURITES
              9 AccessText at=154,48,106,25 box=106,25 by=clip caps=13.999 drawn face=Barlow layout=105.54,24 limit=138,40,138,44 owner=8 paints=#D4D4D4FF role=none size=20 text=FAVOURITES
            10 SectionTab at=284,40,103,44 by=panel drawn focus heard=Recent limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=RECENT
              11 AccessText at=300,48,71,25 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.14,24 limit=284,40,103,44 owner=10 paints=#D4D4D4FF role=none size=20 text=RECENT
            12 SectionTab at=395,40,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=NES
              13 AccessText at=411,48,36,25 box=36,25 by=clip caps=13.999 drawn face=Barlow layout=35.76,24 limit=395,40,68,44 owner=12 paints=#D4D4D4FF role=none size=20 text=NES
            14 SectionTab at=471,40,119,44 by=panel drawn focus heard="Game Boy" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="GAME BOY"
              15 AccessText at=487,48,87,25 box=87,25 by=clip caps=13.999 drawn face=Barlow layout=86.74,24 limit=471,40,119,44 owner=14 paints=#D4D4D4FF role=none size=20 text="GAME BOY"
            16 SectionTab at=598,40,186,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="SUPER NINTENDO"
              17 AccessText at=614,48,154,25 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=598,40,186,44 owner=16 paints=#D4D4D4FF role=none size=20 text="SUPER NINTENDO"
        18 TextBox at=64,100,1152,44 by=panel drawn heard=Search limit=0,0,1280,800 name=GallerySearch paints=#0E0E10FF,#707070FF,#EEEEF0FF role=edit text=gran
          19 ScrollViewer at=81,101,1118,42 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
        20 Panel at=142,156,996,240 by=panel heard=All limit=0,0,1280,800 name=GalleryGrid role=none
          21 CoverCard at=142,156,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="GalleryTile./tmp/EmuSenWindowFit/001/Roms/Granite Choir (Synthetic).nes" role=none
            22 Border at=142,270,180,126 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              23 Image at=142,270,180,126 by=clip drawn limit=142,270,180,126 nohit role=image
        24 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=GalleryHelpBar paints=#1B1B1EFF,#707070FF role=none
          25 HintBar at=64,716,1152,30 by=panel heard="Type, Delete, Space, Done, Move to grid, Home" limit=0,0,1280,800 role=group
        26 OnScreenKeyboard at=0,398,1280,304 by=panel focus heard="On-screen keyboard" limit=0,0,1280,800 name=GalleryKeyboard paints=#1B1B1EFF,#707070FF role=none
          27 StackPanel at=84,415,1112,271 by=panel limit=0,0,1280,800 role=none
            28 StackPanel at=244,415,792,48 by=panel limit=0,0,1280,800 role=none
              29 Button at=244,415,72,48 by=panel drawn heard=1 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=1
                30 TextBlock at=276,425,7,27 box=7,27 by=clip caps=15.3989 drawn face=Barlow heard=1 layout=6.93,26.4 limit=244,415,72,48 owner=29 paints=#D4D4D4FF part role=text size=22 text=1
              31 Button at=324,415,72,48 by=panel drawn heard=2 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=2
                32 TextBlock at=354,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=2 layout=11.15,26.4 limit=324,415,72,48 owner=31 paints=#D4D4D4FF part role=text size=22 text=2
              33 Button at=404,415,72,48 by=panel drawn heard=3 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=3
                34 TextBlock at=434,425,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=3 layout=10.91,26.4 limit=404,415,72,48 owner=33 paints=#D4D4D4FF part role=text size=22 text=3
              35 Button at=484,415,72,48 by=panel drawn heard=4 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=4
                36 TextBlock at=514,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=4 layout=11.11,26.4 limit=484,415,72,48 owner=35 paints=#D4D4D4FF part role=text size=22 text=4
              37 Button at=564,415,72,48 by=panel drawn heard=5 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=5
                38 TextBlock at=594,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=5 layout=11.18,26.4 limit=564,415,72,48 owner=37 paints=#D4D4D4FF part role=text size=22 text=5
              39 Button at=644,415,72,48 by=panel drawn heard=6 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=6
                40 TextBlock at=674,425,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=6 layout=10.96,26.4 limit=644,415,72,48 owner=39 paints=#D4D4D4FF part role=text size=22 text=6
              41 Button at=724,415,72,48 by=panel drawn heard=7 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=7
                42 TextBlock at=754,425,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=7 layout=10.32,26.4 limit=724,415,72,48 owner=41 paints=#D4D4D4FF part role=text size=22 text=7
              43 Button at=804,415,72,48 by=panel drawn heard=8 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=8
                44 TextBlock at=834,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=8 layout=11.35,26.4 limit=804,415,72,48 owner=43 paints=#D4D4D4FF part role=text size=22 text=8
              45 Button at=884,415,72,48 by=panel drawn heard=9 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=9
                46 TextBlock at=914,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=9 layout=11.46,26.4 limit=884,415,72,48 owner=45 paints=#D4D4D4FF part role=text size=22 text=9
              47 Button at=964,415,72,48 by=panel drawn heard=0 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=0
                48 TextBlock at=994,425,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=0 layout=12.03,26.4 limit=964,415,72,48 owner=47 paints=#D4D4D4FF part role=text size=22 text=0
            49 StackPanel at=244,471,792,48 by=panel limit=0,0,1280,800 role=none
              50 Button at=244,471,72,48 by=panel drawn heard=q limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=q
                51 TextBlock at=274,481,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=q layout=12.14,26.4 limit=244,471,72,48 owner=50 paints=#D4D4D4FF part role=text size=22 text=q
              52 Button at=324,471,72,48 by=panel drawn heard=w limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=w
                53 TextBlock at=352,481,16,27 box=16,27 by=clip caps=15.3989 drawn face=Barlow heard=w layout=15.88,26.4 limit=324,471,72,48 owner=52 paints=#D4D4D4FF part role=text size=22 text=w
              54 Button at=404,471,72,48 by=panel drawn heard=e limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=e
                55 TextBlock at=434,481,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=e layout=11.09,26.4 limit=404,471,72,48 owner=54 paints=#D4D4D4FF part role=text size=22 text=e
              56 Button at=484,471,72,48 by=panel drawn heard=r limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=r
                57 TextBlock at=516,481,8,27 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=r layout=7.99,26.4 limit=484,471,72,48 owner=56 paints=#D4D4D4FF part role=text size=22 text=r
              58 Button at=564,471,72,48 by=panel drawn heard=t limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=t
                59 TextBlock at=596,481,8,27 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=t layout=7.13,26.4 limit=564,471,72,48 owner=58 paints=#D4D4D4FF part role=text size=22 text=t
              60 Button at=644,471,72,48 by=panel drawn heard=y limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=y
                61 TextBlock at=674,481,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=y layout=10.34,26.4 limit=644,471,72,48 owner=60 paints=#D4D4D4FF part role=text size=22 text=y
              62 Button at=724,471,72,48 by=panel drawn heard=u limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=u
                63 TextBlock at=754,481,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=u layout=11.7,26.4 limit=724,471,72,48 owner=62 paints=#D4D4D4FF part role=text size=22 text=u
              64 Button at=804,471,72,48 by=panel drawn heard=i limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=i
                65 TextBlock at=837,481,6,27 box=6,27 by=clip caps=15.3989 drawn face=Barlow heard=i layout=5.5,26.4 limit=804,471,72,48 owner=64 paints=#D4D4D4FF part role=text size=22 text=i
              66 Button at=884,471,72,48 by=panel drawn heard=o limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=o
                67 TextBlock at=914,481,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=o layout=11.46,26.4 limit=884,471,72,48 owner=66 paints=#D4D4D4FF part role=text size=22 text=o
              68 Button at=964,471,72,48 by=panel drawn heard=p limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=p
                69 TextBlock at=994,481,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=p layout=12.14,26.4 limit=964,471,72,48 owner=68 paints=#D4D4D4FF part role=text size=22 text=p
            70 StackPanel at=244,527,792,48 by=panel limit=0,0,1280,800 role=none
              71 Button at=244,527,72,48 by=panel drawn heard=a limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=a
                72 TextBlock at=274,537,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=a layout=11.24,26.4 limit=244,527,72,48 owner=71 paints=#D4D4D4FF part role=text size=22 text=a
              73 Button at=324,527,72,48 by=panel drawn heard=s limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=s
                74 TextBlock at=354,537,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=s layout=10.1,26.4 limit=324,527,72,48 owner=73 paints=#D4D4D4FF part role=text size=22 text=s
              75 Button at=404,527,72,48 by=panel drawn heard=d limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=d
                76 TextBlock at=434,537,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=d layout=12.01,26.4 limit=404,527,72,48 owner=75 paints=#D4D4D4FF part role=text size=22 text=d
              77 Button at=484,527,72,48 by=panel drawn heard=f limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=f
                78 TextBlock at=516,537,8,27 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=f layout=7.17,26.4 limit=484,527,72,48 owner=77 paints=#D4D4D4FF part role=text size=22 text=f
              79 Button at=564,527,72,48 by=panel drawn heard=g limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=g
                80 TextBlock at=594,537,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=g layout=11.66,26.4 limit=564,527,72,48 owner=79 paints=#D4D4D4FF part role=text size=22 text=g
              81 Button at=644,527,72,48 by=panel drawn heard=h limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=h
                82 TextBlock at=674,537,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=h layout=11.81,26.4 limit=644,527,72,48 owner=81 paints=#D4D4D4FF part role=text size=22 text=h
              83 Button at=724,527,72,48 by=panel drawn heard=j limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=j
                84 TextBlock at=757,537,6,27 box=6,27 by=clip caps=15.3989 drawn face=Barlow heard=j layout=5.06,26.4 limit=724,527,72,48 owner=83 paints=#D4D4D4FF part role=text size=22 text=j
              85 Button at=804,527,72,48 by=panel drawn heard=k limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=k
                86 TextBlock at=834,537,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=k layout=10.82,26.4 limit=804,527,72,48 owner=85 paints=#D4D4D4FF part role=text size=22 text=k
              87 Button at=884,527,72,48 by=panel drawn heard=l limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=l
                88 TextBlock at=918,537,5,27 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=l layout=4.93,26.4 limit=884,527,72,48 owner=87 paints=#D4D4D4FF part role=text size=22 text=l
              89 Button at=964,527,72,48 by=panel drawn heard=' limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text='
                90 TextBlock at=998,537,5,27 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=' layout=4.31,26.4 limit=964,527,72,48 owner=89 paints=#D4D4D4FF part role=text size=22 text='
            91 StackPanel at=84,583,1112,48 by=panel limit=0,0,1280,800 role=none
              92 Button at=84,583,152,48 by=panel drawn heard=Shift limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Shift
                93 TextBlock at=138,593,44,27 box=44,27 by=clip caps=15.3989 drawn face=Barlow heard=Shift layout=43.71,26.4 limit=84,583,152,48 owner=92 paints=#D4D4D4FF part role=text size=22 text=Shift
              94 Button at=244,583,72,48 by=panel drawn heard=z limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=z
                95 TextBlock at=274,593,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=z layout=10.12,26.4 limit=244,583,72,48 owner=94 paints=#D4D4D4FF part role=text size=22 text=z
              96 Button at=324,583,72,48 by=panel drawn heard=x limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=x
                97 TextBlock at=354,593,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=x layout=10.52,26.4 limit=324,583,72,48 owner=96 paints=#D4D4D4FF part role=text size=22 text=x
              98 Button at=404,583,72,48 by=panel drawn heard=c limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=c
                99 TextBlock at=434,593,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=c layout=11.02,26.4 limit=404,583,72,48 owner=98 paints=#D4D4D4FF part role=text size=22 text=c
              100 Button at=484,583,72,48 by=panel drawn heard=v limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=v
                101 TextBlock at=514,593,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=v layout=10.43,26.4 limit=484,583,72,48 owner=100 paints=#D4D4D4FF part role=text size=22 text=v
              102 Button at=564,583,72,48 by=panel drawn heard=b limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=b
                103 TextBlock at=594,593,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=b layout=11.66,26.4 limit=564,583,72,48 owner=102 paints=#D4D4D4FF part role=text size=22 text=b
              104 Button at=644,583,72,48 by=panel drawn heard=n limit=0,0,1280,800 paints=#26262BFF,#4C9AE8FF,#EEEEF0FF role=button text=n
                105 TextBlock at=674,593,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=n layout=11.81,26.4 limit=644,583,72,48 owner=104 paints=#EEEEF0FF part role=text size=22 text=n
              106 Button at=724,583,72,48 by=panel drawn heard=m limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=m
                107 TextBlock at=750,593,19,27 box=19,27 by=clip caps=15.3989 drawn face=Barlow heard=m layout=18.13,26.4 limit=724,583,72,48 owner=106 paints=#D4D4D4FF part role=text size=22 text=m
              108 Button at=804,583,72,48 by=panel drawn heard=, limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=,
                109 TextBlock at=838,593,5,27 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=, layout=4.49,26.4 limit=804,583,72,48 owner=108 paints=#D4D4D4FF part role=text size=22 text=,
              110 Button at=884,583,72,48 by=panel drawn heard=. limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=.
                111 TextBlock at=916,593,7,27 box=7,27 by=clip caps=15.3989 drawn face=Barlow heard=. layout=6.12,26.4 limit=884,583,72,48 owner=110 paints=#D4D4D4FF part role=text size=22 text=.
              112 Button at=964,583,72,48 by=panel drawn heard=- limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=-
                113 TextBlock at=995,593,10,27 box=10,27 by=clip caps=15.3989 drawn face=Barlow heard=- layout=9.88,26.4 limit=964,583,72,48 owner=112 paints=#D4D4D4FF part role=text size=22 text=-
              114 Button at=1044,583,152,48 by=panel drawn heard="⌫" limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text="⌫"
                115 TextBlock at=1113,593,14,29 box=14,29 by=clip caps=16.1724 drawn face="Adwaita Mono" heard="⌫" layout=13.2,28.27 limit=1044,583,152,48 owner=114 paints=#D4D4D4FF part role=text size=22 text="⌫"
            116 StackPanel at=284,639,712,48 by=panel limit=0,0,1280,800 role=none
              117 Button at=284,639,152,48 by=panel drawn heard=Symbols limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Symbols
                118 TextBlock at=320,649,79,27 box=79,27 by=clip caps=15.3989 drawn face=Barlow heard=Symbols layout=78.72,26.4 limit=284,639,152,48 owner=117 paints=#D4D4D4FF part role=text size=22 text=Symbols
              119 Button at=444,639,392,48 by=panel drawn heard=Space limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Space
                120 TextBlock at=611,649,58,27 box=58,27 by=clip caps=15.3989 drawn face=Barlow heard=Space layout=57.6,26.4 limit=444,639,392,48 owner=119 paints=#D4D4D4FF part role=text size=22 text=Space
              121 Button at=844,639,152,48 by=panel drawn heard=Done limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Done
                122 TextBlock at=896,649,48,27 box=48,27 by=clip caps=15.3989 drawn face=Barlow heard=Done layout=47.7,26.4 limit=844,639,152,48 owner=121 paints=#D4D4D4FF part role=text size=22 text=Done
focus none
press down focus="GalleryLayout GamesScreen" games="all keyboard" top=none
press up focus="GalleryLayout GamesScreen" games="all keyboard" top=none
press back focus="GalleryLayout GamesScreen" games="all keyboard" top=none
