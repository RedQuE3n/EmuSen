# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-Screensaver-1920x1200
size 1920 1200
kind overlay
root Screensaver#Screensaver
unit 1.5
slack 2
floor 0
frame 0,0,1920,1200
case Screensaver
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Border at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#000000FF role=none
  1 CrossFadeImage at=0,0,1920,1200 by=panel limit=0,0,1920,1200 nohit role=none
    2 FittedImage at=0,0,1920,1200 by=panel drawn limit=0,0,1920,1200 nohit role=image
  3 Canvas at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    4 Border at=26,24,393,126 by=panel limit=0,0,1920,1200 paints=#000000AA role=none
    5 FontText at=47,34,346,41 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1920,1200 role=text
    6 FontText at=47,97,239,41 by=panel heard="Super Nintendo" limit=0,0,1920,1200 role=text
focus none
