# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesCollection-1920x1200
size 1920 1200
kind menu
root GalleryLayout#GamesScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamesCollection
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LunaSky at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=GallerySky nohit role=none
    2 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
        4 DockPanel at=96,51,1728,78 by=panel limit=0,0,1920,1200 role=none
          5 StackPanel at=96,51,475.5,78 by=panel limit=0,0,1920,1200 name=GalleryHeading role=none
            6 TextBlock at=96,70.5,135,39 box=90,26 by=panel caps=13.999 drawn face=Barlow heard="GAMES ▸" layout=89.6,25.7 limit=0,0,1920,1200 name=GalleryTrail paints=#8D8D91FF role=text size=20 text="GAMES ▸"
            7 TextBlock at=255,60,186,58.5 box=124,39 by=panel caps=22.3984 drawn face="Barlow Condensed Medium" heard=WEEKEND layout=123.98,38.4 limit=0,0,1920,1200 name=GallerySection paints=#EEEEF0FF role=text size=32 text=WEEKEND
            8 TextBlock at=465,72,106.5,36 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="3 games" layout=70.48,24 limit=0,0,1920,1200 name=GalleryCount paints=#8D8D91FF role=text size=20 text="3 games"
        9 Panel at=213,162,1494,756 by=panel heard=Weekend limit=0,0,1920,1200 name=GalleryGrid role=none
          10 CoverCard at=204.9,137.18,286.2,381.6 by=panel heard="Aurora Drift (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Aurora Drift (Synthetic).sfc" role=none
            11 Border at=204.9,315.26,286.2,205.11 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              12 Image at=204.9,315.26,286.2,204.16 by=clip drawn limit=204.9,315.26,286.2,205.11 nohit role=image
            13 Border at=193.77,304.13,308.46,227.37 by=panel limit=0,0,1920,1200 name=CoverRing nohit paints=#4C9AE8FF role=none
          14 CoverCard at=519,162,270,360 by=panel heard="Brass Lantern (Synthetic), favourite" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Brass Lantern (Synthetic).sfc" role=none
            15 Border at=519,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              16 Image at=519,330,270,192.6 by=clip drawn limit=519,330,270,193.5 nohit role=image
            17 Path at=741,342,36,36 by=panel limit=0,0,1920,1200 name=CoverHeart nohit role=none
          18 CoverCard at=825,162,270,360 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1200 name="GalleryTile.<run>/Roms/Cobalt Harbor (Synthetic).sfc" role=none
            19 Border at=825,330,270,193.5 by=panel limit=0,0,1920,1200 name=CoverBox paints=#26262BFF role=none
              20 Image at=825,330,270,192.6 by=clip drawn limit=825,330,270,193.5 nohit role=image
        21 TextBlock at=96,942,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic)" layout=234.91,38.4 limit=0,0,1920,1200 name=GalleryName paints=#EEEEF0FF role=text size=32 text="Aurora Drift (Synthetic)"
        22 TextBlock at=96,1005,1728,40.5 box=1152,27 by=panel caps=15.3989 drawn face=Barlow heard="Super Nintendo · 3 h ago" layout=224.05,26.4 limit=0,0,1920,1200 name=GalleryFacts paints=#8D8D91FF role=text size=22 text="Super Nintendo · 3 h ago"
        23 Border at=0,1113,1920,87 by=panel limit=0,0,1920,1200 name=GalleryHelpBar role=none
          24 Panel at=0,1113,1920,87 by=panel limit=0,0,1920,1200 role=none
            25 Border at=0,1113,1920,87 by=panel limit=0,0,1920,1200 nohit paints=#0E0E10FF,#707070FF role=none
            26 HintBar at=96,1134,1728,45 by=panel heard="Play, Game page, Favourite, Section, List, Consoles, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press up focus="GalleryLayout GamesScreen" games="collection:1 <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
