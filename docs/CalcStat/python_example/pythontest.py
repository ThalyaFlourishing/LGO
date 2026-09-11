from calcstat import CalcStat
from calcstat import DecSng
from calcstat import RoundDbl

LevelCap=CalcStat('LevelCap',1)
print("Current Level Cap= "+str(LevelCap))
ILvlCap=RoundDbl(CalcStat('LvlToILvl',LevelCap))
print("Associated Item Level Cap= "+str(ILvlCap))
print()

print("Tactical Mitigation essence iLvl 474= "+str(CalcStat('TacMit',474,0.8)))
print("Tactical Mitigation essence iLvl 481= "+str(CalcStat('TacMit',481,0.8)))
print()
print("Hunter mitigation class (1=Light, 2=Medium, 3=Heavy)= "+str(CalcStat('HunterCDArmourType',1)))
print("Mitigation (medium classes) Cap Rating at Level "+str(LevelCap)+"= "+str(CalcStat('TacMitMPRatPCapR',LevelCap)))
print()
print("Tier 2 Armour penetration at Level "+str(LevelCap)+"= "+str(CalcStat('TPenArmour',LevelCap,2)))
print("Tier 3 Armour penetration at Level "+str(LevelCap)+"= "+str(CalcStat('TPenArmour',LevelCap,3)))
print()
print("Virtue Fidelity Tactical Mitigation rank 90= "+str(CalcStat('VirtFidelityTacMit',90)))
print()
print("Total Character Experience needed to reach level "+str(LevelCap)+"= "+str(CalcStat('LvlExpCostTot',LevelCap)))
print()
print("ICMRT140#1.8= "+str(CalcStat('ICMRT',140,1.8)))
print("ICMRT141#1.8= "+str(CalcStat('ICMRT',141,1.8)))
print()

basevalue=450
level=1
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=25
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=50
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=60
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=65
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=75
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=85
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=95
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=100
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=105
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=106
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=115
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=116
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=120
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=121
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=130
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=131
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=140
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=141
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=150
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=151
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
level=160
print("StdProgRatings"+str(level)+"#"+str(basevalue)+"= "+str(CalcStat('StdProgRatings',level,basevalue)))
print()

CombatBaseLevel = 25
CombatBaseILevel = RoundDbl(CalcStat('LvlToILvl',CombatBaseLevel))
print("- Base TDR/THR By Level for tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel))+")")
print("CombatBaseTacHPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel))+")")
print("- Base TDR/THR By Level for non tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel))+")")
print("CombatBaseTacHPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel))+")")
print("- Li TDR/THR at iLvl:"+str(CombatBaseILevel))
print("CombatBaseTacDPS= "+str(DecSng(CalcStat('CombatBaseTacDPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacDPS',CombatBaseILevel))+")")
print("CombatBaseTacHPS= "+str(DecSng(CalcStat('CombatBaseTacHPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacHPS',CombatBaseILevel))+")")
print()
CombatBaseLevel = 52
CombatBaseILevel = RoundDbl(CalcStat('LvlToILvl',CombatBaseLevel))
print("- Base TDR/THR By Level for tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel))+")")
print("CombatBaseTacHPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel))+")")
print("- Base TDR/THR By Level for non tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel))+")")
print("CombatBaseTacHPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel))+")")
print("- Li TDR/THR at iLvl:"+str(CombatBaseILevel))
print("CombatBaseTacDPS= "+str(DecSng(CalcStat('CombatBaseTacDPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacDPS',CombatBaseILevel))+")")
print("CombatBaseTacHPS= "+str(DecSng(CalcStat('CombatBaseTacHPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacHPS',CombatBaseILevel))+")")
print()
CombatBaseLevel = 129
CombatBaseILevel = RoundDbl(CalcStat('LvlToILvl',CombatBaseLevel))
print("- Base TDR/THR By Level for tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel))+")")
print("CombatBaseTacHPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel))+")")
print("- Base TDR/THR By Level for non tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel))+")")
print("CombatBaseTacHPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel))+")")
print("- Li TDR/THR at iLvl:"+str(CombatBaseILevel))
print("CombatBaseTacDPS= "+str(DecSng(CalcStat('CombatBaseTacDPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacDPS',CombatBaseILevel))+")")
print("CombatBaseTacHPS= "+str(DecSng(CalcStat('CombatBaseTacHPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacHPS',CombatBaseILevel))+")")
print()
CombatBaseLevel = LevelCap
CombatBaseILevel = RoundDbl(CalcStat('LvlToILvl',CombatBaseLevel))
print("- Base TDR/THR By Level for tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSByLevel',CombatBaseLevel))+")")
print("CombatBaseTacHPSByLevel= "+str(DecSng(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSByLevel',CombatBaseLevel))+")")
print("- Base TDR/THR By Level for non tactical classes at cLvl:"+str(CombatBaseLevel))
print("CombatBaseTacDPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacDPSNoClass',CombatBaseLevel))+")")
print("CombatBaseTacHPSNoClass= "+str(DecSng(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel)))+" ("+str(CalcStat('CombatBaseTacHPSNoClass',CombatBaseLevel))+")")
print("- Li TDR/THR at iLvl:"+str(CombatBaseILevel))
print("CombatBaseTacDPS= "+str(DecSng(CalcStat('CombatBaseTacDPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacDPS',CombatBaseILevel))+")")
print("CombatBaseTacHPS= "+str(DecSng(CalcStat('CombatBaseTacHPS',CombatBaseILevel)))+" ("+str(CalcStat('CombatBaseTacHPS',CombatBaseILevel))+")")
print()

CreepILvlCurr=CalcStat('CreepILvlCurr',1)
print("Current Creep Item Level= "+str(CreepILvlCurr))
print()

stat='Resist'
num=0.6
print(stat+"C1#"+str(num)+"= "+str(CalcStat(stat+'C',1,num)))
print(stat+"C"+str(LevelCap)+"#"+str(num)+"= "+str(CalcStat(stat+'C',LevelCap,num)))
print(stat+"CI510#"+str(num)+"= "+str(CalcStat(stat+'CI',510,num)))
print(stat+"CI515#"+str(num)+"= "+str(CalcStat(stat+'CI',515,num)))
print(stat+"CI520#"+str(num)+"= "+str(CalcStat(stat+'CI',520,num)))
print(stat+"CI525#"+str(num)+"= "+str(CalcStat(stat+'CI',525,num)))
print(stat+"CI"+str(CreepILvlCurr)+"#"+str(num)+"= "+str(CalcStat(stat+'CI',CreepILvlCurr,num)))
num=3.2
print(stat+"C1#"+str(num)+"= "+str(CalcStat(stat+'C',1,num)))
print(stat+"C"+str(LevelCap)+"#"+str(num)+"= "+str(CalcStat(stat+'C',LevelCap,num)))
print(stat+"CI510#"+str(num)+"= "+str(CalcStat(stat+'CI',510,num)))
print(stat+"CI515#"+str(num)+"= "+str(CalcStat(stat+'CI',515,num)))
print(stat+"CI520#"+str(num)+"= "+str(CalcStat(stat+'CI',520,num)))
print(stat+"CI525#"+str(num)+"= "+str(CalcStat(stat+'CI',525,num)))
print(stat+"CI"+str(CreepILvlCurr)+"#"+str(num)+"= "+str(CalcStat(stat+'CI',CreepILvlCurr,num)))
stat='TacMit'
num=0.28
print(stat+"C1#"+str(num)+"= "+str(CalcStat(stat+'C',1,num)))
print(stat+"C"+str(LevelCap)+"#"+str(num)+"= "+str(CalcStat(stat+'C',LevelCap,num)))
print(stat+"CI510#"+str(num)+"= "+str(CalcStat(stat+'CI',510,num)))
print(stat+"CI515#"+str(num)+"= "+str(CalcStat(stat+'CI',515,num)))
print(stat+"CI520#"+str(num)+"= "+str(CalcStat(stat+'CI',520,num)))
print(stat+"CI525#"+str(num)+"= "+str(CalcStat(stat+'CI',525,num)))
print(stat+"CI"+str(CreepILvlCurr)+"#"+str(num)+"= "+str(CalcStat(stat+'CI',CreepILvlCurr,num)))
stat='PhyMit'
num=0.3
print(stat+"C1#"+str(num)+"= "+str(CalcStat(stat+'C',1,num)))
print(stat+"C"+str(LevelCap)+"#"+str(num)+"= "+str(CalcStat(stat+'C',LevelCap,num)))
print(stat+"CI510#"+str(num)+"= "+str(CalcStat(stat+'CI',510,num)))
print(stat+"CI515#"+str(num)+"= "+str(CalcStat(stat+'CI',515,num)))
print(stat+"CI520#"+str(num)+"= "+str(CalcStat(stat+'CI',520,num)))
print(stat+"CI525#"+str(num)+"= "+str(CalcStat(stat+'CI',525,num)))
print(stat+"CI"+str(CreepILvlCurr)+"#"+str(num)+"= "+str(CalcStat(stat+'CI',CreepILvlCurr,num)))

print()

print("CalcStat test finished.")
