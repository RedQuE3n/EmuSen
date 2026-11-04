# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageCover-1920x1200
size 1920 1200
kind menu
root PageLayout#GamePage
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamePageCover
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 PosterSlot at=0,0,1920,540 by=panel heard=Poster limit=0,0,1920,1200 name=PagePoster nohit role=image
        4 Border at=96,270,301.5,402 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1920,1200 name=PageCover paints=#26262BFF role=none
          5 Panel at=96,270,301.5,402 by=clip limit=96,270,301.5,402 role=none
            6 Image at=96,270,301.5,402 by=clip drawn limit=96,270,301.5,402 nohit role=image
        7 Viewbox at=504,339,552,79.5 by=panel limit=0,0,1920,1200 role=none
          8 ViewboxContainer at=504,339,552,79.5 by=clip limit=504,339,552,79.5 role=none
            9 TextBlock at=504,339,552,79.5 box=368,53 by=clip caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=504,339,552,79.5 name=PageTitle paints=#FFFFFFFF role=text size=44 text="Cobalt Harbor (Synthetic)"
        10 Viewbox at=504,429,763.5,40.5 by=panel limit=0,0,1920,1200 role=none
          11 ViewboxContainer at=504,429,763.5,40.5 by=clip limit=504,429,763.5,40.5 role=none
            12 TextBlock at=504,429,763.5,40.5 box=509,27 by=clip caps=15.3989 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=508.11,26.4 limit=504,429,763.5,40.5 name=PageFacts paints=#FFFFFFFF role=text size=22 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
        13 SelectionStack at=504,492,1320,66 by=panel heard=Actions limit=0,0,1920,1200 name=PageActions role=list
          14 SelectableCell at=504,492,172.5,66 by=panel focus heard=Play limit=0,0,1920,1200 role=listitem
            15 Border at=504,492,172.5,66 by=panel limit=0,0,1920,1200 name=PageAction.play paints=#4C9AE8FF role=none
              16 StackPanel at=516,501,130.5,48 by=panel limit=0,0,1920,1200 role=none
                17 PadGlyph at=516,502.5,45,45 by=panel heard=A limit=0,0,1920,1200 role=image
                18 TextBlock at=576,501,70.5,48 box=47,32 by=panel caps=18.1987 drawn face=Barlow heard=Play layout=46.7,31.2 limit=0,0,1920,1200 paints=#0E0E10FF role=text size=26 text=Play
          19 SelectableCell at=694.5,492,535.5,66 by=panel focus heard="Resume, Auto · 12 min ago" limit=0,0,1920,1200 role=listitem
            20 Border at=694.5,492,535.5,66 by=panel limit=0,0,1920,1200 name=PageAction.resume paints=#707070FF role=none
              21 StackPanel at=726,501,472.5,48 by=panel limit=0,0,1920,1200 role=none
                22 TextBlock at=726,501,133.5,48 box=89,32 by=panel caps=18.1987 drawn face=Barlow heard=Resume layout=88.92,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Resume
                23 Path at=874.5,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
                24 TextBlock at=900,501,273,48 box=182,32 by=panel caps=18.1987 drawn face=Barlow heard="Auto · 12 min ago" layout=181.19,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text="Auto · 12 min ago"
                25 Path at=1188,516,10.5,18 by=panel limit=0,0,1920,1200 nohit role=none
          26 SelectableCell at=1248,492,307.5,66 by=panel focus heard=Favourite limit=0,0,1920,1200 role=listitem
            27 Border at=1248,492,307.5,66 by=panel limit=0,0,1920,1200 name=PageAction.favourite paints=#707070FF role=none
              28 StackPanel at=1261.5,501,262.5,48 by=panel limit=0,0,1920,1200 role=none
                29 PadGlyph at=1261.5,502.5,45,45 by=panel heard=Y limit=0,0,1920,1200 role=image
                30 Path at=1321.5,508.5,33,33 by=panel limit=0,0,1920,1200 nohit role=none
                31 TextBlock at=1369.5,501,154.5,48 box=103,32 by=panel caps=18.1987 drawn face=Barlow heard=Favourite layout=102.36,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Favourite
        32 WholeRow at=504,594,1320,66 by=panel heard=Sections limit=0,0,1920,1200 name=PageSections role=tab
          33 SectionTab at=504,594,154.5,66 by=panel drawn focus heard=Details limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=DETAILS
            34 AccessText at=528,606,106.5,37.5 box=71,25 by=clip caps=13.999 drawn face=Barlow layout=70.98,24 limit=504,594,154.5,66 owner=33 paints=#D4D4D4FF role=none size=20 text=DETAILS
          35 SectionTab at=670.5,594,192,66 by=panel drawn focus heard="Cover Art" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="COVER ART"
            36 AccessText at=694.5,606,144,37.5 box=96,25 by=clip caps=13.999 drawn face=Barlow layout=95.68,24 limit=670.5,594,192,66 owner=35 paints=#EEEEF0FF role=none size=20 text="COVER ART"
        37 Border at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageContent role=none
          38 StackPanel at=96,696,1728,357 by=panel limit=0,0,1920,1200 name=PageTiles role=none
            39 StackPanel at=96,696,225,357 by=panel limit=0,0,1920,1200 role=none
              40 Panel at=96,696,225,264 by=panel heard="Its own" limit=0,0,1920,1200 name=PageTile.own role=none
                41 Border at=96,696,225,264 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  42 Panel at=96,696,225,264 by=clip limit=96,696,225,264 role=none
                    43 Image at=96,696,225,264 by=clip drawn limit=96,696,225,264 nohit role=image
                44 Ellipse at=279,708,30,30 by=panel limit=0,0,1920,1200 nohit role=none
                45 Border at=85.5,685.5,246,285 by=panel limit=0,0,1920,1200 nohit paints=#4C9AE8FF role=none
              46 TextBlock at=96,978,225,36 box=150,24 by=panel caps=13.999 drawn face=Barlow heard="Its own" layout=60.12,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text="Its own"
            47 StackPanel at=357,696,225,357 by=panel limit=0,0,1920,1200 role=none
              48 Panel at=357,696,225,264 by=panel heard="Borrow from Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="PageTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
                49 Border at=357,696,225,264 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  50 Panel at=357,696,225,264 by=clip limit=357,696,225,264 role=none
                    51 Image at=357,696,225,264 by=clip drawn limit=357,696,225,264 nohit role=image
              52 TextBlock at=357,978,225,72 box=150,48 by=panel caps=13.999 drawn face=Barlow heard="Borrow from Aurora Drift (Synthetic)" layout=106.44,48 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Borrow from Aurora Drift (Synthetic)"
            53 StackPanel at=618,696,225,357 by=panel limit=0,0,1920,1200 role=none
              54 Panel at=618,696,225,264 by=panel heard="Borrow from Brass Lantern (Synthetic)" limit=0,0,1920,1200 name="PageTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
                55 Border at=618,696,225,264 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  56 Panel at=618,696,225,264 by=clip limit=618,696,225,264 role=none
                    57 Image at=618,696,225,264 by=clip drawn limit=618,696,225,264 nohit role=image
              58 TextBlock at=618,978,225,72 box=150,48 by=panel caps=13.999 drawn face=Barlow heard="Borrow from Brass Lantern (Synthetic)" layout=117.88,48 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Borrow from Brass Lantern (Synthetic)"
            59 StackPanel at=879,696,225,357 by=panel limit=0,0,1920,1200 role=none
              60 Panel at=879,696,225,264 by=panel heard="Borrow from Dune Relay (Synthetic)" limit=0,0,1920,1200 name="PageTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
                61 Border at=879,696,225,264 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  62 Panel at=879,696,225,264 by=clip limit=879,696,225,264 role=none
                    63 Image at=879,696,225,264 by=clip drawn limit=879,696,225,264 nohit role=image
              64 TextBlock at=879,978,225,72 box=150,48 by=panel caps=13.999 drawn face=Barlow heard="Borrow from Dune Relay (Synthetic)" layout=106.44,48 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Borrow from Dune Relay (Synthetic)"
            65 StackPanel at=1140,696,225,357 by=panel limit=0,0,1920,1200 role=none
              66 Panel at=1140,696,225,264 by=panel heard="Borrow from Ember Circuit (Synthetic)" limit=0,0,1920,1200 name="PageTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
                67 Border at=1140,696,225,264 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  68 Panel at=1140,696,225,264 by=clip limit=1140,696,225,264 role=none
                    69 Image at=1140,696,225,264 by=clip drawn limit=1140,696,225,264 nohit role=image
              70 TextBlock at=1140,978,225,72 box=150,48 by=panel caps=13.999 drawn face=Barlow heard="Borrow from Ember Circuit (Synthetic)" layout=115.78,48 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Borrow from Ember Circuit (Synthetic)"
            71 StackPanel at=1401,696,225,357 by=panel limit=0,0,1920,1200 role=none
              72 Panel at=1401,696,225,264 by=panel heard="Borrow from Fable of Tiles (Synthetic)" limit=0,0,1920,1200 name="PageTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
                73 Border at=1401,696,225,264 by=panel limit=0,0,1920,1200 paints=#26262BFF role=none
                  74 Panel at=1401,696,225,264 by=clip limit=1401,696,225,264 role=none
                    75 Image at=1401,696,225,264 by=clip drawn limit=1401,696,225,264 nohit role=image
              76 TextBlock at=1401,978,225,72 box=150,48 by=panel caps=13.999 drawn face=Barlow heard="Borrow from Fable of Tiles (Synthetic)" layout=108.62,48 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="Borrow from Fable of Tiles (Synthetic)"
        77 Border at=0,1053,1920,87 by=panel limit=0,0,1920,1200 name=PageHelpBar paints=#1B1B1EFF,#707070FF role=none
          78 HintBar at=96,1069.5,1728,61.5 by=panel heard="Use this one, Favourite, Details, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="PageLayout GamePage" page="Cover Art"
press up focus="SelectableCell  (Play)" page="Cover Art"
press back focus="MenuRow PageRow.cover" page=details
