# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-Gamelist-1280x800
size 1280 800
kind scene
root Panel
unit 1
slack 1.5
floor 0
frame 0,0,1280,800
case Gamelist
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
view gamelist snes "Aurora Drift (Synthetic)"
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 Panel at=0,0,1280,800 by=clip limit=0,0,1280,800 role=none
      2 NormalizedCanvas at=0,0,1280,800 by=clip limit=0,0,1280,800 role=group
        3 TextRowList at=64,64,576,640 by=clip element=textlist:gamelist limit=0,0,1280,800 role=list
        4 DeviceStatusBar at=1228.96,12.8,28,28 by=clip element=systemstatus:(default) heard="Bluetooth on" limit=0,0,1280,800 role=statusbar
        5 HintBar at=38.4,736,1230.4,24 by=clip element=helpsystem:help heard="D-pad up and down Choose, D-pad left and right System, LB Jump, RB Jump, LT First, RT Last, A Launch, B Back, Y Favorite, View Options, Thumbstick click Random, Menu Menu" limit=0,0,1280,800 role=group
      6 ScrollLetterOverlay at=0,0,1280,800 by=clip limit=0,0,1280,800 nohit role=text
focus none
press down game="Brass Lantern (Synthetic)" sheet=none system=snes view=gamelist
press up game="Aurora Drift (Synthetic)" sheet=none system=snes view=gamelist
press pagedown game="Ember Circuit (Synthetic)" sheet=none system=snes view=gamelist
press pageup game="Aurora Drift (Synthetic)" sheet=none system=snes view=gamelist
press last game="Ember Circuit (Synthetic)" sheet=none system=snes view=gamelist
press first game="Aurora Drift (Synthetic)" sheet=none system=snes view=gamelist
press right game="Fable of Tiles (Synthetic)" sheet=none system=nes view=gamelist
press left game="Aurora Drift (Synthetic)" sheet=none system=snes view=gamelist
press back game="" sheet=none system=snes view=system
