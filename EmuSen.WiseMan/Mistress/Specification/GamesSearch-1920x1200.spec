# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesSearch-1920x1200
size 1920 1200
kind menu
root GalleryLayout#GamesScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamesSearch
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LunaSky at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=GallerySky nohit role=none
    2 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
        4 DockPanel at=96,51,1728,78 by=panel limit=0,0,1920,1200 role=none
          5 StackPanel at=1357.5,51,466.5,78 by=panel limit=0,0,1920,1200 name=GalleryNeighbours role=none
            6 PadGlyph at=1357.5,67.5,45,45 by=panel heard=LB limit=0,0,1920,1200 name=GalleryLeftShoulder role=image
            7 TextBlock at=1420.5,73.5,178.5,33 box=119,22 by=panel caps=12.5991 drawn face=Barlow heard="Super Nintendo" layout=118.42,21.6 limit=0,0,1920,1200 name=GalleryPrevious paints=#8D8D91FF role=text size=18 text="Super Nintendo"
            8 TextBlock at=1617,73.5,6,33 box=4,22 by=panel caps=12.5991 drawn face=Barlow heard="·" layout=3.78,21.6 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=18 text="·"
            9 TextBlock at=1641,73.5,120,33 box=80,22 by=panel caps=12.5991 drawn face=Barlow heard=Favourites layout=79.13,21.6 limit=0,0,1920,1200 name=GalleryNext paints=#8D8D91FF role=text size=18 text=Favourites
            10 PadGlyph at=1779,67.5,45,45 by=panel heard=RB limit=0,0,1920,1200 name=GalleryRightShoulder role=image
          11 StackPanel at=96,51,409.5,78 by=panel limit=0,0,1920,1200 name=GalleryHeading role=none
            12 TextBlock at=96,70.5,135,39 box=90,26 by=panel caps=13.999 drawn face=Barlow heard="GAMES ▸" layout=89.6,25.7 limit=0,0,1920,1200 name=GalleryTrail paints=#8D8D91FF role=text size=20 text="GAMES ▸"
            13 TextBlock at=255,60,73.5,58.5 box=49,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=ALL layout=48.46,38.4 limit=0,0,1920,1200 name=GallerySection paints=#EEEEF0FF role=text size=32 text=ALL
            14 TextBlock at=352.5,72,153,36 box=102,24 by=panel caps=13.999 drawn face=Barlow heard="1 of 8 games" layout=101.62,24 limit=0,0,1920,1200 name=GalleryCount paints=#8D8D91FF role=text size=20 text="1 of 8 games"
        15 TextBox at=96,150,1728,66 by=panel drawn heard=Search limit=0,0,1920,1200 name=GallerySearch paints=#0E0E10FF,#707070FF,#EEEEF0FF role=edit text=gran
          16 ScrollViewer at=121.5,151.5,1677,63 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
        17 Panel at=213,234,1494,360 by=panel heard=All limit=0,0,1920,1200 name=GalleryGrid role=none
          18 CoverCard at=213,234,270,360 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            19 Border at=222,234,253.5,360 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              20 Image at=222,234,252.34,360 by=clip drawn limit=222,234,253.5,360 nohit role=image
        21 Border at=0,1113,1920,87 by=panel limit=0,0,1920,1200 name=GalleryHelpBar role=none
          22 Panel at=0,1113,1920,87 by=panel limit=0,0,1920,1200 role=none
            23 Border at=0,1113,1920,87 by=panel limit=0,0,1920,1200 nohit paints=#0E0E10FF,#707070FF role=none
            24 HintBar at=96,1134,1728,45 by=panel heard="Type, Delete, Space, Done, Move to grid, Home" limit=0,0,1920,1200 role=group
        25 OnScreenKeyboard at=0,597,1920,456 by=panel focus heard="On-screen keyboard" limit=0,0,1920,1200 name=GalleryKeyboard paints=#1B1B1EFF,#707070FF role=none
          26 StackPanel at=126,622.5,1668,406.5 by=panel limit=0,0,1920,1200 role=none
            27 StackPanel at=366,622.5,1188,72 by=panel limit=0,0,1920,1200 role=none
              28 Button at=366,622.5,108,72 by=panel drawn heard=1 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=1
                29 TextBlock at=414,637.5,10.5,40.5 box=7,27 by=clip caps=15.3989 drawn face=Barlow heard=1 layout=6.93,26.4 limit=366,622.5,108,72 owner=28 paints=#D4D4D4FF part role=text size=22 text=1
              30 Button at=486,622.5,108,72 by=panel drawn heard=2 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=2
                31 TextBlock at=531,637.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=2 layout=11.15,26.4 limit=486,622.5,108,72 owner=30 paints=#D4D4D4FF part role=text size=22 text=2
              32 Button at=606,622.5,108,72 by=panel drawn heard=3 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=3
                33 TextBlock at=651,637.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=3 layout=10.91,26.4 limit=606,622.5,108,72 owner=32 paints=#D4D4D4FF part role=text size=22 text=3
              34 Button at=726,622.5,108,72 by=panel drawn heard=4 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=4
                35 TextBlock at=771,637.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=4 layout=11.11,26.4 limit=726,622.5,108,72 owner=34 paints=#D4D4D4FF part role=text size=22 text=4
              36 Button at=846,622.5,108,72 by=panel drawn heard=5 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=5
                37 TextBlock at=891,637.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=5 layout=11.18,26.4 limit=846,622.5,108,72 owner=36 paints=#D4D4D4FF part role=text size=22 text=5
              38 Button at=966,622.5,108,72 by=panel drawn heard=6 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=6
                39 TextBlock at=1011,637.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=6 layout=10.96,26.4 limit=966,622.5,108,72 owner=38 paints=#D4D4D4FF part role=text size=22 text=6
              40 Button at=1086,622.5,108,72 by=panel drawn heard=7 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=7
                41 TextBlock at=1131,637.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=7 layout=10.32,26.4 limit=1086,622.5,108,72 owner=40 paints=#D4D4D4FF part role=text size=22 text=7
              42 Button at=1206,622.5,108,72 by=panel drawn heard=8 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=8
                43 TextBlock at=1251,637.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=8 layout=11.35,26.4 limit=1206,622.5,108,72 owner=42 paints=#D4D4D4FF part role=text size=22 text=8
              44 Button at=1326,622.5,108,72 by=panel drawn heard=9 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=9
                45 TextBlock at=1371,637.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=9 layout=11.46,26.4 limit=1326,622.5,108,72 owner=44 paints=#D4D4D4FF part role=text size=22 text=9
              46 Button at=1446,622.5,108,72 by=panel drawn heard=0 limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=0
                47 TextBlock at=1491,637.5,19.5,40.5 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=0 layout=12.03,26.4 limit=1446,622.5,108,72 owner=46 paints=#D4D4D4FF part role=text size=22 text=0
            48 StackPanel at=366,706.5,1188,72 by=panel limit=0,0,1920,1200 role=none
              49 Button at=366,706.5,108,72 by=panel drawn heard=q limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=q
                50 TextBlock at=411,721.5,19.5,40.5 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=q layout=12.14,26.4 limit=366,706.5,108,72 owner=49 paints=#D4D4D4FF part role=text size=22 text=q
              51 Button at=486,706.5,108,72 by=panel drawn heard=w limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=w
                52 TextBlock at=528,721.5,24,40.5 box=16,27 by=clip caps=15.3989 drawn face=Barlow heard=w layout=15.88,26.4 limit=486,706.5,108,72 owner=51 paints=#D4D4D4FF part role=text size=22 text=w
              53 Button at=606,706.5,108,72 by=panel drawn heard=e limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=e
                54 TextBlock at=651,721.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=e layout=11.09,26.4 limit=606,706.5,108,72 owner=53 paints=#D4D4D4FF part role=text size=22 text=e
              55 Button at=726,706.5,108,72 by=panel drawn heard=r limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=r
                56 TextBlock at=774,721.5,12,40.5 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=r layout=7.99,26.4 limit=726,706.5,108,72 owner=55 paints=#D4D4D4FF part role=text size=22 text=r
              57 Button at=846,706.5,108,72 by=panel drawn heard=t limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=t
                58 TextBlock at=894,721.5,12,40.5 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=t layout=7.13,26.4 limit=846,706.5,108,72 owner=57 paints=#D4D4D4FF part role=text size=22 text=t
              59 Button at=966,706.5,108,72 by=panel drawn heard=y limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=y
                60 TextBlock at=1011,721.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=y layout=10.34,26.4 limit=966,706.5,108,72 owner=59 paints=#D4D4D4FF part role=text size=22 text=y
              61 Button at=1086,706.5,108,72 by=panel drawn heard=u limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=u
                62 TextBlock at=1131,721.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=u layout=11.7,26.4 limit=1086,706.5,108,72 owner=61 paints=#D4D4D4FF part role=text size=22 text=u
              63 Button at=1206,706.5,108,72 by=panel drawn heard=i limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=i
                64 TextBlock at=1255.5,721.5,9,40.5 box=6,27 by=clip caps=15.3989 drawn face=Barlow heard=i layout=5.5,26.4 limit=1206,706.5,108,72 owner=63 paints=#D4D4D4FF part role=text size=22 text=i
              65 Button at=1326,706.5,108,72 by=panel drawn heard=o limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=o
                66 TextBlock at=1371,721.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=o layout=11.46,26.4 limit=1326,706.5,108,72 owner=65 paints=#D4D4D4FF part role=text size=22 text=o
              67 Button at=1446,706.5,108,72 by=panel drawn heard=p limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=p
                68 TextBlock at=1491,721.5,19.5,40.5 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=p layout=12.14,26.4 limit=1446,706.5,108,72 owner=67 paints=#D4D4D4FF part role=text size=22 text=p
            69 StackPanel at=366,790.5,1188,72 by=panel limit=0,0,1920,1200 role=none
              70 Button at=366,790.5,108,72 by=panel drawn heard=a limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=a
                71 TextBlock at=411,805.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=a layout=11.24,26.4 limit=366,790.5,108,72 owner=70 paints=#D4D4D4FF part role=text size=22 text=a
              72 Button at=486,790.5,108,72 by=panel drawn heard=s limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=s
                73 TextBlock at=531,805.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=s layout=10.1,26.4 limit=486,790.5,108,72 owner=72 paints=#D4D4D4FF part role=text size=22 text=s
              74 Button at=606,790.5,108,72 by=panel drawn heard=d limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=d
                75 TextBlock at=651,805.5,19.5,40.5 box=13,27 by=clip caps=15.3989 drawn face=Barlow heard=d layout=12.01,26.4 limit=606,790.5,108,72 owner=74 paints=#D4D4D4FF part role=text size=22 text=d
              76 Button at=726,790.5,108,72 by=panel drawn heard=f limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=f
                77 TextBlock at=774,805.5,12,40.5 box=8,27 by=clip caps=15.3989 drawn face=Barlow heard=f layout=7.17,26.4 limit=726,790.5,108,72 owner=76 paints=#D4D4D4FF part role=text size=22 text=f
              78 Button at=846,790.5,108,72 by=panel drawn heard=g limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=g
                79 TextBlock at=891,805.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=g layout=11.66,26.4 limit=846,790.5,108,72 owner=78 paints=#D4D4D4FF part role=text size=22 text=g
              80 Button at=966,790.5,108,72 by=panel drawn heard=h limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=h
                81 TextBlock at=1011,805.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=h layout=11.81,26.4 limit=966,790.5,108,72 owner=80 paints=#D4D4D4FF part role=text size=22 text=h
              82 Button at=1086,790.5,108,72 by=panel drawn heard=j limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=j
                83 TextBlock at=1135.5,805.5,9,40.5 box=6,27 by=clip caps=15.3989 drawn face=Barlow heard=j layout=5.06,26.4 limit=1086,790.5,108,72 owner=82 paints=#D4D4D4FF part role=text size=22 text=j
              84 Button at=1206,790.5,108,72 by=panel drawn heard=k limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=k
                85 TextBlock at=1251,805.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=k layout=10.82,26.4 limit=1206,790.5,108,72 owner=84 paints=#D4D4D4FF part role=text size=22 text=k
              86 Button at=1326,790.5,108,72 by=panel drawn heard=l limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=l
                87 TextBlock at=1377,805.5,7.5,40.5 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=l layout=4.93,26.4 limit=1326,790.5,108,72 owner=86 paints=#D4D4D4FF part role=text size=22 text=l
              88 Button at=1446,790.5,108,72 by=panel drawn heard=' limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text='
                89 TextBlock at=1497,805.5,7.5,40.5 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=' layout=4.31,26.4 limit=1446,790.5,108,72 owner=88 paints=#D4D4D4FF part role=text size=22 text='
            90 StackPanel at=126,874.5,1668,72 by=panel limit=0,0,1920,1200 role=none
              91 Button at=126,874.5,228,72 by=panel drawn heard=Shift limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=Shift
                92 TextBlock at=207,889.5,66,40.5 box=44,27 by=clip caps=15.3989 drawn face=Barlow heard=Shift layout=43.71,26.4 limit=126,874.5,228,72 owner=91 paints=#D4D4D4FF part role=text size=22 text=Shift
              93 Button at=366,874.5,108,72 by=panel drawn heard=z limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=z
                94 TextBlock at=411,889.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=z layout=10.12,26.4 limit=366,874.5,108,72 owner=93 paints=#D4D4D4FF part role=text size=22 text=z
              95 Button at=486,874.5,108,72 by=panel drawn heard=x limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=x
                96 TextBlock at=531,889.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=x layout=10.52,26.4 limit=486,874.5,108,72 owner=95 paints=#D4D4D4FF part role=text size=22 text=x
              97 Button at=606,874.5,108,72 by=panel drawn heard=c limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=c
                98 TextBlock at=651,889.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=c layout=11.02,26.4 limit=606,874.5,108,72 owner=97 paints=#D4D4D4FF part role=text size=22 text=c
              99 Button at=726,874.5,108,72 by=panel drawn heard=v limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=v
                100 TextBlock at=771,889.5,16.5,40.5 box=11,27 by=clip caps=15.3989 drawn face=Barlow heard=v layout=10.43,26.4 limit=726,874.5,108,72 owner=99 paints=#D4D4D4FF part role=text size=22 text=v
              101 Button at=846,874.5,108,72 by=panel drawn heard=b limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=b
                102 TextBlock at=891,889.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=b layout=11.66,26.4 limit=846,874.5,108,72 owner=101 paints=#D4D4D4FF part role=text size=22 text=b
              103 Button at=966,874.5,108,72 by=panel drawn heard=n limit=0,0,1920,1200 paints=#26262BFF,#4C9AE8FF,#EEEEF0FF role=button text=n
                104 TextBlock at=1011,889.5,18,40.5 box=12,27 by=clip caps=15.3989 drawn face=Barlow heard=n layout=11.81,26.4 limit=966,874.5,108,72 owner=103 paints=#EEEEF0FF part role=text size=22 text=n
              105 Button at=1086,874.5,108,72 by=panel drawn heard=m limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=m
                106 TextBlock at=1125,889.5,28.5,40.5 box=19,27 by=clip caps=15.3989 drawn face=Barlow heard=m layout=18.13,26.4 limit=1086,874.5,108,72 owner=105 paints=#D4D4D4FF part role=text size=22 text=m
              107 Button at=1206,874.5,108,72 by=panel drawn heard=, limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=,
                108 TextBlock at=1257,889.5,7.5,40.5 box=5,27 by=clip caps=15.3989 drawn face=Barlow heard=, layout=4.49,26.4 limit=1206,874.5,108,72 owner=107 paints=#D4D4D4FF part role=text size=22 text=,
              109 Button at=1326,874.5,108,72 by=panel drawn heard=. limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=.
                110 TextBlock at=1374,889.5,10.5,40.5 box=7,27 by=clip caps=15.3989 drawn face=Barlow heard=. layout=6.12,26.4 limit=1326,874.5,108,72 owner=109 paints=#D4D4D4FF part role=text size=22 text=.
              111 Button at=1446,874.5,108,72 by=panel drawn heard=- limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=-
                112 TextBlock at=1492.5,889.5,15,40.5 box=10,27 by=clip caps=15.3989 drawn face=Barlow heard=- layout=9.88,26.4 limit=1446,874.5,108,72 owner=111 paints=#D4D4D4FF part role=text size=22 text=-
              113 Button at=1566,874.5,228,72 by=panel drawn heard="⌫" limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text="⌫"
                114 TextBlock at=1669.5,889.5,21,43.5 box=14,29 by=clip caps=16.1724 drawn face="Adwaita Mono" heard="⌫" layout=13.2,28.27 limit=1566,874.5,228,72 owner=113 paints=#D4D4D4FF part role=text size=22 text="⌫"
            115 StackPanel at=426,958.5,1068,72 by=panel limit=0,0,1920,1200 role=none
              116 Button at=426,958.5,228,72 by=panel drawn heard=Symbols limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=Symbols
                117 TextBlock at=480,973.5,118.5,40.5 box=79,27 by=clip caps=15.3989 drawn face=Barlow heard=Symbols layout=78.72,26.4 limit=426,958.5,228,72 owner=116 paints=#D4D4D4FF part role=text size=22 text=Symbols
              118 Button at=666,958.5,588,72 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=Space
                119 TextBlock at=916.5,973.5,87,40.5 box=58,27 by=clip caps=15.3989 drawn face=Barlow heard=Space layout=57.6,26.4 limit=666,958.5,588,72 owner=118 paints=#D4D4D4FF part role=text size=22 text=Space
              120 Button at=1266,958.5,228,72 by=panel drawn heard=Done limit=0,0,1920,1200 paints=#26262BFF,#D4D4D4FF role=button text=Done
                121 TextBlock at=1344,973.5,72,40.5 box=48,27 by=clip caps=15.3989 drawn face=Barlow heard=Done layout=47.7,26.4 limit=1266,958.5,228,72 owner=120 paints=#D4D4D4FF part role=text size=22 text=Done
focus none
press down focus="GalleryLayout GamesScreen" games="all keyboard" top=none
press up focus="GalleryLayout GamesScreen" games="all keyboard" top=none
press back focus="GalleryLayout GamesScreen" games="all keyboard" top=none
