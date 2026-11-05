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
    1 LunaSky at=0,0,1280,800 by=panel limit=0,0,1280,800 name=GallerySky nohit role=none
    2 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
        4 DockPanel at=64,34,1152,52 by=panel limit=0,0,1280,800 role=none
          5 StackPanel at=905,34,311,52 by=panel limit=0,0,1280,800 name=GalleryNeighbours role=none
            6 PadGlyph at=905,45,30,30 by=panel heard=LB limit=0,0,1280,800 name=GalleryLeftShoulder role=image
            7 TextBlock at=947,49,119,22 box=119,22 by=panel caps=12.5991 drawn face=Barlow heard="Super Nintendo" layout=118.42,21.6 limit=0,0,1280,800 name=GalleryPrevious paints=#8D8D91FF role=text size=18 text="Super Nintendo"
            8 TextBlock at=1078,49,4,22 box=4,22 by=panel caps=12.5991 drawn face=Barlow heard="·" layout=3.78,21.6 limit=0,0,1280,800 paints=#8D8D91FF role=text size=18 text="·"
            9 TextBlock at=1094,49,80,22 box=80,22 by=panel caps=12.5991 drawn face=Barlow heard=Favourites layout=79.13,21.6 limit=0,0,1280,800 name=GalleryNext paints=#8D8D91FF role=text size=18 text=Favourites
            10 PadGlyph at=1186,45,30,30 by=panel heard=RB limit=0,0,1280,800 name=GalleryRightShoulder role=image
          11 StackPanel at=64,34,273,52 by=panel limit=0,0,1280,800 name=GalleryHeading role=none
            12 TextBlock at=64,47,90,26 box=90,26 by=panel caps=13.999 drawn face=Barlow heard="GAMES ▸" layout=89.6,25.7 limit=0,0,1280,800 name=GalleryTrail paints=#8D8D91FF role=text size=20 text="GAMES ▸"
            13 TextBlock at=170,40,49,39 box=49,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=ALL layout=48.46,38.4 limit=0,0,1280,800 name=GallerySection paints=#EEEEF0FF role=text size=32 text=ALL
            14 TextBlock at=235,48,102,24 box=102,24 by=panel caps=13.999 drawn face=Barlow heard="1 of 8 games" layout=101.62,24 limit=0,0,1280,800 name=GalleryCount paints=#8D8D91FF role=text size=20 text="1 of 8 games"
        15 TextBox at=64,100,1152,44 by=panel drawn heard=Search limit=0,0,1280,800 name=GallerySearch paints=#0E0E10FF,#707070FF,#EEEEF0FF role=edit text=gran
          16 ScrollViewer at=81,101,1118,42 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
        17 Panel at=142,156,996,240 by=panel heard=All limit=0,0,1280,800 name=GalleryGrid role=none
          18 CoverCard at=142,156,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            19 Border at=148,156,169,240 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              20 Image at=148,156,168.22,240 by=clip drawn limit=148,156,169,240 nohit role=image
        21 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=GalleryHelpBar role=none
          22 Panel at=0,702,1280,98 by=panel limit=0,0,1280,800 role=none
            23 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 nohit paints=#0E0E10FF,#707070FF role=none
            24 HintBar at=64,716,1152,30 by=panel heard="Type, Delete, Space, Done, Move to grid, Home" limit=0,0,1280,800 role=group
        25 OnScreenKeyboard at=0,398,1280,304 by=panel focus heard="On-screen keyboard" limit=0,0,1280,800 name=GalleryKeyboard paints=#1B1B1EFF,#707070FF role=none
          26 StackPanel at=84,415,1112,271 by=panel limit=0,0,1280,800 role=none
            27 StackPanel at=244,415,792,48 by=panel limit=0,0,1280,800 role=none
              28 Button at=244,415,72,48 by=panel drawn heard=1 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=1
                29 TextBlock at=276,425,7,27 box=7,27 by=clip caps=15.3989 drawn face=Barlow heard=1 layout=6.93,26.4 limit=244,415,72,48 owner=28 paints=#D4D4D4FF part role=text size=22 text=1
              30 Button at=324,415,72,48 by=panel drawn heard=2 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=2
                31 TextBlock at=354,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=2 layout=11.15,26.4 limit=324,415,72,48 owner=30 paints=#D4D4D4FF part role=text size=22 text=2
              32 Button at=404,415,72,48 by=panel drawn heard=3 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=3
                33 TextBlock at=434,425,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=3 layout=10.91,26.4 limit=404,415,72,48 owner=32 paints=#D4D4D4FF part role=text size=22 text=3
              34 Button at=484,415,72,48 by=panel drawn heard=4 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=4
                35 TextBlock at=514,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=4 layout=11.11,26.4 limit=484,415,72,48 owner=34 paints=#D4D4D4FF part role=text size=22 text=4
              36 Button at=564,415,72,48 by=panel drawn heard=5 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=5
                37 TextBlock at=594,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=5 layout=11.18,26.4 limit=564,415,72,48 owner=36 paints=#D4D4D4FF part role=text size=22 text=5
              38 Button at=644,415,72,48 by=panel drawn heard=6 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=6
                39 TextBlock at=674,425,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=6 layout=10.96,26.4 limit=644,415,72,48 owner=38 paints=#D4D4D4FF part role=text size=22 text=6
              40 Button at=724,415,72,48 by=panel drawn heard=7 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=7
                41 TextBlock at=754,425,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=7 layout=10.32,26.4 limit=724,415,72,48 owner=40 paints=#D4D4D4FF part role=text size=22 text=7
              42 Button at=804,415,72,48 by=panel drawn heard=8 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=8
                43 TextBlock at=834,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=8 layout=11.35,26.4 limit=804,415,72,48 owner=42 paints=#D4D4D4FF part role=text size=22 text=8
              44 Button at=884,415,72,48 by=panel drawn heard=9 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=9
                45 TextBlock at=914,425,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=9 layout=11.46,26.4 limit=884,415,72,48 owner=44 paints=#D4D4D4FF part role=text size=22 text=9
              46 Button at=964,415,72,48 by=panel drawn heard=0 limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=0
                47 TextBlock at=994,425,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=0 layout=12.03,26.4 limit=964,415,72,48 owner=46 paints=#D4D4D4FF part role=text size=22 text=0
            48 StackPanel at=244,471,792,48 by=panel limit=0,0,1280,800 role=none
              49 Button at=244,471,72,48 by=panel drawn heard=q limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=q
                50 TextBlock at=274,481,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=q layout=12.14,26.4 limit=244,471,72,48 owner=49 paints=#D4D4D4FF part role=text size=22 text=q
              51 Button at=324,471,72,48 by=panel drawn heard=w limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=w
                52 TextBlock at=352,481,16,27 box=16,27 by=clip caps=15.3989 drawn face=Barlow heard=w layout=15.88,26.4 limit=324,471,72,48 owner=51 paints=#D4D4D4FF part role=text size=22 text=w
              53 Button at=404,471,72,48 by=panel drawn heard=e limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=e
                54 TextBlock at=434,481,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=e layout=11.09,26.4 limit=404,471,72,48 owner=53 paints=#D4D4D4FF part role=text size=22 text=e
              55 Button at=484,471,72,48 by=panel drawn heard=r limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=r
                56 TextBlock at=516,481,8,27 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=r layout=7.99,26.4 limit=484,471,72,48 owner=55 paints=#D4D4D4FF part role=text size=22 text=r
              57 Button at=564,471,72,48 by=panel drawn heard=t limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=t
                58 TextBlock at=596,481,8,27 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=t layout=7.13,26.4 limit=564,471,72,48 owner=57 paints=#D4D4D4FF part role=text size=22 text=t
              59 Button at=644,471,72,48 by=panel drawn heard=y limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=y
                60 TextBlock at=674,481,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=y layout=10.34,26.4 limit=644,471,72,48 owner=59 paints=#D4D4D4FF part role=text size=22 text=y
              61 Button at=724,471,72,48 by=panel drawn heard=u limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=u
                62 TextBlock at=754,481,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=u layout=11.7,26.4 limit=724,471,72,48 owner=61 paints=#D4D4D4FF part role=text size=22 text=u
              63 Button at=804,471,72,48 by=panel drawn heard=i limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=i
                64 TextBlock at=837,481,6,27 box=6,27 by=clip caps=15.3989 drawn face=Barlow heard=i layout=5.5,26.4 limit=804,471,72,48 owner=63 paints=#D4D4D4FF part role=text size=22 text=i
              65 Button at=884,471,72,48 by=panel drawn heard=o limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=o
                66 TextBlock at=914,481,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=o layout=11.46,26.4 limit=884,471,72,48 owner=65 paints=#D4D4D4FF part role=text size=22 text=o
              67 Button at=964,471,72,48 by=panel drawn heard=p limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=p
                68 TextBlock at=994,481,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=p layout=12.14,26.4 limit=964,471,72,48 owner=67 paints=#D4D4D4FF part role=text size=22 text=p
            69 StackPanel at=244,527,792,48 by=panel limit=0,0,1280,800 role=none
              70 Button at=244,527,72,48 by=panel drawn heard=a limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=a
                71 TextBlock at=274,537,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=a layout=11.24,26.4 limit=244,527,72,48 owner=70 paints=#D4D4D4FF part role=text size=22 text=a
              72 Button at=324,527,72,48 by=panel drawn heard=s limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=s
                73 TextBlock at=354,537,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=s layout=10.1,26.4 limit=324,527,72,48 owner=72 paints=#D4D4D4FF part role=text size=22 text=s
              74 Button at=404,527,72,48 by=panel drawn heard=d limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=d
                75 TextBlock at=434,537,13,27 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=d layout=12.01,26.4 limit=404,527,72,48 owner=74 paints=#D4D4D4FF part role=text size=22 text=d
              76 Button at=484,527,72,48 by=panel drawn heard=f limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=f
                77 TextBlock at=516,537,8,27 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=f layout=7.17,26.4 limit=484,527,72,48 owner=76 paints=#D4D4D4FF part role=text size=22 text=f
              78 Button at=564,527,72,48 by=panel drawn heard=g limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=g
                79 TextBlock at=594,537,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=g layout=11.66,26.4 limit=564,527,72,48 owner=78 paints=#D4D4D4FF part role=text size=22 text=g
              80 Button at=644,527,72,48 by=panel drawn heard=h limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=h
                81 TextBlock at=674,537,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=h layout=11.81,26.4 limit=644,527,72,48 owner=80 paints=#D4D4D4FF part role=text size=22 text=h
              82 Button at=724,527,72,48 by=panel drawn heard=j limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=j
                83 TextBlock at=757,537,6,27 box=6,27 by=clip caps=15.3989 drawn face=Barlow heard=j layout=5.06,26.4 limit=724,527,72,48 owner=82 paints=#D4D4D4FF part role=text size=22 text=j
              84 Button at=804,527,72,48 by=panel drawn heard=k limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=k
                85 TextBlock at=834,537,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=k layout=10.82,26.4 limit=804,527,72,48 owner=84 paints=#D4D4D4FF part role=text size=22 text=k
              86 Button at=884,527,72,48 by=panel drawn heard=l limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=l
                87 TextBlock at=918,537,5,27 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=l layout=4.93,26.4 limit=884,527,72,48 owner=86 paints=#D4D4D4FF part role=text size=22 text=l
              88 Button at=964,527,72,48 by=panel drawn heard=' limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text='
                89 TextBlock at=998,537,5,27 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=' layout=4.31,26.4 limit=964,527,72,48 owner=88 paints=#D4D4D4FF part role=text size=22 text='
            90 StackPanel at=84,583,1112,48 by=panel limit=0,0,1280,800 role=none
              91 Button at=84,583,152,48 by=panel drawn heard=Shift limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Shift
                92 TextBlock at=138,593,44,27 box=44,27 by=clip caps=15.3989 drawn face=Barlow heard=Shift layout=43.71,26.4 limit=84,583,152,48 owner=91 paints=#D4D4D4FF part role=text size=22 text=Shift
              93 Button at=244,583,72,48 by=panel drawn heard=z limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=z
                94 TextBlock at=274,593,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=z layout=10.12,26.4 limit=244,583,72,48 owner=93 paints=#D4D4D4FF part role=text size=22 text=z
              95 Button at=324,583,72,48 by=panel drawn heard=x limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=x
                96 TextBlock at=354,593,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=x layout=10.52,26.4 limit=324,583,72,48 owner=95 paints=#D4D4D4FF part role=text size=22 text=x
              97 Button at=404,583,72,48 by=panel drawn heard=c limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=c
                98 TextBlock at=434,593,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=c layout=11.02,26.4 limit=404,583,72,48 owner=97 paints=#D4D4D4FF part role=text size=22 text=c
              99 Button at=484,583,72,48 by=panel drawn heard=v limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=v
                100 TextBlock at=514,593,11,27 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=v layout=10.43,26.4 limit=484,583,72,48 owner=99 paints=#D4D4D4FF part role=text size=22 text=v
              101 Button at=564,583,72,48 by=panel drawn heard=b limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=b
                102 TextBlock at=594,593,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=b layout=11.66,26.4 limit=564,583,72,48 owner=101 paints=#D4D4D4FF part role=text size=22 text=b
              103 Button at=644,583,72,48 by=panel drawn heard=n limit=0,0,1280,800 paints=#26262BFF,#4C9AE8FF,#EEEEF0FF role=button text=n
                104 TextBlock at=674,593,12,27 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=n layout=11.81,26.4 limit=644,583,72,48 owner=103 paints=#EEEEF0FF part role=text size=22 text=n
              105 Button at=724,583,72,48 by=panel drawn heard=m limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=m
                106 TextBlock at=750,593,19,27 box=19,27 by=clip caps=15.3989 drawn face=Barlow heard=m layout=18.13,26.4 limit=724,583,72,48 owner=105 paints=#D4D4D4FF part role=text size=22 text=m
              107 Button at=804,583,72,48 by=panel drawn heard=, limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=,
                108 TextBlock at=838,593,5,27 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=, layout=4.49,26.4 limit=804,583,72,48 owner=107 paints=#D4D4D4FF part role=text size=22 text=,
              109 Button at=884,583,72,48 by=panel drawn heard=. limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=.
                110 TextBlock at=916,593,7,27 box=7,27 by=clip caps=15.3989 drawn face=Barlow heard=. layout=6.12,26.4 limit=884,583,72,48 owner=109 paints=#D4D4D4FF part role=text size=22 text=.
              111 Button at=964,583,72,48 by=panel drawn heard=- limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=-
                112 TextBlock at=995,593,10,27 box=10,27 by=clip caps=15.3989 drawn face=Barlow heard=- layout=9.88,26.4 limit=964,583,72,48 owner=111 paints=#D4D4D4FF part role=text size=22 text=-
              113 Button at=1044,583,152,48 by=panel drawn heard="⌫" limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text="⌫"
                114 TextBlock at=1113,593,14,29 box=14,29 by=clip caps=16.1724 drawn face="Adwaita Mono" heard="⌫" layout=13.2,28.27 limit=1044,583,152,48 owner=113 paints=#D4D4D4FF part role=text size=22 text="⌫"
            115 StackPanel at=284,639,712,48 by=panel limit=0,0,1280,800 role=none
              116 Button at=284,639,152,48 by=panel drawn heard=Symbols limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Symbols
                117 TextBlock at=320,649,79,27 box=79,27 by=clip caps=15.3989 drawn face=Barlow heard=Symbols layout=78.72,26.4 limit=284,639,152,48 owner=116 paints=#D4D4D4FF part role=text size=22 text=Symbols
              118 Button at=444,639,392,48 by=panel drawn heard=Space limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Space
                119 TextBlock at=611,649,58,27 box=58,27 by=clip caps=15.3989 drawn face=Barlow heard=Space layout=57.6,26.4 limit=444,639,392,48 owner=118 paints=#D4D4D4FF part role=text size=22 text=Space
              120 Button at=844,639,152,48 by=panel drawn heard=Done limit=0,0,1280,800 paints=#26262BFF,#D4D4D4FF role=button text=Done
                121 TextBlock at=896,649,48,27 box=48,27 by=clip caps=15.3989 drawn face=Barlow heard=Done layout=47.7,26.4 limit=844,639,152,48 owner=120 paints=#D4D4D4FF part role=text size=22 text=Done
focus none
press down focus="GalleryLayout GamesScreen" games="all keyboard" top=none
press up focus="GalleryLayout GamesScreen" games="all keyboard" top=none
press back focus="GalleryLayout GamesScreen" games="all keyboard" top=none
