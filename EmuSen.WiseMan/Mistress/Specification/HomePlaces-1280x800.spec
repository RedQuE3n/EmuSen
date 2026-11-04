# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomePlaces-1280x800
size 1280 800
kind menu
root HubLayout#HomeScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case HomePlaces
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
        7 Panel at=64,84,373,280 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name=HubContinue role=none
          8 Border at=64,84,373,280 by=panel limit=0,0,1280,800 role=none
            9 PosterSlot at=64,84,373,280 by=clip heard=Poster limit=64,84,373,280 name=HubPoster nohit role=image
        10 TextBlock at=461,112,755,24 box=755,24 by=panel caps=13.999 drawn face=Barlow heard="CONTINUE PLAYING" layout=166.72,24 limit=0,0,1280,800 name=HubCaption paints=#8D8D91FF role=text size=20 text="CONTINUE PLAYING"
        11 TextBlock at=461,136,755,53 box=755,53 by=panel caps=30.7979 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=367.05,52.8 limit=0,0,1280,800 name=HubTitle paints=#EEEEF0FF role=text size=44 text="Cobalt Harbor (Synthetic)"
        12 TextBlock at=461,252,755,27 box=755,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · Paused just now" layout=308.95,26.4 limit=0,0,1280,800 name=HubFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · Paused just now"
        13 TextBlock at=64,374,400,24 box=400,24 by=panel caps=13.999 drawn face=Barlow heard=RECENT layout=70.14,24 limit=0,0,1280,800 name=HubRecentTitle paints=#8D8D91FF role=text size=20 text=RECENT
        14 Panel at=64,416,1152,160 by=panel heard=Recent limit=0,0,1280,800 name=HubShelf role=none
          15 CoverCard at=64,416,120,160 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            16 Border at=64,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              17 Image at=64,490,120,85.6 by=clip drawn limit=64,490,120,86 nohit role=image
          18 CoverCard at=208,416,120,160 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="HubTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            19 Border at=208,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              20 Image at=208,490,120,85.6 by=clip drawn limit=208,490,120,86 nohit role=image
            21 Path at=296,498,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          22 CoverCard at=352,416,120,160 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Dune Relay (Synthetic).sfc" role=none
            23 Border at=352,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              24 Image at=352,490,120,85.6 by=clip drawn limit=352,490,120,86 nohit role=image
          25 CoverCard at=496,416,120,160 by=panel heard="Ember Circuit (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Ember Circuit (Synthetic).sfc" role=none
            26 Border at=496,490,120,86 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              27 Image at=496,490,120,85.6 by=clip drawn limit=496,490,120,86 nohit role=image
          28 CoverCard at=640,416,120,160 by=panel heard="Fable of Tiles (Synthetic), favourite" limit=0,0,1280,800 name="HubTile.<run>/Roms/Fable of Tiles (Synthetic).nes" role=none
            29 Border at=644,416,113,160 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              30 Image at=644,416,112.15,160 by=clip drawn limit=644,416,113,160 nohit role=image
            31 Path at=724,424,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          32 CoverCard at=784,416,120,160 by=panel heard="Granite Choir (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Granite Choir (Synthetic).nes" role=none
            33 Border at=788,416,113,160 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              34 Image at=788,416,112.15,160 by=clip drawn limit=788,416,113,160 nohit role=image
          35 CoverCard at=928,416,120,160 by=panel heard="Hollow Comet (Synthetic)" limit=0,0,1280,800 name="HubTile.<run>/Roms/Hollow Comet (Synthetic).gb" role=none
            36 Border at=928,443,120,134 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              37 Image at=928,443,120,133.2 by=clip drawn limit=928,443,120,134 nohit role=image
        38 Panel at=64,622,1152,64 by=panel limit=0,0,1280,800 name=HubPlaces role=none
          39 Panel at=64,622,150,64 by=panel heard="Games, 8 games" limit=0,0,1280,800 name=HubPlace.games role=none
            40 Border at=64,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              41 StackPanel at=76,630,126,48 by=panel limit=0,0,1280,800 role=none
                42 TextBlock at=76,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Games layout=57.52,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Games
                43 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=76,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="8 games" layout=70.88,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="8 games"
          44 Panel at=231,622,150,64 by=panel heard="Scraper, 41 no art" limit=0,0,1280,800 name=HubPlace.scraper role=none
            45 Border at=231,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              46 StackPanel at=243,630,126,48 by=panel limit=0,0,1280,800 role=none
                47 TextBlock at=243,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Scraper layout=66.6,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Scraper
                48 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=243,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="41 no art" layout=69.16,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="41 no art"
          49 Panel at=398,622,150,64 by=panel heard="Packages, 3 changed" limit=0,0,1280,800 name=HubPlace.packages role=none
            50 Border at=398,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              51 StackPanel at=410,630,126,48 by=panel limit=0,0,1280,800 role=none
                52 TextBlock at=410,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Packages layout=81.6,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Packages
                53 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=410,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard="3 changed" layout=87.24,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="3 changed"
          54 Panel at=565,622,150,64 by=panel heard="Library, 2 suspect" limit=0,0,1280,800 name=HubPlace.library role=none
            55 Border at=565,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              56 StackPanel at=577,629,126,50 by=panel limit=0,0,1280,800 role=none
                57 TextBlock at=577,629,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Library layout=60.36,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Library
                58 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=577,653,126,26 box=126,26 caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 2 suspect" layout=104.76,25.7 name=HubPlaceStatus paints=#DAA520FF role=text size=20 text="⚠ 2 suspect"
          59 Panel at=732,622,150,64 by=panel heard="Cores, 1 to approve" limit=0,0,1280,800 name=HubPlace.cores role=none
            60 Border at=732,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              61 StackPanel at=744,629,126,50 by=panel limit=0,0,1280,800 role=none
                62 TextBlock at=744,629,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Cores layout=48.32,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Cores
                63 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=744,653,126,26 box=126,26 caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 1 to approve" layout=124.34,25.7 name=HubPlaceStatus paints=#DAA520FF role=text size=20 text="⚠ 1 to approve"
            64 Border at=725,615,164,78 by=panel limit=0,0,1280,800 name=HubRing nohit paints=#4C9AE8FF role=none
          65 Panel at=899,622,150,64 by=panel heard="Settings, Dark · 60% sound" limit=0,0,1280,800 name=HubPlace.settings role=none
            66 Border at=899,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              67 StackPanel at=911,630,126,48 by=panel limit=0,0,1280,800 role=none
                68 TextBlock at=911,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Settings layout=69.56,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Settings
                69 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=911,654,126,24 box=126,24 caps=13.999 drawn ellipsis face=Barlow heard="Dark · 60% sound" layout=116.12,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text="Dark · 60% sound"
          70 Panel at=1066,622,150,64 by=panel heard="Power, Exit" limit=0,0,1280,800 name=HubPlace.power role=none
            71 Border at=1066,622,150,64 by=panel limit=0,0,1280,800 paints=#26262BFF role=none
              72 StackPanel at=1078,630,126,48 by=panel limit=0,0,1280,800 role=none
                73 TextBlock at=1078,630,126,24 box=126,24 by=panel caps=13.999 drawn face=Barlow heard=Power layout=53.74,24 limit=0,0,1280,800 paints=#EEEEF0FF role=text size=20 text=Power
                74 TextBlock allow="a place's live line is one line cut with an ellipsis, as the design draws it" at=1078,654,126,24 box=126,24 caps=13.999 drawn face=Barlow heard=Exit layout=32.62,24 name=HubPlaceStatus paints=#8D8D91FF role=text size=20 text=Exit
        75 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=HubHelpBar paints=#1B1B1EFF,#707070FF role=none
          76 HintBar at=64,716,1152,30 by=panel heard="Open, Search, Home" limit=0,0,1280,800 role=group
focus none
press down focus="HubLayout HomeScreen" home="Places cores" top=none
press up focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Granite Choir (Synthetic).nes" top=none
press back focus="HubLayout HomeScreen" home="Shelf <run>/Roms/Granite Choir (Synthetic).nes" top=none
