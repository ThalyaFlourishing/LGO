Sub OutputStat(csn,cl,sparam)
	Dim res: res = CalcStat(csn,cl,sparam)
	If IsNumeric(sparam) Then
		WScript.StdOut.WriteLine "CalcStat(""" & csn & """," & cl & "," & sparam & ") = " & res
	ElseIf VarType(sparam) = 8 Then
		WScript.StdOut.WriteLine "CalcStat(""" & csn & """," & cl & ",""" & sparam & """) = " & res
	Else
		WScript.StdOut.WriteLine "CalcStat(""" & csn & """," & cl & ") = " & res
	End If
End Sub

OutputStat "OutHealPRatP",160,47964
OutputStat "CritHitT",135,2.0
OutputStat "TPenArmour",160,3
OutputStat "T2PenArmour",160,0
OutputStat "Armour",467,"HHT"
OutputStat "TomeTotalMain",0,"XX"
OutputStat "CritHit",81,0.8
OutputStat "PhyMitHPRatPCapR",160,0
OutputStat "LvlExpCostTot",160,0
OutputStat "LvlExpCost",160,0
OutputStat "-VERSION",160,Null
OutputStat "BEOFATE",160,1
OutputStat "BEOVITALITYINCREASE",160,1
OutputStat "BRGALLINFINESSE",160,1
OutputStat "BRGALLINONEXPFINESSE",160,1
OutputStat "BRGTRICKCNTDEFBPE",160,1
OutputStat "BRGTRICKCNTDEFCRITDEF",160,1
OutputStat "CHP2HWPNBLOCK",160,1
OutputStat "CHPCONTRBURNICPR",160,1
OutputStat "CHPFINESSEINCREASE",160,1
OutputStat "CHPFLURRYINCRCRITHIT",160,1
OutputStat "CHPMIGHTINCREASE",160,1
OutputStat "CHPSTALWBLADEVITALITY",160,1
OutputStat "CHPUNBREAKTACMIT",160,1
OutputStat "CPTCOVMAIN",160,1
OutputStat "CPTCOVPHYMIT",160,1
OutputStat "CPTCOVVITALITY",160,1
OutputStat "GRDTENDERIZECRITHIT",160,1
OutputStat "GRDWARDTACTTACMIT",160,1
OutputStat "HNTARMOURRENDBLOCK",160,1
OutputStat "HNTARMOURRENDEVADE",160,1
OutputStat "HNTARMOURRENDPARRY",160,1
OutputStat "LEVELCAP",160,1
OutputStat "LMHEARTYDIETMORALE",160,1
OutputStat "LMPREPFORWARTACMAS",160,1
OutputStat "MINCOMPOSURERESIST",160,1
OutputStat "MINCOMPOSURETACMIT",160,1
OutputStat "MINENDMORALE",160,1
OutputStat "MINTACMAS",160,1
OutputStat "RKDETERMINATIONWILL",160,1
OutputStat "RKFORTUNESMILESFATE",160,1
OutputStat "WRDFINESSE",160,1
OutputStat "WRDPHYMAS",160,1
OutputStat "WRDRECKLESSNCRITHIT",160,1
OutputStat "WRDSHIELDMASBLOCK",160,1
OutputStat "WRDSSSHPIERCERBLOCK",160,1
OutputStat "WRDSTDYOURGRBLOCK",160,1
OutputStat "WRDSTDYOURGREVADE",160,1
OutputStat "WRDSTDYOURGRPARRY",160,1
OutputStat "Might",488,1
OutputStat "WpnDPS",488,"HBT"
