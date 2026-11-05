# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Home-1920x1200
size 1920 1200
kind menu
root HubLayout#HomeScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Home
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LunaSky at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=HubSky nohit role=none
    2 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
        4 TextBlock at=96,60,222,36 box=148,24 by=panel caps=13.999 drawn face=Barlow heard="1 controller · 21:42" layout=147.42,24 limit=0,0,1920,1200 name=HubStatus paints=#8D8D91FF role=text size=20 text="1 controller · 21:42"
        5 StackPanel at=1614,54,210,43.5 by=panel limit=0,0,1920,1200 name=HubWordmark role=none
          6 Border at=1614,57,39,39 by=panel limit=0,0,1920,1200 role=none
            7 Path at=1614,57,39,39 by=panel limit=0,0,1920,1200 role=none
          8 TextBlock at=1671,54,153,43.5 box=102,29 by=panel caps=16.7988 drawn face="Barlow Condensed Light" heard=EMUSEN layout=101.57,28.8 limit=0,0,1920,1200 name=HubMark paints=#EEEEF0FF role=text size=24 text=EMUSEN
        9 Panel at=78,108,1764,444 by=panel limit=0,0,1920,1200 name=HubContinuePanel nohit role=none
          10 Border at=78,108,1764,444 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
          11 Border at=78,108,1764,444 by=panel limit=0,0,1920,1200 name=HubRing nohit paints=#4C9AE8FF role=none
        12 Panel at=96,126,559.5,420 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name=HubContinue role=none
          13 Border at=96,126,559.5,420 by=panel limit=0,0,1920,1200 role=none
            14 PosterSlot at=96,126,559.5,420 by=clip heard=Poster limit=96,126,559.5,420 name=HubPoster nohit role=image
        15 TextBlock at=691.5,168,1132.5,36 box=755,24 by=panel caps=13.999 drawn face=Barlow heard="CONTINUE PLAYING" layout=166.72,24 limit=0,0,1920,1200 name=HubCaption paints=#8D8D91FF role=text size=20 text="CONTINUE PLAYING"
        16 TextBlock at=691.5,204,1132.5,79.5 box=755,53 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=0,0,1920,1200 name=HubTitle paints=#EEEEF0FF role=text size=44 text="Cobalt Harbor (Synthetic)"
        17 PlanetGlyph at=691.5,388.5,21,21 by=panel limit=0,0,1920,1200 name=HubContinuePlanet nohit role=none
        18 TextBlock at=727.5,378,1096.5,40.5 box=731,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · Paused just now" layout=308.95,26.4 limit=0,0,1920,1200 name=HubFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · Paused just now"
        19 TextBlock at=96,564,600,36 box=400,24 by=panel caps=13.999 drawn face=Barlow heard=RECENT layout=70.14,24 limit=0,0,1920,1200 name=HubRecentTitle paints=#8D8D91FF role=text size=20 text=RECENT
        20 Panel at=96,636,1728,240 by=panel heard=Recent limit=0,0,1920,1200 name=HubShelf role=none
          21 CoverCard at=96,636,180,240 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            22 Border at=96,747,180,129 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              23 Image at=96,747,180,128.4 by=clip drawn limit=96,747,180,129 nohit role=image
          24 CoverCard at=312,636,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            25 Border at=312,747,180,129 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              26 Image at=312,747,180,128.4 by=clip drawn limit=312,747,180,129 nohit role=image
            27 Path at=444,759,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          28 CoverCard at=528,636,180,240 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            29 Border at=528,747,180,129 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              30 Image at=528,747,180,128.4 by=clip drawn limit=528,747,180,129 nohit role=image
          31 CoverCard at=744,636,180,240 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            32 Border at=744,747,180,129 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              33 Image at=744,747,180,128.4 by=clip drawn limit=744,747,180,129 nohit role=image
          34 CoverCard at=960,636,180,240 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            35 Border at=966,636,169.5,240 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              36 Image at=966,636,168.22,240 by=clip drawn limit=966,636,169.5,240 nohit role=image
            37 Path at=1086,648,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          38 CoverCard at=1176,636,180,240 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            39 Border at=1182,636,169.5,240 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              40 Image at=1182,636,168.22,240 by=clip drawn limit=1182,636,169.5,240 nohit role=image
          41 CoverCard at=1392,636,180,240 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1920,1200 name="HubTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            42 Border at=1392,676.5,180,201 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              43 Image at=1392,676.5,180,199.8 by=clip drawn limit=1392,676.5,180,201 nohit role=image
        44 Panel at=96,933,1728,96 by=panel limit=0,0,1920,1200 name=HubPlaces role=none
          45 Panel at=96,933,225,96 by=panel heard="Games, 8 games" limit=0,0,1920,1200 name=HubPlace.games role=none
            46 Border at=96,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            47 Border at=96,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            48 Border at=96,933,225,96 by=panel limit=0,0,1920,1200 role=none
              49 StackPanel at=114,945,189,72 by=panel limit=0,0,1920,1200 role=none
                50 TextBlock at=114,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Games layout=57.52,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Games
                51 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=114,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="8 games"
          52 Panel at=346.5,933,225,96 by=panel heard="Scraper, 41 no art" limit=0,0,1920,1200 name=HubPlace.scraper role=none
            53 Border at=346.5,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            54 Border at=346.5,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            55 Border at=346.5,933,225,96 by=panel limit=0,0,1920,1200 role=none
              56 StackPanel at=364.5,945,189,72 by=panel limit=0,0,1920,1200 role=none
                57 TextBlock at=364.5,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Scraper layout=66.6,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Scraper
                58 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=364.5,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="41 no art" layout=69.16,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="41 no art"
          59 Panel at=597,933,225,96 by=panel heard="Packages, 3 changed" limit=0,0,1920,1200 name=HubPlace.packages role=none
            60 Border at=597,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            61 Border at=597,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            62 Border at=597,933,225,96 by=panel limit=0,0,1920,1200 role=none
              63 StackPanel at=615,945,189,72 by=panel limit=0,0,1920,1200 role=none
                64 TextBlock at=615,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Packages layout=81.6,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Packages
                65 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=615,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard="3 changed" layout=87.24,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="3 changed"
          66 Panel at=847.5,933,225,96 by=panel heard="Library, 2 suspect" limit=0,0,1920,1200 name=HubPlace.library role=none
            67 Border at=847.5,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            68 Border at=847.5,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            69 Border at=847.5,933,225,96 by=panel limit=0,0,1920,1200 role=none
              70 StackPanel at=865.5,943.5,189,75 by=panel limit=0,0,1920,1200 role=none
                71 TextBlock at=865.5,943.5,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Library layout=60.36,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Library
                72 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=865.5,979.5,189,39 box=126,26 caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 2 suspect" layout=104.76,25.7 name=HubPlaceStatus paints=#DAA520FF role=text size=20 text="⚠ 2 suspect"
          73 Panel at=1098,933,225,96 by=panel heard="Cores, 1 to approve" limit=0,0,1920,1200 name=HubPlace.cores role=none
            74 Border at=1098,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            75 Border at=1098,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            76 Border at=1098,933,225,96 by=panel limit=0,0,1920,1200 role=none
              77 StackPanel at=1116,943.5,189,75 by=panel limit=0,0,1920,1200 role=none
                78 TextBlock at=1116,943.5,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Cores layout=48.32,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Cores
                79 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1116,979.5,189,39 box=126,26 caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 1 to approve" layout=124.34,25.7 name=HubPlaceStatus paints=#DAA520FF role=text size=20 text="⚠ 1 to approve"
          80 Panel at=1348.5,933,225,96 by=panel heard="Settings, Dark · 60% sound" limit=0,0,1920,1200 name=HubPlace.settings role=none
            81 Border at=1348.5,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            82 Border at=1348.5,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            83 Border at=1348.5,933,225,96 by=panel limit=0,0,1920,1200 role=none
              84 StackPanel at=1366.5,945,189,72 by=panel limit=0,0,1920,1200 role=none
                85 TextBlock at=1366.5,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Settings layout=69.56,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Settings
                86 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1366.5,981,189,36 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Dark · 60% sound" layout=116.12,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Dark · 60% sound"
          87 Panel at=1599,933,225,96 by=panel heard="Power, Exit" limit=0,0,1920,1200 name=HubPlace.power role=none
            88 Border at=1599,933,225,96 by=panel limit=0,0,1920,1200 name=HubGlass nohit paints=#26262BFF role=none
            89 Border at=1599,933,225,96 by=panel limit=0,0,1920,1200 nohit paints=#707070FF role=none
            90 Border at=1599,933,225,96 by=panel limit=0,0,1920,1200 role=none
              91 StackPanel at=1617,945,189,72 by=panel limit=0,0,1920,1200 role=none
                92 TextBlock at=1617,945,189,36 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Power layout=53.74,24 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=20 text=Power
                93 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1617,981,189,36 box=126,24 caps=13.999 drawn face=Barlow heard=Exit layout=32.62,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text=Exit
        94 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=HubHelpBar role=none
          95 Panel at=0,1053,1920,147 by=panel limit=0,0,1920,1200 role=none
            96 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 nohit paints=#0E0E10FF,#707070FF role=none
            97 HintBar at=96,1074,1728,45 by=panel heard="Resume, Game page, Favourite, Search, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press up focus="HubLayout HomeScreen" home="Continue none" top=none
press back focus="HubLayout HomeScreen" home="Continue none" top=none
