# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Consoles-1920x1200
size 1920 1200
kind menu
root PlanetLayout#ConsolesScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Consoles
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LunaSky at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=PlanetSky nohit role=none
    2 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
        4 DockPanel at=96,60,1728,66 by=panel limit=0,0,1920,1200 role=none
          5 TextBlock at=1566,82.5,258,36 box=172,24 by=panel caps=13.999 drawn face=Barlow heard="3 consoles · 8 games" layout=171.52,24 limit=0,0,1920,1200 name=PlanetCount paints=#8D8D91FF role=text size=20 text="3 consoles · 8 games"
          6 StackPanel at=96,60,1434,66 by=panel limit=0,0,1920,1200 name=PlanetHeader role=none
            7 TextBlock at=96,67.5,129,58.5 box=86,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=GAMES layout=85.18,38.4 limit=0,0,1920,1200 name=PlanetTitle paints=#EEEEF0FF role=text size=32 text=GAMES
            8 TextBlock at=255,82.5,219,36 box=146,24 by=panel caps=13.999 drawn face=Barlow heard="Choose a console" layout=145.78,24 limit=0,0,1920,1200 name=PlanetSubtitle paints=#8D8D91FF role=text size=20 text="Choose a console"
        9 Panel at=96,150,558,108 by=panel heard="All games, 8 games" limit=0,0,1920,1200 name=PlanetTile.all role=none
          10 Border at=96,150,558,108 by=panel limit=0,0,1920,1200 name=PlanetGlass nohit paints=#26262BFF role=none
          11 StackPanel at=126,165,498,79.5 by=panel limit=0,0,1920,1200 role=none
            12 Panel at=126,174,60,60 by=panel limit=0,0,1920,1200 role=none
              13 Border at=126,174,60,60 by=panel limit=0,0,1920,1200 nohit paints=#EEEEF0FF role=none
              14 Viewbox at=139.5,187.5,33,33 by=panel limit=0,0,1920,1200 role=none
                15 ViewboxContainer at=139.5,187.5,33,33 by=clip limit=139.5,187.5,33,33 role=none
                  16 Path at=139.5,187.5,33,33 by=clip limit=139.5,187.5,33,33 role=none
            17 StackPanel at=210,165,151.5,79.5 by=panel limit=0,0,1920,1200 role=none
              18 TextBlock at=210,165,151.5,43.5 box=101,29 by=panel caps=16.7988 drawn face="Barlow Condensed Medium" heard="ALL GAMES" layout=100.52,28.8 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=24 text="ALL GAMES"
              19 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=210,211.5,151.5,33 box=101,22 caps=12.5991 drawn face=Barlow heard="8 games" layout=63.79,21.6 name=PlanetStatus paints=#8D8D91FF role=text size=18 text="8 games"
        20 Panel at=681,150,558,108 by=panel heard="Favourites, Your hearts, every console" limit=0,0,1920,1200 name=PlanetTile.favourites role=none
          21 Border at=681,150,558,108 by=panel limit=0,0,1920,1200 name=PlanetGlass nohit paints=#26262BFF role=none
          22 StackPanel at=711,165,498,79.5 by=panel limit=0,0,1920,1200 role=none
            23 Panel at=711,174,60,60 by=panel limit=0,0,1920,1200 role=none
              24 Border at=711,174,60,60 by=panel limit=0,0,1920,1200 nohit paints=#EEEEF0FF role=none
              25 Viewbox at=724.5,187.5,33,33 by=panel limit=0,0,1920,1200 role=none
                26 ViewboxContainer at=724.5,187.5,33,33 by=clip limit=724.5,187.5,33,33 role=none
                  27 Path at=724.5,187.5,33,33 by=clip limit=724.5,187.5,33,33 role=none
            28 StackPanel at=795,165,298.5,79.5 by=panel limit=0,0,1920,1200 role=none
              29 TextBlock at=795,165,298.5,43.5 box=199,29 by=panel caps=16.7988 drawn face="Barlow Condensed Medium" heard=FAVOURITES layout=108.7,28.8 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=24 text=FAVOURITES
              30 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=795,211.5,298.5,33 box=199,22 caps=12.5991 drawn face=Barlow heard="Your hearts, every console" layout=198.56,21.6 name=PlanetStatus paints=#8D8D91FF role=text size=18 text="Your hearts, every console"
        31 Panel at=1266,150,558,108 by=panel heard="Recently played, Last: Aurora Drift (Synthetic)" limit=0,0,1920,1200 name=PlanetTile.recent role=none
          32 Border at=1266,150,558,108 by=panel limit=0,0,1920,1200 name=PlanetGlass nohit paints=#26262BFF role=none
          33 StackPanel at=1296,165,498,79.5 by=panel limit=0,0,1920,1200 role=none
            34 Panel at=1296,174,60,60 by=panel limit=0,0,1920,1200 role=none
              35 Border at=1296,174,60,60 by=panel limit=0,0,1920,1200 nohit paints=#EEEEF0FF role=none
              36 Viewbox at=1309.5,187.5,33,33 by=panel limit=0,0,1920,1200 role=none
                37 ViewboxContainer at=1309.5,187.5,33,33 by=clip limit=1309.5,187.5,33,33 role=none
                  38 Path at=1309.5,187.5,33,33 by=clip limit=1309.5,187.5,33,33 role=none
            39 StackPanel at=1380,165,325.5,79.5 by=panel limit=0,0,1920,1200 role=none
              40 TextBlock at=1380,165,325.5,43.5 box=217,29 by=panel caps=16.7988 drawn face="Barlow Condensed Medium" heard="RECENTLY PLAYED" layout=165.9,28.8 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=24 text="RECENTLY PLAYED"
              41 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=1380,211.5,325.5,33 box=217,22 caps=12.5991 drawn face=Barlow heard="Last: Aurora Drift (Synthetic)" layout=216.41,21.6 name=PlanetStatus paints=#8D8D91FF role=text size=18 text="Last: Aurora Drift (Synthetic)"
        42 Border at=0,384,1920,1.5 by=panel limit=0,0,1920,1200 name=PlanetOrbit nohit role=none
        43 Panel at=537,285,264,264 by=panel heard="NES, 2 games" limit=0,0,1920,1200 name="PlanetCell.NES (Moon)" role=none
          44 Border at=537,285,264,264 by=panel limit=0,0,1920,1200 name=PlanetGlass nohit paints=#26262BFF role=none
          45 StackPanel at=549,285,240,243 by=panel limit=0,0,1920,1200 role=none
            46 Panel at=549,285,240,180 by=panel limit=0,0,1920,1200 role=none
              47 PlanetGlyph at=606,312,126,126 by=panel limit=0,0,1920,1200 name="Planet.NES (Moon)" nohit role=none
            48 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=645,468,49.5,40.5 box=33,27 caps=15.3989 drawn face="Barlow Condensed Medium" heard=NES layout=32.07,26.4 name=PlanetStatus paints=#EEEEF0FF role=text size=22 text=NES
            49 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=624,511.5,91.5,31.5 box=61,21 caps=11.8992 drawn face=Barlow heard="2 games" layout=60.1,20.4 name=PlanetStatus paints=#8D8D91FF role=text size=17 text="2 games"
          50 Border at=526.5,274.5,285,285 by=panel limit=0,0,1920,1200 name=HubRing nohit paints=#4C9AE8FF role=none
        51 Panel at=828,294,264,264 by=panel heard="Game Boy, 1 game" limit=0,0,1920,1200 name="PlanetCell.Game Boy (Mercury)" role=none
          52 StackPanel at=840,294,240,243 by=panel limit=0,0,1920,1200 role=none
            53 Panel at=840,294,240,180 by=panel limit=0,0,1920,1200 role=none
              54 PlanetGlyph at=916.5,340.5,87,87 by=panel limit=0,0,1920,1200 name="Planet.Game Boy (Mercury)" nohit role=none
            55 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=898.5,477,123,40.5 box=82,27 caps=15.3989 drawn face="Barlow Condensed Medium" heard="GAME BOY" layout=81.47,26.4 name=PlanetStatus paints=#EEEEF0FF role=text size=22 text="GAME BOY"
            56 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=922.5,520.5,75,31.5 box=50,21 caps=11.8992 drawn face=Barlow heard="1 game" layout=49.03,20.4 name=PlanetStatus paints=#8D8D91FF role=text size=17 text="1 game"
        57 Panel at=1119,294,264,264 by=panel heard="Super Nintendo, 5 games" limit=0,0,1920,1200 name="PlanetCell.SNES (Venus)" role=none
          58 StackPanel at=1131,294,240,243 by=panel limit=0,0,1920,1200 role=none
            59 Panel at=1131,294,240,180 by=panel limit=0,0,1920,1200 role=none
              60 PlanetGlyph at=1207.5,340.5,87,87 by=panel limit=0,0,1920,1200 name="Planet.SNES (Venus)" nohit role=none
            61 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=1143,477,214.5,40.5 box=143,27 caps=15.3989 drawn face="Barlow Condensed Medium" heard="SUPER NINTENDO" layout=142,26.4 name=PlanetStatus paints=#EEEEF0FF role=text size=22 text="SUPER NINTENDO"
            62 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=1206,520.5,91.5,31.5 box=61,21 caps=11.8992 drawn face=Barlow heard="5 games" layout=60.11,20.4 name=PlanetStatus paints=#8D8D91FF role=text size=17 text="5 games"
        63 Panel at=96,879,1728,156 by=panel limit=0,0,1920,1200 role=none
          64 StackPanel at=96,910.5,856.5,115.5 by=panel limit=0,0,1920,1200 role=none
            65 TextBlock at=96,910.5,856.5,66 box=571,44 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="NINTENDO ENTERTAINMENT SYSTEM" layout=570.51,44 limit=0,0,1920,1200 name=PlanetFocusTitle paints=#EEEEF0FF role=text size=44 text="NINTENDO ENTERTAINMENT SYSTEM"
            66 TextBlock at=96,985.5,856.5,40.5 box=571,27 by=panel caps=15.3989 drawn face=Barlow heard="Nintendo · 1983 · 2 games" layout=230.58,26.4 limit=0,0,1920,1200 name=PlanetFocusFacts paints=#8D8D91FF role=text size=22 text="Nintendo · 1983 · 2 games"
        67 Panel at=1194,879,630,156 by=panel limit=0,0,1920,1200 name=PlanetFan role=none
          68 CoverCard at=1605.41,883.19,111.67,153.63 by=panel heard="Fable of Tiles (Synthetic)" limit=0,0,1920,1200 name="PlanetCover.<run>/Roms/Fable of Tiles (Synthetic).nes" nohit role=none
            69 Border at=1605.41,883.19,111.67,153.63 by=panel limit=0,0,1920,1200 name=CoverBox nohit paints=#26262BFF role=none
              70 Image at=1605.41,883.19,110.31,153.58 by=clip drawn limit=1605.41,883.19,111.67,153.63 nohit role=image
          71 CoverCard at=1711.45,879.84,121.6,160.31 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1920,1200 name="PlanetCover.<run>/Roms/Granite Choir (Synthetic).nes" nohit role=none
            72 Border at=1711.45,879.84,121.6,160.31 by=panel limit=0,0,1920,1200 name=CoverBox nohit paints=#26262BFF role=none
              73 Image at=1711.45,879.84,120.24,160.17 by=clip drawn limit=1711.45,879.84,121.6,160.31 nohit role=image
        74 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=PlanetHelpBar role=none
          75 Panel at=0,1053,1920,147 by=panel limit=0,0,1920,1200 role=none
            76 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 nohit paints=#0E0E10FF,#707070FF role=none
            77 HintBar at=96,1074,1728,45 by=panel heard="Open, Back, Favourites, Random game" limit=0,0,1920,1200 role=group
focus none
press down consoles="NES (Moon)" focus="PlanetLayout ConsolesScreen" top=none
press up consoles=favourites focus="PlanetLayout ConsolesScreen" top=none
press back focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
