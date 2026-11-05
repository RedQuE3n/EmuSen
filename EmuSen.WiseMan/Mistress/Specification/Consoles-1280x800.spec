# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Consoles-1280x800
size 1280 800
kind menu
root PlanetLayout#ConsolesScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Consoles
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LunaSky at=0,0,1280,800 by=panel limit=0,0,1280,800 name=PlanetSky nohit role=none
    2 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
        4 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
          5 TextBlock at=1044,55,172,24 box=172,24 by=panel caps=13.999 drawn face=Barlow heard="3 consoles · 8 games" layout=171.52,24 limit=0,0,1280,800 name=PlanetCount paints=#8D8D91FF role=text size=20 text="3 consoles · 8 games"
          6 StackPanel at=64,40,956,44 by=panel limit=0,0,1280,800 name=PlanetHeader role=none
            7 TextBlock at=64,45,86,39 box=86,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=GAMES layout=85.18,38.4 limit=0,0,1280,800 name=PlanetTitle paints=#EEEEF0FF role=text size=32 text=GAMES
            8 TextBlock at=170,55,146,24 box=146,24 by=panel caps=13.999 drawn face=Barlow heard="Choose a console" layout=145.78,24 limit=0,0,1280,800 name=PlanetSubtitle paints=#8D8D91FF role=text size=20 text="Choose a console"
        9 Panel at=64,100,372,72 by=panel heard="All games, 8 games" limit=0,0,1280,800 name=PlanetTile.all role=none
          10 Border at=64,100,372,72 by=panel limit=0,0,1280,800 name=PlanetGlass nohit paints=#26262BFF role=none
          11 StackPanel at=84,110,332,53 by=panel limit=0,0,1280,800 role=none
            12 Panel at=84,116,40,40 by=panel limit=0,0,1280,800 role=none
              13 Border at=84,116,40,40 by=panel limit=0,0,1280,800 nohit paints=#EEEEF0FF role=none
              14 Viewbox at=93,125,22,22 by=panel limit=0,0,1280,800 role=none
                15 ViewboxContainer at=93,125,22,22 by=clip limit=93,125,22,22 role=none
                  16 Path at=93,125,22,22 by=clip limit=93,125,22,22 role=none
            17 StackPanel at=140,110,101,53 by=panel limit=0,0,1280,800 role=none
              18 TextBlock at=140,110,101,29 box=101,29 by=panel caps=16.7988 drawn face="Barlow Condensed Medium" heard="ALL GAMES" layout=100.52,28.8 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=24 text="ALL GAMES"
              19 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=140,141,101,22 box=101,22 caps=12.5991 drawn face=Barlow heard="8 games" layout=63.79,21.6 name=PlanetStatus paints=#8D8D91FF role=text size=18 text="8 games"
        20 Panel at=454,100,372,72 by=panel heard="Favourites, Your hearts, every console" limit=0,0,1280,800 name=PlanetTile.favourites role=none
          21 Border at=454,100,372,72 by=panel limit=0,0,1280,800 name=PlanetGlass nohit paints=#26262BFF role=none
          22 StackPanel at=474,110,332,53 by=panel limit=0,0,1280,800 role=none
            23 Panel at=474,116,40,40 by=panel limit=0,0,1280,800 role=none
              24 Border at=474,116,40,40 by=panel limit=0,0,1280,800 nohit paints=#EEEEF0FF role=none
              25 Viewbox at=483,125,22,22 by=panel limit=0,0,1280,800 role=none
                26 ViewboxContainer at=483,125,22,22 by=clip limit=483,125,22,22 role=none
                  27 Path at=483,125,22,22 by=clip limit=483,125,22,22 role=none
            28 StackPanel at=530,110,199,53 by=panel limit=0,0,1280,800 role=none
              29 TextBlock at=530,110,199,29 box=199,29 by=panel caps=16.7988 drawn face="Barlow Condensed Medium" heard=FAVOURITES layout=108.7,28.8 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=24 text=FAVOURITES
              30 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=530,141,199,22 box=199,22 caps=12.5991 drawn face=Barlow heard="Your hearts, every console" layout=198.56,21.6 name=PlanetStatus paints=#8D8D91FF role=text size=18 text="Your hearts, every console"
        31 Panel at=844,100,372,72 by=panel heard="Recently played, Last: Aurora Drift (Synthetic)" limit=0,0,1280,800 name=PlanetTile.recent role=none
          32 Border at=844,100,372,72 by=panel limit=0,0,1280,800 name=PlanetGlass nohit paints=#26262BFF role=none
          33 StackPanel at=864,110,332,53 by=panel limit=0,0,1280,800 role=none
            34 Panel at=864,116,40,40 by=panel limit=0,0,1280,800 role=none
              35 Border at=864,116,40,40 by=panel limit=0,0,1280,800 nohit paints=#EEEEF0FF role=none
              36 Viewbox at=873,125,22,22 by=panel limit=0,0,1280,800 role=none
                37 ViewboxContainer at=873,125,22,22 by=clip limit=873,125,22,22 role=none
                  38 Path at=873,125,22,22 by=clip limit=873,125,22,22 role=none
            39 StackPanel at=920,110,217,53 by=panel limit=0,0,1280,800 role=none
              40 TextBlock at=920,110,217,29 box=217,29 by=panel caps=16.7988 drawn face="Barlow Condensed Medium" heard="RECENTLY PLAYED" layout=165.9,28.8 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=24 text="RECENTLY PLAYED"
              41 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=920,141,217,22 box=217,22 caps=12.5991 drawn face=Barlow heard="Last: Aurora Drift (Synthetic)" layout=216.41,21.6 name=PlanetStatus paints=#8D8D91FF role=text size=18 text="Last: Aurora Drift (Synthetic)"
        42 Border at=0,256,1280,1 by=panel limit=0,0,1280,800 name=PlanetOrbit nohit role=none
        43 Panel at=358,190,176,176 by=panel heard="NES, 2 games" limit=0,0,1280,800 name="PlanetCell.NES (Moon)" role=none
          44 Border at=358,190,176,176 by=panel limit=0,0,1280,800 name=PlanetGlass nohit paints=#26262BFF role=none
          45 StackPanel at=366,190,160,162 by=panel limit=0,0,1280,800 role=none
            46 Panel at=366,190,160,120 by=panel limit=0,0,1280,800 role=none
              47 PlanetGlyph at=404,208,84,84 by=panel limit=0,0,1280,800 name="Planet.NES (Moon)" nohit role=none
            48 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=430,312,33,27 box=33,27 caps=15.3989 drawn face="Barlow Condensed Medium" heard=NES layout=32.07,26.4 name=PlanetStatus paints=#EEEEF0FF role=text size=22 text=NES
            49 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=416,341,61,21 box=61,21 caps=11.8992 drawn face=Barlow heard="2 games" layout=60.1,20.4 name=PlanetStatus paints=#8D8D91FF role=text size=17 text="2 games"
          50 Border at=351,183,190,190 by=panel limit=0,0,1280,800 name=HubRing nohit paints=#4C9AE8FF role=none
        51 Panel at=552,196,176,176 by=panel heard="Game Boy, 1 game" limit=0,0,1280,800 name="PlanetCell.Game Boy (Mercury)" role=none
          52 StackPanel at=560,196,160,162 by=panel limit=0,0,1280,800 role=none
            53 Panel at=560,196,160,120 by=panel limit=0,0,1280,800 role=none
              54 PlanetGlyph at=611,227,58,58 by=panel limit=0,0,1280,800 name="Planet.Game Boy (Mercury)" nohit role=none
            55 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=599,318,82,27 box=82,27 caps=15.3989 drawn face="Barlow Condensed Medium" heard="GAME BOY" layout=81.47,26.4 name=PlanetStatus paints=#EEEEF0FF role=text size=22 text="GAME BOY"
            56 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=615,347,50,21 box=50,21 caps=11.8992 drawn face=Barlow heard="1 game" layout=49.03,20.4 name=PlanetStatus paints=#8D8D91FF role=text size=17 text="1 game"
        57 Panel at=746,196,176,176 by=panel heard="Super Nintendo, 5 games" limit=0,0,1280,800 name="PlanetCell.SNES (Venus)" role=none
          58 StackPanel at=754,196,160,162 by=panel limit=0,0,1280,800 role=none
            59 Panel at=754,196,160,120 by=panel limit=0,0,1280,800 role=none
              60 PlanetGlyph at=805,227,58,58 by=panel limit=0,0,1280,800 name="Planet.SNES (Venus)" nohit role=none
            61 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=762,318,143,27 box=143,27 caps=15.3989 drawn face="Barlow Condensed Medium" heard="SUPER NINTENDO" layout=142,26.4 name=PlanetStatus paints=#EEEEF0FF role=text size=22 text="SUPER NINTENDO"
            62 TextBlock allow="a console's words are one line cut with an ellipsis, its full name said beneath the rows" at=804,347,61,21 box=61,21 caps=11.8992 drawn face=Barlow heard="5 games" layout=60.11,20.4 name=PlanetStatus paints=#8D8D91FF role=text size=17 text="5 games"
        63 Panel at=64,586,1152,104 by=panel limit=0,0,1280,800 role=none
          64 StackPanel at=64,607,571,77 by=panel limit=0,0,1280,800 role=none
            65 TextBlock at=64,607,571,44 box=571,44 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="NINTENDO ENTERTAINMENT SYSTEM" layout=570.51,44 limit=0,0,1280,800 name=PlanetFocusTitle paints=#EEEEF0FF role=text size=44 text="NINTENDO ENTERTAINMENT SYSTEM"
            66 TextBlock at=64,657,571,27 box=571,27 by=panel caps=15.3989 drawn face=Barlow heard="Nintendo · 1983 · 2 games" layout=230.58,26.4 limit=0,0,1280,800 name=PlanetFocusFacts paints=#8D8D91FF role=text size=22 text="Nintendo · 1983 · 2 games"
        67 Panel at=796,586,420,104 by=panel limit=0,0,1280,800 name=PlanetFan role=none
          68 CoverCard at=1070.28,588.79,74.45,102.42 by=panel heard="Fable of Tiles (Synthetic)" limit=0,0,1280,800 name="PlanetCover.<run>/Roms/Fable of Tiles (Synthetic).nes" nohit role=none
            69 Border at=1070.28,588.79,74.45,102.42 by=panel limit=0,0,1280,800 name=CoverBox nohit paints=#26262BFF role=none
              70 Image at=1070.28,588.79,73.54,102.39 by=clip drawn limit=1070.28,588.79,74.45,102.42 nohit role=image
          71 CoverCard at=1140.97,586.56,81.06,106.87 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="PlanetCover.<run>/Roms/Granite Choir (Synthetic).nes" nohit role=none
            72 Border at=1140.97,586.56,81.06,106.87 by=panel limit=0,0,1280,800 name=CoverBox nohit paints=#26262BFF role=none
              73 Image at=1140.97,586.56,80.16,106.78 by=clip drawn limit=1140.97,586.56,81.06,106.87 nohit role=image
        74 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=PlanetHelpBar role=none
          75 Panel at=0,702,1280,98 by=panel limit=0,0,1280,800 role=none
            76 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 nohit paints=#0E0E10FF,#707070FF role=none
            77 HintBar at=64,716,1152,30 by=panel heard="Open, Back, Favourites, Random game" limit=0,0,1280,800 role=group
focus none
press down consoles="NES (Moon)" focus="PlanetLayout ConsolesScreen" top=none
press up consoles=favourites focus="PlanetLayout ConsolesScreen" top=none
press back focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
