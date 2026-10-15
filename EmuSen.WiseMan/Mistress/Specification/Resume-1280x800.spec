# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Resume-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,84.5,812.7,605
title "Continue Playing"
case Resume
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,184,813,485 by=panel limit=234,184,812.35,485 paints=#9C9CA0FF role=none seen=234,184,812.35,485
    1 StackPanel at=274,212,733,441 by=panel limit=234,184,812.35,485 role=none
      2 Image at=503,212,274.29,240 by=panel drawn limit=234,184,812.35,485 role=image
      3 TextBlock at=274,464,733,29 box=733,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Would you like to continue your last game?" layout=346.8,28.8 limit=234,184,812.35,485 paints=#9C9CA0FF role=text size=24 text="Would you like to continue your last game?"
      4 HintText at=274,505,733,48 box=733,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again." layout=690.72,48 limit=234,184,812.35,485 paints=#86868AFF role=text size=20 text="Cobalt Harbor (Synthetic) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again."
      5 LunaSwitch at=274,565,194,32 by=panel drawn focus heard="Do not ask again" limit=234,184,812.35,485 name=RememberResumeSwitch op paints=#86868AFF,#9C9CA0FF role=button
        6 TextBlock at=326,567,136,29 box=136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Do not ask again" layout=135.58,28.8 limit=274,565,194,32 owner=5 paints=#9C9CA0FF role=text size=24 text="Do not ask again"
      7 ButtonBar at=783,609,224,44 by=panel limit=234,184,812.35,485 paints=#9C9CA0FF role=toolbar
        8 Button at=783,609,105,44 by=clip drawn focus heard=RESUME limit=783,609,224,44 name=ResumeButton op paints=#55555AFF,#9C9CA0FF role=button text=Resume
          9 TextBlock at=800,615,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME layout=70.56,31.2 limit=783,609,105,44 owner=8 paints=#EEEEF0FF role=text size=26 text=RESUME
        10 Button at=896,609,111,44 by=clip drawn focus heard=RESTART limit=783,609,224,44 name=RestartButton op paints=#55555AFF,#9C9CA0FF role=button text=Restart
          11 TextBlock at=913,615,77,32 box=77,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESTART layout=76.41,31.2 limit=896,609,111,44 owner=10 paints=#9C9CA0FF role=text size=26 text=RESTART
focus 8
edge 5 up same
edge 5 down 8
edge 5 left same
edge 5 right 8
edge 8 up 5
edge 8 down same
edge 8 left 5
edge 8 right 10
edge 10 up 5
edge 10 down same
edge 10 left 8
edge 10 right same
press pageup focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press pagedown focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press first focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press last focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press back top=none
