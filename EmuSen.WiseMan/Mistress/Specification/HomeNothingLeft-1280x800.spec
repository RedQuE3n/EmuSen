# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeNothingLeft-1280x800
size 1280 800
kind menu
root HubLayout#HomeScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case HomeNothingLeft
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
        3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=HubBackdrop nohit paints=#1B1B1EFF role=none
          4 Image at=0,0,1280,800 by=panel drawn limit=0,0,1280,800 name=HubArt nohit role=image
          5 Border at=0,0,1280,800 by=panel limit=0,0,1280,800 nohit role=none
        6 TextBlock at=1068,40,148,24 box=148,24 by=panel caps=13.999 drawn face=Barlow heard="1 controller · 21:42" layout=147.42,24 limit=0,0,1280,800 name=HubStatus paints=#8D8D91FF role=text size=20 text="1 controller · 21:42"
        7 Panel at=64,84,373,280 by=panel limit=0,0,1280,800 name=HubContinue role=none
          8 Border at=64,84,373,280 by=panel limit=0,0,1280,800 role=none
            9 PosterSlot at=64,84,373,280 by=clip heard=Poster limit=64,84,373,280 name=HubPoster nohit role=image
        10 TextBlock at=461,136,755,53 box=755,53 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="Aurora Drift (Synthetic)" layout=337.3,52.8 limit=0,0,1280,800 name=HubTitle paints=#EEEEF0FF role=text size=44 text="Aurora Drift (Synthetic)"
        11 TextBlock at=461,252,755,27 box=755,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1280,800 name=HubFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        12 TextBlock at=64,374,400,24 box=400,24 by=panel caps=13.999 drawn face=Barlow heard=RECENT layout=70.14,24 limit=0,0,1280,800 name=HubRecentTitle paints=#8D8D91FF role=text size=20 text=RECENT
        13 Panel at=64,416,1152,160 by=panel heard=Recent limit=0,0,1280,800 name=HubShelf role=none
          14 CoverCard at=60.4,408.97,127.2,169.6 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            15 Border at=60.4,487.41,127.2,91.16 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              16 Image at=60.4,487.41,127.2,90.74 by=clip drawn limit=60.4,487.41,127.2,91.16 nohit role=image
            17 Border at=52.98,479.99,142.04,106 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          18 CoverCard at=208,416,120,160 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="HubTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            19 Border at=208,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              20 Image at=208,490,120,85.6 by=clip drawn limit=208,490,120,86 nohit role=image
            21 Path at=296,498,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          22 CoverCard at=352,416,120,160 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            23 Border at=352,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              24 Image at=352,490,120,85.6 by=clip drawn limit=352,490,120,86 nohit role=image
          25 CoverCard at=496,416,120,160 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            26 Border at=496,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              27 Image at=496,490,120,85.6 by=clip drawn limit=496,490,120,86 nohit role=image
          28 CoverCard at=640,416,120,160 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            29 Border at=640,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              30 Image at=640,490,120,85.6 by=clip drawn limit=640,490,120,86 nohit role=image
          31 CoverCard at=784,416,120,160 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1280,800 name="HubTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            32 Border at=788,416,113,160 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              33 Image at=788,416,112.15,160 by=clip drawn limit=788,416,113,160 nohit role=image
            34 Path at=868,424,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          35 CoverCard at=928,416,120,160 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            36 Border at=932,416,113,160 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              37 Image at=932,416,112.15,160 by=clip drawn limit=932,416,113,160 nohit role=image
          38 CoverCard at=1072,416,120,160 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            39 Border at=1072,443,120,134 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              40 Image at=1072,443,120,133.2 by=clip drawn limit=1072,443,120,134 nohit role=image
        41 TextBlock at=64,582,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Aurora Drift (Synthetic)" layout=212.3,26.4 limit=0,0,1280,800 name=HubShelfTitle paints=#D4D4D4FF role=text size=22 text="Aurora Drift (Synthetic)"
        42 Panel at=64,622,1152,64 by=panel limit=0,0,1280,800 name=HubPlaces role=none
          43 Panel at=64,622,150,64 by=panel heard="Games, 8 games" limit=0,0,1280,800 name=HubPlace.games role=none
            44 Border at=64,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              45 StackPanel at=76,630,126,48 by=panel limit=0,0,1280,800 role=none
                46 TextBlock at=76,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Games layout=57.52,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Games
                47 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=76,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="8 games"
          48 Panel at=231,622,150,64 by=panel heard="Scraper, Art and facts" limit=0,0,1280,800 name=HubPlace.scraper role=none
            49 Border at=231,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              50 StackPanel at=243,630,126,48 by=panel limit=0,0,1280,800 role=none
                51 TextBlock at=243,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Scraper layout=66.6,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Scraper
                52 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=243,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="Art and facts" layout=107.54,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Art and facts"
          53 Panel at=398,622,150,64 by=panel heard="Packages, Game packages" limit=0,0,1280,800 name=HubPlace.packages role=none
            54 Border at=398,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              55 StackPanel at=410,630,126,48 by=panel limit=0,0,1280,800 role=none
                56 TextBlock at=410,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Packages layout=81.6,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Packages
                57 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=410,654,126,24 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Game packages" layout=119.4,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Game packages"
          58 Panel at=565,622,150,64 by=panel heard="Library, Checks dumps" limit=0,0,1280,800 name=HubPlace.library role=none
            59 Border at=565,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              60 StackPanel at=577,630,126,48 by=panel limit=0,0,1280,800 role=none
                61 TextBlock at=577,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Library layout=60.36,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Library
                62 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=577,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="Checks dumps" layout=123.44,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Checks dumps"
          63 Panel at=732,622,150,64 by=panel heard="Cores, The engines" limit=0,0,1280,800 name=HubPlace.cores role=none
            64 Border at=732,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              65 StackPanel at=744,630,126,48 by=panel limit=0,0,1280,800 role=none
                66 TextBlock at=744,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Cores layout=48.32,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Cores
                67 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=744,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="The engines" layout=101.72,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="The engines"
          68 Panel at=899,622,150,64 by=panel heard="Settings, Dark · 60% sound" limit=0,0,1280,800 name=HubPlace.settings role=none
            69 Border at=899,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              70 StackPanel at=911,630,126,48 by=panel limit=0,0,1280,800 role=none
                71 TextBlock at=911,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Settings layout=69.56,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Settings
                72 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=911,654,126,24 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Dark · 60% sound" layout=116.12,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Dark · 60% sound"
          73 Panel at=1066,622,150,64 by=panel heard="Power, Exit" limit=0,0,1280,800 name=HubPlace.power role=none
            74 Border at=1066,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              75 StackPanel at=1078,630,126,48 by=panel limit=0,0,1280,800 role=none
                76 TextBlock at=1078,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Power layout=53.74,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Power
                77 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1078,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard=Exit layout=32.62,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text=Exit
        78 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=HubHelpBar paints=#1B1B1EFF,#707070FF role=none
          79 HintBar at=64,716,1152,30 by=panel heard="Play, Game page, Favourite, Search, Home" limit=0,0,1280,800 role=group
focus none
press down home="Places games" top=none
press up home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back home="Shelf <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
