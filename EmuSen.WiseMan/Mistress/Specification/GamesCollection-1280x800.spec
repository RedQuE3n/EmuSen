# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesCollection-1280x800
size 1280 800
kind menu
root GalleryLayout#GamesScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case GamesCollection
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LunaSky at=0,0,1280,800 by=panel limit=0,0,1280,800 name=GallerySky nohit role=none
    2 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
        4 DockPanel at=64,34,1152,52 by=panel limit=0,0,1280,800 role=none
          5 StackPanel at=64,34,317,52 by=panel limit=0,0,1280,800 name=GalleryHeading role=none
            6 TextBlock at=64,47,90,26 box=90,26 by=panel caps=13.999 drawn face=Barlow heard="GAMES ▸" layout=89.6,25.7 limit=0,0,1280,800 name=GalleryTrail paints=#8D8D91FF role=text size=20 text="GAMES ▸"
            7 TextBlock at=170,40,124,39 box=124,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=WEEKEND layout=123.98,38.4 limit=0,0,1280,800 name=GallerySection paints=#EEEEF0FF role=text size=32 text=WEEKEND
            8 TextBlock at=310,48,71,24 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="3 games" layout=70.48,24 limit=0,0,1280,800 name=GalleryCount paints=#8D8D91FF role=text size=20 text="3 games"
        9 Panel at=142,108,996,504 by=panel heard=Weekend limit=0,0,1280,800 name=GalleryGrid role=none
          10 CoverCard at=136.6,91.45,190.8,254.4 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            11 Border at=136.6,210.17,190.8,136.74 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              12 Image at=136.6,210.17,190.8,136.1 by=clip drawn limit=136.6,210.17,190.8,136.74 nohit role=image
            13 Border at=129.18,202.75,205.64,151.58 by=panel limit=0,0,1280,800 name=CoverRing nohit paints=#4C9AE8FF role=none
          14 CoverCard at=346,108,180,240 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            15 Border at=346,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              16 Image at=346,220,180,128.4 by=clip drawn limit=346,220,180,129 nohit role=image
            17 Path at=494,228,24,24 by=panel limit=0,0,1280,800 name=CoverHeart nohit role=none
          18 CoverCard at=550,108,180,240 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            19 Border at=550,220,180,129 by=panel limit=0,0,1280,800 name=CoverBox paints=#26262BFF role=none
              20 Image at=550,220,180,128.4 by=clip drawn limit=550,220,180,129 nohit role=image
        21 TextBlock at=64,628,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1280,800 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        22 TextBlock at=64,670,1152,27 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1280,800 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        23 Border at=0,742,1280,58 by=panel limit=0,0,1280,800 name=GalleryHelpBar role=none
          24 Panel at=0,742,1280,58 by=panel limit=0,0,1280,800 role=none
            25 Border at=0,742,1280,58 by=panel limit=0,0,1280,800 nohit paints=#0E0E10FF,#707070FF role=none
            26 HintBar at=64,756,1152,30 by=panel heard="Play, Game page, Favourite, Section, List, Consoles, Home" limit=0,0,1280,800 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press up focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
