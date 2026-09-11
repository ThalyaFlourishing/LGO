Const adSaveCreateNotExist = 1
Const adSaveCreateOverWrite = 2
Const adTypeBinary = 1 'Binary data
Const adTypeText = 2 '(Default) Text data
Const adLF = 10
Const adCR = 13
Const adCRLF = -1

Const adReadLine = -2
Const adWriteChar = 0
Const adWriteLine = 1

'indexes StatData record fields
Const SD_STAT = 0
Const SD_SCRIPT = 1
Const SD_LSTART = 2
Const SD_LEND = 3
Const SD_TYPE = 4
Const SD_PROC = 5
Const SD_P1 = 6
Const SD_P2 = 7
Const SD_P3 = 8
Const SD_P4 = 9
Const SD_P5 = 10
Const SD_P6 = 11
Const SD_COMPF = 12
Const SDFLDCNT = 13

Const SDFLDSEP = ","
Const SDSTRQUOT = """"

Const InFileName = "statdata.csv"
Const DataFileName = "statdcmp.dat"

'script text variables
Const STV_SN = "$STATNAME"
Dim STV_SN_TEXT
Const STV_COND = "$COND"
Dim STV_COND_TEXT
Const STV_LSTART = "$LVLSTART"
Dim STV_LSTART_TEXT
Const STV_LEND = "$LVLEND"
Dim STV_LEND_TEXT
Const STV_LVLEVAL = "$LVLEVAL"
Dim STV_LVLEVAL_TEXT
Const STV_CALC = "$CALC"
Dim STV_CALC_TEXT
Const STV_DATA = "$DATA"
Dim STV_DATA_TEXT
Const STV_DATA2 = "$DATA2"
Dim STV_DATA2_TEXT
Const STV_NODEID = "$NODEID"
Dim STV_NODEID_TEXT
Const STV_CHILDID = "$CHILDID"
Dim STV_CHILDID_TEXT

'indexes Stats Index list record
Const SI_SD_STAT = 0
Const SI_SD_INDEX = 1
Const SI_SD_COUNT = 2
Const SIFLDCNT = 3

'indexes Binary Search Tree node variables
Const BST_KEY = 0
Const BST_VALUE = 1
Const BST_SIZE = 2
Const BST_LEFT = 3
Const BST_RIGHT = 4
Const BSTFLDCNT = 5

'scripting types
Const SCRIPT_DEFINE = "D" 'define $name for replacements
Const SCRIPT_STAT = "S" 'define stat

'rounding types
Const RND_NORMAL = "N" 'normal rounding up/down
Const RND_UP = "U" 'rounding up
Const RND_DOWN = "D" 'rounding down
Const RND_LOTRO = "L" 'Lotro rounding
Const RND_MORREG = "M" 'Morale Regen rounding
Const RND_PROGRESSION = "P" 'Progression rounding
Const RND_EQUSNG = "E" 'convert double to equivalant value of a single float
Const RND_DECSNG = "T" 'convert double to equivalant value of a single float in decimal textual format

'defines
Const DF_DEF = 0 'define name to replace
Const DF_REPL = 1 'replacement text
Const DF_TYPE = 2 'define type
Const DF_DECIMALS = 3 'optional rounding
'define types
Const DFT_TEXT = 1 'define is type text
Const DFT_EXPR = 2 'define is type expression

Const optCompileFull = 0 'full version with all stats
Const optCompilePercentages = 1 'percentage calculations only
Const optCompileTraitTrees = 2 'trait trees stats

'script data
Dim ScriptDescr
Dim OutFileName
Dim STextStart
Dim STextMainStart
Dim STextMainEnd
Dim STextSubStart
Dim STextSubEnd
Dim STextSNSearchSubResult
Dim STextEnd
Dim STextSNSearchCondStartSm
Dim STextSNSearchCondSm
Dim STextSNSearchCondStartEq
Dim STextSNSearchCondEq
Dim STextSNSearchCondStartGr
Dim STextSNSearchCondGr
Dim STextSNSearchAlternative
Dim STextSNSearchEnd
Dim STextLSearchCondStart
Dim STextLSearchCond
Dim STextLSearchAlternative
Dim STextLSearchEnd
Dim STextSearchResult
Dim STextSearchResultD
Dim STextSearchResultG
Dim STextSearchResultI
Dim STextSearchResultV
Dim STextDefineC
Dim STextDefineL
Dim STextDefineN
Dim STextDefinePUNDEF
Dim STextDefinePNULL
Dim STextDefineSN
Dim STextListParamD
Dim STextListParamG
Dim STextListParamI
Dim STextListParamV
Dim STextListSingleTrail
Dim STextDefineCastFloat
Dim STextDefineCastInt
Dim STextDefineLvlEvalStart
Dim STextDefineLvlEvalEnd
Dim STextDefineLvlEvalEqual
Dim STextDefineLvlEvalBoth
Dim STextDefineCondCombi
Dim STextIndent

Dim CompileOption 'compile option choice
Dim SDVersion

Main
Sub Main

	SetLocale("en-us")

	Dim DataArray

	If Not ReadDataFile(DataFileName,DataArray) Then
		MsgBox Replace("Could not read #S# file. Operation cancelled.","#S#",DataFileName)
		Exit Sub
	End If
	
	Dim ScrSelText
	Dim ScrSelDefault
	Dim ScrSelCount
	
	GetScrSelData DataArray,ScrSelText,ScrSelDefault,ScrSelCount
	
	Dim OutFileOption

	OutFileOption = CInt(InputBox(ScrSelText&Chr(10),"Choose output script type",ScrSelDefault))

	If Not (1 <= OutFileOption And OutFileOption <= ScrSelCount) Then
		MsgBox "Invalid option. Operation cancelled."
		Exit Sub
	End If
	
	CompileOption = CInt(InputBox("Enter a Number:"&Chr(10)&Chr(10)&"1 = Full version with all stats"&Chr(10)&Chr(10)&"2 = Percentage calculations only <- normal for plugins"&Chr(10)&Chr(10)&"3 = Trait trees stats"&Chr(10),"Choose output version",2))-1

	If Not (CompileOption = optCompileFull Or CompileOption = optCompilePercentages Or CompileOption = optCompileTraitTrees) Then
		MsgBox "Invalid option. Operation cancelled."
		Exit Sub
	End If
	
	GetScriptData DataArray,OutFileOption
	
	Dim SDInfoLst
	Dim aDefines

	SDVersion = "?"
	ReadSDFile InFileName,SDInfoLst,aDefines
	
	Dim aStatsIndex
	aStatsIndex = CreateStatsIndex(SDInfoLst)

	Dim aBalancedBST
	aBalancedBST = CreateBalancedBST(aStatsIndex)
	
	Dim OutStream
	'This is a workaround. antivirus software seems to detect the normal, complete, string as a threat since ~ december 2024.
	'This object is used to write an output file like calcstat.java or with an other extension, for an other chosen output scripting language.
	'The names of the involved files are always shown by the MsgBox dialogue further down.
	Set OutStream = CreateObject("ado" + "db.stre" + "am")
	With OutStream
		.Type = adTypeText
		.CharSet = "us-ascii"
		.LineSeparator = adCRLF
		.Open

		'Init. script text variables
		STV_SN_TEXT = ""
		STV_COND_TEXT = ""
		STV_LSTART_TEXT = ""
		STV_LEND_TEXT = ""
		STV_CALC_TEXT = ""
		STV_DATA_TEXT = ""
		STV_DATA2_TEXT = ""
		STV_NODEID_TEXT = ""
		STV_CHILDID_TEXT = ""

		WriteSText OutStream,STextStart,""

		Dim iNodeIndex
		iNodeIndex = LBound(aBalancedBST)	'Head node of the tree is first in the list

		If IsEmpty(STextSNSearchSubResult) Then
			WriteScrStatsNode OutStream,SDInfoLst,aDefines,aBalancedBST,iNodeIndex,"",1,-1,Nothing
		Else
			WriteSText OutStream,STextMainStart,""

			Dim aMaxDepthNodes
			Dim iMaxDepth: iMaxDepth = GetHeightBSTnode(aBalancedBST,0)-7 '8
			If iMaxDepth < 1 Then
				iMaxDepth = 1
			End If

			WriteScrStatsNode OutStream,SDInfoLst,aDefines,aBalancedBST,iNodeIndex,"",1,iMaxDepth,aMaxDepthNodes

			WriteSText OutStream,STextMainEnd,""

			If Not IsEmpty(aMaxDepthNodes) Then
				Dim oldSTV_NODEID_TEXT: oldSTV_NODEID_TEXT = STV_NODEID_TEXT
				For nSubNode = LBound(aMaxDepthNodes) To UBound(aMaxDepthNodes)
					STV_NODEID_TEXT = CStr(aMaxDepthNodes(nSubNode))
					WriteSText OutStream,STextSubStart,""

					WriteScrStatsNode OutStream,SDInfoLst,aDefines,aBalancedBST,aMaxDepthNodes(nSubNode),"",1,-1,Nothing				

					WriteSText OutStream,STextSubEnd,""
				Next
				STV_NODEID_TEXT = oldSTV_NODEID_TEXT
			End If
			
		End If

		WriteSText OutStream,STextEnd,""

		.SaveToFile OutFileName,adSaveCreateOverWrite
		.Close
	End With
	Set OutStream = Nothing

	Dim nRecCount: nRecCount = 0
	If Not IsEmpty(SDInfoLst) Then
		nRecCount = UBound(SDInfoLst)+1
	End If
	Dim nStatCount: nStatCount = 0
	If Not IsEmpty(aStatsIndex) Then
		nStatCount = UBound(aStatsIndex)+1
	End If

	MsgBox Replace(Replace("Compilation of #InFileName# to #OutFileName# completed."+CHR(10)+CHR(10)+"CalcStat Version: "+SDVersion+CHR(10)+"Script Data Records: "+CStr(nRecCount)+CHR(10)+"Number of Stats: "+CStr(nStatCount)+CHR(10)+"Height of Binary Search Tree: "+CStr(GetHeightBSTnode(aBalancedBST,0)),"#InFileName#",InFileName),"#OutFileName#",OutFileName)
	
End Sub

Function ReadDataFile(ByVal theDataFileName, ByRef theDataArray)

	Dim Result
	Result = True

	Dim DataStream
	'This is a workaround. antivirus software seems to detect the normal, complete, string as a threat since ~ december 2024.
	'The code below is only reading data from file statdcmp.dat
	Set DataStream = CreateObject("ado" + "db.stre" + "am")
	With DataStream
		.Type = adTypeText
		.CharSet = "UTF-8"
		.LineSeparator = adLF
		.Open
		.LoadFromFile theDataFileName

		Dim RowText

		Do Until .EOS
			RowText = .ReadText(adReadLine)

			RowText = Replace(RowText,Chr(10),"")
			RowText = Replace(RowText,Chr(13),"")

			If IsEmpty(theDataArray) Then
				ReDim theDataArray(0)
			Else
				ReDim Preserve theDataArray(UBound(theDataArray)+1)
			End If
			theDataArray(UBound(theDataArray)) = RowText
		Loop

		.Close
	End With
	Set DataStream = Nothing

	If IsEmpty(theDataArray) Then
		Result = False
	End If
	
	ReadDataFile = Result
	
End Function

Sub GetScrSelData(ByVal theDataArray, ByRef theScrSelText, ByRef theScrSelDefault, ByRef theScrSelCount)

	Dim SelText
	SelText = "Enter a Number:"

	Dim SelCount
	SelCount = 0
	
	Dim I
	I = LBound(theDataArray)
	Dim MaxI
	MaxI = UBound(theDataArray)
	
	Dim ScrTextLen
	
	'default script number
	theScrSelDefault = CInt(theDataArray(I))
	I = I+1
	
	Do Until MaxI-I < 18
		'script description
		SelCount = SelCount+1
		SelText = SelText & Chr(10) & Chr(10) & CStr(SelCount) & " = " & theDataArray(I)
		I = I+1

		'outfilename
		I = I+1

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script start text
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script main start text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script main end text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script sub start text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script sub end text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search sub result text
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script end text
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition start smaller text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition smaller text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition start equal text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition equal text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition start greater text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition greater text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search alternative text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search end text
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search condition start text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search condition text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search alternative text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search end text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type D
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type G
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type I
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type V
		I = I+ScrTextLen
		
		'script Define C text
		I = I+1
		
		'script Define L text
		I = I+1
		
		'script Define N text
		I = I+1
		
		'script Define PUNDEF text
		I = I+1
		
		'script Define PNULL text
		I = I+1
		
		'script Define SN text
		I = I+1

		'script List Parameter type D text
		I = I+1
		
		'script List Parameter type G text
		I = I+1
		
		'script List Parameter type I text
		I = I+1
		
		'script List Parameter type V text
		I = I+1
		
		'script List Single Element Trail text
		I = I+1

		'script Define Float Cast text
		I = I+1
		
		'script Define Int Cast text
		I = I+1
		
		'script Define LvlEval Start text
		I = I+1
		
		'script Define LvlEval End text
		I = I+1
		
		'script Define LvlEval Equal text
		I = I+1
		
		'script Define LvlEval Both text
		I = I+1
		
		'script Define Condition Combination text
		I = I+1
		
		'script Indent text
		I = I+1
	Loop
	
	theScrSelText = SelText
	theScrSelCount = SelCount
	
End Sub

Sub	GetScriptData(ByVal theDataArray, ByVal theOutFileOption)

	Dim ScriptCount
	ScriptCount = 0
	
	Dim I
	I = LBound(theDataArray)
	Dim MaxI
	MaxI = UBound(theDataArray)
	
	Dim ScrTextLen
	
	'default script number
	I = I+1
	
	Do Until MaxI-I < 18
		ScriptCount = ScriptCount+1

		'script description
		If ScriptCount = theOutFileOption Then
			ScriptDescr = theDataArray(I)
		End If
		I = I+1

		'outfilename
		If ScriptCount = theOutFileOption Then
			OutFileName = theDataArray(I)
		End If
		I = I+1

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script start text
		If ScriptCount = theOutFileOption Then
			STextStart = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script main start text
		If ScriptCount = theOutFileOption Then
			STextMainStart = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script main end text
		If ScriptCount = theOutFileOption Then
			STextMainEnd = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script sub start text
		If ScriptCount = theOutFileOption Then
			STextSubStart = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script sub end text
		If ScriptCount = theOutFileOption Then
			STextSubEnd = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search sub result text
		If ScriptCount = theOutFileOption Then
			STextSNSearchSubResult = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script end text
		If ScriptCount = theOutFileOption Then
			STextEnd = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition start smaller text
		If ScriptCount = theOutFileOption Then
			STextSNSearchCondStartSm = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition smaller text
		If ScriptCount = theOutFileOption Then
			STextSNSearchCondSm = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition start equal text
		If ScriptCount = theOutFileOption Then
			STextSNSearchCondStartEq = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition equal text
		If ScriptCount = theOutFileOption Then
			STextSNSearchCondEq = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition start greater text
		If ScriptCount = theOutFileOption Then
			STextSNSearchCondStartGr = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search condition greater text
		If ScriptCount = theOutFileOption Then
			STextSNSearchCondGr = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search alternative text
		If ScriptCount = theOutFileOption Then
			STextSNSearchAlternative = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script SN search end text
		If ScriptCount = theOutFileOption Then
			STextSNSearchEnd = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search condition start text
		If ScriptCount = theOutFileOption Then
			STextLSearchCondStart = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search condition text
		If ScriptCount = theOutFileOption Then
			STextLSearchCond = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search alternative text
		If ScriptCount = theOutFileOption Then
			STextLSearchAlternative = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Interval Level Search end text
		If ScriptCount = theOutFileOption Then
			STextLSearchEnd = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text
		If ScriptCount = theOutFileOption Then
			STextSearchResult = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type D
		If ScriptCount = theOutFileOption Then
			STextSearchResultD = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen
		
		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type G
		If ScriptCount = theOutFileOption Then
			STextSearchResultG = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type I
		If ScriptCount = theOutFileOption Then
			STextSearchResultI = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen

		ScrTextLen = CInt(theDataArray(I))
		I = I+1
		'script Search result text for type V
		If ScriptCount = theOutFileOption Then
			STextSearchResultV = GetScriptText(theDataArray,I,ScrTextLen)
		End If
		I = I+ScrTextLen

		'script Define C text
		If ScriptCount = theOutFileOption Then
			STextDefineC = theDataArray(I)
		End If
		I = I+1

		'script Define L text
		If ScriptCount = theOutFileOption Then
			STextDefineL = theDataArray(I)
		End If
		I = I+1

		'script Define N text
		If ScriptCount = theOutFileOption Then
			STextDefineN = theDataArray(I)
		End If
		I = I+1

		'script Define PUNDEF text
		If ScriptCount = theOutFileOption Then
			STextDefinePUNDEF = theDataArray(I)
		End If
		I = I+1

		'script Define PNULL text
		If ScriptCount = theOutFileOption Then
			STextDefinePNULL = theDataArray(I)
		End If
		I = I+1

		'script Define SN text
		If ScriptCount = theOutFileOption Then
			STextDefineSN = theDataArray(I)
		End If
		I = I+1

		'script List Parameter type D text
		If ScriptCount = theOutFileOption Then
			STextListParamD = theDataArray(I)
		End If
		I = I+1

		'script List Parameter type G text
		If ScriptCount = theOutFileOption Then
			STextListParamG = theDataArray(I)
		End If
		I = I+1

		'script List Parameter type I text
		If ScriptCount = theOutFileOption Then
			STextListParamI = theDataArray(I)
		End If
		I = I+1

		'script List Parameter type V text
		If ScriptCount = theOutFileOption Then
			STextListParamV = theDataArray(I)
		End If
		I = I+1

		'script List Single Element Trail text
		If ScriptCount = theOutFileOption Then
			STextListSingleTrail = theDataArray(I)
		End If
		I = I+1

		'script Define Float Cast text
		If ScriptCount = theOutFileOption Then
			STextDefineCastFloat = theDataArray(I)
		End If
		I = I+1

		'script Define Int Cast text
		If ScriptCount = theOutFileOption Then
			STextDefineCastInt = theDataArray(I)
		End If
		I = I+1

		'script Define LvlEval Start text
		If ScriptCount = theOutFileOption Then
			STextDefineLvlEvalStart = theDataArray(I)
		End If
		I = I+1
		
		'script Define LvlEval End text
		If ScriptCount = theOutFileOption Then
			STextDefineLvlEvalEnd = theDataArray(I)
		End If
		I = I+1
		
		'script Define LvlEval Equal text
		If ScriptCount = theOutFileOption Then
			STextDefineLvlEvalEqual = theDataArray(I)
		End If
		I = I+1
		
		'script Define LvlEval Both text
		If ScriptCount = theOutFileOption Then
			STextDefineLvlEvalBoth = theDataArray(I)
		End If
		I = I+1
		
		'script Define Condition Combination text
		If ScriptCount = theOutFileOption Then
			STextDefineCondCombi = theDataArray(I)
		End If
		I = I+1
		
		'script Indent text
		If ScriptCount = theOutFileOption Then
			STextIndent = theDataArray(I)
		End If
		I = I+1
		
		If ScriptCount = theOutFileOption Then
			Exit Do
		End If
	Loop
	
End Sub

Function GetScriptText(theDataArray,I,ScrTextLen)

	Dim ScriptText
	
	For II = I To I+ScrTextLen-1
		If IsEmpty(ScriptText) Then
			ReDim ScriptText(0)
		Else
			ReDim Preserve ScriptText(UBound(ScriptText)+1)
		End If
		ScriptText(UBound(ScriptText)) = theDataArray(II)
	Next
	
	GetScriptText = ScriptText
	
End Function

Sub EvalDefines(ByRef theDefines)

	Dim bReprocess: bProcess = True
	Dim DI

	While bProcess
		bProcess = False
		For DI = LBound(theDefines) To UBound(theDefines)
			If theDefines(DI)(DF_TYPE) = DFT_EXPR Then
				If ProcessDefineRefs(theDefines(DI)(DF_REPL),theDefines) Then
					If theDefines(DI)(DF_DECIMALS) = "" Then
						theDefines(DI)(DF_REPL) = CStr(Eval(theDefines(DI)(DF_REPL)))
					Else
						theDefines(DI)(DF_REPL) = CStr(RoundDbl(Eval(theDefines(DI)(DF_REPL)),CInt(theDefines(DI)(DF_DECIMALS))))
					End If
					theDefines(DI)(DF_TYPE) = DFT_TEXT
					theDefines(DI)(DF_DECIMALS) = ""
				Else
					bProcess = True
				End If
			End If
		Next
	Wend

End Sub

Function ProcessDefineRefs(ByRef theDefineReplText, ByVal theDefines)

	Dim sProcessedText: sProcessedText = theDefineReplText
	Dim nLenProcessedText: nLenProcessedText = Len(sProcessedText)

	Dim sStartToken: sStartToken = "$"
	Dim nLenStartToken: nLenStartToken = Len(sStartToken)

	Dim bResult: bResult = True

	Dim nSearch
	Dim RI
	Dim sDefName
	Dim nLenDefName
	Dim cDefChar

	Dim sDefReplText
	Dim nLenDefReplText

	For nSearch = 1 To nLenProcessedText
		If Mid(sProcessedText,nSearch,nLenStartToken) = sStartToken Then
			sDefName = ""
			For RI = nSearch+nLenStartToken To nLenProcessedText
				cDefChar = Mid(sProcessedText,RI,1)
				If ("A" <= cDefChar And cDefChar <= "Z") Or ("a" <= cDefChar And cDefChar <= "z") Or ("0" <= cDefChar And cDefChar <= "9") Or cDefChar = "_" Then
					sDefName = sDefName+cDefChar
				Else
					Exit For	
				End If
			Next
			nLenDefName = RI-(nSearch+nLenStartToken)
			If nLenDefName > 0 Then
				If GetDefineReplText(sDefName,theDefines,sDefReplText) Then
					nLenDefReplText = Len(sDefReplText)
					sProcessedText = Left(sProcessedText,nSearch-1)+sDefReplText+Right(sProcessedText,nLenProcessedText-(nSearch-1)-(nLenStartToken+nLenDefName))
					nLenProcessedText = nLenProcessedText+nLenDefReplText-(nLenStartToken+nLenDefName)
					nSearch = (nSearch-1)+nLenDefReplText
				Else
					bResult = False
				End If
			End If
		End If
	Next

	theDefineReplText = sProcessedText
	
	ProcessDefineRefs = bResult

End Function

Function GetDefineReplText(ByVal theDefName, ByVal theDefines, ByRef theDefReplText)

	Dim bResult: bResult = False
	Dim sSearchDefine: sSearchDefine = "$"+UCase(theDefName)
	Dim DI

	For DI = LBound(theDefines) To UBound(theDefines)
		If theDefines(DI)(DF_DEF) = sSearchDefine Then
			If theDefines(DI)(DF_TYPE) = DFT_TEXT Then
				theDefReplText = theDefines(DI)(DF_REPL)
				bResult = True
			End If
			Exit For
		End If
	Next 

	GetDefineReplText = bResult

End Function

Function ReplaceDefines(theText, theDefines)

	Dim sNewText
	sNewText = theText

	Dim sSearchText
	sSearchText = UCase(theText)

	Dim SDefine
	Dim sRepl
	
	Dim nSearch
	
	For DI = LBound(theDefines) To UBound(theDefines)
		sDefine = theDefines(DI)(DF_DEF)
		sRepl = theDefines(DI)(DF_REPL)
		If UCase(sDefine) <> UCase(sRepl) Then
			nSearch = InStr(sSearchText,sDefine)
			While nSearch > 0
				sNewText = Left(sNewText,nSearch-1)+sRepl+Right(sNewText,Len(sSearchText)-nSearch+1-Len(sDefine))
				sSearchText = Left(sSearchText,nSearch-1)+UCase(sRepl)+Right(sSearchText,Len(sSearchText)-nSearch+1-Len(sDefine))
				nSearch = InStr(sSearchText,sDefine)
			Wend
		End If
	Next
	
	ReplaceDefines = sNewText

End Function

Sub WriteSText(theOutStream,theSText,theIndent)

	If VarType(theSText) <> 8204 Then
		Exit Sub
	End If

	With theOutStream
		For I = LBound(theSText) To UBound(theSText)
			.WriteText theIndent & _
				Replace(Replace(Replace(Replace(Replace(Replace(Replace(Replace(Replace( _
				theSText(I), _
				STV_SN, STV_SN_TEXT), _
				STV_COND, STV_COND_TEXT), _
				STV_LSTART, STV_LSTART_TEXT), _
				STV_LEND, STV_LEND_TEXT), _
				STV_CALC, STV_CALC_TEXT), _
				STV_DATA2, STV_DATA2_TEXT), _
				STV_DATA, STV_DATA_TEXT), _
				STV_NODEID, STV_NODEID_TEXT), _
				STV_CHILDID, STV_CHILDID_TEXT), _
				adWriteLine
		Next
	End With
	
End Sub

Function ReplSText(theSText,theSearch,theRepl)

	Dim NewSText: NewSText = theSText

	For I = LBound(NewSText) To UBound(NewSText)
		NewSText(I) = Replace(NewSText(I),theSearch,theRepl)
	Next

	ReplSText = NewSText
	
End Function

Function NumFloatSText(theSText)

	Dim NewSText: NewSText = theSText

	For I = LBound(NewSText) To UBound(NewSText)
		NewSText(I) = NumFloatText(NewSText(I))
	Next

	NumFloatSText = NewSText

End Function

Function NumFloatText(theText)

	Dim NewText: NewText = ""
	Dim nPos: nPos = 1
	Dim nLen: nLen = Len(theText)
	Dim sChar
	Dim sAlpha: sAlpha = ""
	Dim sNum: sNum = ""
	dim sOther: sOther = ""
	
	While nPos <= nLen
		sChar = Mid(theText,nPos,1)
		If IsAlpha(sChar) Then
			NewText = NewText+sNum
			sNum = ""
			NewText = NewText+sOther
			sOther = ""
			sAlpha = sAlpha+sChar
		ElseIf IsNum(sChar) Then
			NewText = NewText+sAlpha
			sAlpha = ""
			NewText = NewText+sOther
			sOther = ""
			sNum = sNum+sChar
		Else
			NewText = NewText+sAlpha
			sAlpha = ""
			If sNum <> "" And (NewText = "" Or Not IsAlpha(Right(NewText,1))) Then
				'check num which is between non alpha and non alpha or <start> and non alpha
				If InStr(sNum,".") = 0 Then
					'no . so add
					sNum = sNum+".0"
				End If
			End If
			NewText = NewText+sNum
			sNum = ""
			sOther = sOther+sChar
		End If
		nPos = nPos+1
	Wend

	NewText = NewText+sAlpha
	sAlpha = ""
	If sNum <> "" And (NewText = "" Or Not IsAlpha(Right(NewText,1))) Then
		'check num which is between non alpha and <end> or <start> and <end>
		If InStr(sNum,".") = 0 Then
			'no . so add
			sNum = sNum+".0"
		End If
	End If
	NewText = NewText+sNum
	sNum = ""
	NewText = NewText+sOther
	sOther = ""
	
	NumFloatText = NewText

End Function

Function IsAlpha(sChar)

	IsAlpha = (("A" <= sChar And sChar <= "Z") Or ("a" <= sChar And sChar <= "z"))

End Function

Function IsNum(sChar)

	IsNum = (("0" <= sChar And sChar <= "9") Or sChar = ".")

End Function

Function BranchContainsStat(aBST,iNodeIndex,sSN)

	If aBST(iNodeIndex)(BST_KEY) = sSN Then
		BranchContainsStat = True
		Exit Function
	End If

	If aBST(iNodeIndex)(BST_LEFT) <> -1 Then
		If BranchContainsStat(aBST,aBST(iNodeIndex)(BST_LEFT),sSN) Then
			BranchContainsStat = True
			Exit Function
		End If
	End If
	
	If aBST(iNodeIndex)(BST_RIGHT) <> -1 Then
		If BranchContainsStat(aBST,aBST(iNodeIndex)(BST_RIGHT),sSN) Then
			BranchContainsStat = True
			Exit Function
		End If
	End If
	
	BranchContainsStat = False

End Function

Sub WriteScrStatsNode(theOutStream,theSDInfoLst,theDefines,aBST,iNodeIndex,sIndent,iCurDepth,iMaxDepth,ByRef aMaxDepthNodes)
	
	Dim oldSTV_NODEID_TEXT: oldSTV_NODEID_TEXT = STV_NODEID_TEXT
	STV_NODEID_TEXT = CStr(iNodeIndex)
	
	'take care of a tree depth limitation
	If iMaxDepth > 0 Then
		If iCurDepth > iMaxDepth Then
			'current branch is too deep into the tree and needs to be put in it's own sub-function
			'store the node's Id in the list of nodes which need separate functions
			If IsEmpty(aMaxDepthNodes) Then
				ReDim aMaxDepthNodes(0)
			Else
				ReDim Preserve aMaxDepthNodes(UBound(aMaxDepthNodes)+1)
			End If
			aMaxDepthNodes(UBound(aMaxDepthNodes)) = iNodeIndex
			'so output that sub-function's result instead
			WriteSText theOutStream,STextSNSearchSubResult,sIndent
			STV_NODEID_TEXT = oldSTV_NODEID_TEXT
			Exit Sub
		End If
	End If

	Dim oldSTV_SN_TEXT: oldSTV_SN_TEXT = STV_SN_TEXT
	Dim oldSTV_CHILDID_TEXT: oldSTV_CHILDID_TEXT = STV_CHILDID_TEXT

	'exam node properties
	'current stat node has child nodes?
	Dim iSmallerIndex: iSmallerIndex = aBST(iNodeIndex)(BST_LEFT)
	Dim iGreaterIndex: iGreaterIndex = aBST(iNodeIndex)(BST_RIGHT)
	Dim bHasSmallerNode: bHasSmallerNode = (iSmallerIndex <> -1)
	Dim bHasGreaterNode: bHasGreaterNode = (iGreaterIndex <> -1)
	Dim bHasSmallerBranch: bHasSmallerBranch = False
	'if so, do the child nodes have childs? then they're a branch, otherwise just a single stat node
	If bHasSmallerNode Then
		If aBST(iSmallerIndex)(BST_LEFT) <> -1 Or aBST(iSmallerIndex)(BST_RIGHT) <> -1 Then
			bHasSmallerBranch = True
		End If
	End If
	Dim bHasGreaterBranch: bHasGreaterBranch = False
	If bHasGreaterNode Then
		If aBST(iGreaterIndex)(BST_LEFT) <> -1 Or aBST(iGreaterIndex)(BST_RIGHT) <> -1 Then
			bHasGreaterBranch = True
		End If
	End If

	'output node constructs
	'branches have priority / are treated first, because they contain multiple stats
	If bHasSmallerBranch And bHasGreaterBranch Then
		STV_SN_TEXT = aBST(iNodeIndex)(BST_KEY)
		'let's guarantee a shortest path to STDPROGRATINGS stat and alternate smaller/greater order to get a more balanced number of comparison operations
		If BranchContainsStat(aBST,iSmallerIndex,"STDPROGRATINGS") Or (Not BranchContainsStat(aBST,iGreaterIndex,"STDPROGRATINGS") And iCurDepth/2 = CInt(iCurDepth/2)) Then
			'If SN < "$STATNAME" Then
			'	<smaller node branch>
			'ElseIf SN > "$STATNAME" Then
			'	<greater node branch>
			STV_CHILDID_TEXT = CStr(iSmallerIndex)
			WriteSText theOutStream,STextSNSearchCondStartSm,sIndent
			WriteScrStatsNode theOutStream,theSDInfoLst,theDefines,aBST,iSmallerIndex,sIndent+STextIndent,iCurDepth+1,iMaxDepth,aMaxDepthNodes
			STV_CHILDID_TEXT = CStr(iGreaterIndex)
			WriteSText theOutStream,STextSNSearchCondGr,sIndent
			WriteScrStatsNode theOutStream,theSDInfoLst,theDefines,aBST,iGreaterIndex,sIndent+STextIndent,iCurDepth+1,iMaxDepth,aMaxDepthNodes
		Else
			'If SN > "$STATNAME" Then
			'	<greater node branch>
			'ElseIf SN < "$STATNAME" Then
			'	<smaller node branch>
			STV_CHILDID_TEXT = CStr(iGreaterIndex)
			WriteSText theOutStream,STextSNSearchCondStartGr,sIndent
			WriteScrStatsNode theOutStream,theSDInfoLst,theDefines,aBST,iGreaterIndex,sIndent+STextIndent,iCurDepth+1,iMaxDepth,aMaxDepthNodes
			STV_CHILDID_TEXT = CStr(iSmallerIndex)
			WriteSText theOutStream,STextSNSearchCondSm,sIndent
			WriteScrStatsNode theOutStream,theSDInfoLst,theDefines,aBST,iSmallerIndex,sIndent+STextIndent,iCurDepth+1,iMaxDepth,aMaxDepthNodes
		End If
		'Else
		'	<current node stat>
		'End If
		WriteSText theOutStream,STextSNSearchAlternative,sIndent
		WriteScrStat theOutStream,theSDInfoLst,aBST(iNodeIndex)(BST_VALUE),aBST(iNodeIndex)(BST_SIZE),theDefines,sIndent
		WriteSText theOutStream,STextSNSearchEnd,sIndent
	ElseIf bHasSmallerBranch Then
		'If SN < "$STATNAME" Then
		'	<smaller node branch>
		'ElseIf SN = "$STATNAME" Then
		'	<current node stat>
		'ElseIf SN = "$STATNAME" Then
		'	<greater node stat>
		'End If
		STV_SN_TEXT = aBST(iNodeIndex)(BST_KEY)
		STV_CHILDID_TEXT = CStr(iSmallerIndex)
		WriteSText theOutStream,STextSNSearchCondStartSm,sIndent
		WriteScrStatsNode theOutStream,theSDInfoLst,theDefines,aBST,iSmallerIndex,sIndent+STextIndent,iCurDepth+1,iMaxDepth,aMaxDepthNodes
		WriteSText theOutStream,STextSNSearchCondEq,sIndent
		WriteScrStat theOutStream,theSDInfoLst,aBST(iNodeIndex)(BST_VALUE),aBST(iNodeIndex)(BST_SIZE),theDefines,sIndent
		STV_SN_TEXT = aBST(iGreaterIndex)(BST_KEY)
		WriteSText theOutStream,STextSNSearchCondEq,sIndent
		WriteScrStat theOutStream,theSDInfoLst,aBST(iGreaterIndex)(BST_VALUE),aBST(iGreaterIndex)(BST_SIZE),theDefines,sIndent
		WriteSText theOutStream,STextSNSearchEnd,sIndent
	ElseIf bHasGreaterBranch Then
		'If SN > "$STATNAME" Then
		'	<greater node branch>
		'ElseIf SN = "$STATNAME" Then
		'	<smaller node stat>
		'ElseIf SN = "$STATNAME" Then
		'	<current node stat>
		'End If
		STV_SN_TEXT = aBST(iNodeIndex)(BST_KEY)
		STV_CHILDID_TEXT = CStr(iGreaterIndex)
		WriteSText theOutStream,STextSNSearchCondStartGr,sIndent
		WriteScrStatsNode theOutStream,theSDInfoLst,theDefines,aBST,iGreaterIndex,sIndent+STextIndent,iCurDepth+1,iMaxDepth,aMaxDepthNodes
		STV_SN_TEXT = aBST(iSmallerIndex)(BST_KEY)
		WriteSText theOutStream,STextSNSearchCondEq,sIndent
		WriteScrStat theOutStream,theSDInfoLst,aBST(iSmallerIndex)(BST_VALUE),aBST(iSmallerIndex)(BST_SIZE),theDefines,sIndent
		STV_SN_TEXT = aBST(iNodeIndex)(BST_KEY)
		WriteSText theOutStream,STextSNSearchCondEq,sIndent
		WriteScrStat theOutStream,theSDInfoLst,aBST(iNodeIndex)(BST_VALUE),aBST(iNodeIndex)(BST_SIZE),theDefines,sIndent
		WriteSText theOutStream,STextSNSearchEnd,sIndent
	Else
		'no branches and both smaller and greater nodes are optional
		If bHasSmallerNode Then
			'If SN = "$STATNAME" Then
			'	<smaller node stat>
			'ElseIf SN = "$STATNAME" Then
			'	<current node stat>
			STV_SN_TEXT = aBST(iSmallerIndex)(BST_KEY)
			WriteSText theOutStream,STextSNSearchCondStartEq,sIndent
			WriteScrStat theOutStream,theSDInfoLst,aBST(iSmallerIndex)(BST_VALUE),aBST(iSmallerIndex)(BST_SIZE),theDefines,sIndent
			STV_SN_TEXT = aBST(iNodeIndex)(BST_KEY)
			WriteSText theOutStream,STextSNSearchCondEq,sIndent
			WriteScrStat theOutStream,theSDInfoLst,aBST(iNodeIndex)(BST_VALUE),aBST(iNodeIndex)(BST_SIZE),theDefines,sIndent
		Else
			'If SN = "$STATNAME" Then
			'	<current node stat>
			STV_SN_TEXT = aBST(iNodeIndex)(BST_KEY)
			WriteSText theOutStream,STextSNSearchCondStartEq,sIndent
			WriteScrStat theOutStream,theSDInfoLst,aBST(iNodeIndex)(BST_VALUE),aBST(iNodeIndex)(BST_SIZE),theDefines,sIndent
		End If
		If bHasGreaterNode Then
			'ElseIf SN = "$STATNAME" Then
			'	<greater node stat>
			STV_SN_TEXT = aBST(iGreaterIndex)(BST_KEY)
			WriteSText theOutStream,STextSNSearchCondEq,sIndent
			WriteScrStat theOutStream,theSDInfoLst,aBST(iGreaterIndex)(BST_VALUE),aBST(iGreaterIndex)(BST_SIZE),theDefines,sIndent
		End If
		'End If
		WriteSText theOutStream,STextSNSearchEnd,sIndent
	End If

	STV_CHILDID_TEXT = oldSTV_CHILDID_TEXT
	STV_SN_TEXT = oldSTV_SN_TEXT
	STV_NODEID_TEXT = oldSTV_NODEID_TEXT
	
End Sub

Function CreateStatsIndex(theSDInfoLst)

	Dim aStatsIndex

	Dim sStat, sLastStat
	sLastStat = ""

	'create index list of unique stat names
	For SI = LBound(theSDInfoLst) To UBound(theSDInfoLst)
		sStat = theSDInfoLst(SI)(SD_STAT)
		If sStat <> sLastStat Then
			If sLastStat <> "" Then
				Redim Preserve aStatsIndex(UBound(aStatsIndex)+1)
				aStatsIndex(UBound(aStatsIndex)) = Array(sStat,SI,1)
			Else
				aStatsIndex = Array(Array(sStat,SI,1))
			End If
			sLastStat = sStat
		Else
			aStatsIndex(UBound(aStatsIndex))(SI_SD_COUNT) = aStatsIndex(UBound(aStatsIndex))(SI_SD_COUNT)+1
		End If
	Next

	'sort index based on alphabetical order of stat names
	Dim temp
	For I = UBound(aStatsIndex)-1 To LBound(aStatsIndex) Step -1
		For J = 0 to I
			If aStatsIndex(J)(SI_SD_STAT) > aStatsIndex(J+1)(SI_SD_STAT) Then
				temp = aStatsIndex(J+1)
				aStatsIndex(J+1) = aStatsIndex(J)
				aStatsIndex(J) = temp
			End If
		Next
	Next 

	CreateStatsIndex = aStatsIndex

End Function

Function CreateBalancedBST(aStatsIndex)

	Dim iLowIndex
	iLowIndex = LBound(aStatsIndex)
	
	Dim iHighIndex
	iHighIndex = UBound(aStatsIndex)

	Dim iStatIndex
	iStatIndex = Int((iLowIndex+(iHighIndex-iLowIndex)/2))

	Dim aBalancedBST
	aBalancedBST = Array(Array(-1,-1,-1,-1,-1))
	
	Dim iNodeIndex
	iNodeIndex = UBound(aBalancedBST)

	aBalancedBST(iNodeIndex)(BST_KEY) = aStatsIndex(iStatIndex)(SI_SD_STAT)
	aBalancedBST(iNodeIndex)(BST_VALUE) = aStatsIndex(iStatIndex)(SI_SD_INDEX)
	aBalancedBST(iNodeIndex)(BST_SIZE) = aStatsIndex(iStatIndex)(SI_SD_COUNT)
	If iStatIndex-1 >= iLowIndex Then
		aBalancedBST(iNodeIndex)(BST_LEFT) = AddBSTnode(aBalancedBST,aStatsIndex,iLowIndex,iStatIndex-1)
	End If
	If iStatIndex+1 <= iHighIndex Then
		aBalancedBST(iNodeIndex)(BST_RIGHT) = AddBSTnode(aBalancedBST,aStatsIndex,iStatIndex+1,iHighIndex)
	End If

	CreateBalancedBST = aBalancedBST
	
End Function

Function AddBSTnode(ByRef aBST, ByRef aStatsIndex, ByVal iLowIndex, ByVal iHighIndex)

	Dim iStatIndex
	iStatIndex = Int((iLowIndex+(iHighIndex-iLowIndex)/2))
	
	Redim Preserve aBST(UBound(aBST)+1)
	aBST(UBound(aBST)) = Array(-1,-1,-1,-1,-1)

	Dim iNodeIndex
	iNodeIndex = UBound(aBST)

	aBST(iNodeIndex)(BST_KEY) = aStatsIndex(iStatIndex)(SI_SD_STAT)
	aBST(iNodeIndex)(BST_VALUE) = aStatsIndex(iStatIndex)(SI_SD_INDEX)
	aBST(iNodeIndex)(BST_SIZE) = aStatsIndex(iStatIndex)(SI_SD_COUNT)
	If iStatIndex-1 >= iLowIndex Then
		aBST(iNodeIndex)(BST_LEFT) = AddBSTnode(aBST,aStatsIndex,iLowIndex,iStatIndex-1)
	End If
	If iStatIndex+1 <= iHighIndex Then
		aBST(iNodeIndex)(BST_RIGHT) = AddBSTnode(aBST,aStatsIndex,iStatIndex+1,iHighIndex)
	End If
	
	AddBSTnode = iNodeIndex
	
End Function

Function GetHeightBSTnode(ByRef aBST, ByVal iStatIndex)

	Dim iLeftIndex
	iLeftIndex = aBST(iStatIndex)(BST_LEFT)
	Dim iLeftHeight
	iLeftHeight = 0
	If iLeftIndex <> -1 Then
		iLeftHeight = GetHeightBSTnode(aBST,iLeftIndex)
	End If

	Dim iRightIndex
	iRightIndex = aBST(iStatIndex)(BST_RIGHT)
	Dim iRightHeight
	iRightHeight = 0
	If iRightIndex <> -1 Then
		iRightHeight = GetHeightBSTnode(aBST,iRightIndex)
	End If

	If iLeftHeight >= iRightHeight Then
		GetHeightBSTnode = iLeftHeight+1
	Else
		GetHeightBSTnode = iRightHeight+1
	End If
	
End Function

Sub WriteScrStat(theOutStream,theSDInfoLst,iStatIndex,iStatDataCount,theDefines,sIndent)

	Dim oldSTV_COND_TEXT: oldSTV_COND_TEXT = STV_COND_TEXT
	Dim oldSTV_LSTART: oldSTV_LSTART_TEXT = STV_LSTART_TEXT
	Dim oldSTV_LEND: oldSTV_LEND_TEXT = STV_LEND_TEXT
	Dim oldSTV_CALC_TEXT: oldSTV_CALC_TEXT = STV_CALC_TEXT
	Dim oldSTV_DATA_TEXT: oldSTV_DATA_TEXT = STV_DATA_TEXT
	Dim oldSTV_DATA2_TEXT: oldSTV_DATA2_TEXT = STV_DATA2_TEXT

	Dim StatPos: StatPos = iStatIndex
	Dim StatLast: StatLast = StatPos+iStatDataCount-1
	Dim MainSDInfo: MainSDInfo = theSDInfoLst(StatPos)
	Dim S_STAT: S_STAT = MainSDInfo(SD_STAT)
	Dim S_SCRIPT: S_SCRIPT = MainSDInfo(SD_SCRIPT)
	Dim CurrSDInfo
	Dim S_LSTART, S_LEND, S_TYPE, S_PROC, S_P1, S_P2, S_P3, S_P4, S_P5, S_P6

	Dim Build
	Dim Data
	Dim Data2
	Dim nDataCount
	
	Dim nRoundType
	Dim bSingleAssignment: bSingleAssignment = False
	Dim sResultIndent

	Dim aProcDef: aProcDef = Array( _
		Array(RND_NORMAL,"RoundDbl(<B><P>)"), _
		Array(RND_UP,"RoundDblUp(<B><P>)"), _
		Array(RND_DOWN,"RoundDblDown(<B><P>)"), _
		Array("Y","<(><B><)>*"+STextDefineN), _
		Array("F",STextDefineCastFloat+"<B>"), _
		Array("I",STextDefineCastInt+"<B>"), _
		Array(RND_EQUSNG,"EquSng(<B>)"), _
		Array(RND_DECSNG,"DecSng(<B>)"), _
		Array(RND_PROGRESSION,"RoundDblProg(<B>)"), _
		Array(RND_LOTRO,"RoundDblLotro(<B>)"), _
		Array(RND_MORREG,"RoundDblMorReg(<B>)"))
	Dim nProcIndex, nProcLen, sProcText, sProcParam, sProcParamChar

	For SI = StatPos To StatLast
		CurrSDInfo = theSDInfoLst(SI)

		S_LSTART = ReplaceDefines(CurrSDInfo(SD_LSTART),theDefines)
		S_LEND = ReplaceDefines(CurrSDInfo(SD_LEND),theDefines)
		S_TYPE = ReplaceDefines(CurrSDInfo(SD_TYPE),theDefines)
		S_PROC = Ucase(ReplaceDefines(CurrSDInfo(SD_PROC),theDefines))
		S_P1 = ReplaceDefines(CurrSDInfo(SD_P1),theDefines)
		S_P2 = ReplaceDefines(CurrSDInfo(SD_P2),theDefines)
		S_P3 = ReplaceDefines(CurrSDInfo(SD_P3),theDefines)
		S_P4 = ReplaceDefines(CurrSDInfo(SD_P4),theDefines)
		S_P5 = ReplaceDefines(CurrSDInfo(SD_P5),theDefines)
		S_P6 = ReplaceDefines(CurrSDInfo(SD_P6),theDefines)

		'If STextDefineCastFloat <> "" Then
		'	STV_LSTART_TEXT = NumFloatText(S_LSTART)
		'	STV_LEND_TEXT = NumFloatText(S_LEND)
		'Else
			STV_LSTART_TEXT = S_LSTART
			STV_LEND_TEXT = S_LEND
		'End If

		STV_COND_TEXT = ""
		
		If STV_LSTART_TEXT = "" And STV_LEND_TEXT = "" Then
			'no condition: must be single assignment or -else- in -if- construct
			If iStatDataCount = 1 Then
				'must be single assignment
				bSingleAssignment = True
			Else
				'must be -else-
				WriteSText theOutStream,STextLSearchAlternative,sIndent
			End If
		Else
			'we have a condition with possibly multiple level evaluations

			'build condition
			Dim aLStart: aLStart = Split(STV_LSTART_TEXT,"|")
			Dim aLEnd: aLEnd = Split(STV_LEND_TEXT,"|")
			Dim nEvalMax
			If UBound(aLStart) > UBound(aLEnd) Then
				nEvalMax = UBound(aLStart)
			Else
				nEvalMax = UBound(aLEnd)
			End If
			For nEvalIndex = 0 To nEvalMax
				STV_LSTART_TEXT = ""
				If nEvalIndex <= UBOUND(aLStart) Then
					STV_LSTART_TEXT = aLStart(nEvalIndex)
				End If
				STV_LEND_TEXT = ""
				If nEvalIndex <= UBOUND(aLEnd) Then
					STV_LEND_TEXT = aLEnd(nEvalIndex)
				End If
				If STV_LSTART_TEXT <> "" And STV_LEND_TEXT <> "" Then
					If STV_LSTART_TEXT = STV_LEND_TEXT Then
						'start and end level equal evaluation
						STV_LVLEVAL_TEXT = Replace(Replace(STextDefineLvlEvalEqual,STV_LSTART,STV_LSTART_TEXT),STV_LEND,STV_LEND_TEXT)
					Else
						'both start and end level evaluation
						STV_LVLEVAL_TEXT = Replace(Replace(STextDefineLvlEvalBoth,STV_LSTART,STV_LSTART_TEXT),STV_LEND,STV_LEND_TEXT)
					End If
				ElseIf STV_LSTART_TEXT <> "" Then
					'only start level evaluation
					STV_LVLEVAL_TEXT = Replace(STextDefineLvlEvalStart,STV_LSTART,STV_LSTART_TEXT)
				ElseIf STV_LEND_TEXT <> "" Then
					'only end level evaluation
					STV_LVLEVAL_TEXT = Replace(STextDefineLvlEvalEnd,STV_LEND,STV_LEND_TEXT)
				Else
					MsgBox("Houston, we have a problem.")
					STV_LVLEVAL_TEXT = ""
				End If
				If STV_COND_TEXT = "" Then
					STV_COND_TEXT = STV_LVLEVAL_TEXT
				Else
					STV_COND_TEXT = Replace(Replace(STextDefineCondCombi,STV_COND,STV_COND_TEXT),STV_LVLEVAL,STV_LVLEVAL_TEXT)
				End If
			Next
				
			'need to start -if- construct or is it a -else if- ?
			If SI = StatPos Then
				'this is the first segment, so must be the start of an -if-
				WriteSText theOutStream,STextLSearchCondStart,sIndent
			Else
				'this is not the first segment, so must be an -else if-
				WriteSText theOutStream,STextLSearchCond,sIndent
			End If
		End If
				
		Build = ""
		Data = ""
		Data2 = ""

		If S_TYPE = "A" Then
			Build = S_P1
		ElseIf S_TYPE = "C" Then
			Build = S_P1
		ElseIf S_TYPE = "D" Then
			Build = "DataTableValue("+STextListParamD
			Data = Replace(Replace(S_P1,"|",","),"!","""") 'STextListSingleTrail
			nDataCount = UBound(Split(Data,","))
			If nDataCount < 2 Then If STextListSingleTrail <> "" Then Data = Data+STextListSingleTrail
			If S_LSTART <> "" Then
				If CInt(S_LSTART)-1 <> 0 Then
					Build = Build+","+STextDefineL+"-"+CStr(CInt(S_LSTART)-1)+")"
				Else
					Build = Build+","+STextDefineL+")"
				End If
			ElseIf S_LEND <> "" Then
				If CInt(S_LEND)-nDataCount-1 <> 0 Then
					Build = Build+","+STextDefineL+"-"+CStr(CInt(S_LEND)-nDataCount-1)+")"
				Else
					Build = Build+","+STextDefineL+")"
				End If
			Else
				Build = Build+","+STextDefineL+")"
			End If
		ElseIf S_TYPE = "E" Then
			If S_LSTART <> "" Then
				Build = "ExpFmod("+S_P1+","+S_LSTART+","+S_P2+","+STextDefineL
			Else
				Build = "ExpFmod("+S_P1+",1,"+S_P2+","+STextDefineL
			End If
			If S_P3 <> "" Then
				Build = Build+","+S_P3
			ElseIf S_P4 <> "" Then
				Build = Build+",0"
			Else
				Build = Build+STextDefinePUNDEF
			End If
			If S_P4 <> "" Then
				Build = Build+","+S_P4+")"
			Else
				Build = Build+STextDefinePNULL+")"
			End If
		ElseIf S_TYPE = "G" Then
			Build = STextListParamG 'should be 2D list of Int (Lvl to Lvl)
			Data = Replace(S_P1,"|",",")
			Data2 = Replace(S_P2,"|",",")
		ElseIf S_TYPE = "I" Then
			Build = "LinInter("+STextListParamI+","+STextDefineL+")" 'should be 1 list of Int and 1 list of Float (Lvl to Float)
			Data = Replace(S_P1,"|",",")
			Data2 = Replace(S_P2,"|",",")
		ElseIf S_TYPE = "L" Then
			If InStr(S_P1,"/") Or InStr(S_P1,"+") Or InStr(S_P1,"-") Then
				Build = "("+S_P1+")*"+STextDefineL
			ElseIf S_P1 = "1" Then
				Build = STextDefineL
			Else
				Build = S_P1+"*"+STextDefineL
			End If
			If S_P2 <> "" Then
				Build = Build+"+"+S_P2
			End If
		ElseIf S_TYPE = "P" Then
			Build = "CalcPercAB("+S_P1+","+S_P2+","+S_P3+","+STextDefineN+")"
		ElseIf S_TYPE = "R" Then
			Build = "CalcRatAB("+S_P1+","+S_P2+","+S_P3+","+STextDefineN+")"
		ElseIf S_TYPE = "S" Then
			'P1 = PntMP
			'P2 = Progression
			'P3 = Progression Base
			'P4 = Progression Adjustment
			'P5 = $N or $C (default $N)
			'P6 = Rounding Type for graph point values
			Build = "StatLinInter("""+S_P1+""","""+S_P2+""","""+S_P3+""","""+S_P4+""","+STextDefineL
			If 	S_P5 <> "" Then
				Build = Build+","+S_P5
			Else
				Build = Build+","+STextDefineN
			End If
			nRoundType = InStr("PLM",Ucase(S_P6))
			If 	S_P6 <> "" And IsNumeric(nRoundType) And nRoundType > 0 Then
				Build = Build+","+CStr(nRoundType)
			Else
				Build = Build+STextDefinePNULL
			End If
			Build = Build+")"
		ElseIf S_TYPE = "T" Then
			nRoundType = InStr("PLM",Ucase(S_P6))
			If 	S_P6 <> "" And IsNumeric(nRoundType) And nRoundType > 0 Then
				Build = "LinFmod("+S_P1+","+S_P2+","+S_P3+","+S_P4+","+S_P5+","+STextDefineL+","+CStr(nRoundType)+")"
			Else
				Build = "LinFmod("+S_P1+","+S_P2+","+S_P3+","+S_P4+","+S_P5+","+STextDefineL+STextDefinePNULL+")"
			End If
		ElseIf S_TYPE = "V" Then
			Build = "TranslateValue("+STextListParamV
			Data = Replace(Replace(S_P1,"|",","),"!","""")
			nDataCount = UBound(Split(Data,","))
			If nDataCount < 2 Then If STextListSingleTrail <> "" Then Data = Data+STextListSingleTrail
			Data2 = Replace(Replace(S_P2,"|",","),"!","""")
			nDataCount = UBound(Split(Data2,","))
			If nDataCount < 2 Then If STextListSingleTrail <> "" Then Data2 = Data2+STextListSingleTrail
			If 	S_P3 <> "" Then
				Build = Build+","+S_P3+")"
			Else
				Build = Build+","+STextDefineN+")"
			End If
		End If

		If Build <> "" Then
			If S_PROC <> "" Then
				nProcIndex = 1
				nProcLen = Len(S_PROC)
				While nProcIndex <= nProcLen
					For nDefIndex = LBound(aProcDef) To UBound(aProcDef)
						If Mid(S_PROC,nProcIndex,1) = aProcDef(nDefIndex)(0) Then
							sProcText = aProcDef(nDefIndex)(1)
							Build = Replace(sProcText,"<B>",Build)
							If InStr(sProcText,"<P>") > 0 Then
								sProcParam = ""
								sProcParamChar = Mid(S_PROC,nProcIndex+1,1)
								While ("0" <= sProcParamChar And sProcParamChar <= "9") Or sProcParamChar = "." Or sProcParamChar = "-"
									sProcParam = sProcParam+sProcParamChar
									nProcIndex = nProcIndex+1
									sProcParamChar = Mid(S_PROC,nProcIndex+1,1)
								Wend
								If sProcParam <> "" Then
									Build = Replace(Build,"<P>",","+sProcParam)
								Else
									Build = Replace(Build,"<P>",STextDefinePNULL)
								End If
							End If
							If InStr(sProcText,"<(>") > 0 Or InStr(sProcText,"<)>") > 0 Then
								If InStr(Build,"+") Or InStr(Build,"-") Or InStr(Build,"/") Then
									Build = Replace(Build,"<(>","(")
									Build = Replace(Build,"<)>",")")
								Else
									Build = Replace(Build,"<(>","")
									Build = Replace(Build,"<)>","")
								End If
							End If
							Exit For
						End If
					Next
					nProcIndex = nProcIndex+1
				Wend
			End If

			Build = Replace(Build,"!","""")
			Build = Replace(Build,"|",",")
			Build = Replace(Build,"+-","-")
			Build = Replace(Build,"--","+")
			Build = Replace(Build,",,)",")")
			Build = Replace(Build,",)",")")
			Build = Replace(Build,",undefined)",")")
			Build = ReplStatRefs(Build)
			'If STextDefineCastFloat <> "" Then
			'	Build = NumFloatText(Build)
			'	Data = NumFloatText(Data)
			'	Data2 = NumFloatText(Data2)
			'End If

			STV_CALC_TEXT = Build
			STV_DATA_TEXT = Data
			STV_DATA2_TEXT = Data2

			If bSingleAssignment Then
				sResultIndent = sIndent
			Else
				sResultIndent = sIndent+STextIndent
			End If
			If S_TYPE = "D" Then
				WriteSText theOutStream,STextSearchResultD,sResultIndent 'STextSearchResultD
			ElseIf S_TYPE = "G" Then
				WriteSText theOutStream,STextSearchResultG,sResultIndent 'STextSearchResultG
			ElseIf S_TYPE = "I" Then
				WriteSText theOutStream,STextSearchResultI,sResultIndent 'STextSearchResultI
			ElseIf S_TYPE = "V" Then
				WriteSText theOutStream,STextSearchResultV,sResultIndent 'STextSearchResultV
			Else
				WriteSText theOutStream,STextSearchResult,sResultIndent 'STextSearchResult
			End If
		End If
	Next	

	If not bSingleAssignment Then
		'must be -if- construct need to close with -end if-
		WriteSText theOutStream,STextLSearchEnd,sIndent
	End If

	STV_DATA_TEXT = oldSTV_DATA_TEXT
	STV_DATA2_TEXT = oldSTV_DATA2_TEXT
	STV_CALC_TEXT = oldSTV_CALC_TEXT
	STV_LEND_TEXT = oldSTV_LEND_TEXT
	STV_LSTART_TEXT = oldSTV_LSTART_TEXT
	STV_COND_TEXT = oldSTV_COND_TEXT

End Sub

Function ReplStatRefs(ByVal theBuild)

	Dim Result
	Result = theBuild

	Dim nSearch
	Dim BI
	Dim sStartToken, sNumToken, sEncapStartToken, sEncapEndToken
	Dim nEncapOpen
	Dim sStatName
	Dim cStatChar
	Dim sLevel
	Dim sNumber

	sStartToken = "@"
	sNumToken = "#"
	sEncapStartToken = "("
	sEncapEndToken = ")"

	nSearch = InStr(Result,sStartToken)
	While nSearch > 0
		Result = Left(Result,nSearch-1)+Right(Result,Len(Result)-nSearch+1-Len(sStartToken)) 'remove start token

		sStatName = ""
		For BI = nSearch To Len(Result)
			cStatChar = Mid(Result,BI,1)
			If ("A" <= UCase(cStatChar) And UCase(cStatChar) <= "Z") Or ("0" <= UCase(cStatChar) And UCase(cStatChar) <= "9") Then
				sStatName = sStatName+cStatChar
			Else
				Exit For	
			End If
		Next
		For BI = Len(sStatName) To 1 Step -1
			cStatChar = Mid(sStatName,BI,1)
			If "0" <= UCase(cStatChar) And UCase(cStatChar) <= "9" Then
				sStatName = Left(sStatName,BI-1)
			Else
				Exit For
			End If
		Next

		If Len(sStatName) > 0 Then
			Result = Left(Result,nSearch-1)+"CalcStat("""+sStatName+""""+Right(Result,Len(Result)-nSearch+1-Len(sStatName))
			nSearch = nSearch+Len("CalcStat("""+sStatName+"""")

			sLevel = ""
			If Mid(Result,nSearch,Len(sEncapStartToken)) = sEncapStartToken Then
				Result = Left(Result,nSearch-1)+Right(Result,Len(Result)-nSearch+1-Len(sEncapStartToken))
				nEncapOpen = 1
				For BI = nSearch To Len(Result)
					cStatChar = Mid(Result,BI,1)
					If Mid(Result,BI,Len(sEncapStartToken)) = sEncapStartToken Then
						nEncapOpen = nEncapOpen+1
					End If
					If Mid(Result,BI,Len(sEncapEndToken)) = sEncapEndToken Then
						nEncapOpen = nEncapOpen-1
					End If
					If nEncapOpen > 0 Then
						sLevel = sLevel+cStatChar
					Else
						Result = Left(Result,BI-1)+Right(Result,Len(Result)-BI+1-Len(sEncapEndToken))
						Exit For
					End If
				Next
			ElseIf "0" <= Mid(Result,nSearch,1) And Mid(Result,nSearch,1) <= "9" Then
				For BI = nSearch To Len(Result)
					cStatChar = Mid(Result,BI,1)
					If "0" <= cStatChar And cStatChar <= "9" Then
						sLevel = sLevel+cStatChar
					Else
						Exit For
					End If
				Next
			End If
			If sLevel = "" Then
				Result = Left(Result,nSearch-1)+","+STextDefineL+Right(Result,Len(Result)-nSearch+1)
				nSearch = nSearch+Len(","+STextDefineL)
			Else
				Result = Left(Result,nSearch-1)+","+sLevel+Right(Result,Len(Result)-nSearch+1-Len(sLevel))
				nSearch = nSearch+Len(","+sLevel)
			End If

			If Mid(Result,nSearch,Len(sNumToken)) = sNumToken Then
				Result = Left(Result,nSearch-1)+Right(Result,Len(Result)-nSearch+1-Len(sNumToken))

				sNumber = ""
				If Mid(Result,nSearch,Len(sEncapStartToken)) = sEncapStartToken Then
					Result = Left(Result,nSearch-1)+Right(Result,Len(Result)-nSearch+1-Len(sEncapStartToken))
					nEncapOpen = 1
					For BI = nSearch To Len(Result)
						cStatChar = Mid(Result,BI,1)
						If Mid(Result,BI,Len(sEncapStartToken)) = sEncapStartToken Then
							nEncapOpen = nEncapOpen+1
						End If
						If Mid(Result,BI,Len(sEncapEndToken)) = sEncapEndToken Then
							nEncapOpen = nEncapOpen-1
						End If
						If nEncapOpen > 0 Then
							sNumber = sNumber+cStatChar
						Else
							Result = Left(Result,BI-1)+Right(Result,Len(Result)-BI+1-Len(sEncapEndToken))
							Exit For
						End If
					Next
				ElseIf "0" <= Mid(Result,nSearch,1) And Mid(Result,nSearch,1) <= "9" Then
					For BI = nSearch To Len(Result)
						cStatChar = Mid(Result,BI,1)
						If ("0" <= cStatChar And cStatChar <= "9") Or cStatChar = "." Then
							sNumber = sNumber+cStatChar
						Else
							Exit For
						End If
					Next
				End If
				If sNumber = "" Then
					Result = Left(Result,nSearch-1)+","+STextDefineN+")"+Right(Result,Len(Result)-nSearch+1)
					nSearch = nSearch+Len(","+STextDefineN+")")
				Else
					Result = Left(Result,nSearch-1)+","+sNumber+")"+Right(Result,Len(Result)-nSearch+1-Len(sNumber))
					nSearch = nSearch+Len(","+sNumber+")")
				End If
			Else
				Result = Left(Result,nSearch-1)+STextDefinePNULL+")"+Right(Result,Len(Result)-nSearch+1)
				nSearch = nSearch+Len(STextDefinePNULL+")")
			End If

		End If

		nSearch = InStr(Result,sStartToken)
	Wend

	ReplStatRefs = Result

End Function

Sub ReadSDFile(ByVal theInFileName, ByRef theSDInfoLst, ByRef aDefines)

	If IsEmpty(aDefines) Then
		Redim aDefines(6)
		aDefines(0) = Array("$C",STextDefineC,DFT_TEXT,"")
		aDefines(1) = Array("$L",STextDefineL,DFT_TEXT,"")
		aDefines(2) = Array("$N",STextDefineN,DFT_TEXT,"")
		aDefines(3) = Array("$FLOAT",STextDefineCastFloat,DFT_TEXT,"")
		aDefines(4) = Array("$INT",STextDefineCastInt,DFT_TEXT,"")
		aDefines(5) = Array("$PNULL",STextDefinePNULL,DFT_TEXT,"")
		aDefines(6) = Array("$SN",STextDefineSN,DFT_TEXT,"")
	End If

	Dim InStream
	'This is a workaround. antivirus software seems to detect the normal, complete, string as a threat since ~ december 2024.
	'The code below is only reading data from file statdata.csv
	Set InStream = CreateObject("ado" + "db.stre" + "am")
	With InStream
		.Type = adTypeText
		.CharSet = "UTF-8"
		.LineSeparator = adLF
		.Open
		.LoadFromFile theInFileName

		Dim ReadError
		ReadError = False

		Dim RowText
		RowText = ""

		Dim nRowCounter: nRowCounter = 0

		'skip header line
		RowText = .ReadText(adReadLine)
		nRowCounter = nRowCounter+1

		Dim StoreSDInfo
		Dim CurrStatIndex: CurrStatIndex = -1
		Dim StoreDefine
		Dim DefName
		Dim DefAssign

		Do Until .EOS OR ReadError
			RowText = .ReadText(adReadLine)
			nRowCounter = nRowCounter+1

			RowText = Replace(RowText,Chr(10),"")
			RowText = Replace(RowText,Chr(13),"")

			Dim NewSDInfo
			InitSDInfo NewSDInfo

			For FldI = LBound(NewSDInfo) To UBound(NewSDInfo)
				ReadSDField RowText, NewSDInfo(FldI)
			Next

			StoreSDInfo = False
			StoreDefine = False
			If NewSDInfo(SD_SCRIPT) = SCRIPT_DEFINE Then
				'define definition line
				StoreDefine = True
			ElseIf NewSDInfo(SD_SCRIPT) = SCRIPT_STAT Then
				'might be start of stat definition section if included in compile set.
				'reset current stat index anyway. it's the end of any previous section.
				CurrStatIndex = -1
				'compile flag needs to be a number
				If NewSDInfo(SD_COMPF) = "" Then
					NewSDInfo(SD_COMPF) = "0"
				End If
				'put compile option in version stat
				If NewSDInfo(SD_STAT) = "-VERSION" Then
					Select Case CompileOption
						Case optCompileFull
							NewSDInfo(SD_P1) = Replace(NewSDInfo(SD_P1),"v","f")
						Case optCompilePercentages
							NewSDInfo(SD_P1) = Replace(NewSDInfo(SD_P1),"v","p")
						Case optCompileTraitTrees
							NewSDInfo(SD_P1) = Replace(NewSDInfo(SD_P1),"v","t")
					End Select
					SDVersion = Replace(NewSDInfo(SD_P1),"!","")
					'allways include version stat
					StoreSDInfo = True
				ElseIf CompileOption = optCompileFull Or (CInt(NewSDInfo(SD_COMPF)) And (2^(CompileOption-1))) Then
					'new stat should be in this compile set
					StoreSDInfo = True
				End If
			ElseIf NewSDInfo(SD_TYPE) = "" Then
				'no calculation just skip. it's probably a line with commentary.
			ElseIf CurrStatIndex <> -1 Then
				'must be part of current (indexed) stat section: copy some fields from main record
				NewSDInfo(SD_STAT) = theSDInfoLst(CurrStatIndex)(SD_STAT)
				NewSDInfo(SD_SCRIPT) = theSDInfoLst(CurrStatIndex)(SD_SCRIPT)
				StoreSDInfo = True
			End If
			
			If StoreSDInfo Then
				If IsEmpty(theSDInfoLst) Then
					ReDim theSDInfoLst(0)
				Else
					ReDim Preserve theSDInfoLst(UBound(theSDInfoLst)+1)
				End If
				theSDInfoLst(UBound(theSDInfoLst)) = NewSDInfo
				If NewSDInfo(SD_SCRIPT) = SCRIPT_STAT Then
					If CurrStatIndex = -1 Then
						'newly indexed stat
						CurrStatIndex = UBound(theSDInfoLst)
					End If
					'check level intervals of stat
					If NewSDInfo(SD_LSTART) = "" And NewSDInfo(SD_LEND) = "" Then
						'no intervals, so it's the end of the section (Else in case of an If construct or Assignment)
						CurrStatIndex = -1
					End If
				End If
			ElseIf StoreDefine Then
				Redim Preserve aDefines(UBound(aDefines)+1)
				DefName = UCase(NewSDInfo(SD_STAT))
				DefAssign = Replace(NewSDInfo(SD_P1),"|",",")
				If Left(DefAssign,1) = "!" And Right(DefAssign,1) = "!" Then
					aDefines(UBound(aDefines)) = Array(DefName,Mid(DefAssign,2,Len(DefAssign)-2),"")
				Else
					aDefines(UBound(aDefines)) = Array(DefName,DefAssign,DFT_EXPR,NewSDInfo(SD_PROC))
				End If
			End If
		Loop

		.Close
	End With
	Set InStream = Nothing

	'sort defines based on length of define name (longest first)
	Dim temp
	For I = UBound(aDefines)-1 To LBound(aDefines) Step -1
		For J = 0 to I
			If Len(aDefines(J)(DF_DEF)) < Len(aDefines(J+1)(DF_DEF)) Then
				temp = aDefines(J+1)
				aDefines(J+1) = aDefines(J)
				aDefines(J) = temp
			End If
		Next
	Next 
	'calculate defines which are of the expression type
	EvalDefines aDefines
	
End Sub

Sub InitSDInfo(ByRef theSDInfo)

	theSDInfo = Split(String(SDFLDCNT-1,","),",",SDFLDCNT)
				
End Sub

Sub ReadSDField(ByRef RowText, ByRef NewSDFld)

	Dim SepPos
	SepPos = InStr(RowText,SDFLDSEP)
	If SepPos > 0 Then
		NewSDFld = RTrim(Mid(RowText,1,SepPos-1))
		RowText = LTrim(Mid(RowText,SepPos+1,Len(RowText)-SepPos))
	Else
		NewSDFld = Rtrim(RowText)
		RowText = ""
	End If

	If Len(NewSDFld) >= 2 Then
		If Mid(NewSDFld,1,1) = SDSTRQUOT AND Mid(NewSDFld,Len(NewSDFld),1) = SDSTRQUOT Then
			NewSDFld = Mid(NewSDFld,2,Len(NewSDFld)-2)
		End If
	End If

End Sub

Function Eval(ByVal Expr)
  Do While HandleParentheses(Expr): Loop

  Dim L, R
  If Spl(Expr, ",", L, R) Then
    Eval = Eval(L) & "@" & Eval(R)
	Exit Function
  End If
  If Spl(Expr, "Or", L, R) Then:   Eval = Eval(L) Or Eval(R):   Exit Function
  If Spl(Expr, "And", L, R) Then:  Eval = Eval(L) And Eval(R):  Exit Function
  If Spl(Expr, ">=", L, R) Then:   Eval = Eval(L) >= Eval(R):   Exit Function
  If Spl(Expr, "<=", L, R) Then:   Eval = Eval(L) <= Eval(R):   Exit Function
  If Spl(Expr, "=", L, R) Then:    Eval = Eval(L) = Eval(R):    Exit Function
  If Spl(Expr, ">", L, R) Then:    Eval = Eval(L) > Eval(R):    Exit Function
  If Spl(Expr, "<", L, R) Then:    Eval = Eval(L) < Eval(R):    Exit Function
  If Spl(Expr, "Like", L, R) Then: Eval = Eval(L) Like Eval(R): Exit Function
  If Spl(Expr, "&", L, R) Then:    Eval = Eval(L) & Eval(R):    Exit Function
  If Spl(Expr, "-", L, R) Then:    Eval = Eval(L) - Eval(R):    Exit Function
  If Spl(Expr, "+", L, R) Then:    Eval = Eval(L) + Eval(R):    Exit Function
  If Spl(Expr, "Mod", L, R) Then:  Eval = Eval(L) Mod Eval(R):  Exit Function
  If Spl(Expr, "\", L, R) Then:    Eval = Eval(L) \ Eval(R):    Exit Function
  If Spl(Expr, "*", L, R) Then:    Eval = Eval(L) * Eval(R):    Exit Function
  If Spl(Expr, "/", L, R) Then:    Eval = Eval(L) / Eval(R):    Exit Function
  If Spl(Expr, "^", L, R) Then:    Eval = Eval(L) ^ Eval(R):    Exit Function
  If Trim(Expr) >= "A" Then:       Eval = Fnc(Expr):            Exit Function
  If Len(Expr) Then:               Eval = IIf(InStr(Expr, "'"), Replace(Trim(Expr), "'", ""), IIf(InStr(Expr, "@"), Expr, Val(Expr)) )
End Function

Private Function HandleParentheses(Expr)
  Dim P: P = InStr(Expr, "(")
  If P Then HandleParentheses = True Else Exit Function

  Dim i, C: C = 0
  For i = P To Len(Expr)
    If Mid(Expr, i, 1) = "(" Then C = C + 1
    If Mid(Expr, i, 1) = ")" Then C = C - 1
    If C = 0 Then Exit For
  Next

  Dim xEval: xEval = Eval(Mid(Expr, P + 1, i - P - 1))
  If VarType(xEval) <> 8 Then
    xEval = Str(xEval)
  End If
  Expr = Left(Expr, P - 1) & xEval & Mid(Expr, i + 1)
End Function

Private Function Spl(Expr, Op, L, R)
  Dim P: P = InStrRev(Expr, Op, -1 , 1)
  If P Then Spl = True Else Exit Function
  If P < InStrRev(Expr, "'") And InStr("*-", Op) Then P = InStrRev(Expr, "'", P) - 1

  R = Mid(Expr, P + Len(Op))
  L = Trim(Left(Expr, IIf(P > 0, P - 1, 0)))

  Dim testthis: testthis = Right(L, 1)
  Select Case testthis
    Case "", "+", "*", "/", "@": Spl = False
    Case "-": R = "-" & R
	Case Else: If testthis >= "A" And testthis <= "z" Then Spl = False
  End Select
End Function

Private Function Fnc(Expr)
  Expr = LCase(Trim(Expr))

  Select Case Left(Expr, 5)
	Case "round":
		Dim Params: Params = Mid(Expr, 6)
		Dim L, R
		Dim iDiv: iDiv = InStr(Params, "@")
		If iDiv > 0 Then
			L = Left(Params,iDiv-1)
			R = Mid(Params,iDiv+1)
			Fnc = RoundDbl(Val(L),Val(R))
		Else
			Fnc = RoundDbl(Val(Params),Nothing)
		End If
		Exit Function
  End Select

  Select Case Left(Expr, 3)
    Case "abs": Fnc = Abs(Val(Mid(Expr, 4)))
    Case "sin": Fnc = Sin(Val(Mid(Expr, 4)))
    Case "cos": Fnc = Cos(Val(Mid(Expr, 4)))
    Case "atn": Fnc = Atn(Val(Mid(Expr, 4)))
    Case "log": Fnc = Log(Val(Mid(Expr, 4)))
    Case "exp": Fnc = Exp(Val(Mid(Expr, 4)))
    'etc...
  End Select
End Function

Private Function IIf(Cond, Value1, Value2)
  If Cond Then IIf = Value1 Else IIf = Value2
End Function

Private Function Str(NumValue)
  Str = CStr(NumValue)
End Function

Private Function Val(StrValue)
  Val = CDbl(StrValue)
End Function

Private Const DblCalcDev = 0.00000001

' ****************** Misc. floating point support functions ******************

' Misc. functions for floating point rounding.
' 2nd parameter is number of decimals.

Private Function RoundDbl(ByVal dNum, ByVal vDec)

	Dim dCorrection: dCorrection = 0.5+DblCalcDev
	Dim iSign: iSign = 1
	If dNum < 0 Then
		iSign = -1
	End If
	If VarType(vDec) = 9 Then
		RoundDbl = iSign*Int(iSign*dNum+dCorrection)
	ElseIf -DblCalcDev <= vDec And vDec <= DblCalcDev Then
		RoundDbl = iSign*Int(iSign*dNum+dCorrection)
	Else
		Dim dFactor: dFactor = 10^vDec
		RoundDbl = iSign*Int(iSign*dNum*dFactor+dCorrection)/dFactor
	End If

End Function
