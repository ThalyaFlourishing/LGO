-- Created by Giseldah

local p = _G --p stands for package

import "Giseldah.CalcStat.CalcStatFull" -- import from Lotro plugin environment

-- local used library functions
local mathabs = math.abs
local mathceil = math.ceil
local mathfloor = math.floor
local mathlog10 = math.log10
local mathmax = math.max
local mathmin = math.min
local mathsqrt = math.sqrt
local strfind = string.find
local strformat = string.format
local strgsub = string.gsub
local strmatch = string.match
local strreverse = string.reverse
local strsub = string.sub
local strupper = string.upper
local tableconcat = table.concat
local tableinsert = table.insert
local tableremove = table.remove
local tablesort = table.sort

-- CalcStat module access
local csm = _G
-- CalcStat
local trim = csm.trim
local DblCalcDev = csm.DblCalcDev
local RoundDbl = csm.RoundDbl
local RoundDblDown = csm.RoundDblDown
local RoundDblUp = csm.RoundDblUp
local RoundDblLotro = csm.RoundDblLotro
local RoundDblMorReg = csm.RoundDblMorReg
local RoundDblProg = csm.RoundDblProg
local EquSng = csm.EquSng
local DecSng = csm.DecSng
local LinFmod = csm.LinFmod
local CalcStat = csm.CalcStat

-- ******************** Start Formatting/Display Support **********************

-- small number correction for display purposes only (like in-game)
local function correctvalue(nValue)
	if (nValue % 1) == 0 then
		return nValue -- don't adjust whole numbers
	elseif nValue < -DblCalcDev then
		return nValue-0.0002
	elseif nValue > DblCalcDev then
		return nValue+0.0002
	else
		return 0
	end
end

-- uses a template to output value(format ..ggggssscc) as currency (..gggg gold sss silver cc copper)
local function currencyvalue(nValue,frame,sTemplateTitle)
	local nTemp = mathfloor(nValue+0.5+DblCalcDev)
	local nCopper = nTemp%100
	nTemp = (nTemp-nCopper)/100
	local nSilver = nTemp%1000
	nTemp = (nTemp-nSilver)/1000
	local nGold = nTemp

	local aTemplateArgs
	if nGold > 0 then
		aTemplateArgs = {g=nGold,s=nSilver,c=nCopper}
	elseif nSilver > 0 then
		aTemplateArgs = {s=nSilver,c=nCopper}
	else
		aTemplateArgs = {c=nCopper}
	end

	return frame:expandTemplate{title=sTemplateTitle,args=aTemplateArgs}
end

local function adjustfordisplay(sOption,xValue,frame,bTable)
	local xTemp = xValue
	local bTableDisplay = false
	if type(bTable) == "boolean" then
		bTableDisplay = bTable
	end
	if type(xTemp) == "number" then
		-- optional display options for percentages
		-- for display purposes only (same as in-game)
		if sOption == "MULTIP" then
			-- multiplier percentage
			xTemp = correctvalue(xTemp-100)
		elseif sOption == "MULTIP100" then
			-- multiplier percentage & multiply by 100
			xTemp = correctvalue(xTemp*100-100)
		elseif sOption == "ADDP" then
			-- additive percentage
			xTemp = correctvalue(xTemp)
		elseif sOption == "ADDP100" then
			-- additive percentage & multiply by 100
			xTemp = correctvalue(xTemp*100)
		elseif sOption == "WORTH" then
			-- currency display
			if frame then
				if bTableDisplay then
					xTemp = currencyvalue(xTemp,frame,"Cost")
				else
					xTemp = currencyvalue(xTemp,frame,"Worth")
				end
			end
		elseif sOption == "CORR" then
			-- fractured value which needs a correction for rounded display
			xTemp = correctvalue(xTemp)
		elseif sOption == "NOADJ" then
			-- no adjustment
		end
	end
	return xTemp
end

local function AddCommas(pre,post) return pre..strreverse(strgsub(strreverse(post),"(%d%d%d)","%1,")) end
local function SplitDecimal(str) return strgsub(str,"^(%d)(%d+)",AddCommas) end
local function ReformatThousSep(str) return strgsub(str,"[%d%.]+",SplitDecimal) end

-- adds trailing zero removal option (set minimum precision): %.0.3f
-- adds thousand separator option to formatting: %'d
local function stringformatvalue(sFormat,xValue)
	if type(xValue) == "number" then
		local sPreFormat = sFormat

		local bPostThousSep = strfind(sPreFormat,"%%'")
		if bPostThousSep then
			sPreFormat = strgsub(sPreFormat,"%%'","%%")
		end

		local bPostFormatTrailZeroes = strfind(sPreFormat,".%d+.%d+%a")
		local nTrailZeroesMinPrec

		if bPostFormatTrailZeroes then
			local _, _, sTrailZeroesMinPrec = strfind(sPreFormat,".(%d+).%d+%a")
			nTrailZeroesMinPrec = tonumber(sTrailZeroesMinPrec)
			sPreFormat = strgsub(sPreFormat,".%d+(.%d+%a)","%1")
		end

		local sPostFormat = strformat(sPreFormat,xValue)

		if bPostFormatTrailZeroes then
			local sTrailZeroesGetDecimals = "%d+(.%d+)"
			local sTrailZeroesSetDecimals = "(%d+).%d+"
			local _, _, sDotDecimals = strfind(sPostFormat,sTrailZeroesGetDecimals)

			while sDotDecimals and #sDotDecimals > nTrailZeroesMinPrec+1 and strsub(sDotDecimals,-1,-1) == "0" do
				sDotDecimals = strsub(sDotDecimals,1,-2)
			end

			if sDotDecimals == "." then
				sPostFormat = strgsub(sPostFormat,sTrailZeroesSetDecimals,"%1")
			elseif sDotDecimals then
				sPostFormat = strgsub(sPostFormat,sTrailZeroesSetDecimals,"%1"..sDotDecimals)
			end
		end

		if bPostThousSep then
			sPostFormat = ReformatThousSep(sPostFormat)
		end

		return sPostFormat
	else
		return strformat(sFormat,xValue)
	end
end

local function findstatname(sStatName,aStats)
	for _, sStat in ipairs(aStats) do
		sSNmatch, sPmatch = strmatch(sStatName,"("..sStat..")(P?)")
		if sSNmatch and sPmatch ~= "P" then
			-- found statname and without p(ercent) behind it
			return true
		end
	end
	return false
end

local function statdefaultformat(sStatName)
	-- armour main stats: whole number with thousand separator
	if findstatname(sStatName,{"ARMOURLOW","CLOTHARMOUR"}) then
		return "%'d"
	end
	-- player level armour: whole number without thousand separator
	if findstatname(sStatName,{"ARMOUR%a+","%w+ARMOUR"}) then
		return "%.0f"
	end
	-- main stats: whole number with thousand separator
	if findstatname(sStatName,{"MAIN","AGILITY","FATE","MIGHT","VITALITY","WILL","ARMOUR"}) then
		return "%'d"
	end
	-- fractured stats: whole number with thousand separator
	if findstatname(sStatName,{"WPNDMGMIN","WPNDMGMAX","SKILLPOWERCOST"}) then
		return "%'.0f"
	end
	-- fractured stats: thousand separator and 1 decimal
	if findstatname(sStatName,{"WPNDPS"}) then
		return "%'.1f"
	end
	-- fractured stats: minimum no decimals and maximum 3 decimals
	if findstatname(sStatName,{"ICMR","ICPR","NCMR","NCPR","POWER"}) then
		return "%.0.3f"
	end
	-- default: whole number without thousand separator
	return "%.0f"
end

local function statdefaultdisplay(sStatName)
	if findstatname(sStatName,{"WORTHVAL"}) then
		return "WORTH"
	end
	return "CORR"
end

local function DisplayableStatValue(sSN,xValue,sFormat,sDisplay,frame,bTable)
	local result = xValue

	-- display option. default = "CORR" for numbers.
	local sStatDisplay = ""
	if sDisplay then
		sStatDisplay = sDisplay
	end
	if (sStatDisplay == "") and (type(result) == "number") then
		sStatDisplay = statdefaultdisplay(sSN,bTable)
	end
	if sStatDisplay ~= "" then
		result = adjustfordisplay(sStatDisplay,result,frame,bTable)
	end

	-- optional format string. default = "%.0f" for numbers.
	local sStatFormat = ""
	if sFormat then
		sStatFormat = sFormat
	end
	if (sStatFormat == "") and (type(result) == "number") then
		sStatFormat = statdefaultformat(sSN,bTable)
	end
	if sStatFormat ~= "" then
		result = stringformatvalue(sStatFormat,result,frame,bTable)
	end

	return result
end

local EXPRERR_AT_TRANSLATION = 1
local EXPRERR_AT_EXECUTION = 2
local EXPRERR_AT_INITIALIZATION = 3
-- Fields:
-- Source - text describing origin
-- Code - error code number
-- Message - error text
-- Detail - detailed description
-- Stage - Expressions stage
local function DisplayableError(e,nl)
	local sNewLine = "<br>"
	if nl then sNewLine = nl end
	local aDisplay = {}
	if e.Stage then
		local aErrStage = {"Translation error","Execution error","Initialization error"}
		tableinsert(aDisplay,aErrStage[e.Stage])
	else
		tableinsert(aDisplay,"Error")
	end
	if e.Source then
		tableinsert(aDisplay," in ")
		tableinsert(aDisplay,e.Source)
	end
	tableinsert(aDisplay,sNewLine)
	if e.Code then
		tableinsert(aDisplay,"Code: ")
		tableinsert(aDisplay,e.Code)
		if e.Message then
			tableinsert(aDisplay," (")
			tableinsert(aDisplay,e.Message)
			tableinsert(aDisplay,")")
		end
		tableinsert(aDisplay,sNewLine)
	elseif e.Message then
		tableinsert(aDisplay,e.Message)
		tableinsert(aDisplay,sNewLine)
	end
	if e.Detail then
		tableinsert(aDisplay,"Detail: ")
		tableinsert(aDisplay,e.Detail)
		tableinsert(aDisplay,sNewLine)
	end
	return tableconcat(aDisplay)
end

-- to be used by other modules
p.DisplayableStatValue = DisplayableStatValue
p.DisplayableError = DisplayableError

-- ********************* End Formatting/Display Support ***********************

-- *********************** Start CalcStat Expressions *************************

local ERR =
	{
		TOOMANYVARIABLES = {1, "Too many variables used"},
		SYNTAX = {2, "Syntax error"},
		INVALIDOPCOUNT = {3, "Invalid number of operands"},
		INVALIDOPTYPE = {4, "Invalid operand type"},
		OUTOFRANGE = {5, "Out of range"},
		EXECUTION = {6, "Execution error"},
		UNKNOWNUNARY = {7, "Unknown unary operator"},
		UNKNOWNBINARY = {8, "Unknown binary operator"},
		MISSINGBINARY = {9, "Missing binary operator"},
		MISSINGOPERAND = {10, "Unidentified operand"},
		UNKNOWNFUNCTION = {11, "Unknown function"},
		MISSINGEXPRESSION = {12, "Missing expression"},
		NONMATCHINGOPTYPE = {13, "Non-matching operand type"},
		UNINITIALIZEDVARIABLE = {14, "Uninitialized variable"},
		MAXPROCESSING = {15, "Reached maximum processing requests"},
		MISSINGSWITCHDEFAULT = {16, "Missing Switch Default expression"},
		MISSINGSWITCHCONDITION = {17, "Missing Switch Case condition"},
		SYNTAXSWITCH = {18, "Syntax error in Switch"},
		MISSINGASSIGNVARIABLE = {19, "Missing assignment variable"},
		MISSINGASSIGNSIGN = {20, "Missing assignment token '='"},
		MISSINGASSIGNEXPRESSION = {21, "Missing assignment expression"},
		MISSINGSESELDEFAULT = {22, "Missing SEsel Default expression"},
		PARAMGROUPSOVERFLOW = {23, "Too many parameter groups"},
		MISSINGPARAMGROUPS = {24, "Missing parameter group(s)"},
		SWITCHGROUPSOVERFLOW = {25, "Parameter group after Switch Default"},
		CALCSTATSHORTSYNTAX = {26, "Invalid stat reference"},
		CALCSTATSHORTDOUBLELVL = {27, "Double defined level in stat reference"},
		UNKNOWNERROR = {28, "Unknown error"},
		MISSINGTRANSLATION = {29, "Missing Translation object"},
		DOUBLEGRAPHKEY = {30, "Graph point Key already exists"}
	}

local TRANS_EXPRESSION = 1 -- source expression
local TRANS_EXPRWIP = 2 -- expression in various stages of decomposition during translation - will contain result operand [n] when successfully finished
local TRANS_RESULTOP = 3 -- index of result operand
local TRANS_ERROR = 4 -- encountered error during translation or execution
local TRANS_OPERATIONS = 5 -- extracted operations, to be performed on..
local TRANS_OPERANDS = 6 -- extracted operands
local TRANS_OPSOURCEIDX = 7 -- search index: key=operand source, value=operand index
local TRANS_VARIABLES = 8 -- extracted and preset variables
local TRANS_VARNAMEIDX = 9 -- search index: key=variable name, value=variable index
local TRANS_OPERANDREQUESTS = 10 -- processed number of operand requests
local TRANS_MAXOPERANDREQUESTS = 11 -- maximum number of operand requests allowed in each execution
local TRANS_EXECUTIONS = 12 -- number of executions run

local OPERAND_VALUE = 1 -- value of operand directly in case it's a constant
local OPERAND_OPERATION = 2 -- operation (index) which gives value to this operand (negative indicates a variable index instead), absent for a constant

local VAR_NAME = 1 -- name of variable
local VAR_VALUE = 2 -- current value of variable

local OPERATION_DEFINITION = 1 -- index into aOperationDefs table
local OPERATION_OPERANDS = 2 -- parameter operand indexes into TRANS_OPERANDS

local OPERATION_FORMAT_UNARY = 1 -- like: operator op2 - always needs 1 operand
local OPERATION_FORMAT_BINARY = 2 -- like: op1 operator op2 - always needs 2 operands
local OPERATION_FORMAT_FUNCTION = 3 -- like functionname(op1,op2,op3,etc) - needs an unspecified number of operands
local OPERATION_FORMAT_SWITCH = 4 -- like Switch($L, case<=10: case>=100: 25, default: 13)
local OPERATION_FORMAT_WITH = 5 -- like With($A=@Agility#2, $B=20: IIf($A>$B,$A,$B))
local OPERATION_FORMAT_UNTIL = 6 -- like Until(init vars:loop vars:end condition:return value)
local OPERATION_FORMAT_WHILE = 7 -- like While(init vars:run condition:loop vars:return value)

local OPERATIONDEF_OPERATOR = 1 -- simple mathematical/logical or functions/branching/loops
local OPERATIONDEF_FORMAT = 2 -- type of operation
local OPERATIONDEF_EXECUTE = 3 -- execution function, which performs the operation
local aOperationDefs
-- Operation Definition 'sections' for optimized scanning
local nOpDefUnaryStart, nOpDefUnaryEnd = 1, 3
local nOpDefBinaryStart, nOpDefBinaryEnd = 4, 18
local nOpDefFunctionStart, nOpDefFunctionEnd = 19, 0

local TYPEBOOL = "boolean"
local TYPENUM = "number"
local TYPENUMBOOLSTR = "number;boolean;string"
local TYPENUMSTR = "number;string"
local TYPEOP1 = "operand1"
local TYPESTR = "string"

local function RaiseExternalError(aTranslation,e)
	if aTranslation[TRANS_EXECUTIONS] == 0 then
		e.Stage = EXPRERR_AT_TRANSLATION
	else
		e.Stage = EXPRERR_AT_EXECUTION
	end
	aTranslation[TRANS_ERROR] = e
	error(e,0)
end

local function SetExpressionsError(aTranslation,aERR,sDetail)
	local e = {}
	e.Source = "CalcStat:Expressions"
	if aTranslation[TRANS_EXECUTIONS] == 0 then
		e.Stage = EXPRERR_AT_TRANSLATION
	else
		e.Stage = EXPRERR_AT_EXECUTION
	end
	e.Code = aERR[1]
	e.Message = aERR[2]
	if sDetail then
		if trim(sDetail) ~= "" then
			e.Detail = sDetail
		end
	elseif trim(aTranslation[TRANS_EXPRWIP]) ~= "" then
		e.Detail = trim(aTranslation[TRANS_EXPRWIP])
	end
	aTranslation[TRANS_ERROR] = e
end

local function RaiseExpressionsError(aTranslation,aERR,sDetail)
	SetExpressionsError(aTranslation,aERR,sDetail)
	error(aTranslation[TRANS_ERROR],0)
end

-- to be used by public functions: return either a valid value/object or an Error object depending on internal error condition
local function ErrorProtectedCall(aTranslation,fProtFunction,xFuncParam)
	local bWithoutError, xResult = pcall(fProtFunction,aTranslation,xFuncParam)
	if bWithoutError then
		return xResult
	elseif type(xResult) == "table" then
		-- xResult is a CalcStat/Expressions error object
		return nil, xResult
	else
		-- might be a programming error
		SetExpressionsError(aTranslation,ERR.UNKNOWNERROR,xResult)
		return nil, aTranslation[TRANS_ERROR]
	end
end

-- executes a single operation and returns the result
local function ExecuteOperation(aTranslation,aOperation)
	return aOperationDefs[aOperation[OPERATION_DEFINITION]][OPERATIONDEF_EXECUTE](aTranslation,aOperation)
end

local function GetOperandValue(aTranslation,nOp)
	if aTranslation[TRANS_OPERANDREQUESTS] >= aTranslation[TRANS_MAXOPERANDREQUESTS] then
		RaiseExpressionsError(aTranslation,ERR.MAXPROCESSING)
	end
	aTranslation[TRANS_OPERANDREQUESTS] = aTranslation[TRANS_OPERANDREQUESTS]+1
	local aOperand = aTranslation[TRANS_OPERANDS][nOp]
	local nOperationIdx = aOperand[OPERAND_OPERATION]
	if nOperationIdx then
		-- need to get the value from an operation
		if nOperationIdx < 0 then
			-- need to get value from variable
			local aVar = aTranslation[TRANS_VARIABLES][-nOperationIdx]
			if aVar[VAR_VALUE] == nil then
				RaiseExpressionsError(aTranslation,ERR.UNINITIALIZEDVARIABLE,"$"..aVar[VAR_NAME])
			end
			return aVar[VAR_VALUE]
		else
			-- we should get the value when we execute its operation
			return ExecuteOperation(aTranslation,aTranslation[TRANS_OPERATIONS][nOperationIdx])
		end
	else
		-- otherwise return a constant value
		return aOperand[OPERAND_VALUE]
	end
end

local function ErrorDisplayOperation(aOperation)
	local sDisp = aOperationDefs[aOperation[OPERATION_DEFINITION]][OPERATIONDEF_OPERATOR]
	if nOpDefUnaryStart <= aOperation[OPERATION_DEFINITION] and aOperation[OPERATION_DEFINITION] <= nOpDefUnaryEnd then return "Unary operation '"..sDisp.."'"
	elseif nOpDefBinaryStart <= aOperation[OPERATION_DEFINITION] and aOperation[OPERATION_DEFINITION] <= nOpDefBinaryEnd then return "Binary operation '"..sDisp.."'"
	elseif nOpDefFunctionStart <= aOperation[OPERATION_DEFINITION] and aOperation[OPERATION_DEFINITION] <= nOpDefFunctionEnd then return "Function '"..sDisp.."'"
	else return "" end
end

local function ErrorDisplayOperand(xOperand)
	if type(xOperand) == "string" then
		local sDisp = "\""..xOperand.."\""
		if #sDisp > 20 then
			return strsub(sDisp,1,18)..".."
		else
			return sDisp
		end
	elseif type(xOperand) == "boolean" then if xOperand then return "True" else return "False" end
	else return ""..xOperand end
end

local PARAMDEF_TYPE = 1 -- allowed operand type(s)
local PARAMDEF_REQUIRED = 2 -- parameter/operand is required (optional if false)
local PARAMDEF_DEFAULT = 3 -- default value

-- executes operation function on provided operands under constraints of parameter definitions
local function ExecDefault(aTranslation,aOperation,fTheFunc,aParamDefs)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount, nOpMaxCount = 0, #aParamDefs
	for _, aParamDef in ipairs(aParamDefs) do
		if aParamDef[PARAMDEF_REQUIRED] then
			nOpMinCount = nOpMinCount+1
		else
			break -- remainder must be non-required - no need to check further
		end
	end
	if nOpCount < nOpMinCount or nOpMaxCount < nOpCount  then
		if nOpMinCount == nOpMaxCount then
			RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting:"..nOpMinCount.."]")
		else
			RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting:"..nOpMinCount.."-"..nOpMaxCount.."]")
		end
	end
	local xOperand = {nil,nil,nil} -- maximum is 3 operands
	local sTypeParam, nTypeOp, sTypeOp
	for nParam, aParamDef in ipairs(aParamDefs) do
		if aParamDef[PARAMDEF_REQUIRED] or nParam <= nOpCount then
			xOperand[nParam] = GetOperandValue(aTranslation,aOperands[nParam])
			sTypeParam = type(xOperand[nParam])
			if strsub(aParamDef[PARAMDEF_TYPE],1,7) == "operand" then
				nTypeOp = tonumber(strsub(aParamDef[PARAMDEF_TYPE],8,-1))
				sTypeOp = type(xOperand[nTypeOp])
				if sTypeOp ~= sTypeParam then
					RaiseExpressionsError(aTranslation,ERR.NONMATCHINGOPTYPE,ErrorDisplayOperation(aOperation).." [Operand"..nParam.."("..ErrorDisplayOperand(xOperand[nParam])..")"..":"..sTypeParam..", Expecting same type as Operand"..nTypeOp.."("..ErrorDisplayOperand(xOperand[nTypeOp])..")"..":"..sTypeOp.."]")
				end
			else
				if not strfind(aParamDef[PARAMDEF_TYPE],sTypeParam) then
					RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Operand"..nParam.."("..ErrorDisplayOperand(xOperand[nParam])..")"..":"..sTypeParam..", Expecting:"..aParamDef[PARAMDEF_TYPE].."]")
				end
			end
		else
			xOperand[nParam] = aParamDef[PARAMDEF_DEFAULT]
		end
	end
	if xOperand[3] ~= nil then return fTheFunc(xOperand[1],xOperand[2],xOperand[3])
	elseif xOperand[2] ~= nil then return fTheFunc(xOperand[1],xOperand[2])
	elseif xOperand[1] ~= nil then return fTheFunc(xOperand[1])
	else return fTheFunc()
	end
end

-- executes a loop operation with pre-condition (While) or post-condition (Until)
local function ExecLoopOp(aTranslation,aOperation,bPreCondition)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local xOperand, sTypeOp
	-- process initial variable assignments
	local nInitVarCount = GetOperandValue(aTranslation,aOperands[1])
	local nInitVarOpIdx = 2
	for nInitVarIdx = 1, nInitVarCount do
		aTranslation[TRANS_VARIABLES][-aTranslation[TRANS_OPERANDS][aOperands[nInitVarOpIdx]][OPERAND_OPERATION]][VAR_VALUE] = GetOperandValue(aTranslation,aOperands[nInitVarOpIdx+1])
		nInitVarOpIdx = nInitVarOpIdx+2
	end
	-- init indexes
	local nLoopCondOpIdx
	local nLoopVarCount
	local nLoopVarOpIdxInit
	local nResultingOpIdx
	if bPreCondition then
		nLoopCondOpIdx = nInitVarOpIdx
		nLoopVarCount = GetOperandValue(aTranslation,aOperands[nInitVarOpIdx+1])
		nLoopVarOpIdxInit = nInitVarOpIdx+2
		nResultingOpIdx = nInitVarOpIdx+2+nLoopVarCount*2
	else
		nLoopVarCount = GetOperandValue(aTranslation,aOperands[nInitVarOpIdx])
		nLoopCondOpIdx = nInitVarOpIdx+nLoopVarCount*2+1
		nLoopVarOpIdxInit = nInitVarOpIdx+1
		nResultingOpIdx = nLoopCondOpIdx+1
	end
	local nLoopVarOpIdx
	while true do
		if bPreCondition then
			-- process pre-loop condition
			xOperand = GetOperandValue(aTranslation,aOperands[nLoopCondOpIdx])
			sTypeOp = type(xOperand)
			if not strfind(TYPENUMBOOLSTR,sTypeOp) then
				RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Loop Condition Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
			end
			if not xOperand then
				break -- end of loop if false
			end
		end
		-- process loop variable assignments
		nLoopVarOpIdx = nLoopVarOpIdxInit
		for nLoopVarIdx = 1, nLoopVarCount do
			aTranslation[TRANS_VARIABLES][-aTranslation[TRANS_OPERANDS][aOperands[nLoopVarOpIdx]][OPERAND_OPERATION]][VAR_VALUE] = GetOperandValue(aTranslation,aOperands[nLoopVarOpIdx+1])
			nLoopVarOpIdx = nLoopVarOpIdx+2
		end
		if not bPreCondition then
			-- process post-loop condition
			xOperand = GetOperandValue(aTranslation,aOperands[nLoopCondOpIdx])
			sTypeOp = type(xOperand)
			if not strfind(TYPENUMBOOLSTR,sTypeOp) then
				RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Loop Condition Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
			end
			if xOperand then
				break -- end of loop if true
			end
		end
	end
	-- process result
	return GetOperandValue(aTranslation,aOperands[nResultingOpIdx])
end

-- list of operation definitions
-- longer function names/operators should come first in the list if one starts with the other: like '<=' before '<' or 'ROUNDUP' before 'ROUND'

aOperationDefs = {

{"NOT",OPERATION_FORMAT_UNARY, -- 1
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand) return not xOperand end,{{TYPENUMBOOLSTR,true}})
end
},

{"-",OPERATION_FORMAT_UNARY, -- 2
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand) return -xOperand end,{{TYPENUM,true}})
end
},

{"+",OPERATION_FORMAT_UNARY, -- 3
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand) return xOperand end,{{TYPENUM,true}})
end
},

{"..",OPERATION_FORMAT_BINARY, -- 4
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,
		function(xOperand1,xOperand2)
			if type(xOperand1) == "boolean" then
				if xOperand1 then
					xOperand1 = "True"
				else
					xOperand1 = "False"
				end
			end
			if type(xOperand2) == "boolean" then
				if xOperand2 then
					xOperand2 = "True"
				else
					xOperand2 = "False"
				end
			end
			return xOperand1..""..xOperand2
		end,
		{{TYPENUMBOOLSTR,true},{TYPENUMBOOLSTR,true}})
end
},

{"OR",OPERATION_FORMAT_BINARY, -- 5
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount, nOpMaxCount = 2, 2
	if nOpCount < nOpMinCount or nOpMaxCount < nOpCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting:"..nOpMinCount.."]")
	end
	local xOperand, sTypeOp
	xOperand = GetOperandValue(aTranslation,aOperands[1])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUMBOOLSTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Operand1("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
	end
	if xOperand then return xOperand end
	xOperand = GetOperandValue(aTranslation,aOperands[2])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUMBOOLSTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Operand2("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
	end
	if xOperand then return xOperand end
	return false
end
},

{"AND",OPERATION_FORMAT_BINARY, -- 6
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount, nOpMaxCount = 2, 2
	if nOpCount < nOpMinCount or nOpMaxCount < nOpCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting:"..nOpMinCount.."]")
	end
	local xOperand, sTypeOp
	xOperand = GetOperandValue(aTranslation,aOperands[1])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUMBOOLSTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Operand1("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
	end
	if not xOperand then return false end
	xOperand = GetOperandValue(aTranslation,aOperands[2])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUMBOOLSTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Operand2("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
	end
	if not xOperand then return false end
	return xOperand
end
},

{">=",OPERATION_FORMAT_BINARY, -- 7
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 >= xOperand2 end,{{TYPENUMSTR,true},{TYPEOP1,true}})
end
},

{"<=",OPERATION_FORMAT_BINARY, -- 8
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 <= xOperand2 end,{{TYPENUMSTR,true},{TYPEOP1,true}})
end
},

{"==",OPERATION_FORMAT_BINARY, -- 9
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 == xOperand2 end,{{TYPENUMSTR,true},{TYPEOP1,true}})
end
},

{"~=",OPERATION_FORMAT_BINARY, -- 10
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 ~= xOperand2 end,{{TYPENUMSTR,true},{TYPEOP1,true}})
end
},

{">",OPERATION_FORMAT_BINARY, -- 11
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 > xOperand2 end,{{TYPENUMSTR,true},{TYPEOP1,true}})
end
},

{"<",OPERATION_FORMAT_BINARY, -- 12
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 < xOperand2 end,{{TYPENUMSTR,true},{TYPEOP1,true}})
end
},

{"-",OPERATION_FORMAT_BINARY, -- 13
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 - xOperand2 end,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"+",OPERATION_FORMAT_BINARY, -- 14
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 + xOperand2 end,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"%",OPERATION_FORMAT_BINARY, -- 15
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 % xOperand2 end,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"*",OPERATION_FORMAT_BINARY, -- 16
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 * xOperand2 end,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"/",OPERATION_FORMAT_BINARY, -- 17
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 / xOperand2 end,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"^",OPERATION_FORMAT_BINARY, -- 18
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,function(xOperand1,xOperand2) return xOperand1 ^ xOperand2 end,{{TYPENUM,true},{TYPENUM,true}})
end
},

-- from here only functions -- 19

{"CALCSTAT",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,
		function(sStatName,nLvl,xNorC)
			local xResult, e = CalcStat(sStatName,nLvl,xNorC)
			if e then
				-- pass error encountered in CalcStat
				RaiseExternalError(aTranslation,e)
			end
			return xResult
		end,
		{{TYPESTR,true},{TYPENUM,false},{TYPENUMSTR,false}})
end
},

{"IIF",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount, nOpMaxCount = 3, 3
	if nOpCount < nOpMinCount or nOpMaxCount < nOpCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting:"..nOpMinCount.."]")
	end
	local xOperand, sTypeOp
	xOperand = GetOperandValue(aTranslation,aOperands[1])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUMBOOLSTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Condition Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
	end
	if xOperand then
		return GetOperandValue(aTranslation,aOperands[2])
	else
		return GetOperandValue(aTranslation,aOperands[3])
	end
end
},

{"ISVALID",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount, nOpMaxCount = 1, 1 -- need exactly 1 operand (the variable)
	if nOpCount < nOpMinCount or nOpMaxCount < nOpCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting:"..nOpMinCount.."]")
	end
	-- get operation index for the operand
	local nOperationIdx = aTranslation[TRANS_OPERANDS][aOperands[1]][OPERAND_OPERATION]
	if not (nOperationIdx and nOperationIdx < 0) then
		-- operand can't be a variable if it doesn't have an operation or if it's operation index is not negative (index into variable table)
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Expecting: $Variable]")
	end
	-- count as an operand request
	if aTranslation[TRANS_OPERANDREQUESTS] >= aTranslation[TRANS_MAXOPERANDREQUESTS] then
		RaiseExpressionsError(aTranslation,ERR.MAXPROCESSING)
	end
	aTranslation[TRANS_OPERANDREQUESTS] = aTranslation[TRANS_OPERANDREQUESTS]+1
	-- get value from variable and test it
	return aTranslation[TRANS_VARIABLES][-nOperationIdx][VAR_VALUE] ~= nil
end
},

{"ABS",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathabs,{{TYPENUM,true}})
end
},

{"CEIL",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathceil,{{TYPENUM,true}})
end
},

{"FLOOR",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathfloor,{{TYPENUM,true}})
end
},

{"LOG10",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathlog10,{{TYPENUM,true}})
end
},

{"MAX",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathmax,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"MIN",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathmin,{{TYPENUM,true},{TYPENUM,true}})
end
},

{"SQRT",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,mathsqrt,{{TYPENUM,true}})
end
},

{"AFD",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,
		function(xOperand1,xOperand2)
			return adjustfordisplay(strupper(xOperand2),xOperand1)
		end,
		{{TYPENUMBOOLSTR,true},{TYPESTR,false,"CORR"}})
end
},

{"FORMAT",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,
		function(xOperand1,xOperand2)
			if xOperand2 == nil then
				if type(xOperand1) == "number" then
					xOperand2 = "%.0f" --default: number without decimals
				elseif type(xOperand1) == "string" then
					xOperand2 = "%s" --default: just same string
				end
			end
			return stringformatvalue(xOperand2,xOperand1)
		end,
		{{TYPENUMSTR,true},{TYPESTR,false}})
end
},

{"SWITCH",OPERATION_FORMAT_SWITCH,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local xOperand, sTypeOp
	for nOp = 1, nOpCount, 2 do
		-- process logical + result operand pairs
		-- default logical is a constant true, so will always be the result once encountered
		xOperand = GetOperandValue(aTranslation,aOperands[nOp])
		sTypeOp = type(xOperand)
		if not strfind(TYPEBOOL,sTypeOp) then
			RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Condition Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPEBOOL.."]")
		end
		if xOperand then
			xOperand = GetOperandValue(aTranslation,aOperands[nOp+1])
			return xOperand
		end
	end
end
},

{"SESEL",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount = 2
	if nOpCount < nOpMinCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting at least:"..nOpMinCount.."(even number)]")
	end
	local xOperand, xCompOperand
	local sTypeParam, nTypeOp, sTypeOp
	nTypeOp = 1
	xCompOperand = GetOperandValue(aTranslation,aOperands[nTypeOp])
	sTypeOp = type(xCompOperand)
	if not strfind(TYPENUMSTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Compare Operand("..ErrorDisplayOperand(xCompOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMSTR.."]")
	end
	if sTypeOp == "number" then
		-- compare number value minus a little for small discrepancies
		-- like 79.0000001 should be treated like 79 so <= 79
		xCompOperand = xCompOperand-DblCalcDev
	end
	for nOp = 2, nOpCount, 2 do
		xOperand = GetOperandValue(aTranslation,aOperands[nOp])
		if nOp == nOpCount then
			-- last operand is default value
			return xOperand
		end
		sTypeParam = type(xOperand)
		if sTypeOp ~= sTypeParam then
			RaiseExpressionsError(aTranslation,ERR.NONMATCHINGOPTYPE,ErrorDisplayOperation(aOperation).." [Operand"..nOp.."("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeParam..", Expecting same type as Compare Operand("..ErrorDisplayOperand(xCompOperand)..")"..":"..sTypeOp.."]")
		end
		if xCompOperand <= xOperand then
			xOperand = GetOperandValue(aTranslation,aOperands[nOp+1])
			return xOperand
		end
	end
	-- doesn't include a default operand
	RaiseExpressionsError(aTranslation,ERR.MISSINGSESELDEFAULT)
end
},

{"WITH",OPERATION_FORMAT_WITH,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	-- process initial variable assignments
	local nInitVarCount = GetOperandValue(aTranslation,aOperands[1])
	local nInitVarOpIdx = 2
	for nInitVarIdx = 1, nInitVarCount do
		aTranslation[TRANS_VARIABLES][-aTranslation[TRANS_OPERANDS][aOperands[nInitVarOpIdx]][OPERAND_OPERATION]][VAR_VALUE] = GetOperandValue(aTranslation,aOperands[nInitVarOpIdx+1])
		nInitVarOpIdx = nInitVarOpIdx+2
	end
	-- last operand is result value
	return GetOperandValue(aTranslation,aOperands[#aOperands])
end
},

{"WHILE",OPERATION_FORMAT_WHILE,
function (aTranslation,aOperation)
	return ExecLoopOp(aTranslation,aOperation,true)
end
},

{"CHOOSE",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount = 2
	if nOpCount < nOpMinCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting at least:"..nOpMinCount.."]")
	end
	local xOperand, sTypeOp
	xOperand = GetOperandValue(aTranslation,aOperands[1])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUM,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Index Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUM.."]")
	end
	xOperand = mathfloor(xOperand+0.5+DblCalcDev)
	if xOperand < 1 or nOpCount-1 < xOperand then
		RaiseExpressionsError(aTranslation,ERR.OUTOFRANGE,ErrorDisplayOperation(aOperation).." [Index Operand:"..ErrorDisplayOperand(xOperand)..", Expecting:1-"..(nOpCount-1).."]")
	end
	xOperand = GetOperandValue(aTranslation,aOperands[1+xOperand])
	return xOperand
end
},

{"REPLACE",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount = 3
	if nOpCount < nOpMinCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting at least:"..nOpMinCount.."(odd number)]")
	end
	if nOpCount % 2 == 0 then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting an odd number]")
	end
	local xOperand, sTypeOp
	xOperand = GetOperandValue(aTranslation,aOperands[1])
	sTypeOp = type(xOperand)
	if not strfind(TYPESTR,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Source Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPESTR.."]")
	end
	local sResultText = xOperand
	local sSearchText, sReplacingText
	local sSearch, sReplace
	for nOp = 2, nOpCount, 2 do
		xOperand = GetOperandValue(aTranslation,aOperands[nOp])
		sTypeOp = type(xOperand)
		if not strfind(TYPESTR,sTypeOp) then
			RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Search Operand"..nOp.."("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPESTR.."]")
		end
		sSearchText = xOperand
		if sSearchText ~= "" then
			xOperand = GetOperandValue(aTranslation,aOperands[nOp+1])
			sTypeOp = type(xOperand)
			if not strfind(TYPENUMBOOLSTR,sTypeOp) then
				RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Replace Operand"..(nOp+1).."("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUMBOOLSTR.."]")
			end
			if type(xOperand) == "number" then
				sReplacingText = tostring(xOperand)
			elseif type(xOperand) == "boolean" then
				if xOperand then
					sReplacingText = "True"
				else
					sReplacingText = "False"
				end
			else
				sReplacingText = xOperand
			end
			sSearch = strgsub(sSearchText,"[%(%)%.%%%+%-%*%?%[%^%$%]]","%%%1")
			sReplace = strgsub(sReplacingText,"%%","%%%%")
			sResultText = strgsub(sResultText,sSearch,sReplace)
		end
	end
	return sResultText
end
},

{"GRAPHVAL",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	local aOperands = aOperation[OPERATION_OPERANDS]
	local nOpCount = #aOperands
	local nOpMinCount = 5 -- 1 for level + 2 * graphpoints (2)
	if nOpCount < nOpMinCount then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting at least:"..nOpMinCount.."(odd number)]")
	end
	if nOpCount % 2 == 0 then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPCOUNT,ErrorDisplayOperation(aOperation).." [Operand count:"..nOpCount..", Expecting an odd number]")
	end
	local xOperand, sTypeOp
	xOperand = GetOperandValue(aTranslation,aOperands[1])
	sTypeOp = type(xOperand)
	if not strfind(TYPENUM,sTypeOp) then
		RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Key Operand("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUM.."]")
	end
	local iKey = RoundDbl(xOperand) -- key should be a whole number
	local aGraphPoints = {}
	local aNewPoint
	for nOp = 2, nOpCount, 2 do
		aNewPoint = {}
		xOperand = GetOperandValue(aTranslation,aOperands[nOp])
		sTypeOp = type(xOperand)
		if not strfind(TYPENUM,sTypeOp) then
			RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Point Key Operand"..nOp.."("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUM.."]")
		end
		aNewPoint.Key = RoundDbl(xOperand) -- key should be a whole number
		-- graph point key should be unique: check if already exists
		for _, aGraphPoint in pairs(aGraphPoints) do
			if aGraphPoint.Key == aNewPoint.Key then
				-- key already exists, double defined
				RaiseExpressionsError(aTranslation,ERR.DOUBLEGRAPHKEY,ErrorDisplayOperation(aOperation).." [Point Key Operand"..nOp.."("..ErrorDisplayOperand(xOperand)..")]")
			end
		end
		xOperand = GetOperandValue(aTranslation,aOperands[nOp+1])
		sTypeOp = type(xOperand)
		if not strfind(TYPENUM,sTypeOp) then
			RaiseExpressionsError(aTranslation,ERR.INVALIDOPTYPE,ErrorDisplayOperation(aOperation).." [Point Value Operand"..(nOp+1).."("..ErrorDisplayOperand(xOperand)..")"..":"..sTypeOp..", Expecting:"..TYPENUM.."]")
		end
		aNewPoint.Value = xOperand
		tableinsert(aGraphPoints,aNewPoint)
	end
	-- make sure that point keys are in ascending order
	tablesort(aGraphPoints,function(p1,p2) return p1.Key < p2.Key end)
	-- find key interval
	local iPointIndexSecond = 2
	local iPointIndexMax = #aGraphPoints
	while iPointIndexSecond < iPointIndexMax do
		if iKey <= aGraphPoints[iPointIndexSecond].Key then
			break
		end
		iPointIndexSecond = iPointIndexSecond+1
	end
	local iPointIndexFirst = iPointIndexSecond-1
	local aFirstPoint = aGraphPoints[iPointIndexFirst]
	local aSecondPoint = aGraphPoints[iPointIndexSecond]
	-- interpolate value
	return LinFmod(1.0,aFirstPoint.Value,aSecondPoint.Value,aFirstPoint.Key,aSecondPoint.Key,iKey)
end
},

{"UNTIL",OPERATION_FORMAT_UNTIL,
function (aTranslation,aOperation)
	return ExecLoopOp(aTranslation,aOperation,false)
end
},

{"ROUNDDOWN",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,RoundDblDown,{{TYPENUM,true},{TYPENUM,false,0}})
end
},

{"ROUNDLOTRO",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,RoundDblLotro,{{TYPENUM,true}})
end
},

{"ROUNDMORREG",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,RoundDblMorReg,{{TYPENUM,true}})
end
},

{"ROUNDPROG",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,RoundDblProg,{{TYPENUM,true}})
end
},

{"ROUNDUP",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,RoundDblUp,{{TYPENUM,true},{TYPENUM,false,0}})
end
},

{"ROUND",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,RoundDbl,{{TYPENUM,true},{TYPENUM,false,0}})
end
},

{"EQUSNG",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,EquSng,{{TYPENUM,true}})
end
},

{"DECSNG",OPERATION_FORMAT_FUNCTION,
function (aTranslation,aOperation)
	return ExecDefault(aTranslation,aOperation,DecSng,{{TYPENUM,true}})
end
}

}

nOpDefFunctionEnd = #aOperationDefs
 
local OPPRIOGRP_OPERATIONDEF_IDS = 1 -- set of operation definition IDs (indexes)
local OPPRIOGRP_SCANDIRECTION = 2
local OPPRIOGRP_SCANFORMAT = 3

local SCANDIRECTION_LEFTRIGHT = 1
local SCANDIRECTION_RIGHTLEFT = 2

-- defines priority process order and scan direction for grouped binary and unary operations
-- same as Lua operation order, highest priority first
-- you can't mix unary and binary operations in the same group: it's either all unary or all binary
local aOperationPriorityGroups =
	{
		{{[18]=true},SCANDIRECTION_RIGHTLEFT,OPERATION_FORMAT_BINARY}, -- POWER
		{{[1]=true,[2]=true,[3]=true},SCANDIRECTION_RIGHTLEFT,OPERATION_FORMAT_UNARY}, -- LOGICNEG,NEGATE,POSITIVE
		{{[16]=true,[17]=true,[15]=true},SCANDIRECTION_LEFTRIGHT,OPERATION_FORMAT_BINARY}, -- MULTIPLY,DIVIDE,MOD
		{{[14]=true,[13]=true},SCANDIRECTION_LEFTRIGHT,OPERATION_FORMAT_BINARY}, -- ADD,SUBSTRACT
		{{[4]=true},SCANDIRECTION_RIGHTLEFT,OPERATION_FORMAT_BINARY}, -- CONCAT
		{{[12]=true,[11]=true,[8]=true,[7]=true,[10]=true,[9]=true},SCANDIRECTION_LEFTRIGHT,OPERATION_FORMAT_BINARY}, -- SMTH,GRTH,SMEQTH,GREQTH,NOTEQUALS,EQUALS
		{{[6]=true},SCANDIRECTION_LEFTRIGHT,OPERATION_FORMAT_BINARY}, -- AND
		{{[5]=true},SCANDIRECTION_LEFTRIGHT,OPERATION_FORMAT_BINARY} -- OR
	}

-- returns operand number from operand string like 9 from "[9]"
local function OPStonumber(sOPS)
	if sOPS == nil then
		return
	else
		local sOPn = strmatch(sOPS,"^%s*%[(%d+)%]%s*$")
		if sOPn then
			return tonumber(sOPn)
		else
			return
		end
	end
end

-- returns operand string from operand number/string like "[9]" from 9
local function OPNtostring(xOPN)
	if xOPN == nil then
		return
	else
		return "["..xOPN.."]"
	end
end

local function CreateConstantOperand(aTranslation,sSource,xValue)
	local nOp = aTranslation[TRANS_OPSOURCEIDX][sSource]
	if not nOp then
		-- add new operand if not exists yet
		nOp = #aTranslation[TRANS_OPERANDS]+1
		aTranslation[TRANS_OPERANDS][nOp] = {xValue}
		aTranslation[TRANS_OPSOURCEIDX][sSource] = nOp -- store source+index into operand source searchindex
	end
	return nOp
end

local function CreateBooleanConstant(aTranslation,bConstant)
	if bConstant then
		return CreateConstantOperand(aTranslation,"TRUE",true)
	else
		return CreateConstantOperand(aTranslation,"FALSE",false)
	end
end

-- detects a boolean value in a text, creates an operand for it and returns operand index
-- if the value already exists then it returns the existing operand index
-- pattern matches FALSE, TRUE
local function FetchBooleanConstant(aTranslation,sExpression)
	local sSource = strmatch(sExpression,"%s*(FALSE)%s*")
	if not sSource then
		sSource = strmatch(sExpression,"%s*(TRUE)%s*")
	end
	if sSource == "TRUE" then
		return OPNtostring(CreateConstantOperand(aTranslation,"TRUE",true))
	else
		return OPNtostring(CreateConstantOperand(aTranslation,"FALSE",false))
	end
end

local function CreateNumberConstant(aTranslation,nConstant)
	return CreateConstantOperand(aTranslation,tostring(nConstant),nConstant)
end

-- detects a number value in a text, creates an operand for it and returns operand index
-- if the value already exists then it returns the existing operand index
-- pattern matches the minimal 9 and 9.9 or more digits
local function FetchNumberConstant(aTranslation,sExpression)
	local sSource = strmatch(sExpression,"%s*(%d+%.%d+)%s*")
	if not sSource then
		sSource = strmatch(sExpression,"%s*(%d+)%s*")
	end
	return OPNtostring(CreateConstantOperand(aTranslation,sSource,tonumber(sSource)))
end

local function CreateStringConstant(aTranslation,sConstant)
	return CreateConstantOperand(aTranslation,"\""..sConstant.."\"",sConstant)
end

-- detects a string value in a text, creates an operand for it and returns operand index
-- if the value already exists then it returns the existing operand index
-- pattern matches characters between ""
local function FetchStringConstant(aTranslation,sExpression)
	local sSource = strmatch(sExpression,"%s*(\".-\")%s*")
	return OPNtostring(CreateConstantOperand(aTranslation,sSource,strgsub(strsub(sSource,2,-2),"\\.",
				function(sEscapeChar)
					if sEscapeChar == "\\n" then return "\n"
					elseif sEscapeChar == "\\q" then return "\""
					elseif sEscapeChar == "\\\\" then return "\\"
					else return ""
					end
				end)))
end

-- if an operand for the variable already exists then it returns the existing operand index
-- operand index is registered with the variable's entry in the variable table
-- dependson flag ensures that every operand that uses this operand will also depend on the variable and be registered as such
local function CreateVariableOperand(aTranslation,sVarName)
	local nVar = aTranslation[TRANS_VARNAMEIDX][sVarName]
	if not nVar then
		-- add new 'on the fly' variable. inside With function?
		nVar = #aTranslation[TRANS_VARIABLES]+1
		if nVar > 1000 then
			RaiseExpressionsError(aTranslation,ERR.TOOMANYVARIABLES)
		end
		aTranslation[TRANS_VARIABLES][nVar] = {sVarName,nil}
		aTranslation[TRANS_VARNAMEIDX][sVarName] = nVar -- store name+index into variable name searchindex
	end
	local nOp = aTranslation[TRANS_OPSOURCEIDX][sVarName]
	if not nOp then
		-- add new operand if not exists yet
		nOp = #aTranslation[TRANS_OPERANDS]+1
		aTranslation[TRANS_OPERANDS][nOp] = {nil,-nVar} -- negative operation index: value needs to be picked up from variable
		aTranslation[TRANS_OPSOURCEIDX][sVarName] = nOp -- store source+index into operand source searchindex
	end
	return nOp
end

-- detects a variable in a text, creates an operand for it and returns operand index
-- pattern matches the minimal $A (or with more letters)
local function FetchVariableOperand(aTranslation,sVar)
	return OPNtostring(CreateVariableOperand(aTranslation,strmatch(sVar,"%s*%$(%a+)%s*")))
end

-- splits an expression into (parameter)parts, based on a splitting token
-- calls a supplied function for each detected part
-- tokens in nested expressions between () are ignored
local function DecomposeParameters(aTranslation,sParams,sSplitToken,fDecomposeParameter)
	local nParam = 0
	local nLevel = 0 -- to keep track of nested ()
	local nLastToken = 0
	local sSearchChars = "([%(%)"..sSplitToken.."])"
	local nCharIndex = 0
	while true do
		nCharIndex, _, sChar = strfind(sParams,sSearchChars,nCharIndex+1)
		if not sChar then
			-- last parameter at the end
			fDecomposeParameter(strsub(sParams,nLastToken+1),nParam+1,true)
			return
		elseif sChar == "(" then
			nLevel = nLevel+1
		elseif sChar == ")" then
			nLevel = nLevel-1
		elseif nLevel == 0 then
			-- we have a parameter when we detect a splitting token at level 0
			nParam = nParam+1
			fDecomposeParameter(strsub(sParams,nLastToken+1,nCharIndex-1),nParam,false)
			nLastToken = nCharIndex
		end
	end
end

local CreateExpressionOperand
local DecomposeExpression

-- sParams = param1,param2,etc
-- returns an array with operand numbers, containing parameter expression results
local function DecomposeFunctionParameters(aTranslation,sParams)
	-- extract parameters
	local aParamOps = {}

	local DecomposeFunctionParameter = function(sParam,nParam,bIsLastParam)
		aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sParam)
	end

	DecomposeParameters(aTranslation,sParams,",",DecomposeFunctionParameter)

	return aParamOps
end

-- transform CASE>=compexpr1:CASE<=compexpr2:..etc:resultexpr
-- expand to: (testexpr>=compexpr1)OR(testexpr<=compexpr2)OR..etc
local function DecomposeCaseConditions(aTranslation,sParams,sTestOp,aParamOps)
	local sCombinedExpr = ""
	local sCompareExpr
	local sNewExpr

	local DecomposeCaseCondition = function(sParam,nParam,bIsLastParam)
		if bIsLastParam and nParam < 2 then
			-- not enough parameter groups
			RaiseExpressionsError(aTranslation,ERR.MISSINGPARAMGROUPS,sParams)
		elseif bIsLastParam then
			-- add combined logical expression
			aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sCombinedExpr)
			-- add last: should be result expression
			aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sParam)
		else
			-- construct combined logical expression
			sCompareExpr = strmatch(sParam,"^%s*CASE%s*(.-)%s*$")
			if not sCompareExpr or sCompareExpr == "" then
				RaiseExpressionsError(aTranslation,ERR.MISSINGSWITCHCONDITION,sParam)
			end
			sNewExpr = DecomposeExpression(aTranslation,sTestOp.." "..sCompareExpr)
			if sCombinedExpr == "" then
				sCombinedExpr = sNewExpr
			else
				sCombinedExpr = sCombinedExpr.." OR "..sNewExpr
			end
		end
	end

	DecomposeParameters(aTranslation,sParams,":",DecomposeCaseCondition)
end

-- SWITCH(testexpr,CASE>=compexpr1:CASE<=compexpr2:resultexpr1,DEFAULT:resultexpr2)
-- expand to: SWITCH((testexpr>=compexpr1)OR(testexpr<=compexpr2),resultexpr1,True,resultexpr2)
-- parameter results should be like bBoolean1,xValue1,bBoolean2,xValue2,bBoolean3,xValue3,etc
local function DecomposeSwitchParameters(aTranslation,sParams)
	-- extract parameters
	local aParamOps = {}
	local sTestOp
	local sDefaultExpr
	local bDefaultDone = false

	local DecomposeSwitchParameter = function(sParam,nParam,bIsLastParam)
		if bIsLastParam and nParam < 2 then
			-- not enough parameter groups
			RaiseExpressionsError(aTranslation,ERR.MISSINGPARAMGROUPS,sParams)
		elseif nParam == 1 then
			-- should be test expression
			sTestOp = DecomposeExpression(aTranslation,sParam)
		elseif not bDefaultDone then
			-- test for CASE or DEFAULT at the start
			if strmatch(sParam,"^%s*CASE") then
				-- CASE parameter, possibly multiple conditions
				-- CASE>=compexpr1:CASE<=compexpr2:resultexpr1
				DecomposeCaseConditions(aTranslation,sParam,sTestOp,aParamOps)
			elseif strmatch(sParam,"^%s*DEFAULT") then
				-- default parameter
				-- DEFAULT:resultexpr2
				sDefaultExpr = strmatch(sParam,"^%s*DEFAULT%s*:%s*(.-)%s*$")
				if not sDefaultExpr or sDefaultExpr == "" then
					RaiseExpressionsError(aTranslation,ERR.MISSINGSWITCHDEFAULT,sParam)
				end
				-- add true parameter operand
				aParamOps[#aParamOps+1] = CreateBooleanConstant(aTranslation,true)
				-- add default result parameter operand
				aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sDefaultExpr)
				-- default done, should be last
				bDefaultDone = true
			else
				RaiseExpressionsError(aTranslation,ERR.SYNTAXSWITCH,sParam)
			end
		else
			-- too many parameter groups (after being closed by a default)
			if sParam == "" then
				RaiseExpressionsError(aTranslation,ERR.SWITCHGROUPSOVERFLOW,sParams)
			else
				RaiseExpressionsError(aTranslation,ERR.SWITCHGROUPSOVERFLOW,sParam)
			end
		end
	end

	DecomposeParameters(aTranslation,sParams,",",DecomposeSwitchParameter)

	if not bDefaultDone then
		-- didn't include a default condition
		RaiseExpressionsError(aTranslation,ERR.MISSINGSWITCHDEFAULT,sParams)
	end

	return aParamOps
end

-- var1=varexpr1, var2=varexpr2, etc.
-- expand to: varcount, var1, varexpr1, var2, varexpr2, etc.
local function DecomposeVarAssignments(aTranslation,sParams,aParamOps)
	local nVarCountOpIdx = #aParamOps+1
	aParamOps[nVarCountOpIdx] = -1 -- reserve position for assignment count

	local DecomposeVarAssignment = function(sParam,nParam,bIsLastParam)
		local sVar, sVarExpr = strmatch(sParam,"^%s*(%[%d+%])%s*=%s*(.-)%s*$") -- variable should already be an operand
		if not sVar or sVarExpr == "" then
			sVar = strmatch(sParam,"^%s*(%[%d+%])")
			if not sVar then
				RaiseExpressionsError(aTranslation,ERR.MISSINGASSIGNVARIABLE,sParam)
			end
			sVar = strmatch(sParam,"^%s*(%[%d+%])%s*=")
			if not sVar then
				RaiseExpressionsError(aTranslation,ERR.MISSINGASSIGNSIGN,sParam)
			end
			RaiseExpressionsError(aTranslation,ERR.MISSINGASSIGNEXPRESSION,sParam)
		end
		-- add variable operand
		aParamOps[#aParamOps+1] = OPStonumber(sVar)
		-- add assigned expression
		aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sVarExpr)
	end

	DecomposeParameters(aTranslation,sParams,",",DecomposeVarAssignment)

	-- store variable assignment count
	aParamOps[nVarCountOpIdx] = CreateNumberConstant(aTranslation,(#aParamOps-nVarCountOpIdx)/2)
end

-- WITH(var1=varexpr1, var2=varexpr2, etc. : resultexpr)
-- expand to: WITH(varcount, var1, varexpr1, var2, varexpr2, etc. , resultexpr)
-- parameter results should be like a pair for each variable assignment plus one result operand at the end
local function DecomposeWithParameters(aTranslation,sParams)
	-- extract parameters
	local aParamOps = {}

	local DecomposeWithParameter = function(sParam,nParam,bIsLastParam)
		if bIsLastParam and nParam < 2 then
			-- not enough parameter groups
			RaiseExpressionsError(aTranslation,ERR.MISSINGPARAMGROUPS,sParams)
		elseif nParam == 1 then
			-- should be variable assignments
			DecomposeVarAssignments(aTranslation,sParam,aParamOps)
		elseif nParam == 2 then
			-- should be the result expression
			-- add result parameter operand
			aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sParam)
		elseif nParam == 3 then
			-- too many parameter groups
			if sParam == "" then
				RaiseExpressionsError(aTranslation,ERR.PARAMGROUPSOVERFLOW,sParams)
			else
				RaiseExpressionsError(aTranslation,ERR.PARAMGROUPSOVERFLOW,sParam)
			end
		end
	end

	DecomposeParameters(aTranslation,sParams,":",DecomposeWithParameter)

	return aParamOps
end

-- WHILE(initvar1=varexpr1, initvar2=varexpr2, etc. : loopconditionexpr : loopvar1=varexp1, loopvar2=varexp2, etc. : resultexpr)
-- expand to: WHILE(initvarcount, initvar1, initvarexpr1, initvar2, initvarexpr2, etc. , loopconditionexpr, loopvarcount, loopvar1, loopvarexpr1, loopvar2, loopvarexpr2, etc. , resultexpr)
-- UNTIL(initvar1=varexpr1, initvar2=varexpr2, etc. : loopvar1=varexp1, loopvar2=varexp2, etc. : loopconditionexpr : resultexpr)
-- expand to: UNTIL(initvarcount, initvar1, initvarexpr1, initvar2, initvarexpr2, etc., loopvarcount, loopvar1, loopvarexpr1, loopvar2, loopvarexpr2, etc. , loopconditionexpr , resultexpr)
-- parameter results should be like a pair for each variable assignment plus one operand for loopcondition and one for result at the end
local function DecomposeLoopParameters(aTranslation,sParams,bPreCondition)
	-- extract parameters
	local aParamOps = {}

	local DecomposeLoopParameter = function(sParam,nParam,bIsLastParam)
		if bIsLastParam and nParam < 4 then
			-- not enough parameter groups
			RaiseExpressionsError(aTranslation,ERR.MISSINGPARAMGROUPS,sParams)
		elseif nParam == 1 then
			-- should be initial variable assignments
			DecomposeVarAssignments(aTranslation,sParam,aParamOps)
		elseif (nParam == 2 and bPreCondition) or (nParam == 3 and not bPreCondition) then
			-- should be loop condition expression
			aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sParam)
		elseif (nParam == 3 and bPreCondition) or (nParam == 2 and not bPreCondition) then
			-- should be loop variable assignments
			DecomposeVarAssignments(aTranslation,sParam,aParamOps)
		elseif nParam == 4 then
			-- should be the result expression
			-- add result parameter operand
			aParamOps[#aParamOps+1] = CreateExpressionOperand(aTranslation,sParam)
		elseif nParam == 5 then
			-- too many parameter groups
			if sParam == "" then
				RaiseExpressionsError(aTranslation,ERR.PARAMGROUPSOVERFLOW,sParams)
			else
				RaiseExpressionsError(aTranslation,ERR.PARAMGROUPSOVERFLOW,sParam)
			end
		end
	end

	DecomposeParameters(aTranslation,sParams,":",DecomposeLoopParameter)

	return aParamOps
end

-- check if list of operands contains dynamic content
local function OperandsContainOperation(aTranslation,aOperands)
	if aOperands then
		for _, nOp in ipairs(aOperands) do
			if nOp then
				if aTranslation[TRANS_OPERANDS][nOp][OPERAND_OPERATION] then
					return true
				end
			end
		end
	end
	return false
end

local function CreateOperation(aTranslation,sSource,nOperationDef,aOperands)
	-- test if a result not yet exists for this source
	local nResultOp = aTranslation[TRANS_OPSOURCEIDX][sSource]
	if not nResultOp then
		-- index result operand
		nResultOp = #aTranslation[TRANS_OPERANDS]+1
		aTranslation[TRANS_OPSOURCEIDX][sSource] = nResultOp -- store source+index into operand source searchindex
		-- define the operation
		local aOperation = {nOperationDef,aOperands}
		-- check if source operands provide any dynamic result (at least one operation present) from variable(s) or are all constants
		if OperandsContainOperation(aTranslation,aOperands) then
			-- should depend on variable(s), so operation needs to be stored, but not executed now
			local nResultOperation = #aTranslation[TRANS_OPERATIONS]+1
			aTranslation[TRANS_OPERATIONS][nResultOperation] = aOperation -- store in operation table
			aTranslation[TRANS_OPERANDS][nResultOp] = {nil,nResultOperation} -- define operand
		else
			-- does not depend on any variable, so operand values should be all constants
			-- calculate constant value now, no need to store the operation
			aTranslation[TRANS_OPERANDS][nResultOp] = {ExecuteOperation(aTranslation,aOperation)} -- define operand, will hold only a result
		end
	end
	return nResultOp
end

local function GetSourceParameterOperands(aParamOps)
	local sSource = ""
	for _, nOp in ipairs(aParamOps) do
		if sSource ~= "" then
			sSource = sSource..","
		end
		if nOp then
			sSource = sSource..OPNtostring(nOp)
		end
	end
	return sSource
end

-- sName = name of function like CALCSTAT, LINFMOD, etc.
-- sParams = param1,param2,etc (normally)
-- parameters need to be extracted and each treated like an expression (which should return a result in an operand)
-- an operation for this function needs to be created, which takes a list of operands (the parameters) and gives a result in an operand
local function DecomposeFunction(aTranslation,sName,sParams)
	 -- set focus for error reporting
	aTranslation[TRANS_EXPRWIP] = sName.."("..sParams..")"

	-- get function definition
	local nOperationDef
	local nFunctionFormat
	for nOpDef = nOpDefUnaryStart, nOpDefFunctionEnd do
		if aOperationDefs[nOpDef][OPERATIONDEF_OPERATOR] == sName then
			nFunctionFormat = aOperationDefs[nOpDef][OPERATIONDEF_FORMAT]
			if nFunctionFormat == OPERATION_FORMAT_UNARY or nFunctionFormat == OPERATION_FORMAT_BINARY then
				-- this is not a function, but a basic operation followed by parantheses, like: NOT ($A < 10)
				return aTranslation[TRANS_EXPRWIP] -- return unaltered so they can be processed as basic operations later
			end
			nOperationDef = nOpDef
			break
		end
	end
	if not nOperationDef then
		-- unknown function
		RaiseExpressionsError(aTranslation,ERR.UNKNOWNFUNCTION)
	end

	-- get an array with parameter operand numbers
	local aParamOps
	if nFunctionFormat == OPERATION_FORMAT_SWITCH then
		aParamOps = DecomposeSwitchParameters(aTranslation,sParams) -- Switch($L, case<=10: case>=100: 25, default: 13)
	elseif nFunctionFormat == OPERATION_FORMAT_WITH then
		aParamOps = DecomposeWithParameters(aTranslation,sParams) -- With($A=@Agility#2, $B=20: IIf($A>$B,$A,$B))
	elseif nFunctionFormat == OPERATION_FORMAT_WHILE then
		aParamOps = DecomposeLoopParameters(aTranslation,sParams,true)
	elseif nFunctionFormat == OPERATION_FORMAT_UNTIL then
		aParamOps = DecomposeLoopParameters(aTranslation,sParams,false)
	else
		aParamOps = DecomposeFunctionParameters(aTranslation,sParams) -- sName(operand1, operand2, etc.)
	end

	-- compose funcname([n],[n],etc) as source identifier
	local sSource = sName.."("..GetSourceParameterOperands(aParamOps)..")"

	-- create an operation for this function (returns result operand number)
	local nResultOp = CreateOperation(aTranslation,sSource,nOperationDef,aParamOps)

	return OPNtostring(nResultOp)
end

-- extract operation from expression snippet in Work in Progress
local function ExtractOpDefWIP(aTranslation,nOpDefStart,nOpDefEnd)
	local sOperator, nTestLen
	for nOpDef = nOpDefStart, nOpDefEnd do
		sOperator = aOperationDefs[nOpDef][OPERATIONDEF_OPERATOR]
		nTestLen = #sOperator
		if strsub(aTranslation[TRANS_EXPRWIP],1,nTestLen) == sOperator then
			-- remove found operator from operators and remove any leading whitespace
			aTranslation[TRANS_EXPRWIP] = strmatch(strsub(aTranslation[TRANS_EXPRWIP],nTestLen+1),"%s*(.*)")
			return nOpDef
		end
	end
end

-- extract unary operations from expression snippet in Work in Progress
local function ExtractUnaryOpDefsWIP(aTranslation)
	local aOpDefs = {}
	local nOpDefCount = 0
	local nUnaryOpDef
	while aTranslation[TRANS_EXPRWIP] ~= "" do
		nUnaryOpDef = ExtractOpDefWIP(aTranslation,nOpDefUnaryStart,nOpDefUnaryEnd)
		if not nUnaryOpDef then
			-- something was there, but not a valid unary operation
			RaiseExpressionsError(aTranslation,ERR.UNKNOWNUNARY)
		end	
		-- store unary operation definition index
		nOpDefCount = nOpDefCount+1
		aOpDefs[nOpDefCount] = nUnaryOpDef
	end
	return aOpDefs
end

local ELEMENT_OPERAND = 1 -- operand in expression element
local ELEMENT_UNARYOPDEFS = 2 -- unary operations to be performed on this operand
local ELEMENT_BINARYOPDEF = 3 -- binary operation to be performed on this operand and the one in the next element

-- create a list of Expression Elements out of an Expression
-- <unary operator 1>..<unary operator N>[<operand>]<binary operator> --> Element = {<operand>,{<unary operation definitions>},<binary operation definition>}
-- or at the end this is
-- <unary operator 1>..<unary operator N>[<operand>] --> Element = {<operand>,{<unary operation definitions>},nil}
local function CreateElementsFromExpression(aTranslation,sExpression)
	local sTempExpr = trim(sExpression) -- trim leading/trailing white space
	local aElements = {}
	local nElemIndex = 0
	local aElement
	local sBinaryOpSearch = "^(.-)%s*%[(%d+)%]%s*(.-)%s*%[(%d+)%]%s*(.-)$" -- binary operation search from the left
	local sUnaryOpSearch = "^(.-)%s*%[(%d+)%]%s*(.-)$" -- unary operation search from the left
	local sLeft, sFoundOperand1, sMiddle, sFoundOperand2, sRight

	while sTempExpr ~= "" do
		-- try to find a binary operation, involving 2 operands
		sLeft, sFoundOperand1, sMiddle, sFoundOperand2, sRight = strmatch(sTempExpr,sBinaryOpSearch)
		if sFoundOperand1 and sFoundOperand2 then
			-- process operand1
			aElement = {tonumber(sFoundOperand1),nil,nil}
			if sLeft ~= "" then
				-- process unaries for operand1
				aTranslation[TRANS_EXPRWIP] = sLeft
				aElement[ELEMENT_UNARYOPDEFS] = ExtractUnaryOpDefsWIP(aTranslation)
			end
			if sMiddle == "" then
				-- syntax error: missing binary operation
				RaiseExpressionsError(aTranslation,ERR.MISSINGBINARY,sTempExpr)
			end
			-- extract the binary operation, but leave any unaries for operand in next element
			aTranslation[TRANS_EXPRWIP] = sMiddle
			aElement[ELEMENT_BINARYOPDEF] = ExtractOpDefWIP(aTranslation,nOpDefBinaryStart,nOpDefBinaryEnd)
			if not aElement[ELEMENT_BINARYOPDEF] then
				RaiseExpressionsError(aTranslation,ERR.UNKNOWNBINARY)
			end
			-- more to be processed in next cycle
			sTempExpr = aTranslation[TRANS_EXPRWIP]..OPNtostring(sFoundOperand2)..sRight
		else
			-- else try to find a single operand (should be at the end of the expression)
			sLeft, sFoundOperand1, sRight = strmatch(sTempExpr,sUnaryOpSearch)
			if not sFoundOperand1 then
				-- syntax error: didn't find any operand - something's there, but not an operand
				RaiseExpressionsError(aTranslation,ERR.MISSINGOPERAND,sTempExpr)
			end
			-- process the operand
			aElement = {tonumber(sFoundOperand1),nil,nil}
			if sLeft ~= "" then
				-- process unaries
				aTranslation[TRANS_EXPRWIP] = sLeft
				aElement[ELEMENT_UNARYOPDEFS] = ExtractUnaryOpDefsWIP(aTranslation)
			end
			sTempExpr = sRight -- should be empty..
		end
		nElemIndex = nElemIndex+1
		aElements[nElemIndex] = aElement
	end

	return aElements
end

local function CreateExpressionFromElements(aElements)
	local aResult = {}
	for _, aElement in ipairs(aElements) do
		if aElement[ELEMENT_UNARYOPDEFS] then
			-- unary operatordefid(s)
			for _, o in ipairs(aElement[ELEMENT_UNARYOPDEFS]) do
				tableinsert(aResult,aOperationDefs[o][OPERATIONDEF_OPERATOR])
			end
		end
		if aElement[ELEMENT_OPERAND] then
			-- operand number
			tableinsert(aResult,OPNtostring(aElement[ELEMENT_OPERAND]))
		end
		if aElement[ELEMENT_BINARYOPDEF] then
			-- binary operatordefid
			tableinsert(aResult,aOperationDefs[aElement[ELEMENT_BINARYOPDEF]][OPERATIONDEF_OPERATOR])
		end
	end
	return tableconcat(aResult)
end

-- decomposes an Element's unary operations
local function ProcessUnaryOperations(aTranslation,aElement,nOperationPriorityGroup)
	local aUnaryOperators = aElement[ELEMENT_UNARYOPDEFS]

	local aOperationPriorityGroup = aOperationPriorityGroups[nOperationPriorityGroup]
	local aOperationDefIds = aOperationPriorityGroup[OPPRIOGRP_OPERATIONDEF_IDS] -- need to search for operators in this set
	local nScanDirection = aOperationPriorityGroup[OPPRIOGRP_SCANDIRECTION] -- from which direction to search
	local nScanFormat = aOperationPriorityGroup[OPPRIOGRP_SCANFORMAT] -- search binary or unary operations

	if nScanFormat ~= OPERATION_FORMAT_UNARY then
		-- nothing to do here if not a unary group
		return
	end

	local nStart, nEnd, nStep, nCurrent
	local nUnaryOperator

	-- process unary operators in given element, which belong to the given priority group
	if nScanDirection == SCANDIRECTION_RIGHTLEFT then
		nStart, nEnd, nStep = #aUnaryOperators, 1, -1
	elseif nScanDirection == SCANDIRECTION_LEFTRIGHT then
		nStart, nEnd, nStep = 1, #aUnaryOperators, 1
	end
	nCurrent = nStart-nStep
	repeat
		nCurrent = nCurrent+nStep
		nUnaryOperator = aUnaryOperators[nCurrent]
		if aOperationDefIds[nUnaryOperator] then
			-- unary operator is in current prio group
			-- create the unary operation
			aElement[ELEMENT_OPERAND] = CreateOperation(aTranslation,
														aOperationDefs[nUnaryOperator][OPERATIONDEF_OPERATOR]..OPNtostring(aElement[ELEMENT_OPERAND]),
														nUnaryOperator,
														{aElement[ELEMENT_OPERAND]})
			-- remove unary operation
			tableremove(aUnaryOperators,nCurrent) -- will also shift all remaining unaries right, one position to the left
			if nScanDirection == SCANDIRECTION_LEFTRIGHT then
				-- need to re-do current index unary if from left to right only, because unaries have shifted
				 -- also, the list has been shortened by 1
				nCurrent, nEnd = nCurrent-nStep, nEnd-nStep
			end
		end
	until nCurrent == nEnd

	if #aUnaryOperators == 0 then
		aElement[ELEMENT_UNARYOPDEFS] = nil -- all done, no need to re-visit in the future
	end
end

-- decomposes a simple expression in unary and binary operations
local function CreateBasicOperations(aTranslation,sExpression)
	-- turn the expression in a list of Elements. each Element contains an Operand, usually a Binary Operator (not the last one) & a list of possible Unary operators.
	-- like -10+24/-32 is turned into 3 elements:
	-- Element 1: operand holding value '10', binary operator '+', unary operator '-': -10+
	-- Element 2: operand holding value '24', binary operator '/', unary operator nil: 24/
	-- Element 3: operand holding value '32', binary operator nil, unary operator '-': -32
	local aElements = CreateElementsFromExpression(aTranslation,sExpression)

	-- the Elements must now be turned into the right sequence of operations.
	-- in the example above we have 4 recognised operators (2 binary and 2 unary), so for this we need to create 4 operations,
	-- BUT in sequence of the right priority (division before addition etc) - for this we have priority groups, each for a defined set of operators [OPPRIOGRP_OPERATIONDEF_IDS]
	-- AND in process order from left or right direction - which is also defined in each priority group [OPPRIOGRP_SCANDIRECTION]

	local aOperationDefIds, nScanDirection, nScanFormat
	local nStart, nEnd, nStep, nCurrent
	local aPrimaryElem, aSecondaryElem, nBinaryOperator

	-- cycle all operator priority groups, in sequence from high to low priority
	for nOperationPriorityGroup, aOperationPriorityGroup in ipairs(aOperationPriorityGroups) do
		aOperationDefIds = aOperationPriorityGroup[OPPRIOGRP_OPERATIONDEF_IDS] -- need to search for operators in this set
		nScanDirection = aOperationPriorityGroup[OPPRIOGRP_SCANDIRECTION] -- from which direction to search
		nScanFormat = aOperationPriorityGroup[OPPRIOGRP_SCANFORMAT] -- search binary or unary operations

		-- process all operators in all elements, which belong to current priority group
		if nScanDirection == SCANDIRECTION_RIGHTLEFT then
			nStart, nEnd, nStep = #aElements, 1, -1
		elseif nScanDirection == SCANDIRECTION_LEFTRIGHT then
			nStart, nEnd, nStep = 1, #aElements, 1
		end
		nCurrent = nStart-nStep
		repeat
			nCurrent = nCurrent+nStep
			if nScanFormat == OPERATION_FORMAT_BINARY then
				aPrimaryElem = aElements[nCurrent]
				nBinaryOperator = aPrimaryElem[ELEMENT_BINARYOPDEF]
				if nBinaryOperator then
					if aOperationDefIds[nBinaryOperator] then
						-- binary operator is in current prio group
						aSecondaryElem = aElements[nCurrent+1]
						-- check for unary operators on secondary operand
						if aSecondaryElem[ELEMENT_UNARYOPDEFS] then
							-- always process any unary operators on secondary operand first
							ProcessUnaryOperations(aTranslation,aSecondaryElem,2) --group 2 contains the unary operations
						end
						-- create the binary operation
						aPrimaryElem[ELEMENT_OPERAND] = CreateOperation(aTranslation,
																		OPNtostring(aPrimaryElem[ELEMENT_OPERAND])..aOperationDefs[nBinaryOperator][OPERATIONDEF_OPERATOR]..OPNtostring(aSecondaryElem[ELEMENT_OPERAND]),
																		nBinaryOperator,
																		{aPrimaryElem[ELEMENT_OPERAND],aSecondaryElem[ELEMENT_OPERAND]})
						-- move any binary operation from secondary to primary element
						aPrimaryElem[ELEMENT_BINARYOPDEF] = aSecondaryElem[ELEMENT_BINARYOPDEF]
						-- remove secondary element
						tableremove(aElements,nCurrent+1) -- will also shift all remaining elements right, one position to the left
						if nScanDirection == SCANDIRECTION_LEFTRIGHT then
							-- need to re-do current index element if from left to right only, because elements have shifted
							-- also, the list has been shortened by 1
							nCurrent, nEnd = nCurrent-nStep, nEnd-nStep
						end
					end
				end
			elseif nScanFormat == OPERATION_FORMAT_UNARY then
				aPrimaryElem = aElements[nCurrent]
				if aPrimaryElem[ELEMENT_UNARYOPDEFS] then
					ProcessUnaryOperations(aTranslation,aPrimaryElem,nOperationPriorityGroup)
				end
			end
		until nCurrent == nEnd
	end

	return CreateExpressionFromElements(aElements) -- should only return "[nResultOp]"
end

-- removes number and boolean constant values from an expression text and replaces them with operand identifiers (indexes in operand table)
-- for example " TRUE , 1.55 " will look something like "[1],[2]"
-- operands with indexes 1 and 2 have then been created in the operand table
local function CreateConstantOperands(aTranslation,sExpression)
	local sReplExpr = sExpression
	-- booleans
	sReplExpr = strgsub(sReplExpr,"%s*FALSE%s*",function(sBoolean) return FetchBooleanConstant(aTranslation,sBoolean) end)
	sReplExpr = strgsub(sReplExpr,"%s*TRUE%s*",function(sBoolean) return FetchBooleanConstant(aTranslation,sBoolean) end)
	-- numbers
	sReplExpr = strgsub(sReplExpr,"%s*%d+%.%d+%s*",function(sNumber) return FetchNumberConstant(aTranslation,sNumber) end)
	-- problem: numbers without dot in operand references [n]
	sReplExpr = strgsub(sReplExpr,"(%s*)([^%d]?)(%d+)([^%d]?)(%s*)",function(sStart,sTest1,sNumber,sTest2,sEnd) if not (sTest1 == "[" and sTest2 == "]") then return sStart..sTest1..FetchNumberConstant(aTranslation,sNumber)..sTest2..sEnd end end) -- just %s*%d+%s* would match n in [n]
	return sReplExpr
end

local function DecomposeParantheses(aTranslation,sExpression)
	local sReplExpr = strgsub(sExpression,"%s*(%b())%s*",function(sNestedExpr) return DecomposeExpression(aTranslation,strsub(sNestedExpr,2,-2)) end)
	return sReplExpr
end

local function DecomposeFunctions(aTranslation,sExpression)
	local sReplExpr = strgsub(sExpression,"%s*(%a+%w*)%s*(%b())%s*",function(sName,sParams) return DecomposeFunction(aTranslation,sName,strsub(sParams,2,-2)) end)
	return sReplExpr
end

-- recursive function which decomposes (parts of) an expression
CreateExpressionOperand = function(aTranslation,sExpression)
	local nStage = 1
	local nResultOp
	local sTempExpr = sExpression
	while true do
		aTranslation[TRANS_EXPRWIP] = sTempExpr
		if nStage == 1 then
			-- check initial state
			if trim(sTempExpr) == "" then
				RaiseExpressionsError(aTranslation,ERR.MISSINGEXPRESSION)
			end
		elseif nStage == 2 then
			-- process functions funcname(expr1,expr2,expr3,..)
			sTempExpr = DecomposeFunctions(aTranslation,sTempExpr)
		elseif nStage == 3 then
			-- process parantheses (expr)
			sTempExpr = DecomposeParantheses(aTranslation,sTempExpr)
		elseif nStage == 4 then
			-- create operands from remaining constant types (number, boolean)
			-- not sooner because of possible numbers in function names
			sTempExpr = CreateConstantOperands(aTranslation,sTempExpr)
		elseif nStage == 5 then
			-- create simple (unary/binary) operations, now we only have operands to work with
			sTempExpr = CreateBasicOperations(aTranslation,sTempExpr)
		elseif nStage == 6 then
			-- still don't have a result operand
			RaiseExpressionsError(aTranslation,ERR.SYNTAX)
		end
		-- contains only [n] ?
		nResultOp = OPStonumber(sTempExpr)
		if nResultOp then
			-- expression contains final result operand
			return nResultOp
		end
		nStage = nStage+1
	end
end

DecomposeExpression = function(aTranslation,sExpression)
	return OPNtostring(CreateExpressionOperand(aTranslation,sExpression))
end

-- removes all variable references from the expression and replaces them with operand identifiers (indexes in operand table)
-- for example "blah blah $TY lefhoihfoe $HIG fslhf" will look like "blah blah [1] lefhoihfoe [2] fslhf"
-- operands with indexes 1 and 2 have then been created in the operand table
local function CreateVariableOperands(aTranslation,sExpression)
	local sReplExpr = strgsub(sExpression,"%s*%$%a+%s*",function(sVar) return FetchVariableOperand(aTranslation,sVar) end)
	return sReplExpr
end

-- replaces the short stat notations for the calcstat function, like @STATNAME# etc
-- the short notations will be replaced by the normal CALCSTAT("STATNAME",$L,$N) or more complex
-- while we are at it, any variable operand use for $L/$N implicated or constant number operands detected, are being processed on the fly
local function GatherCalcStatShorts(aTranslation,sExpression,aProcExpr)
	local nExprIndex = 1
	local nLastIndex
	local nPatEnd, sSN, sL, sHash, sNorC
	local nSNLevelStart, sSNLevel
	while true do
		nLastIndex = nExprIndex
		nExprIndex = strfind(sExpression,"@",nExprIndex,true)
		if not nExprIndex then
			if #sExpression >= nLastIndex then
				-- add remaining text part after last short
				tableinsert(aProcExpr,strsub(sExpression,nLastIndex))
			end
			return
		end
		if nExprIndex > nLastIndex then
			-- add text part between previous and current short
			tableinsert(aProcExpr,strsub(sExpression,nLastIndex,nExprIndex-1))
		end
		nExprIndex = nExprIndex+1

		-- get stat name & optional attached level
		_, nPatEnd, sSN = strfind(sExpression,"^(-?%a%w*)%s*",nExprIndex)
		if not sSN then
			-- syntax error
			RaiseExpressionsError(aTranslation,ERR.CALCSTATSHORTSYNTAX,strsub(sExpression,nExprIndex-1) )
		end
		nExprIndex = nPatEnd+1

		tableinsert(aProcExpr,"CALCSTAT(")

		-- test for attached level number constant in sSN
		nSNLevelStart, _, sSNLevel = strfind(sSN,"(%d+)$")
		-- test for level expression
		_, nPatEnd, sL = strfind(sExpression,"^(%b())%s*",nExprIndex)
		-- process stat name and level
		if sL then
			-- found level expression
			nExprIndex = nPatEnd+1
			if sSNLevel then
				-- found attached level, which is double when we have a level expression as well
				RaiseExpressionsError(aTranslation,ERR.CALCSTATSHORTDOUBLELVL,"@"..sSN..sL)
			end
			sSN = FetchStringConstant(aTranslation,"\""..sSN.."\"") -- transform statname to string constant
			tableinsert(aProcExpr,sSN)
			tableinsert(aProcExpr,",")
			-- contains level expression between parantheses
			GatherCalcStatShorts(aTranslation,strmatch(sL,"^%(%s*(.-)%s*%)$"),aProcExpr) -- remove parantheses, get expression
		else
			if sSNLevel then
				-- split off number constant from stat name
				sSN = strsub(sSN,1,nSNLevelStart-1) -- first part minus level
				sL = FetchNumberConstant(aTranslation,sSNLevel) -- transform level to number constant
			else
				-- no level number constant found, so use default variable $L
				sL = FetchVariableOperand(aTranslation,"$L") -- create level variable (or get if already exists)
			end
			-- process stat name
			sSN = FetchStringConstant(aTranslation,"\""..sSN.."\"") -- transform statname to string constant
			tableinsert(aProcExpr,sSN)
			tableinsert(aProcExpr,",")
			tableinsert(aProcExpr,sL)
		end
		
		-- test for hash
		_, nPatEnd, sHash = strfind(sExpression,"^(#)%s*",nExprIndex)
		-- process NorC
		if sHash then
			nExprIndex = nPatEnd+1
			tableinsert(aProcExpr,",")
			-- test for NorC expression
			_, nPatEnd, sNorC = strfind(sExpression,"^(%b())%s*",nExprIndex)
			if sNorC then
				nExprIndex = nPatEnd+1
				-- contains NorC expression between parantheses
				GatherCalcStatShorts(aTranslation,strmatch(sNorC,"^%(%s*(.-)%s*%)$"),aProcExpr) -- remove parantheses, get expression
			else
				-- test for N constant with decimals
				_, nPatEnd, sNorC = strfind(sExpression,"^(%d+%.%d+)%s*",nExprIndex)
				if not sNorC then
					-- test for N constant without decimals
					_, nPatEnd, sNorC = strfind(sExpression,"^(%d+)%s*",nExprIndex)
				end
				if sNorC then
					nExprIndex = nPatEnd+1
					-- contains N number constant
					sNorC = FetchNumberConstant(aTranslation,sNorC) -- transform value for $N to number constant
				else
					-- only hash
					sNorC = FetchVariableOperand(aTranslation,"$N") -- create N variable (or get if already exists)
				end
				tableinsert(aProcExpr,sNorC)
			end
		end

		tableinsert(aProcExpr,")")
	end
end

local function ReplaceCalcStatShorts(aTranslation,sExpression)
	local aProcExpr = {}
	GatherCalcStatShorts(aTranslation,sExpression,aProcExpr)
	return tableconcat(aProcExpr)
end

-- extracts string operands from the expression and replaces them with the operand index indicator '[n]'
local function CreateStringConstants(aTranslation,sExpression)
	return strgsub(strgsub(sExpression,"\\.",function(sEscapeChar)
					if sEscapeChar == "\\\"" then return "\\q"
					else return sEscapeChar
					end
				end),"%s*\".-\"%s*",function(sString) return FetchStringConstant(aTranslation,sString) end)
end

-- sets a variable to a value 
local function SetExpressionVariable(aTranslation,sVarName,xValue)
	if type(aTranslation) ~= "table" then
		return nil, {Source = "CalcStat:Expressions", Stage = EXPRERR_AT_INITIALIZATION, Code = ERR.MISSINGTRANSLATION[1], Message = ERR.MISSINGTRANSLATION[2], Detail = "Function 'SetExpressionVariable'"}
	end
	if type(sVarName) ~= "string" then return end
	local nVar = aTranslation[TRANS_VARNAMEIDX][strupper(sVarName)]
	if not nVar then return end
	aTranslation[TRANS_VARIABLES][nVar][VAR_VALUE] = xValue
end

-- sets the values of multiple variables (table keys = variable names, table values = variable values)
local function SetExpressionVariables(aTranslation,aSetVariables)
	if type(aTranslation) ~= "table" then
		return nil, {Source = "CalcStat:Expressions", Stage = EXPRERR_AT_INITIALIZATION, Code = ERR.MISSINGTRANSLATION[1], Message = ERR.MISSINGTRANSLATION[2], Detail = "Function 'SetExpressionVariables'"}
	end
	if type(aSetVariables) ~= "table" then return end
	local nVar
	for sVarName, xValue in pairs(aSetVariables) do
		if type(sVarName) == "string" then
			nVar = aTranslation[TRANS_VARNAMEIDX][strupper(sVarName)]
			if nVar then
				aTranslation[TRANS_VARIABLES][nVar][VAR_VALUE] = xValue
			end
		end
	end
end

-- gets a variable's value, which makes sense after execution (like requesting the $Style variable)
local function GetExpressionVariable(aTranslation,sVarName)
	if type(aTranslation) ~= "table" then
		return nil, {Source = "CalcStat:Expressions", Code = ERR.MISSINGTRANSLATION[1], Message = ERR.MISSINGTRANSLATION[2], Detail = "Function 'GetExpressionVariable'"}
	end
	if type(sVarName) ~= "string" then return end
	local nVar = aTranslation[TRANS_VARNAMEIDX][strupper(sVarName)]
	if not nVar then return end
	return aTranslation[TRANS_VARIABLES][nVar][VAR_VALUE]
end

-- process translation stages
local function ProcessExpression(aTranslation,sExpression)
	local nStage = 1
	local nResultOp
	local sTempExpr = sExpression
	while true do
		aTranslation[TRANS_EXPRWIP] = sTempExpr
		if nStage == 1 then
			-- check initial state
			if type(sTempExpr) ~= "string" then
				RaiseExpressionsError(aTranslation,ERR.MISSINGEXPRESSION,"Expression:"..type(sTempExpr)..", Expecting:string")
			end
		elseif nStage == 2 then
			-- remove comments between /* */
			sTempExpr = trim(strgsub(sTempExpr,"(/%*.-%*/)",""))
			if sTempExpr == "" then
				-- nothing left there
				RaiseExpressionsError(aTranslation,ERR.MISSINGEXPRESSION,"")
			end
		elseif nStage == 3 then
			-- create operands for string constants
			sTempExpr = CreateStringConstants(aTranslation,sTempExpr)
		elseif nStage == 4 then
			-- convert expression to uppercase now we have processed the strings +
			-- replace CalcStat shorts: @StatName etc with function call expressions
			sTempExpr = ReplaceCalcStatShorts(aTranslation,strupper(sTempExpr))
		elseif nStage == 5 then
			-- create operands for variables
			sTempExpr = CreateVariableOperands(aTranslation,sTempExpr)
		elseif nStage == 6 then
			-- extract operations - returns "[n]", with indicating operand index n containing the final result
			sTempExpr = DecomposeExpression(aTranslation,sTempExpr)
		elseif nStage == 7 then
			-- still don't have a result operand
			RaiseExpressionsError(aTranslation,ERR.SYNTAX)
		end
		-- contains only [n] ?
		nResultOp = OPStonumber(sTempExpr)
		if nResultOp then
			-- expression contains final result operand
			aTranslation[TRANS_EXPRWIP] = OPNtostring(nResultOp)
			aTranslation[TRANS_RESULTOP] = nResultOp
			-- check availability TIMESTAMP variable
			if not os then
				-- if not: set to unsupported (when referenced at all)
				SetExpressionVariable(aTranslation,"TIMESTAMP","<unsupported>")
			end
			return aTranslation
		end
		nStage = nStage+1
	end
end

-- translates an expression to an intermediate object, given a definition of runtime variables.
-- resulting object may be evaluated/executed (see ExecuteExpression()), while providing runtime variables with values.
local function TranslateExpression(sExpression,aInitVariables)
	-- basic initialization of translation object
	local aTranslation =
		{
			sExpression, -- TRANS_EXPRESSION Original expression which translation will represent
			"", -- TRANS_EXPRWIP expression work in progress: expression in various states of translation
			0, -- TRANS_RESULTOP operand index which will hold the result
			nil, -- TRANS_ERROR indicating last error during translation or execution
			{}, -- TRANS_OPERATIONS Operations table
			{}, -- TRANS_OPERANDS Operands table
			{}, -- TRANS_OPSOURCEIDX Operands table index on operand source
			{}, -- TRANS_VARIABLES Variables table
			{}, -- TRANS_VARNAMEIDX Variables table index on variable name
			0, -- TRANS_OPERANDREQUESTS current number of requests for an operand
			10000, -- TRANS_MAXOPERANDREQUESTS maximum number of requests for an operand (should be enough for translating an expression)
			0 -- TRANS_EXECUTIONS executions counter
		}
	if type(aInitVariables) == "table" then
		-- populate variable table with supplied variables
		local aTransVars = aTranslation[TRANS_VARIABLES]
		local aTransVarNameIdx = aTranslation[TRANS_VARNAMEIDX]
		local sVarName
		for INITVAR_NAME, INITVAR_VALUE in pairs(aInitVariables) do
			sVarName = strupper(INITVAR_NAME)
			aTransVars[#aTransVars+1] =
				{
					sVarName, -- OPERAND_VALUE variable's name
					INITVAR_VALUE -- OPERAND_OPERATION initial value
				}
			aTransVarNameIdx[sVarName] = #aTransVars -- store name+index into variable name searchindex
		end
	end
	return ErrorProtectedCall(aTranslation,ProcessExpression,sExpression)
end

-- execute translated expression and return result
-- needs a translation object, optionally number of maximum operand requests, default: 10000
local function ExecuteExpression(aTranslation,aSetVariables,nMaxOpRequests)
	if type(aTranslation) ~= "table" then
		return nil, {Source = "CalcStat:Expressions", Stage = EXPRERR_AT_EXECUTION, Code = ERR.MISSINGTRANSLATION[1], Message = ERR.MISSINGTRANSLATION[2], Detail = "Function 'ExecuteExpression'"}
	end
	aTranslation[TRANS_EXECUTIONS] = aTranslation[TRANS_EXECUTIONS]+1
	if type(aSetVariables) == "table" then
		SetExpressionVariables(aTranslation,aSetVariables)
	end
	-- check availability TIMESTAMP variable
	if os then
		-- if so: set execution time now (when referenced at all)
		SetExpressionVariable(aTranslation,"TIMESTAMP",strgsub(os.date("%H:%M, %d %B %Y (UTC)"),", 0",", "))
	end
	 -- init operand request counter
	aTranslation[TRANS_OPERANDREQUESTS] = 0
	if type(nMaxOpRequests) == "number" then
		aTranslation[TRANS_MAXOPERANDREQUESTS] = nMaxOpRequests
	else
		aTranslation[TRANS_MAXOPERANDREQUESTS] = 10000
	end
	return ErrorProtectedCall(aTranslation,GetOperandValue,aTranslation[TRANS_RESULTOP])
end

-- translate and execute for a one time expression evaluation result
local function OneShotExpression(sExpression,aInitVariables,nMaxOpRequests)
	local xResult
	-- translate expression
	local aTranslation, e = TranslateExpression(sExpression,aInitVariables)
	if not e then
		-- execute expression
		xResult, e = ExecuteExpression(aTranslation,nil,nMaxOpRequests)
	end
	return xResult, e
end

-- to be used by other modules
p.SetExpressionVariable = SetExpressionVariable -- sets the value of a variable (if it exists - doesn't create a new one)
p.SetExpressionVariables = SetExpressionVariables -- sets the values of multiple variables at once (if they exists - doesn't create new ones)
p.GetExpressionVariable = GetExpressionVariable -- gets the value of a variable (if it exists)
p.TranslateExpression = TranslateExpression -- translates an expression and returns translated object / interpreter code
p.ExecuteExpression = ExecuteExpression -- executes translated object / interpreter code and returns result value
p.OneShotExpression = OneShotExpression -- translate and execute for a one time expression evaluation result

-- ************************ End CalcStat Expressions **************************
