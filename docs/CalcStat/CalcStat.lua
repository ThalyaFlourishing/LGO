-- Created by Giseldah

local p = _G --p stands for package

-- local used library functions
local mathabs = math.abs
local mathfloor = math.floor
local mathfrexp = math.frexp
local mathhuge = math.huge
local mathldexp = math.ldexp
local mathlog10 = math.log10
local mathmodf = math.modf
local strfind = string.find
local strmatch = string.match
local strsub = string.sub
local strupper = string.upper

-- ****************************** Start CalcStat ******************************

local CalcStat
local LinFmod

-- removes leading and trailing white space characters from a string
local function trim(s)
	if type(s) ~= "string" then return "" end
	return strmatch(s,"^()%s*$") and "" or strmatch(s,"^%s*(.*%S)")
end

-- **************** Misc. floating point support functions ****************

-- Misc. functions for floating point rounding.
-- 2nd parameter is number of decimals.

-- Floating point numbers bring errors into the calculation, both inside the Lotro-client and in this function collection. This is why a 100% match with the stats in Lotro is impossible.
-- Anyway, to compensate for some errors, we use a calculation deviation correction value. This makes for instance 24.49999999 round to 25, as it's assumed that 24.5 was intended as outcome of a formula.
local DblCalcDev = 1e-8
local DblCorrDown = DblCalcDev
local DblCorrNormal = 0.5+DblCalcDev
local DblCorrUp = 1.0-DblCalcDev

local IntPow10 = {10,100,1000,10000,100000,1000000,10000000,100000000,1000000000,10000000000} -- 1-10 decimals

local function CorrectDbl(vNum,dCorrection,iDec)
	if vNum == 0.0 then return 0 end -- integer result

	local dSignedCorrection = dCorrection
	if vNum < 0.0 then dSignedCorrection = -dCorrection end

	if iDec == 0 then return (mathmodf(vNum+dSignedCorrection)) end -- integer result

	local iFactor

	if iDec < 0 then
		iFactor = IntPow10[-iDec]
		return (mathmodf(vNum/iFactor+dSignedCorrection))*iFactor -- integer result
	end

	iFactor = IntPow10[iDec]
	local dRounded = (mathmodf(vNum*iFactor+dSignedCorrection))/iFactor
	local iInt, fFrac = mathmodf(dRounded)
	if fFrac == 0.0 then return iInt end -- integer result

	return dRounded -- double result
end

local function RoundDbl(vNum,vDec)
	if vDec == nil then return CorrectDbl(vNum,DblCorrNormal,0) end
	return CorrectDbl(vNum,DblCorrNormal,(mathmodf(vDec)))
end

local function RoundDblDown(vNum,vDec)
	if vDec == nil then return CorrectDbl(vNum,DblCorrDown,0) end
	return CorrectDbl(vNum,DblCorrDown,(mathmodf(vDec)))
end

local function RoundDblUp(vNum,vDec)
	if vDec == nil then return CorrectDbl(vNum,DblCorrUp,0) end
	return CorrectDbl(vNum,DblCorrUp,(mathmodf(vDec)))
end

local function RoundDblLotro(vNum)
	if (0.0 > vNum and vNum >= -2.0) then return -2 end
	local dAbsNum = mathabs(vNum)
	if dAbsNum <= 1000.0 then return CorrectDbl(vNum,DblCorrUp,0) end
	return CorrectDbl(vNum,DblCorrUp,2-mathfloor(mathlog10(dAbsNum)))
end

local function RoundDblMorReg(vNum)
	local dAbsNum = mathabs(vNum)
	if dAbsNum <= 100.0 then return CorrectDbl(vNum,DblCorrUp,2) end
	if dAbsNum <= 1000.0 then return CorrectDbl(vNum,DblCorrUp,1) end
	return CorrectDbl(vNum,DblCorrUp,2-mathfloor(mathlog10(dAbsNum)))
end

local function RoundDblProg(vNum)
	if vNum == 0.0 then return vNum end
	return CorrectDbl(vNum,DblCorrNormal,2-mathfloor(mathlog10(mathabs(vNum))+0.5))
end

-- Constants for single-precision floating-point calculations
local IEEE754_MANTISSA_SIZE = 23 -- 23 bits for single-precision IEEE 754
local IEEE754_MANTISSA_BITSCALE = 2^IEEE754_MANTISSA_SIZE -- Scales mantissa [0,1) to bits value
local IEEE754_MANTISSA_VALUESCALE = 2^-IEEE754_MANTISSA_SIZE -- Scales mantissa bits value to [0,1)
local IEEE754_EXPONENT_SIZE = 8 -- 8 bits for single-precision IEEE 754
local IEEE754_MIN_NORMAL_EXPONENT = 2-2^(IEEE754_EXPONENT_SIZE-1) -- Smallest exponent for normalized numbers
local IEEE754_MAX_NORMAL_EXPONENT = 2^(IEEE754_EXPONENT_SIZE-1)-1 -- Largest exponent for normalized numbers

-- Converts a double-precision value into the equivalent of a single-precision value
local function EquSng(vVal)
	-- Handle special cases (zero/-zero, infinity, -infinity, NaN)
	if vVal == 0 or vVal == mathhuge or vVal == -mathhuge or vVal ~= vVal then
		return vVal
	end

	-- Get the sign and absolute value of the (double float) number
	local nSign = 1
	local nValAbs = vVal
	if vVal < 0 then
		nSign = -1
		nValAbs = -vVal
	end

	-- Use math.frexp to get mantissa and exponent, where nValAbs = mantissa * 2^exponent, with mantissa in the range [0.5,1.0)
	local nMantissa, nExponent = mathfrexp(nValAbs)
	-- math.frexp did not return a mantissa value in the right interval for a normalized number
	nMantissa = nMantissa*2 -- Transform mantissa from [0.5,1.0) to [1.0,2.0) for now. Later becomes implicit leading 1.0 + [0.0,1.0) for normalized numbers.
	nExponent = nExponent-1 -- Mantissa became larger by factor 2^1, need to compensate for this in the exponent.

	if nExponent > IEEE754_MAX_NORMAL_EXPONENT then
		-- Overflow to infinity
		return (nSign < 0) and -mathhuge or mathhuge
	end

	-- Explicit leading 1 used for calculating the single float value: will be 1(normalized value) or 0(subnormal number)
	local nLeadingOne = 0
	if nExponent < IEEE754_MIN_NORMAL_EXPONENT then
		-- Subnormal number
		nMantissa = mathldexp(nMantissa,nExponent-IEEE754_MIN_NORMAL_EXPONENT) -- Transfer old exponent into mantissa and extract new exponent(MIN_NORMAL_EXPONENT) in one go
		if nMantissa == 0 then
			return 0*nSign  -- flush to zero, preserve sign
		end
		nExponent = IEEE754_MIN_NORMAL_EXPONENT
	else
		-- Normalized number
		nLeadingOne = 1 -- Transfer 1 from mantissa to 'explicit leading 1'
		nMantissa = nMantissa-1 -- Mantissa is now in the interval [0.0,1.0)
	end

	-- Scale mantissa to bitfield representation integer
	local nFraction; nMantissa, nFraction = mathmodf(nMantissa*IEEE754_MANTISSA_BITSCALE)
	-- Round to nearest, ties to even (sticky to even)
    if nFraction > 0.5 or (nFraction == 0.5 and nMantissa%2 ~= 0) then
        nMantissa = nMantissa+1 -- Round-up if fraction is larger than 0.5 or if fraction is 0.5 and current mantissa is not an even number
    end
	-- Handle overflow
	if nMantissa == IEEE754_MANTISSA_BITSCALE then
		-- Overflow in mantissa
		if nLeadingOne == 0 then
			-- Subnormal number: transform to Normalized number
			nLeadingOne = 1
		else
			-- Normalized number: increment exponent
			nExponent = nExponent+1
			if nExponent > IEEE754_MAX_NORMAL_EXPONENT then
				-- Overflow to infinity
				return (nSign < 0) and -mathhuge or mathhuge
			end
		end
		nMantissa = 0 -- Reset mantissa
	end

	return nSign*mathldexp(nLeadingOne+nMantissa*IEEE754_MANTISSA_VALUESCALE,nExponent)
end

-- Converts a double value into the decimal representation of an equivalent single float value
local function DecSng(vVal)
	local dVal = EquSng(vVal)
	if dVal == 0.0 then return 0.0 end -- return 0 when 0
	
	-- calculate decimals interval for a max total of 8 digit precision
	-- 0.09#######: 9 to 2
	-- 0.9#######: 8 to 1
	-- 9.#######: 7 to 0
	-- 9#.######: 6 to -1
	-- 9##.#####: 5 to -2 etc
	local iDecMin = 8-(mathfloor(mathlog10(mathabs(dVal)))+1)
	local iDecMax = iDecMin-7
	-- result always needs to be rounded at least once, even if the result is not the same as the original equiv. float value
	local dResult = CorrectDbl(dVal,DblCorrNormal,iDecMin)
	-- search for the least number of decimals, while still keeping the same single value
	local dTest
	for iDec = iDecMin-1,iDecMax,-1 do
		dTest = CorrectDbl(dVal,DblCorrNormal,iDec) -- test value with ever less precision
		if dTest ~= dResult then
			if EquSng(dTest) == dVal then
				dResult = dTest
			else
				-- (test)value contains no longer the same single value
				break
			end
		end
	end
	return dResult
end

-- ****************** Calculation Type support functions ******************

-- DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD
-- DataTableValue: Takes a value from an array table.
local function DataTableValue(vDataArray,dIndex)
	local iIndex = RoundDbl(dIndex)
	if iIndex <= 1 then return vDataArray[1] end
	if iIndex > #vDataArray then return vDataArray[#vDataArray] end
	return vDataArray[iIndex]
end

-- EEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE
-- ExpFmod: Exponential function based on percentage.
-- Common percentage values are around ~5.5% for between levels and ~20% jumps between level segments.
local function ExpFmod(dVal,dLstart,dPlvl,dLvl,vAdd,vDec)
	local dRng = dLvl-dLstart+1.0
	if dRng <= 0.0 then return dVal end
	
	local dFac = 1.0+dPlvl/100.0

	local dAdd = 0.0
	if vAdd ~= nil then dAdd = vAdd end

	if vDec == nil then
		local dFacExp = dFac^dRng
		return dVal*dFacExp+dAdd*((dFacExp-1.0)/(dFac-1.0))
	end
	
	local dResult = dVal
	local dL = dLstart
	while dL <= dLvl do
		dResult = RoundDbl(dResult*dFac+dAdd,vDec)
		dL = dL+1.0
	end
	return dResult
end

-- IIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIIII
-- LinInter: Linear Interpolation, given simple graph point data {{levels},{values}}
local function LinInter(dProgGraph,dLvl)
	-- parameter processing
	local dLevels = dProgGraph[1]
	local dValues = dProgGraph[2]

	-- find interval points for requested level
	local iHigh = 2
	local iMax = #dLevels
	while iHigh < iMax and dLvl > dLevels[iHigh] do iHigh = iHigh+1 end
	local iLow = iHigh-1

	-- return interpolated value from the calculated graph points
	return LinFmod(1.0,dValues[iLow],dValues[iHigh],dLevels[iLow],dLevels[iHigh],dLvl)
end

-- PPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPP
-- CalcPercAB: Calculates the percentage out of a rating based on the AB formula.
local function CalcPercAB(dA,dB,dPCap,dR)
	if dR <= 0.0 then return 0.0 end
	local dResult = dA/(1.0+dB/dR)
	if dResult >= dPCap then return dPCap end
	return dResult
end

-- RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR
-- CalcRatAB: Calculates the rating out of a percentage based on the AB formula.
local function CalcRatAB(dA,dB,dCapR,dP)
	if dP <= 0.0 then return 0.0 end
	local dResult = dB/(dA/dP-1.0)
	if dResult >= dCapR then return dCapR end
	return dResult
end

-- SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS
-- StatLinInter: (Normalized) Stat Linear Interpolating
local function StatLinInter(sPntMP,sProgScheme,sProgBase,sAdj,dLvl,vNorC,vRoundType)
	-- parameter processing
	local dN = 1.0
	local sC = ""
	if vNorC then
		if type(vNorC) == "number" then dN = vNorC else sC = vNorC end
	end

	local dProgScheme = CalcStat(sProgScheme,dLvl)
	local dAccessLvls = dProgScheme[1]
	local dBaseLvls = dProgScheme[2]

	-- find interval points for requested level
	local iHigh = 2
	local iMax = #dAccessLvls
	while iHigh < iMax and dLvl > dAccessLvls[iHigh] do iHigh = iHigh+1 end
	local iLow = iHigh-1

	local dAccessLvlLow = dAccessLvls[iLow]
	local dAccessLvlHigh = dAccessLvls[iHigh]

	local dValLow, dValHigh
	if sProgBase == "" then
		-- if not given: expect the base levels to contain the values directly
		dValLow = dBaseLvls[iLow]
		dValHigh = dBaseLvls[iHigh]
	else
		-- get values from base progression if given
		dValLow = CalcStat(sProgBase,dBaseLvls[iLow],sC)
		dValHigh = CalcStat(sProgBase,dBaseLvls[iHigh],sC)
	end
	
	-- graph point multiplications
	if sPntMP ~= "" then
		dValLow = dValLow*CalcStat(sPntMP,dAccessLvlLow,sC)
		dValHigh = dValHigh*CalcStat(sPntMP,dAccessLvlHigh,sC)
	end
	if sAdj ~= "" then
		dValLow = dValLow*CalcStat(sAdj,dAccessLvlLow,sC)
		dValHigh = dValHigh*CalcStat(sAdj,dAccessLvlHigh,sC)
	end

	-- return interpolated value from the calculated graph points
	return LinFmod(dN,dValLow,dValHigh,dAccessLvlLow,dAccessLvlHigh,dLvl,vRoundType)
end

-- TTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTTT
-- LinFmod: Linear line function between 2 points with some optional modifications.
-- Connects point (dLstart,dVal*dFstart) with (dLend,dVal*dFend).
-- Usually used with dVal=1 and dFstart/dFend containing unrelated points or dVal=base and dFstart/dFend containing multiplier factors.
-- Modification for points: rounding.

-- list of functions which are used for rounding graph point values
local fRoundTypes = {
	[0] = function(dValue) return dValue end, -- no rounding function
	RoundDblProg,
	RoundDblLotro,
	RoundDblMorReg
}

LinFmod = function(dVal,dFstart,dFend,dLstart,dLend,dLvl,vRoundType)
	-- parameter processing
	local fRound = fRoundTypes[vRoundType or 0]

	-- finalize interval values: multiply base Value by Factor and apply rounding by requested type
	-- return point value directly if requested level is a low/high interval point
	local dVstart = fRound(dVal*dFstart)
	if dLvl == dLstart then return dVstart end
	local dVend = fRound(dVal*dFend)
	if dLvl == dLend then return dVend end

	if dLstart == dLend then return 0.0 end -- can't interpolate, return 0

	-- return interpolated value from the calculated graph points
	return (dVstart*(dLend-dLvl)+(dLvl-dLstart)*dVend)/(dLend-dLstart)
end

-- VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV
-- TranslateValue: translates a value to an other by using a lookup table and a result table.
-- If the value is not found in the lookup table then the function returns the default value, which is the last element in the result table.
-- Object types allowed are both numbers (doubles, integers) and strings (non case sensitive matching) and may be mixed.

local function TranslateValue(vSourceData,vResultData,vSearch)
	if type(vSearch) == "number" then
		for iSrc, vSrc in ipairs(vSourceData) do
			if type(vSrc) == "number" and vSrc == vSearch then return vResultData[iSrc] end
		end
	elseif type(vSearch) == "string" then
		local sSearch = strupper(trim(vSearch))
		for iSrc, vSrc in ipairs(vSourceData) do
			if type(vSrc) == "string" and vSrc == sSearch then return vResultData[iSrc] end
		end
	end

	return vResultData[#vResultData] -- return default value
end

-- **************** Parameter "C" decode support functions ****************

-- ArmCodeIndex: returns a specified index from an Armour Code.
-- sACode string:
-- 1st position: H=heavy, M=medium, L=light
-- 2nd position: H=head, S=shoulders, CL=cloak/back, C=chest, G=gloves, L=leggings, B=boots, Sh=shield
-- 3rd position: W=white/common, Y=yellow/uncommon, P=purple/rare, T=teal/blue/incomparable, G=gold/legendary/epic
-- Note: no such thing exists as a heavy, medium or light cloak, so no H/M/L in cloak codes (cloaks go automatically in the M class since U23, although historically this was L)
local function ArmCodeIndex(sACode,iI)
	local sArmCode = trim(sACode)
	if sArmCode == "" then return 0 end
	sArmCode = strupper(sArmCode).."    "

	local sArmCat = strsub(sArmCode,1,1)
	local sArmType = strsub(sArmCode,2,2)
	local sArmCol = strsub(sArmCode,3,3)

	if sArmType == "S" and sArmCol == "H" then
		sArmType = "SH"
		sArmCol = strsub(sArmCode,4,4)
	elseif sArmCat == "C" and sArmType == "L" then
		sArmCat = "M"
		sArmType = "CL"
	elseif sArmType then
		sArmType = " "..sArmType
	end

	local result = 0
	if iI == 1 then
		if sArmCat then
			result = strfind("HML",sArmCat)
		end
	elseif iI == 2 then
		if sArmType then
			result = strfind(" H SCL C G L BSH",sArmType)
			if result then
				result = (result+1)/2
			end
		end
	elseif iI == 3 then
		if sArmCol then
			result = strfind("WYPTG",sArmCol)
		end
	end

	if result then return result else return 0 end
end

-- EnumIndex: returns an index in a Enum string for a character at a specified index in a Code string
-- returns 0 for unknown or missing character
local function EnumIndex(sCodeChars,iCodeIndex,sEnumChars)
	local sCode = trim(sCodeChars)
	if sCode == "" then return 0 end -- no code given
	sCode = strupper(sCode)

	local sChar = strsub(sCode,iCodeIndex,iCodeIndex)
	if sChar == "" then return 0 end -- no character found at specified index

	return strfind(sEnumChars,sChar) or 0 -- returns index of char in Enum string or 0 if not found
end

-- RomanRankDecode: converts a string with a Roman number in characters, to an integer number.
-- used for Legendary Item Title calculation.
local sRomanRankChars = {"M","CM","D","CD","C","XC","L","XL","X","IX","V","IV","I"}
local iRomanRankValues = {1000,900,500,400,100,90,50,40,10,9,5,4,1}

local function RomanRankDecode(sRCode)
	local sRomanCode = strupper(trim(sRCode))
	if sRomanCode == "" then return 0 end

	local iValue = 0

	local iCharFound
	local iCharEnd
	
	local C = 1
	local iCodeLen = #sRomanCode
	while C <= iCodeLen do
		iCharFound = nil
		for I = 1, 13 do
			iCharEnd = C+(I-1)%2
			if iCharEnd <= iCodeLen and strsub(sRomanCode,C,iCharEnd) == sRomanRankChars[I] then
				iCharFound = I
				break
			end
		end
		if iCharFound == nil then break end -- found unknown character: terminate
		iValue = iValue+iRomanRankValues[iCharFound]
		C = iCharEnd+1
	end

	return iValue
end

-- ************************ Main CalcStat function ************************

CalcStat = function(SName,SLvl,SParam)
	-- process parameters and parameter defaults

	local sStatName = trim(SName)
	if sStatName == "" then
		return 0, {Source = "CalcStat", Code = -1, Message = "Missing stat name"}
	end

	local SN
	repeat
		SN = strmatch(sStatName,"^-?%a%w*%a$") -- allow nested digits in stat name and a starting -
		if SN then break end
		SN = strmatch(sStatName,"^-?%a$") -- to allow single character stat name as well
		if SN then break end
		return 0, {Source = "CalcStat", Code = -2, Message = "Illegal stat name", Detail = "Stat '"..sStatName.."'"}
	until true
	SN = strupper(SN) -- uppercase and trimmed for keyword matching
	
	local L = 1.0 -- default L
	if type(SLvl) == "number" then
		L = SLvl
	elseif type(SLvl) ~= "nil" then
		return 0, {Source = "CalcStat", Code = -3, Message = "Illegal level", Detail = "Stat '"..SN.."' [Level:"..type(SLvl)..", Expecting:number,nil]"}
	end

	local N = 1.0 -- default N
	local C = "" -- default C
	if type(SParam) == "number" then
		N = SParam
	elseif type(SParam) == "string" then
		C = SParam
	elseif type(SParam) ~= "nil" then
		return 0, {Source = "CalcStat", Code = -4, Message = "Illegal N or C", Detail = "Stat '"..SN.."' [N or C:"..type(SParam)..", Expecting:number,string,nil]"}
	end

	local Result = 0.0

	-- binary search tree (generated code)

	if SN > "MARINERCDBASEICPR" then
		if SN > "PHYMITLPPRAT" then
			if SN > "STDPROGRATINGS" then
				if SN < "TACMITMPBONUS" then
					if SN > "TACDMGPRATPA" then
						if SN < "TACMITHPRATPC" then
							if SN > "TACMITHPBONUS" then
								if SN > "TACMITHPRATP" then
									if SN == "TACMITHPRATPA" then
										Result = CalcStat("MitHeavyPRatPA",L)
									elseif SN == "TACMITHPRATPB" then
										Result = CalcStat("MitHeavyPRatPB",L)
									end
								elseif SN == "TACMITHPPRAT" then
									Result = CalcStat("MitHeavyPPRat",L,N)
								elseif SN == "TACMITHPRATP" then
									Result = CalcStat("MitHeavyPRatP",L,N)
								end
							elseif SN < "TACMITHPBONUS" then
								if SN > "TACDMGPRATPC" then
									if SN == "TACDMGPRATPCAP" then
										Result = CalcStat("OutDmgPRatPCap",L)
									elseif SN == "TACDMGPRATPCAPR" then
										Result = CalcStat("OutDmgPRatPCapR",L)
									end
								elseif SN == "TACDMGPRATPB" then
									Result = CalcStat("OutDmgPRatPB",L)
								elseif SN == "TACDMGPRATPC" then
									Result = CalcStat("OutDmgPRatPC",L)
								end
							else
								Result = CalcStat("MitHeavyPBonus",L)
							end
						elseif SN > "TACMITHPRATPC" then
							if SN > "TACMITLPRATP" then
								if SN < "TACMITLPRATPC" then
									if SN == "TACMITLPRATPA" then
										Result = CalcStat("MitLightPRatPA",L)
									elseif SN == "TACMITLPRATPB" then
										Result = CalcStat("MitLightPRatPB",L)
									end
								elseif SN > "TACMITLPRATPC" then
									if SN == "TACMITLPRATPCAP" then
										Result = CalcStat("MitLightPRatPCap",L)
									elseif SN == "TACMITLPRATPCAPR" then
										Result = CalcStat("MitLightPRatPCapR",L)
									end
								else
									Result = CalcStat("MitLightPRatPC",L)
								end
							elseif SN < "TACMITLPRATP" then
								if SN > "TACMITHPRATPCAPR" then
									if SN == "TACMITLPBONUS" then
										Result = CalcStat("MitLightPBonus",L)
									elseif SN == "TACMITLPPRAT" then
										Result = CalcStat("MitLightPPRat",L,N)
									end
								elseif SN == "TACMITHPRATPCAP" then
									Result = CalcStat("MitHeavyPRatPCap",L)
								elseif SN == "TACMITHPRATPCAPR" then
									Result = CalcStat("MitHeavyPRatPCapR",L)
								end
							else
								Result = CalcStat("MitLightPRatP",L,N)
							end
						else
							Result = CalcStat("MitHeavyPRatPC",L)
						end
					elseif SN < "TACDMGPRATPA" then
						if SN < "STOUTDOOMDRASAFATE" then
							if SN > "STOUTAXERDTRAITMIGHT" then
								if SN > "STOUTAXERDTRAITSHADOWMITP" then
									if SN == "STOUTAXERDTRAITVITALITY" then
										Result = CalcStat("StoutShadowEyeVitality",L)
									elseif SN == "STOUTAXERDTRAITWILL" then
										Result = CalcStat("StoutUnyieldingWill",L)
									end
								elseif SN == "STOUTAXERDTRAITPHYMITP" then
									Result = CalcStat("StoutUnyieldingPhyMitP",L)
								elseif SN == "STOUTAXERDTRAITSHADOWMITP" then
									Result = CalcStat("StoutWrBlackLShadowMitP",L)
								end
							elseif SN < "STOUTAXERDTRAITMIGHT" then
								if SN > "STOUTAXERDTRAITAGILITY" then
									if SN == "STOUTAXERDTRAITDISEASERESISTP" then
										Result = CalcStat("StoutWrBlackLDiseaseResistP",L)
									elseif SN == "STOUTAXERDTRAITFATE" then
										Result = CalcStat("StoutDoomDrasaFate",L)
									end
								elseif SN == "STDPROGRATINGSOLD" then
									if 151 <= L and L <= 160 then
										Result = LinFmod(CalcStat("StdProgRatings",150,N),1,2.0,151,160,L,1)
									else
										Result = CalcStat("StdProgRatings",L,N)
									end
								elseif SN == "STOUTAXERDTRAITAGILITY" then
									Result = CalcStat("StoutWrBlackLAgility",L)
								end
							else
								Result = CalcStat("StoutWrBlackLMight",L)
							end
						elseif SN > "STOUTDOOMDRASAFATE" then
							if SN > "STOUTWRBLACKLDISEASERESISTP" then
								if SN < "TACDMGPBONUS" then
									if SN == "STOUTWRBLACKLMIGHT" then
										Result = CalcStat("MightT",L,1.0)
									elseif SN == "STOUTWRBLACKLSHADOWMITP" then
										Result = 1.0
									end
								elseif SN > "TACDMGPBONUS" then
									if SN == "TACDMGPPRAT" then
										Result = CalcStat("OutDmgPPRat",L,N)
									elseif SN == "TACDMGPRATP" then
										Result = CalcStat("OutDmgPRatP",L,N)
									end
								else
									Result = CalcStat("OutDmgPBonus",L)
								end
							elseif SN < "STOUTWRBLACKLDISEASERESISTP" then
								if SN > "STOUTUNYIELDINGPHYMITP" then
									if SN == "STOUTUNYIELDINGWILL" then
										Result = CalcStat("WillT",L,1.0)
									elseif SN == "STOUTWRBLACKLAGILITY" then
										Result = CalcStat("AgilityT",L,1.0)
									end
								elseif SN == "STOUTSHADOWEYEVITALITY" then
									Result = -CalcStat("VitalityT",L,0.4)
								elseif SN == "STOUTUNYIELDINGPHYMITP" then
									Result = 1.0
								end
							else
								Result = 1.0
							end
						else
							Result = -CalcStat("FateT",L,0.4)
						end
					else
						Result = CalcStat("OutDmgPRatPA",L)
					end
				elseif SN > "TACMITMPBONUS" then
					if SN > "WARDENCDBASEMORALE" then
						if SN < "WARLEADERCANBLOCK" then
							if SN > "WARDENCDBASEWILL" then
								if SN < "WARDENCDCALCTYPETACMIT" then
									if SN == "WARDENCDCALCTYPECOMPHYMIT" then
										Result = 13
									elseif SN == "WARDENCDCALCTYPENONPHYMIT" then
										Result = 13
									end
								elseif SN > "WARDENCDCALCTYPETACMIT" then
									if SN == "WARDENCDCANBLOCK" then
										Result = 1
									elseif SN == "WARDENCDHASPOWER" then
										Result = 1
									end
								else
									Result = 26
								end
							elseif SN < "WARDENCDBASEWILL" then
								if SN > "WARDENCDBASENCPR" then
									if SN == "WARDENCDBASEPOWER" then
										Result = CalcStat("ClassBasePower",L)
									elseif SN == "WARDENCDBASEVITALITY" then
										Result = CalcStat("ClassBaseVitality",L)
									end
								elseif SN == "WARDENCDBASENCMR" then
									Result = CalcStat("ClassBaseNCMRH",L)
								elseif SN == "WARDENCDBASENCPR" then
									Result = CalcStat("ClassBaseNCPR",L)
								end
							else
								Result = CalcStat("ClassBaseWillL",L)
							end
						elseif SN > "WARLEADERCANBLOCK" then
							if SN > "WEAVERCANBLOCK" then
								if SN < "WEAVERCDCALCTYPETACMIT" then
									if SN == "WEAVERCDCALCTYPECOMPHYMIT" then
										Result = 13
									elseif SN == "WEAVERCDCALCTYPENONPHYMIT" then
										Result = 14
									end
								elseif SN > "WEAVERCDCALCTYPETACMIT" then
									if SN == "WEAVERCDHASPOWER" then
										Result = 1
									elseif SN == "WILLT" then
										Result = CalcStat("MainT",L,N)
									end
								else
									Result = 27
								end
							elseif SN < "WEAVERCANBLOCK" then
								if SN > "WARLEADERCDCALCTYPENONPHYMIT" then
									if SN == "WARLEADERCDCALCTYPETACMIT" then
										Result = 27
									elseif SN == "WARLEADERCDHASPOWER" then
										Result = 1
									end
								elseif SN == "WARLEADERCDCALCTYPECOMPHYMIT" then
									Result = 14
								elseif SN == "WARLEADERCDCALCTYPENONPHYMIT" then
									Result = 14
								end
							else
								Result = 1
							end
						else
							Result = 1
						end
					elseif SN < "WARDENCDBASEMORALE" then
						if SN < "TPENCHOICE" then
							if SN > "TACMITMPRATPC" then
								if SN > "TACMITMPRATPCAPR" then
									if SN == "TPENARMOUR" then
										Result = -CalcStat("ArmourPenT",L,CalcStat("TpenChoice",N))
									elseif SN == "TPENBPE" then
										Result = -CalcStat("BPET",L,CalcStat("TpenChoice",N))
									end
								elseif SN == "TACMITMPRATPCAP" then
									Result = CalcStat("MitMediumPRatPCap",L)
								elseif SN == "TACMITMPRATPCAPR" then
									Result = CalcStat("MitMediumPRatPCapR",L)
								end
							elseif SN < "TACMITMPRATPC" then
								if SN > "TACMITMPRATP" then
									if SN == "TACMITMPRATPA" then
										Result = CalcStat("MitMediumPRatPA",L)
									elseif SN == "TACMITMPRATPB" then
										Result = CalcStat("MitMediumPRatPB",L)
									end
								elseif SN == "TACMITMPPRAT" then
									Result = CalcStat("MitMediumPPRat",L,N)
								elseif SN == "TACMITMPRATP" then
									Result = CalcStat("MitMediumPRatP",L,N)
								end
							else
								Result = CalcStat("MitMediumPRatPC",L)
							end
						elseif SN > "TPENCHOICE" then
							if SN > "WARDENCDARMOURTYPE" then
								if SN < "WARDENCDBASEICMR" then
									if SN == "WARDENCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityH",L)
									elseif SN == "WARDENCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									end
								elseif SN > "WARDENCDBASEICMR" then
									if SN == "WARDENCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									elseif SN == "WARDENCDBASEMIGHT" then
										Result = CalcStat("ClassBaseMightM",L)
									end
								else
									Result = CalcStat("ClassBaseICMRH",L)
								end
							elseif SN < "WARDENCDARMOURTYPE" then
								if SN > "TRAITPNTS" then
									if SN == "TRAITPNTSVITAL" then
										Result = {{1,25,50,60,65,75,85,95,100,105,115,120,130,140,141,150,151,160,161,170},{1,25,50,60,65,75,85,95,100,105,115,120,130,140,141,150,151,160,161,170}}
									elseif SN == "VITALITYT" then
										Result = RoundDblDown(StatLinInter("PntMPVitalityT","TraitPntSVital","ProgBHealth","AdjTraitHealth",L,N,2),0)
									end
								elseif SN == "TPENRESIST" then
									Result = -CalcStat("ResistT",L,CalcStat("TpenChoice",N)*2.0)
								elseif SN == "TRAITPNTS" then
									Result = {{1,25,50,60,65,75,85,95,100,105,115,120,130,131,140,141,150,151,160},{1,25,50,60,65,75,85,95,100,105,115,120,130,131,140,141,150,151,160}}
								end
							else
								Result = 2
							end
						else
							if 1 <= L then
								Result = DataTableValue({0.5,1.0,2.0},L)
							end
						end
					else
						Result = CalcStat("ClassBaseMorale",L)
					end
				else
					Result = CalcStat("MitMediumPBonus",L)
				end
			elseif SN < "STDPROGRATINGS" then
				if SN < "RESISTPRATPA" then
					if SN > "PNTMPICPR" then
						if SN < "PROGBHEALTH" then
							if SN > "PNTMPPOWERT" then
								if SN > "PNTMPVITALITYT" then
									if SN == "POWERT" then
										Result = EquSng(StatLinInter("PntMPPowerT","TraitPntSVital","ProgBEnergy","",L,N,2))
									elseif SN == "PROGBENERGY" then
										Result = CalcStat("StdProgEnergy",L,2.0)
									end
								elseif SN == "PNTMPRESIST" then
									Result = 0.03
								elseif SN == "PNTMPVITALITYT" then
									Result = 0.45
								end
							elseif SN < "PNTMPPOWERT" then
								if SN > "PNTMPMORALE" then
									if SN == "PNTMPNCMR" then
										Result = 0.3
									elseif SN == "PNTMPNCPR" then
										Result = 1.0
									end
								elseif SN == "PNTMPMAIN" then
									Result = 0.5
								elseif SN == "PNTMPMORALE" then
									Result = 2.0
								end
							else
								Result = 1.333
							end
						elseif SN > "PROGBHEALTH" then
							if SN > "REAVERCDCALCTYPECOMPHYMIT" then
								if SN < "REAVERCDHASPOWER" then
									if SN == "REAVERCDCALCTYPENONPHYMIT" then
										Result = 14
									elseif SN == "REAVERCDCALCTYPETACMIT" then
										Result = 27
									end
								elseif SN > "REAVERCDHASPOWER" then
									if SN == "RESISTPPRAT" then
										Result = CalcRatAB(CalcStat("ResistPRatPA",L),CalcStat("ResistPRatPB",L),CalcStat("ResistPRatPCapR",L),N)
									elseif SN == "RESISTPRATP" then
										Result = CalcPercAB(CalcStat("ResistPRatPA",L),CalcStat("ResistPRatPB",L),CalcStat("ResistPRatPCap",L),N)
									end
								else
									Result = 1
								end
							elseif SN < "REAVERCDCALCTYPECOMPHYMIT" then
								if SN > "PROGBMAINOLD" then
									if SN == "RACENAME" then
										Result = TranslateValue({6,7,12,23,27,39,65,66,73,81,114,117,120,125},{"Uruk","Orc","Spider","Man","Critter","Angmarim","Elf","Warg","Dwarf","Hobbit","Beorning","HighElf","StoutAxe","RiverHobbit",""},L)
									elseif SN == "REAVERCANBLOCK" then
										Result = 1
									end
								elseif SN == "PROGBMAIN" then
									Result = CalcStat("StdProgRatings",L,1.75)
								elseif SN == "PROGBMAINOLD" then
									Result = CalcStat("StdProgRatingsOld",L,1.75)
								end
							else
								Result = 13
							end
						else
							Result = CalcStat("StdProgHealth",L,4.0)
						end
					elseif SN < "PNTMPICPR" then
						if SN < "PHYMITMPRATPA" then
							if SN > "PHYMITLPRATPCAP" then
								if SN > "PHYMITMPBONUS" then
									if SN == "PHYMITMPPRAT" then
										Result = CalcStat("MitMediumPPRat",L,N)
									elseif SN == "PHYMITMPRATP" then
										Result = CalcStat("MitMediumPRatP",L,N)
									end
								elseif SN == "PHYMITLPRATPCAPR" then
									Result = CalcStat("MitLightPRatPCapR",L)
								elseif SN == "PHYMITMPBONUS" then
									Result = CalcStat("MitMediumPBonus",L)
								end
							elseif SN < "PHYMITLPRATPCAP" then
								if SN > "PHYMITLPRATPA" then
									if SN == "PHYMITLPRATPB" then
										Result = CalcStat("MitLightPRatPB",L)
									elseif SN == "PHYMITLPRATPC" then
										Result = CalcStat("MitLightPRatPC",L)
									end
								elseif SN == "PHYMITLPRATP" then
									Result = CalcStat("MitLightPRatP",L,N)
								elseif SN == "PHYMITLPRATPA" then
									Result = CalcStat("MitLightPRatPA",L)
								end
							else
								Result = CalcStat("MitLightPRatPCap",L)
							end
						elseif SN > "PHYMITMPRATPA" then
							if SN > "PNTMPARMOURPENT" then
								if SN < "PNTMPCLASSBASENCPR" then
									if SN == "PNTMPBPE" then
										Result = 0.035
									elseif SN == "PNTMPCLASSBASEICPR" then
										Result = 0.15
									end
								elseif SN > "PNTMPCLASSBASENCPR" then
									if SN == "PNTMPFATE" then
										Result = 2.5
									elseif SN == "PNTMPICMR" then
										Result = 0.03
									end
								else
									Result = 0.5
								end
							elseif SN < "PNTMPARMOURPENT" then
								if SN > "PHYMITMPRATPC" then
									if SN == "PHYMITMPRATPCAP" then
										Result = CalcStat("MitMediumPRatPCap",L)
									elseif SN == "PHYMITMPRATPCAPR" then
										Result = CalcStat("MitMediumPRatPCapR",L)
									end
								elseif SN == "PHYMITMPRATPB" then
									Result = CalcStat("MitMediumPRatPB",L)
								elseif SN == "PHYMITMPRATPC" then
									Result = CalcStat("MitMediumPRatPC",L)
								end
							else
								Result = 0.06
							end
						else
							Result = CalcStat("MitMediumPRatPA",L)
						end
					else
						Result = 0.125
					end
				elseif SN > "RESISTPRATPA" then
					if SN > "RUNEKEEPERCDBASENCMR" then
						if SN < "SORCERESSCDCALCTYPENONPHYMIT" then
							if SN > "RUNEKEEPERCDCALCTYPECOMPHYMIT" then
								if SN < "RUNEKEEPERCDHASPOWER" then
									if SN == "RUNEKEEPERCDCALCTYPENONPHYMIT" then
										Result = 12
									elseif SN == "RUNEKEEPERCDCALCTYPETACMIT" then
										Result = 25
									end
								elseif SN > "RUNEKEEPERCDHASPOWER" then
									if SN == "SORCERESSCANBLOCK" then
										Result = 1
									elseif SN == "SORCERESSCDCALCTYPECOMPHYMIT" then
										Result = 13
									end
								else
									Result = 1
								end
							elseif SN < "RUNEKEEPERCDCALCTYPECOMPHYMIT" then
								if SN > "RUNEKEEPERCDBASEPOWER" then
									if SN == "RUNEKEEPERCDBASEVITALITY" then
										Result = CalcStat("ClassBaseVitality",L)
									elseif SN == "RUNEKEEPERCDBASEWILL" then
										Result = CalcStat("ClassBaseWillH",L)
									end
								elseif SN == "RUNEKEEPERCDBASENCPR" then
									Result = CalcStat("ClassBaseNCPR",L)
								elseif SN == "RUNEKEEPERCDBASEPOWER" then
									Result = CalcStat("ClassBasePower",L)
								end
							else
								Result = 12
							end
						elseif SN > "SORCERESSCDCALCTYPENONPHYMIT" then
							if SN > "STALKERCDCALCTYPENONPHYMIT" then
								if SN < "STDPNTS" then
									if SN == "STALKERCDCALCTYPETACMIT" then
										Result = 27
									elseif SN == "STALKERCDHASPOWER" then
										Result = 1
									end
								elseif SN > "STDPNTS" then
									if SN == "STDPROGENERGY" then
										if L <= 0 then
											Result = 0.0
										elseif L <= 50 then
											Result = LinFmod(N,1.0,2.0,1,50,L,1)
										elseif L <= 60 then
											Result = LinFmod(CalcStat("StdProgEnergy",50,N),1.0,1.33,50,60,L,1)
										elseif L <= 65 then
											Result = LinFmod(CalcStat("StdProgEnergy",60,N),1.0,1.25,60,65,L,1)
										elseif L <= 75 then
											Result = LinFmod(CalcStat("StdProgEnergy",65,N),1.0,1.5,65,75,L,1)
										elseif L <= 85 then
											Result = LinFmod(CalcStat("StdProgEnergy",75,N),1.0,1.5,75,85,L,1)
										elseif L <= 95 then
											Result = LinFmod(CalcStat("StdProgEnergy",85,N),1.0,1.33,85,95,L,1)
										elseif L <= 100 then
											Result = LinFmod(CalcStat("StdProgEnergy",95,N),1.0,1.315,95,100,L,1)
										elseif L <= 105 then
											Result = LinFmod(CalcStat("StdProgEnergy",100,N),1.0,1.333,100,105,L,1)
										elseif L <= 115 then
											Result = LinFmod(CalcStat("StdProgEnergy",105,N),1.1,1.5,106,115,L,1)
										elseif L <= 120 then
											Result = LinFmod(CalcStat("StdProgEnergy",115,N),1.15,1.25,116,120,L,1)
										elseif L <= 130 then
											Result = LinFmod(CalcStat("StdProgEnergy",120,N),1.15,1.5,121,130,L,1)
										elseif L <= 140 then
											Result = LinFmod(CalcStat("StdProgEnergy",130,N),1.15,2.0,131,140,L,1)
										elseif L <= 150 then
											Result = LinFmod(CalcStat("StdProgEnergy",140,N),1.15,2.0,141,150,L,1)
										elseif L <= 160 then
											Result = LinFmod(CalcStat("StdProgEnergy",150,N),1.15,2.0,151,160,L,1)
										else
											Result = LinFmod(CalcStat("StdProgEnergy",160,N),1.15,2.0,161,170,L,1)
										end
									elseif SN == "STDPROGHEALTH" then
										if L <= 0 then
											Result = 0.0
										elseif L <= 50 then
											Result = LinFmod(N,1.0,7.5,1,50,L,1)
										elseif L <= 60 then
											Result = LinFmod(CalcStat("StdProgHealth",50,N),1.0,1.33,50,60,L,1)
										elseif L <= 65 then
											Result = LinFmod(CalcStat("StdProgHealth",60,N),1.0,1.25,60,65,L,1)
										elseif L <= 75 then
											Result = LinFmod(CalcStat("StdProgHealth",65,N),1.0,1.5,65,75,L,1)
										elseif L <= 85 then
											Result = LinFmod(CalcStat("StdProgHealth",75,N),1.0,1.5,75,85,L,1)
										elseif L <= 95 then
											Result = LinFmod(CalcStat("StdProgHealth",85,N),1.0,1.33,85,95,L,1)
										elseif L <= 100 then
											Result = LinFmod(CalcStat("StdProgHealth",95,N),1.0,1.5,95,100,L,1)
										elseif L <= 105 then
											Result = LinFmod(CalcStat("StdProgHealth",100,N),1.0,1.333,100,105,L,1)
										elseif L <= 115 then
											Result = LinFmod(CalcStat("StdProgHealth",105,N),1.1,1.5,106,115,L,1)
										elseif L <= 120 then
											Result = LinFmod(CalcStat("StdProgHealth",115,N),1.15,1.25,116,120,L,1)
										elseif L <= 130 then
											Result = LinFmod(CalcStat("StdProgHealth",120,N),1.15,1.5,121,130,L,1)
										elseif L <= 140 then
											Result = LinFmod(CalcStat("StdProgHealth",130,N),1.15,2.0,131,140,L,1)
										elseif L <= 150 then
											Result = LinFmod(CalcStat("StdProgHealth",140,N),1.15,2.0,141,150,L,1)
										elseif L <= 160 then
											Result = LinFmod(CalcStat("StdProgHealth",150,N),1.15,2.0,151,160,L,1)
										else
											Result = LinFmod(CalcStat("StdProgHealth",160,N),1.15,2.0,161,170,L,1)
										end
									end
								else
									Result = {{1,50,60,65,75,85,95,100,105,106,115,116,120,121,130,131,140,141,150,151,160,161,170},{1,50,60,65,75,85,95,100,105,106,115,116,120,121,130,131,140,141,150,151,160,161,170}}
								end
							elseif SN < "STALKERCDCALCTYPENONPHYMIT" then
								if SN > "SORCERESSCDHASPOWER" then
									if SN == "STALKERCANBLOCK" then
										Result = 1
									elseif SN == "STALKERCDCALCTYPECOMPHYMIT" then
										Result = 13
									end
								elseif SN == "SORCERESSCDCALCTYPETACMIT" then
									Result = 27
								elseif SN == "SORCERESSCDHASPOWER" then
									Result = 1
								end
							else
								Result = 14
							end
						else
							Result = 14
						end
					elseif SN < "RUNEKEEPERCDBASENCMR" then
						if SN < "RIVHOBHARDYHOLBMORALE" then
							if SN > "RESISTT" then
								if SN > "RIVERHOBBITRDTRAITFROSTMITP" then
									if SN == "RIVERHOBBITRDTRAITMORALE" then
										Result = CalcStat("RivHobHardyHolbMorale",L)
									elseif SN == "RIVERHOBBITRDTRAITWILL" then
										Result = CalcStat("RivHobSeclusionWill",L)
									end
								elseif SN == "RIVERHOBBITRDTRAITAGILITY" then
									Result = CalcStat("RivHobSlipperyAgility",L)
								elseif SN == "RIVERHOBBITRDTRAITFROSTMITP" then
									Result = CalcStat("RivHobSwimmerFrostMitP",L)
								end
							elseif SN < "RESISTT" then
								if SN > "RESISTPRATPC" then
									if SN == "RESISTPRATPCAP" then
										Result = 50.0
									elseif SN == "RESISTPRATPCAPR" then
										Result = CalcStat("ResistPRatPB",L)*CalcStat("ResistPRatPC",L)
									end
								elseif SN == "RESISTPRATPB" then
									Result = CalcStat("BRatExtra",L)
								elseif SN == "RESISTPRATPC" then
									Result = 0.5
								end
							else
								Result = EquSng(StatLinInter("PntMPResist","TraitPntS","ResistPRatPB","AdjTraitRat",L,N,2))
							end
						elseif SN > "RIVHOBHARDYHOLBMORALE" then
							if SN > "RUNEKEEPERCDBASEAGILITY" then
								if SN < "RUNEKEEPERCDBASEICPR" then
									if SN == "RUNEKEEPERCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									elseif SN == "RUNEKEEPERCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRL",L)
									end
								elseif SN > "RUNEKEEPERCDBASEICPR" then
									if SN == "RUNEKEEPERCDBASEMIGHT" then
										Result = CalcStat("ClassBaseMightM",L)
									elseif SN == "RUNEKEEPERCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									end
								else
									Result = CalcStat("ClassBaseICPR",L)
								end
							elseif SN < "RUNEKEEPERCDBASEAGILITY" then
								if SN > "RIVHOBSLIPPERYAGILITY" then
									if SN == "RIVHOBSWIMMERFROSTMITP" then
										Result = 1.0
									elseif SN == "RUNEKEEPERCDARMOURTYPE" then
										Result = 1
									end
								elseif SN == "RIVHOBSECLUSIONWILL" then
									Result = -CalcStat("WillT",L,0.4)
								elseif SN == "RIVHOBSLIPPERYAGILITY" then
									Result = CalcStat("AgilityT",L,1.0)
								end
							else
								Result = CalcStat("ClassBaseAgilityL",L)
							end
						else
							Result = CalcStat("MoraleT",L,1.0)
						end
					else
						Result = CalcStat("ClassBaseNCMRL",L)
					end
				else
					Result = 150.0
				end
			else
				if L <= 0 then
					Result = 0.0
				elseif L <= 50 then
					Result = LinFmod(N,1.0,10.0,1,50,L,1)
				elseif L <= 60 then
					Result = LinFmod(CalcStat("StdProgRatings",50,N),1.0,1.5,50,60,L,1)
				elseif L <= 65 then
					Result = LinFmod(CalcStat("StdProgRatings",60,N),1.0,1.333,60,65,L,1)
				elseif L <= 75 then
					Result = LinFmod(CalcStat("StdProgRatings",65,N),1.0,1.5,65,75,L,1)
				elseif L <= 85 then
					Result = LinFmod(CalcStat("StdProgRatings",75,N),1.0,1.5,75,85,L,1)
				elseif L <= 95 then
					Result = LinFmod(CalcStat("StdProgRatings",85,N),1.0,1.445,85,95,L,1)
				elseif L <= 100 then
					Result = LinFmod(CalcStat("StdProgRatings",95,N),1.0,1.39,95,100,L,1)
				elseif L <= 105 then
					Result = LinFmod(CalcStat("StdProgRatings",100,N),1.0,1.33,100,105,L,1)
				elseif L <= 115 then
					Result = LinFmod(CalcStat("StdProgRatings",105,N),1.1,1.5,106,115,L,1)
				elseif L <= 120 then
					Result = LinFmod(CalcStat("StdProgRatings",115,N),1.15,1.25,116,120,L,1)
				elseif L <= 130 then
					Result = LinFmod(CalcStat("StdProgRatings",120,N),1.15,1.5,121,130,L,1)
				elseif L <= 140 then
					Result = LinFmod(CalcStat("StdProgRatings",130,N),1.15,2.0,131,140,L,1)
				elseif L <= 150 then
					Result = LinFmod(CalcStat("StdProgRatings",140,N),1.3,2.205,141,150,L,1)
				elseif L <= 160 then
					Result = LinFmod(CalcStat("StdProgRatings",150,N),1.3,2.0,151,160,L,1)
				else
					Result = LinFmod(CalcStat("StdProgRatings",160,N),1.0,2.0,161,170,L,1)
				end
			end
		elseif SN < "PHYMITLPPRAT" then
			if SN > "PARTBLOCKPPRAT" then
				if SN < "PARTFINESSEPRATPCAP" then
					if SN > "PARTEVADEMITPRATPCAPR" then
						if SN < "PARTFINESSEDMGPRATP" then
							if SN > "PARTEVADEPRATPB" then
								if SN > "PARTEVADEPRATPCAP" then
									if SN == "PARTEVADEPRATPCAPR" then
										Result = CalcStat("PartBPEPRatPCapR",L)
									elseif SN == "PARTFINESSEDMGPPRAT" then
										Result = CalcRatAB(CalcStat("PartFinesseDmgPRatPA",L),CalcStat("PartFinesseDmgPRatPB",L),CalcStat("PartFinesseDmgPRatPCapR",L),N)
									end
								elseif SN == "PARTEVADEPRATPC" then
									Result = CalcStat("PartBPEPRatPC",L)
								elseif SN == "PARTEVADEPRATPCAP" then
									Result = CalcStat("PartBPEPRatPCap",L)
								end
							elseif SN < "PARTEVADEPRATPB" then
								if SN > "PARTEVADEPPRAT" then
									if SN == "PARTEVADEPRATP" then
										Result = CalcStat("PartBPEPRatP",L,N)
									elseif SN == "PARTEVADEPRATPA" then
										Result = CalcStat("PartBPEPRatPA",L)
									end
								elseif SN == "PARTEVADEPBONUS" then
									Result = CalcStat("PartBPEPBonus",L)
								elseif SN == "PARTEVADEPPRAT" then
									Result = CalcStat("PartBPEPPRat",L,N)
								end
							else
								Result = CalcStat("PartBPEPRatPB",L)
							end
						elseif SN > "PARTFINESSEDMGPRATP" then
							if SN > "PARTFINESSEDMGPRATPCAPR" then
								if SN < "PARTFINESSEPRATPA" then
									if SN == "PARTFINESSEPPRAT" then
										Result = CalcRatAB(CalcStat("PartFinessePRatPA",L),CalcStat("PartFinessePRatPB",L),CalcStat("PartFinessePRatPCapR",L),N)
									elseif SN == "PARTFINESSEPRATP" then
										Result = CalcPercAB(CalcStat("PartFinessePRatPA",L),CalcStat("PartFinessePRatPB",L),CalcStat("PartFinessePRatPCap",L),N)
									end
								elseif SN > "PARTFINESSEPRATPA" then
									if SN == "PARTFINESSEPRATPB" then
										Result = CalcStat("BRatStandard",L)
									elseif SN == "PARTFINESSEPRATPC" then
										Result = 0.5
									end
								else
									Result = 150.0
								end
							elseif SN < "PARTFINESSEDMGPRATPCAPR" then
								if SN > "PARTFINESSEDMGPRATPB" then
									if SN == "PARTFINESSEDMGPRATPC" then
										Result = 0.5
									elseif SN == "PARTFINESSEDMGPRATPCAP" then
										Result = 50.0
									end
								elseif SN == "PARTFINESSEDMGPRATPA" then
									Result = 150.0
								elseif SN == "PARTFINESSEDMGPRATPB" then
									Result = CalcStat("BRatStandard",L)
								end
							else
								Result = CalcStat("PartFinesseDmgPRatPB",L)*CalcStat("PartFinesseDmgPRatPC",L)
							end
						else
							Result = CalcPercAB(CalcStat("PartFinesseDmgPRatPA",L),CalcStat("PartFinesseDmgPRatPB",L),CalcStat("PartFinesseDmgPRatPCap",L),N)
						end
					elseif SN < "PARTEVADEMITPRATPCAPR" then
						if SN < "PARTBPEPRATPB" then
							if SN > "PARTBLOCKPRATPCAP" then
								if SN > "PARTBPEPPRAT" then
									if SN == "PARTBPEPRATP" then
										Result = CalcPercAB(CalcStat("PartBPEPRatPA",L),CalcStat("PartBPEPRatPB",L),CalcStat("PartBPEPRatPCap",L),N)
									elseif SN == "PARTBPEPRATPA" then
										Result = 75.0
									end
								elseif SN == "PARTBLOCKPRATPCAPR" then
									Result = CalcStat("PartBPEPRatPCapR",L)
								elseif SN == "PARTBPEPPRAT" then
									Result = CalcRatAB(CalcStat("PartBPEPRatPA",L),CalcStat("PartBPEPRatPB",L),CalcStat("PartBPEPRatPCapR",L),N)
								end
							elseif SN < "PARTBLOCKPRATPCAP" then
								if SN > "PARTBLOCKPRATPA" then
									if SN == "PARTBLOCKPRATPB" then
										Result = CalcStat("PartBPEPRatPB",L)
									elseif SN == "PARTBLOCKPRATPC" then
										Result = CalcStat("PartBPEPRatPC",L)
									end
								elseif SN == "PARTBLOCKPRATP" then
									Result = CalcStat("PartBPEPRatP",L,N)
								elseif SN == "PARTBLOCKPRATPA" then
									Result = CalcStat("PartBPEPRatPA",L)
								end
							else
								Result = CalcStat("PartBPEPRatPCap",L)
							end
						elseif SN > "PARTBPEPRATPB" then
							if SN > "PARTEVADEMITPPRAT" then
								if SN < "PARTEVADEMITPRATPB" then
									if SN == "PARTEVADEMITPRATP" then
										Result = CalcStat("PartMitPRatP",L,N)
									elseif SN == "PARTEVADEMITPRATPA" then
										Result = CalcStat("PartMitPRatPA",L)
									end
								elseif SN > "PARTEVADEMITPRATPB" then
									if SN == "PARTEVADEMITPRATPC" then
										Result = CalcStat("PartMitPRatPC",L)
									elseif SN == "PARTEVADEMITPRATPCAP" then
										Result = CalcStat("PartMitPRatPCap",L)
									end
								else
									Result = CalcStat("PartMitPRatPB",L)
								end
							elseif SN < "PARTEVADEMITPPRAT" then
								if SN > "PARTBPEPRATPCAP" then
									if SN == "PARTBPEPRATPCAPR" then
										Result = CalcStat("PartBPEPRatPB",L)*CalcStat("PartBPEPRatPC",L)
									elseif SN == "PARTEVADEMITPBONUS" then
										Result = CalcStat("PartMitPBonus",L)
									end
								elseif SN == "PARTBPEPRATPC" then
									Result = 0.5
								elseif SN == "PARTBPEPRATPCAP" then
									Result = 25.0
								end
							else
								Result = CalcStat("PartMitPPRat",L,N)
							end
						else
							Result = CalcStat("BRatPartBPE",L)
						end
					else
						Result = CalcStat("PartMitPRatPCapR",L)
					end
				elseif SN > "PARTFINESSEPRATPCAP" then
					if SN > "PARTPARRYPRATPA" then
						if SN < "PHYDMGPRATPCAP" then
							if SN > "PHYDMGPBONUS" then
								if SN < "PHYDMGPRATPA" then
									if SN == "PHYDMGPPRAT" then
										Result = CalcStat("OutDmgPPRat",L,N)
									elseif SN == "PHYDMGPRATP" then
										Result = CalcStat("OutDmgPRatP",L,N)
									end
								elseif SN > "PHYDMGPRATPA" then
									if SN == "PHYDMGPRATPB" then
										Result = CalcStat("OutDmgPRatPB",L)
									elseif SN == "PHYDMGPRATPC" then
										Result = CalcStat("OutDmgPRatPC",L)
									end
								else
									Result = CalcStat("OutDmgPRatPA",L)
								end
							elseif SN < "PHYDMGPBONUS" then
								if SN > "PARTPARRYPRATPC" then
									if SN == "PARTPARRYPRATPCAP" then
										Result = CalcStat("PartBPEPRatPCap",L)
									elseif SN == "PARTPARRYPRATPCAPR" then
										Result = CalcStat("PartBPEPRatPCapR",L)
									end
								elseif SN == "PARTPARRYPRATPB" then
									Result = CalcStat("PartBPEPRatPB",L)
								elseif SN == "PARTPARRYPRATPC" then
									Result = CalcStat("PartBPEPRatPC",L)
								end
							else
								Result = CalcStat("OutDmgPBonus",L)
							end
						elseif SN > "PHYDMGPRATPCAP" then
							if SN > "PHYMITHPRATPA" then
								if SN < "PHYMITHPRATPCAP" then
									if SN == "PHYMITHPRATPB" then
										Result = CalcStat("MitHeavyPRatPB",L)
									elseif SN == "PHYMITHPRATPC" then
										Result = CalcStat("MitHeavyPRatPC",L)
									end
								elseif SN > "PHYMITHPRATPCAP" then
									if SN == "PHYMITHPRATPCAPR" then
										Result = CalcStat("MitHeavyPRatPCapR",L)
									elseif SN == "PHYMITLPBONUS" then
										Result = CalcStat("MitLightPBonus",L)
									end
								else
									Result = CalcStat("MitHeavyPRatPCap",L)
								end
							elseif SN < "PHYMITHPRATPA" then
								if SN > "PHYMITHPBONUS" then
									if SN == "PHYMITHPPRAT" then
										Result = CalcStat("MitHeavyPPRat",L,N)
									elseif SN == "PHYMITHPRATP" then
										Result = CalcStat("MitHeavyPRatP",L,N)
									end
								elseif SN == "PHYDMGPRATPCAPR" then
									Result = CalcStat("OutDmgPRatPCapR",L)
								elseif SN == "PHYMITHPBONUS" then
									Result = CalcStat("MitHeavyPBonus",L)
								end
							else
								Result = CalcStat("MitHeavyPRatPA",L)
							end
						else
							Result = CalcStat("OutDmgPRatPCap",L)
						end
					elseif SN < "PARTPARRYPRATPA" then
						if SN < "PARTPARRYMITPBONUS" then
							if SN > "PARTMITPRATPA" then
								if SN > "PARTMITPRATPC" then
									if SN == "PARTMITPRATPCAP" then
										Result = 35.0
									elseif SN == "PARTMITPRATPCAPR" then
										Result = CalcStat("PartMitPRatPB",L)*CalcStat("PartMitPRatPC",L)
									end
								elseif SN == "PARTMITPRATPB" then
									Result = CalcStat("BRatPartBPE",L)
								elseif SN == "PARTMITPRATPC" then
									Result = 0.5
								end
							elseif SN < "PARTMITPRATPA" then
								if SN > "PARTMITPBONUS" then
									if SN == "PARTMITPPRAT" then
										Result = CalcRatAB(CalcStat("PartMitPRatPA",L),CalcStat("PartMitPRatPB",L),CalcStat("PartMitPRatPCapR",L),N)
									elseif SN == "PARTMITPRATP" then
										Result = CalcPercAB(CalcStat("PartMitPRatPA",L),CalcStat("PartMitPRatPB",L),CalcStat("PartMitPRatPCap",L),N)
									end
								elseif SN == "PARTFINESSEPRATPCAPR" then
									Result = CalcStat("PartFinessePRatPB",L)*CalcStat("PartFinessePRatPC",L)
								elseif SN == "PARTMITPBONUS" then
									Result = 0.1
								end
							else
								Result = 105.0
							end
						elseif SN > "PARTPARRYMITPBONUS" then
							if SN > "PARTPARRYMITPRATPC" then
								if SN < "PARTPARRYPBONUS" then
									if SN == "PARTPARRYMITPRATPCAP" then
										Result = CalcStat("PartMitPRatPCap",L)
									elseif SN == "PARTPARRYMITPRATPCAPR" then
										Result = CalcStat("PartMitPRatPCapR",L)
									end
								elseif SN > "PARTPARRYPBONUS" then
									if SN == "PARTPARRYPPRAT" then
										Result = CalcStat("PartBPEPPRat",L,N)
									elseif SN == "PARTPARRYPRATP" then
										Result = CalcStat("PartBPEPRatP",L,N)
									end
								else
									Result = CalcStat("PartBPEPBonus",L)
								end
							elseif SN < "PARTPARRYMITPRATPC" then
								if SN > "PARTPARRYMITPRATP" then
									if SN == "PARTPARRYMITPRATPA" then
										Result = CalcStat("PartMitPRatPA",L)
									elseif SN == "PARTPARRYMITPRATPB" then
										Result = CalcStat("PartMitPRatPB",L)
									end
								elseif SN == "PARTPARRYMITPPRAT" then
									Result = CalcStat("PartMitPPRat",L,N)
								elseif SN == "PARTPARRYMITPRATP" then
									Result = CalcStat("PartMitPRatP",L,N)
								end
							else
								Result = CalcStat("PartMitPRatPC",L)
							end
						else
							Result = CalcStat("PartMitPBonus",L)
						end
					else
						Result = CalcStat("PartBPEPRatPA",L)
					end
				else
					Result = 50.0
				end
			elseif SN < "PARTBLOCKPPRAT" then
				if SN < "MITLIGHTPRATPCAP" then
					if SN > "MINSTRELCDBASENCPR" then
						if SN < "MITHEAVYPRATP" then
							if SN > "MINSTRELCDCALCTYPENONPHYMIT" then
								if SN > "MINSTRELCDCANBLOCK" then
									if SN == "MINSTRELCDHASPOWER" then
										Result = 1
									elseif SN == "MITHEAVYPPRAT" then
										Result = CalcRatAB(CalcStat("MitHeavyPRatPA",L),CalcStat("MitHeavyPRatPB",L),CalcStat("MitHeavyPRatPCapR",L),N)
									end
								elseif SN == "MINSTRELCDCALCTYPETACMIT" then
									Result = 25
								elseif SN == "MINSTRELCDCANBLOCK" then
									if 20 <= L then
										Result = 1
									end
								end
							elseif SN < "MINSTRELCDCALCTYPENONPHYMIT" then
								if SN > "MINSTRELCDBASEVITALITY" then
									if SN == "MINSTRELCDBASEWILL" then
										Result = CalcStat("ClassBaseWillH",L)
									elseif SN == "MINSTRELCDCALCTYPECOMPHYMIT" then
										Result = 12
									end
								elseif SN == "MINSTRELCDBASEPOWER" then
									Result = CalcStat("ClassBasePower",L)
								elseif SN == "MINSTRELCDBASEVITALITY" then
									Result = CalcStat("ClassBaseVitality",L)
								end
							else
								Result = 12
							end
						elseif SN > "MITHEAVYPRATP" then
							if SN > "MITHEAVYPRATPCAPR" then
								if SN < "MITLIGHTPRATPA" then
									if SN == "MITLIGHTPPRAT" then
										Result = CalcRatAB(CalcStat("MitLightPRatPA",L),CalcStat("MitLightPRatPB",L),CalcStat("MitLightPRatPCapR",L),N)
									elseif SN == "MITLIGHTPRATP" then
										Result = CalcPercAB(CalcStat("MitLightPRatPA",L),CalcStat("MitLightPRatPB",L),CalcStat("MitLightPRatPCap",L),N)
									end
								elseif SN > "MITLIGHTPRATPA" then
									if SN == "MITLIGHTPRATPB" then
										Result = CalcStat("BRatMitLight",L)
									elseif SN == "MITLIGHTPRATPC" then
										Result = 0.5
									end
								else
									Result = 120.0
								end
							elseif SN < "MITHEAVYPRATPCAPR" then
								if SN > "MITHEAVYPRATPB" then
									if SN == "MITHEAVYPRATPC" then
										Result = 0.5
									elseif SN == "MITHEAVYPRATPCAP" then
										Result = 60.0
									end
								elseif SN == "MITHEAVYPRATPA" then
									Result = 180.0
								elseif SN == "MITHEAVYPRATPB" then
									Result = CalcStat("BRatMitHeavy",L)
								end
							else
								Result = CalcStat("MitHeavyPRatPB",L)*CalcStat("MitHeavyPRatPC",L)
							end
						else
							Result = CalcPercAB(CalcStat("MitHeavyPRatPA",L),CalcStat("MitHeavyPRatPB",L),CalcStat("MitHeavyPRatPCap",L),N)
						end
					elseif SN < "MINSTRELCDBASENCPR" then
						if SN < "MARINERCDCALCTYPETACMIT" then
							if SN > "MARINERCDBASEPOWER" then
								if SN > "MARINERCDBASEWILL" then
									if SN == "MARINERCDCALCTYPECOMPHYMIT" then
										Result = 13
									elseif SN == "MARINERCDCALCTYPENONPHYMIT" then
										Result = 13
									end
								elseif SN == "MARINERCDBASEVITALITY" then
									Result = CalcStat("ClassBaseVitality",L)
								elseif SN == "MARINERCDBASEWILL" then
									Result = CalcStat("ClassBaseWillM",L)
								end
							elseif SN < "MARINERCDBASEPOWER" then
								if SN > "MARINERCDBASEMORALE" then
									if SN == "MARINERCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRL",L)
									elseif SN == "MARINERCDBASENCPR" then
										Result = CalcStat("ClassBaseNCPR",L)
									end
								elseif SN == "MARINERCDBASEMIGHT" then
									Result = CalcStat("ClassBaseMightM",L)
								elseif SN == "MARINERCDBASEMORALE" then
									Result = CalcStat("ClassBaseMorale",L)
								end
							else
								Result = CalcStat("ClassBasePower",L)
							end
						elseif SN > "MARINERCDCALCTYPETACMIT" then
							if SN > "MINSTRELCDBASEFATE" then
								if SN < "MINSTRELCDBASEMIGHT" then
									if SN == "MINSTRELCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRL",L)
									elseif SN == "MINSTRELCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									end
								elseif SN > "MINSTRELCDBASEMIGHT" then
									if SN == "MINSTRELCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									elseif SN == "MINSTRELCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRL",L)
									end
								else
									Result = CalcStat("ClassBaseMightL",L)
								end
							elseif SN < "MINSTRELCDBASEFATE" then
								if SN > "MIGHTT" then
									if SN == "MINSTRELCDARMOURTYPE" then
										Result = 1
									elseif SN == "MINSTRELCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityM",L)
									end
								elseif SN == "MARINERCDHASPOWER" then
									Result = 1
								elseif SN == "MIGHTT" then
									Result = CalcStat("MainT",L,N)
								end
							else
								Result = CalcStat("ClassBaseFate",L)
							end
						else
							Result = 26
						end
					else
						Result = CalcStat("ClassBaseNCPR",L)
					end
				elseif SN > "MITLIGHTPRATPCAP" then
					if SN > "OUTHEALPRATPA" then
						if SN < "PARRYPRATPCAP" then
							if SN > "PARRYPBONUS" then
								if SN < "PARRYPRATPA" then
									if SN == "PARRYPPRAT" then
										Result = CalcStat("BPEPPRat",L,N)
									elseif SN == "PARRYPRATP" then
										Result = CalcStat("BPEPRatP",L,N)
									end
								elseif SN > "PARRYPRATPA" then
									if SN == "PARRYPRATPB" then
										Result = CalcStat("BPEPRatPB",L)
									elseif SN == "PARRYPRATPC" then
										Result = CalcStat("BPEPRatPC",L)
									end
								else
									Result = CalcStat("BPEPRatPA",L)
								end
							elseif SN < "PARRYPBONUS" then
								if SN > "OUTHEALPRATPC" then
									if SN == "OUTHEALPRATPCAP" then
										Result = 70.0
									elseif SN == "OUTHEALPRATPCAPR" then
										Result = CalcStat("OutHealPRatPB",L)*CalcStat("OutHealPRatPC",L)
									end
								elseif SN == "OUTHEALPRATPB" then
									Result = CalcStat("BRatOutHeal",L)
								elseif SN == "OUTHEALPRATPC" then
									Result = 0.5
								end
							else
								Result = CalcStat("BPEPBonus",L)
							end
						elseif SN > "PARRYPRATPCAP" then
							if SN > "PARTBLOCKMITPRATPA" then
								if SN < "PARTBLOCKMITPRATPCAP" then
									if SN == "PARTBLOCKMITPRATPB" then
										Result = CalcStat("PartMitPRatPB",L)
									elseif SN == "PARTBLOCKMITPRATPC" then
										Result = CalcStat("PartMitPRatPC",L)
									end
								elseif SN > "PARTBLOCKMITPRATPCAP" then
									if SN == "PARTBLOCKMITPRATPCAPR" then
										Result = CalcStat("PartMitPRatPCapR",L)
									elseif SN == "PARTBLOCKPBONUS" then
										Result = CalcStat("PartBPEPBonus",L)
									end
								else
									Result = CalcStat("PartMitPRatPCap",L)
								end
							elseif SN < "PARTBLOCKMITPRATPA" then
								if SN > "PARTBLOCKMITPBONUS" then
									if SN == "PARTBLOCKMITPPRAT" then
										Result = CalcStat("PartMitPPRat",L,N)
									elseif SN == "PARTBLOCKMITPRATP" then
										Result = CalcStat("PartMitPRatP",L,N)
									end
								elseif SN == "PARRYPRATPCAPR" then
									Result = CalcStat("BPEPRatPCapR",L)
								elseif SN == "PARTBLOCKMITPBONUS" then
									Result = CalcStat("PartMitPBonus",L)
								end
							else
								Result = CalcStat("PartMitPRatPA",L)
							end
						else
							Result = CalcStat("BPEPRatPCap",L)
						end
					elseif SN < "OUTHEALPRATPA" then
						if SN < "NCMRT" then
							if SN > "MITMEDIUMPRATPB" then
								if SN > "MITMEDIUMPRATPCAP" then
									if SN == "MITMEDIUMPRATPCAPR" then
										Result = CalcStat("MitMediumPRatPB",L)*CalcStat("MitMediumPRatPC",L)
									elseif SN == "MORALET" then
										Result = EquSng(StatLinInter("PntMPMorale","TraitPntSVital","ProgBHealth","AdjTraitHealth",L,N,2))
									end
								elseif SN == "MITMEDIUMPRATPC" then
									Result = 0.5
								elseif SN == "MITMEDIUMPRATPCAP" then
									Result = 50.0
								end
							elseif SN < "MITMEDIUMPRATPB" then
								if SN > "MITMEDIUMPPRAT" then
									if SN == "MITMEDIUMPRATP" then
										Result = CalcPercAB(CalcStat("MitMediumPRatPA",L),CalcStat("MitMediumPRatPB",L),CalcStat("MitMediumPRatPCap",L),N)
									elseif SN == "MITMEDIUMPRATPA" then
										Result = 150.0
									end
								elseif SN == "MITLIGHTPRATPCAPR" then
									Result = CalcStat("MitLightPRatPB",L)*CalcStat("MitLightPRatPC",L)
								elseif SN == "MITMEDIUMPPRAT" then
									Result = CalcRatAB(CalcStat("MitMediumPRatPA",L),CalcStat("MitMediumPRatPB",L),CalcStat("MitMediumPRatPCapR",L),N)
								end
							else
								Result = CalcStat("BRatMitMedium",L)
							end
						elseif SN > "NCMRT" then
							if SN > "OUTDMGPRATPB" then
								if SN < "OUTDMGPRATPCAPR" then
									if SN == "OUTDMGPRATPC" then
										Result = 0.5
									elseif SN == "OUTDMGPRATPCAP" then
										Result = 200.0
									end
								elseif SN > "OUTDMGPRATPCAPR" then
									if SN == "OUTHEALPPRAT" then
										Result = CalcRatAB(CalcStat("OutHealPRatPA",L),CalcStat("OutHealPRatPB",L),CalcStat("OutHealPRatPCapR",L),N)
									elseif SN == "OUTHEALPRATP" then
										Result = CalcPercAB(CalcStat("OutHealPRatPA",L),CalcStat("OutHealPRatPB",L),CalcStat("OutHealPRatPCap",L),N)
									end
								else
									Result = CalcStat("OutDmgPRatPB",L)*CalcStat("OutDmgPRatPC",L)
								end
							elseif SN < "OUTDMGPRATPB" then
								if SN > "OUTDMGPPRAT" then
									if SN == "OUTDMGPRATP" then
										Result = CalcPercAB(CalcStat("OutDmgPRatPA",L),CalcStat("OutDmgPRatPB",L),CalcStat("OutDmgPRatPCap",L),N)
									elseif SN == "OUTDMGPRATPA" then
										Result = 600.0
									end
								elseif SN == "NCPRT" then
									Result = EquSng(StatLinInter("PntMPNCPR","TraitPntSVital","ProgBEnergy","",L,N))
								elseif SN == "OUTDMGPPRAT" then
									Result = CalcRatAB(CalcStat("OutDmgPRatPA",L),CalcStat("OutDmgPRatPB",L),CalcStat("OutDmgPRatPCapR",L),N)
								end
							else
								Result = CalcStat("BRatExtra",L)
							end
						else
							Result = EquSng(StatLinInter("PntMPNCMR","TraitPntSVital","ProgBHealth","",L,N,3))
						end
					else
						Result = 210.0
					end
				else
					Result = 40.0
				end
			else
				Result = CalcStat("PartBPEPPRat",L,N)
			end
		else
			Result = CalcStat("MitLightPPRat",L,N)
		end
	elseif SN < "MARINERCDBASEICPR" then
		if SN < "CRITHITPRATPB" then
			if SN > "BURGLARCDBASEICMR" then
				if SN < "CHAMPIONCDCALCTYPECOMPHYMIT" then
					if SN > "CAPTAINCDBASENCPR" then
						if SN < "CHAMPIONCDBASEAGILITY" then
							if SN > "CAPTAINCDCALCTYPENONPHYMIT" then
								if SN > "CAPTAINCDCANBLOCK" then
									if SN == "CAPTAINCDHASPOWER" then
										Result = 1
									elseif SN == "CHAMPIONCDARMOURTYPE" then
										Result = 3
									end
								elseif SN == "CAPTAINCDCALCTYPETACMIT" then
									Result = 27
								elseif SN == "CAPTAINCDCANBLOCK" then
									if 15 <= L then
										Result = 1
									end
								end
							elseif SN < "CAPTAINCDCALCTYPENONPHYMIT" then
								if SN > "CAPTAINCDBASEVITALITY" then
									if SN == "CAPTAINCDBASEWILL" then
										Result = CalcStat("ClassBaseWillM",L)
									elseif SN == "CAPTAINCDCALCTYPECOMPHYMIT" then
										Result = 14
									end
								elseif SN == "CAPTAINCDBASEPOWER" then
									Result = CalcStat("ClassBasePower",L)
								elseif SN == "CAPTAINCDBASEVITALITY" then
									Result = CalcStat("ClassBaseVitality",L)
								end
							else
								Result = 14
							end
						elseif SN > "CHAMPIONCDBASEAGILITY" then
							if SN > "CHAMPIONCDBASEMORALE" then
								if SN < "CHAMPIONCDBASEPOWER" then
									if SN == "CHAMPIONCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRH",L)
									elseif SN == "CHAMPIONCDBASENCPR" then
										Result = CalcStat("ClassBaseNCPR",L)
									end
								elseif SN > "CHAMPIONCDBASEPOWER" then
									if SN == "CHAMPIONCDBASEVITALITY" then
										Result = CalcStat("ClassBaseVitality",L)
									elseif SN == "CHAMPIONCDBASEWILL" then
										Result = CalcStat("ClassBaseWillL",L)
									end
								else
									Result = CalcStat("ClassBasePower",L)
								end
							elseif SN < "CHAMPIONCDBASEMORALE" then
								if SN > "CHAMPIONCDBASEICMR" then
									if SN == "CHAMPIONCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									elseif SN == "CHAMPIONCDBASEMIGHT" then
										Result = CalcStat("ClassBaseMightH",L)
									end
								elseif SN == "CHAMPIONCDBASEFATE" then
									Result = CalcStat("ClassBaseFate",L)
								elseif SN == "CHAMPIONCDBASEICMR" then
									Result = CalcStat("ClassBaseICMRH",L)
								end
							else
								Result = CalcStat("ClassBaseMorale",L)
							end
						else
							Result = CalcStat("ClassBaseAgilityM",L)
						end
					elseif SN < "CAPTAINCDBASENCPR" then
						if SN < "BURGLARCDCALCTYPENONPHYMIT" then
							if SN > "BURGLARCDBASENCPR" then
								if SN > "BURGLARCDBASEVITALITY" then
									if SN == "BURGLARCDBASEWILL" then
										Result = CalcStat("ClassBaseWillL",L)
									elseif SN == "BURGLARCDCALCTYPECOMPHYMIT" then
										Result = 13
									end
								elseif SN == "BURGLARCDBASEPOWER" then
									Result = CalcStat("ClassBasePower",L)
								elseif SN == "BURGLARCDBASEVITALITY" then
									Result = CalcStat("ClassBaseVitality",L)
								end
							elseif SN < "BURGLARCDBASENCPR" then
								if SN > "BURGLARCDBASEMIGHT" then
									if SN == "BURGLARCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									elseif SN == "BURGLARCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRL",L)
									end
								elseif SN == "BURGLARCDBASEICPR" then
									Result = CalcStat("ClassBaseICPR",L)
								elseif SN == "BURGLARCDBASEMIGHT" then
									Result = CalcStat("ClassBaseMightM",L)
								end
							else
								Result = CalcStat("ClassBaseNCPR",L)
							end
						elseif SN > "BURGLARCDCALCTYPENONPHYMIT" then
							if SN > "CAPTAINCDBASEFATE" then
								if SN < "CAPTAINCDBASEMIGHT" then
									if SN == "CAPTAINCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRM",L)
									elseif SN == "CAPTAINCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									end
								elseif SN > "CAPTAINCDBASEMIGHT" then
									if SN == "CAPTAINCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									elseif SN == "CAPTAINCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRM",L)
									end
								else
									Result = CalcStat("ClassBaseMightH",L)
								end
							elseif SN < "CAPTAINCDBASEFATE" then
								if SN > "BURGLARCDHASPOWER" then
									if SN == "CAPTAINCDARMOURTYPE" then
										Result = 3
									elseif SN == "CAPTAINCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityL",L)
									end
								elseif SN == "BURGLARCDCALCTYPETACMIT" then
									Result = 26
								elseif SN == "BURGLARCDHASPOWER" then
									Result = 1
								end
							else
								Result = CalcStat("ClassBaseFate",L)
							end
						else
							Result = 13
						end
					else
						Result = CalcStat("ClassBaseNCPR",L)
					end
				elseif SN > "CHAMPIONCDCALCTYPECOMPHYMIT" then
					if SN > "CLASSBASEMORALE" then
						if SN < "CLASSNAME" then
							if SN > "CLASSBASEPOWER" then
								if SN < "CLASSBASEWILLH" then
									if SN == "CLASSBASEPOWERREGENPNTS" then
										Result = {{1,20,50,60,65,75,85,95,100,105,115,120,130,140,141,150,151,160,161,170},{1,20,50,60,65,75,85,95,100,105,115,120,130,140,141,150,151,160,161,170}}
									elseif SN == "CLASSBASEVITALITY" then
										Result = RoundDbl(CalcStat("DirectHealth",L,1.5))
									end
								elseif SN > "CLASSBASEWILLH" then
									if SN == "CLASSBASEWILLL" then
										Result = RoundDbl(CalcStat("DirectRatingsOld",L,0.5))
									elseif SN == "CLASSBASEWILLM" then
										Result = RoundDbl(CalcStat("DirectRatingsOld",L,1.0))
									end
								else
									Result = RoundDbl(CalcStat("DirectRatingsOld",L,1.5))
								end
							elseif SN < "CLASSBASEPOWER" then
								if SN > "CLASSBASENCMRL" then
									if SN == "CLASSBASENCMRM" then
										Result = 1.0
									elseif SN == "CLASSBASENCPR" then
										Result = EquSng(StatLinInter("PntMPClassBaseNCPR","ClassBasePowerRegenPntS","ProgBEnergy","AdjClassBasePowReg",L,N))
									end
								elseif SN == "CLASSBASENCMRH" then
									Result = 2.0
								elseif SN == "CLASSBASENCMRL" then
									Result = 1.0
								end
							else
								if L <= 95 then
									Result = RoundDblDown(CalcStat("DirectEnergy",L,10.0))
								else
									Result = RoundDbl(CalcStat("DirectEnergy",L,10.0))
								end
							end
						elseif SN > "CLASSNAME" then
							if SN > "CRITDEFPRATPC" then
								if SN < "CRITHITPPRAT" then
									if SN == "CRITDEFPRATPCAP" then
										Result = 80.0
									elseif SN == "CRITDEFPRATPCAPR" then
										Result = CalcStat("CritDefPRatPB",L)*CalcStat("CritDefPRatPC",L)
									end
								elseif SN > "CRITHITPPRAT" then
									if SN == "CRITHITPRATP" then
										Result = CalcPercAB(CalcStat("CritHitPRatPA",L),CalcStat("CritHitPRatPB",L),CalcStat("CritHitPRatPCap",L),N)
									elseif SN == "CRITHITPRATPA" then
										Result = 75.0
									end
								else
									Result = CalcRatAB(CalcStat("CritHitPRatPA",L),CalcStat("CritHitPRatPB",L),CalcStat("CritHitPRatPCapR",L),N)
								end
							elseif SN < "CRITDEFPRATPC" then
								if SN > "CRITDEFPRATP" then
									if SN == "CRITDEFPRATPA" then
										Result = 240.0
									elseif SN == "CRITDEFPRATPB" then
										Result = CalcStat("BRatStandard",L)
									end
								elseif SN == "CRITDEFPPRAT" then
									Result = CalcRatAB(CalcStat("CritDefPRatPA",L),CalcStat("CritDefPRatPB",L),CalcStat("CritDefPRatPCapR",L),N)
								elseif SN == "CRITDEFPRATP" then
									Result = CalcPercAB(CalcStat("CritDefPRatPA",L),CalcStat("CritDefPRatPB",L),CalcStat("CritDefPRatPCap",L),N)
								end
							else
								Result = 0.5
							end
						else
							Result = TranslateValue({23,24,31,40,52,71,126,127,128,162,172,179,185,192,193,194,214,215,216,217},{"Guardian","Captain","Minstrel","Burglar","Warleader","Reaver","Stalker","Weaver","Defiler","Hunter","Champion","Blackarrow","LoreMaster","Chicken","RuneKeeper","Warden","Beorning","Brawler","Mariner","Sorceress",""},L)
						end
					elseif SN < "CLASSBASEMORALE" then
						if SN < "CLASSBASEAGILITYH" then
							if SN > "CHICKENCANBLOCK" then
								if SN > "CHICKENCDCALCTYPENONPHYMIT" then
									if SN == "CHICKENCDCALCTYPETACMIT" then
										Result = 27
									elseif SN == "CHICKENCDHASPOWER" then
										Result = 1
									end
								elseif SN == "CHICKENCDCALCTYPECOMPHYMIT" then
									Result = 14
								elseif SN == "CHICKENCDCALCTYPENONPHYMIT" then
									Result = 14
								end
							elseif SN < "CHICKENCANBLOCK" then
								if SN > "CHAMPIONCDCALCTYPETACMIT" then
									if SN == "CHAMPIONCDCANBLOCK" then
										if 6 <= L then
											Result = 1
										end
									elseif SN == "CHAMPIONCDHASPOWER" then
										Result = 1
									end
								elseif SN == "CHAMPIONCDCALCTYPENONPHYMIT" then
									Result = 14
								elseif SN == "CHAMPIONCDCALCTYPETACMIT" then
									Result = 27
								end
							else
								Result = 1
							end
						elseif SN > "CLASSBASEAGILITYH" then
							if SN > "CLASSBASEICMRL" then
								if SN < "CLASSBASEMIGHTH" then
									if SN == "CLASSBASEICMRM" then
										Result = EquSng(0.175)
									elseif SN == "CLASSBASEICPR" then
										Result = EquSng(StatLinInter("PntMPClassBaseICPR","ClassBasePowerRegenPntS","ProgBEnergy","AdjClassBasePowReg",L,N))
									end
								elseif SN > "CLASSBASEMIGHTH" then
									if SN == "CLASSBASEMIGHTL" then
										Result = RoundDbl(CalcStat("DirectRatingsOld",L,0.5))
									elseif SN == "CLASSBASEMIGHTM" then
										Result = RoundDbl(CalcStat("DirectRatingsOld",L,1.0))
									end
								else
									Result = RoundDbl(CalcStat("DirectRatingsOld",L,1.5))
								end
							elseif SN < "CLASSBASEICMRL" then
								if SN > "CLASSBASEAGILITYM" then
									if SN == "CLASSBASEFATE" then
										Result = RoundDbl(CalcStat("DirectEnergy",L,5.0))
									elseif SN == "CLASSBASEICMRH" then
										Result = EquSng(0.2)
									end
								elseif SN == "CLASSBASEAGILITYL" then
									Result = RoundDbl(CalcStat("DirectRatingsOld",L,0.5))
								elseif SN == "CLASSBASEAGILITYM" then
									Result = RoundDbl(CalcStat("DirectRatingsOld",L,1.0))
								end
							else
								Result = EquSng(0.15)
							end
						else
							Result = RoundDbl(CalcStat("DirectRatingsOld",L,1.5))
						end
					else
						if L <= 95 then
							Result = RoundDblDown(CalcStat("DirectHealth",L,10.0))
						else
							Result = RoundDbl(CalcStat("DirectHealth",L,10.0))
						end
					end
				else
					Result = 14
				end
			elseif SN < "BURGLARCDBASEICMR" then
				if SN < "BLOCKPRATPB" then
					if SN > "BEORNINGCDBASENCPR" then
						if SN < "BEORNINGRDTRAITVITALITY" then
							if SN > "BEORNINGCDCALCTYPENONPHYMIT" then
								if SN > "BEORNINGCDCANBLOCK" then
									if SN == "BEORNINGRDTRAITFATE" then
										Result = CalcStat("BeoFewinNumberFate",L)
									elseif SN == "BEORNINGRDTRAITMIGHT" then
										Result = CalcStat("BeoMightoftheWildMight",L)
									end
								elseif SN == "BEORNINGCDCALCTYPETACMIT" then
									Result = 27
								elseif SN == "BEORNINGCDCANBLOCK" then
									if 6 <= L then
										Result = 1
									end
								end
							elseif SN < "BEORNINGCDCALCTYPENONPHYMIT" then
								if SN > "BEORNINGCDBASEVITALITY" then
									if SN == "BEORNINGCDBASEWILL" then
										Result = CalcStat("ClassBaseWillM",L)
									elseif SN == "BEORNINGCDCALCTYPECOMPHYMIT" then
										Result = 14
									end
								elseif SN == "BEORNINGCDBASEPOWER" then
									Result = 10
								elseif SN == "BEORNINGCDBASEVITALITY" then
									Result = CalcStat("ClassBaseVitality",L)
								end
							else
								Result = 14
							end
						elseif SN > "BEORNINGRDTRAITVITALITY" then
							if SN > "BLACKARROWCDCALCTYPETACMIT" then
								if SN < "BLOCKPPRAT" then
									if SN == "BLACKARROWCDHASPOWER" then
										Result = 1
									elseif SN == "BLOCKPBONUS" then
										Result = CalcStat("BPEPBonus",L)
									end
								elseif SN > "BLOCKPPRAT" then
									if SN == "BLOCKPRATP" then
										Result = CalcStat("BPEPRatP",L,N)
									elseif SN == "BLOCKPRATPA" then
										Result = CalcStat("BPEPRatPA",L)
									end
								else
									Result = CalcStat("BPEPPRat",L,N)
								end
							elseif SN < "BLACKARROWCDCALCTYPETACMIT" then
								if SN > "BLACKARROWCANBLOCK" then
									if SN == "BLACKARROWCDCALCTYPECOMPHYMIT" then
										Result = 13
									elseif SN == "BLACKARROWCDCALCTYPENONPHYMIT" then
										Result = 14
									end
								elseif SN == "BEOTHICKHIDEVITALITY" then
									Result = CalcStat("VitalityT",L,1.0)
								elseif SN == "BLACKARROWCANBLOCK" then
									Result = 1
								end
							else
								Result = 27
							end
						else
							Result = CalcStat("BeoThickHideVitality",L)
						end
					elseif SN < "BEORNINGCDBASENCPR" then
						if SN < "AWARDILVLI" then
							if SN > "ADJTRAITMIT" then
								if SN > "AGILITYT" then
									if SN == "ALIGNMENTNAME" then
										Result = TranslateValue({1,2,3},{"Good","Neutral","Evil",""},L)
									elseif SN == "ARMOURPENT" then
										Result = EquSng(StatLinInter("PntMPArmourPenT","TraitPntS","MitMediumPRatPB","",L,N,2))
									end
								elseif SN == "ADJTRAITRAT" then
									if 141 <= L and L <= 150 then
										Result = 0.9
									elseif 151 <= L and L <= 160 then
										Result = 0.8
									else
										Result = 1.0
									end
								elseif SN == "AGILITYT" then
									Result = CalcStat("MainT",L,N)
								end
							elseif SN < "ADJTRAITMIT" then
								if SN > "ADJCLASSBASEPOWREG" then
									if SN == "ADJTRAITHEALTH" then
										if L <= 25 then
											Result = 0.5
										elseif L <= 50 then
											Result = 0.6
										elseif L <= 60 then
											Result = 0.7
										elseif L <= 65 then
											Result = 0.8
										elseif L <= 75 then
											Result = 0.9
										else
											Result = 1.0
										end
									elseif SN == "ADJTRAITMAIN" then
										if 141 <= L and L <= 150 then
											Result = 0.9
										elseif 151 <= L and L <= 160 then
											Result = 0.85
										else
											Result = 1.0
										end
									end
								elseif SN == "-VERSION" then
									Result = "2.5.5p"
								elseif SN == "ADJCLASSBASEPOWREG" then
									if L <= 1 then
										Result = 1.5
									elseif L <= 20 then
										Result = 1.1
									else
										Result = 1.0
									end
								end
							else
								if L == 141 then
									Result = 0.78
								elseif L == 150 then
									Result = 0.7
								elseif 151 <= L and L <= 160 then
									Result = 0.65
								else
									Result = 1.0
								end
							end
						elseif SN > "AWARDILVLI" then
							if SN > "BEORNINGCDBASEFATE" then
								if SN < "BEORNINGCDBASEMIGHT" then
									if SN == "BEORNINGCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRM",L)
									elseif SN == "BEORNINGCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									end
								elseif SN > "BEORNINGCDBASEMIGHT" then
									if SN == "BEORNINGCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									elseif SN == "BEORNINGCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRM",L)
									end
								else
									Result = CalcStat("ClassBaseMightM",L)
								end
							elseif SN < "BEORNINGCDBASEFATE" then
								if SN > "BEOMIGHTOFTHEWILDMIGHT" then
									if SN == "BEORNINGCDARMOURTYPE" then
										Result = 3
									elseif SN == "BEORNINGCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityM",L)
									end
								elseif SN == "BEOFEWINNUMBERFATE" then
									Result = -CalcStat("FateT",L,0.4)
								elseif SN == "BEOMIGHTOFTHEWILDMIGHT" then
									Result = CalcStat("MightT",L,1.0)
								end
							else
								Result = CalcStat("ClassBaseFate",L)
							end
						else
							if L <= 44 then
								Result = 1
							elseif L <= 55 then
								Result = 52
							elseif L <= 75 then
								Result = RoundDblDown((L-56)/5)*5+60
							elseif L <= 85 then
								Result = RoundDblDown((L-76)/5)*29+100
							elseif L <= 95 then
								Result = RoundDblDown((L-86)/5)*20+155
							elseif L <= 100 then
								Result = RoundDblDown((L-96)/4)*10+190
							elseif L <= 105 then
								Result = RoundDblDown((L-101)/4)*35+215
							elseif L <= 110 then
								Result = CalcStat("LvlToILvl",106)+15
							elseif L <= 115 then
								Result = CalcStat("LvlToILvl",115)
							elseif L <= 119 then
								Result = CalcStat("LvlToILvl",116)+15
							elseif L <= 120 then
								Result = CalcStat("LvlToILvl",120)
							elseif L <= 125 then
								Result = CalcStat("LvlToILvl",121)+15
							elseif L <= 130 then
								Result = CalcStat("LvlToILvl",130)
							elseif L <= 135 then
								Result = CalcStat("LvlToILvl",131)+15
							elseif L <= 140 then
								Result = CalcStat("LvlToILvl",140)
							elseif L <= 145 then
								Result = CalcStat("LvlToILvl",141)+15
							elseif L <= 150 then
								Result = CalcStat("LvlToILvl",150)
							elseif L <= 155 then
								Result = CalcStat("LvlToILvl",151)+15
							elseif L <= 160 then
								Result = CalcStat("LvlToILvl",160)
							else
								Result = CalcStat("AwardILvlI",160)
							end
						end
					else
						Result = CalcStat("ClassBaseNCPR",L)
					end
				elseif SN > "BLOCKPRATPB" then
					if SN > "BRATROUNDED" then
						if SN < "BRAWLERCDBASENCPR" then
							if SN > "BRAWLERCDBASEICMR" then
								if SN > "BRAWLERCDBASEMIGHT" then
									if SN == "BRAWLERCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									elseif SN == "BRAWLERCDBASENCMR" then
										Result = CalcStat("ClassBaseNCMRL",L)
									end
								elseif SN == "BRAWLERCDBASEICPR" then
									Result = CalcStat("ClassBaseICPR",L)
								elseif SN == "BRAWLERCDBASEMIGHT" then
									Result = CalcStat("ClassBaseMightH",L)
								end
							elseif SN < "BRAWLERCDBASEICMR" then
								if SN > "BRAWLERCDARMOURTYPE" then
									if SN == "BRAWLERCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityM",L)
									elseif SN == "BRAWLERCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									end
								elseif SN == "BRATSTANDARD" then
									Result = CalcStat("BRatRounded",L,CalcStat("StdProgRatings",L,200.0))
								elseif SN == "BRAWLERCDARMOURTYPE" then
									Result = 3
								end
							else
								Result = CalcStat("ClassBaseICMRL",L)
							end
						elseif SN > "BRAWLERCDBASENCPR" then
							if SN > "BRAWLERCDCALCTYPENONPHYMIT" then
								if SN < "BURGLARCDARMOURTYPE" then
									if SN == "BRAWLERCDCALCTYPETACMIT" then
										Result = 27
									elseif SN == "BRAWLERCDHASPOWER" then
										Result = 1
									end
								elseif SN > "BURGLARCDARMOURTYPE" then
									if SN == "BURGLARCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityH",L)
									elseif SN == "BURGLARCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									end
								else
									Result = 2
								end
							elseif SN < "BRAWLERCDCALCTYPENONPHYMIT" then
								if SN > "BRAWLERCDBASEVITALITY" then
									if SN == "BRAWLERCDBASEWILL" then
										Result = CalcStat("ClassBaseWillL",L)
									elseif SN == "BRAWLERCDCALCTYPECOMPHYMIT" then
										Result = 14
									end
								elseif SN == "BRAWLERCDBASEPOWER" then
									Result = CalcStat("ClassBasePower",L)
								elseif SN == "BRAWLERCDBASEVITALITY" then
									Result = CalcStat("ClassBaseVitality",L)
								end
							else
								Result = 14
							end
						else
							Result = CalcStat("ClassBaseNCPR",L)
						end
					elseif SN < "BRATROUNDED" then
						if SN < "BPEPRATPCAPR" then
							if SN > "BPEPRATP" then
								if SN > "BPEPRATPB" then
									if SN == "BPEPRATPC" then
										Result = 0.5
									elseif SN == "BPEPRATPCAP" then
										Result = 13.0
									end
								elseif SN == "BPEPRATPA" then
									Result = 39.0
								elseif SN == "BPEPRATPB" then
									Result = CalcStat("BRatStandard",L)
								end
							elseif SN < "BPEPRATP" then
								if SN > "BLOCKPRATPCAP" then
									if SN == "BLOCKPRATPCAPR" then
										Result = CalcStat("BPEPRatPCapR",L)
									elseif SN == "BPEPPRAT" then
										Result = CalcRatAB(CalcStat("BPEPRatPA",L),CalcStat("BPEPRatPB",L),CalcStat("BPEPRatPCapR",L),N)
									end
								elseif SN == "BLOCKPRATPC" then
									Result = CalcStat("BPEPRatPC",L)
								elseif SN == "BLOCKPRATPCAP" then
									Result = CalcStat("BPEPRatPCap",L)
								end
							else
								Result = CalcPercAB(CalcStat("BPEPRatPA",L),CalcStat("BPEPRatPB",L),CalcStat("BPEPRatPCap",L),N)
							end
						elseif SN > "BPEPRATPCAPR" then
							if SN > "BRATMITBASE" then
								if SN < "BRATMITMEDIUM" then
									if SN == "BRATMITHEAVY" then
										if L <= 1 then
											Result = 200.0
										elseif 50 <= L then
											Result = CalcStat("BRatRounded",L,CalcStat("BRatMitBase",L,1.0))
										else
											Result = RoundDbl(LinFmod(1.0,CalcStat("BRatMitHeavy",1),CalcStat("BRatMitHeavy",50),1,50,L),0)
										end
									elseif SN == "BRATMITLIGHT" then
										if L <= 1 then
											Result = 105.0
										elseif 50 <= L then
											Result = CalcStat("BRatRounded",L,CalcStat("BRatMitBase",L,0.666))
										else
											Result = RoundDbl(LinFmod(1.0,CalcStat("BRatMitLight",1),CalcStat("BRatMitLight",50),1,50,L),0)
										end
									end
								elseif SN > "BRATMITMEDIUM" then
									if SN == "BRATOUTHEAL" then
										Result = CalcStat("BRatRounded",L,CalcStat("StdProgRatings",L,450.0))
									elseif SN == "BRATPARTBPE" then
										Result = CalcStat("BRatRounded",L,CalcStat("StdProgRatings",L,350.0))
									end
								else
									if L <= 1 then
										Result = 144.0
									elseif 50 <= L then
										Result = CalcStat("BRatRounded",L,CalcStat("BRatMitBase",L,0.833))
									else
										Result = RoundDbl(LinFmod(1.0,CalcStat("BRatMitMedium",1),CalcStat("BRatMitMedium",50),1,50,L),0)
									end
								end
							elseif SN < "BRATMITBASE" then
								if SN > "BRATCRITMAGN" then
									if SN == "BRATDEVHIT" then
										Result = CalcStat("BRatRounded",L,CalcStat("StdProgRatings",L,400.0))
									elseif SN == "BRATEXTRA" then
										Result = CalcStat("BRatRounded",L,CalcStat("StdProgRatings",L,300.0))
									end
								elseif SN == "BPET" then
									Result = EquSng(StatLinInter("PntMPBPE","TraitPntS","BPEPRatPB","AdjTraitRat",L,N,2))
								elseif SN == "BRATCRITMAGN" then
									Result = CalcStat("BRatRounded",L,CalcStat("StdProgRatings",L,600.0))
								end
							else
								Result = StatLinInter("","StdPntS","BRatStandard","",L,N,1)
							end
						else
							Result = CalcStat("BPEPRatPB",L)*CalcStat("BPEPRatPC",L)
						end
					else
						if L <= 50 then
							Result = RoundDbl(N,0)
						elseif L <= 105 then
							Result = RoundDbl(N,-1)
						elseif L <= 115 then
							Result = RoundDbl(N,-2)
						elseif L <= 130 then
							Result = RoundDbl(N,-1)
						elseif L <= 160 then
							Result = RoundDbl(N,-2)
						else
							Result = RoundDbl(N,0)
						end
					end
				else
					Result = CalcStat("BPEPRatPB",L)
				end
			else
				Result = CalcStat("ClassBaseICMRL",L)
			end
		elseif SN > "CRITHITPRATPB" then
			if SN > "GUARDIANCDHASPOWER" then
				if SN < "INHEALPRATPA" then
					if SN > "HUNTERCDBASEMORALE" then
						if SN < "ICMRT" then
							if SN > "HUNTERCDBASEWILL" then
								if SN > "HUNTERCDCALCTYPENONPHYMIT" then
									if SN == "HUNTERCDCALCTYPETACMIT" then
										Result = 26
									elseif SN == "HUNTERCDHASPOWER" then
										Result = 1
									end
								elseif SN == "HUNTERCDCALCTYPECOMPHYMIT" then
									Result = 13
								elseif SN == "HUNTERCDCALCTYPENONPHYMIT" then
									Result = 13
								end
							elseif SN < "HUNTERCDBASEWILL" then
								if SN > "HUNTERCDBASENCPR" then
									if SN == "HUNTERCDBASEPOWER" then
										Result = CalcStat("ClassBasePower",L)
									elseif SN == "HUNTERCDBASEVITALITY" then
										Result = CalcStat("ClassBaseVitality",L)
									end
								elseif SN == "HUNTERCDBASENCMR" then
									Result = CalcStat("ClassBaseNCMRM",L)
								elseif SN == "HUNTERCDBASENCPR" then
									Result = CalcStat("ClassBaseNCPR",L)
								end
							else
								Result = CalcStat("ClassBaseWillL",L)
							end
						elseif SN > "ICMRT" then
							if SN > "INDMGPRATPB" then
								if SN < "INDMGPRATPCAPR" then
									if SN == "INDMGPRATPC" then
										Result = 0.5
									elseif SN == "INDMGPRATPCAP" then
										Result = 400.0
									end
								elseif SN > "INDMGPRATPCAPR" then
									if SN == "INHEALPPRAT" then
										Result = CalcRatAB(CalcStat("InHealPRatPA",L),CalcStat("InHealPRatPB",L),CalcStat("InHealPRatPCapR",L),N)
									elseif SN == "INHEALPRATP" then
										Result = CalcPercAB(CalcStat("InHealPRatPA",L),CalcStat("InHealPRatPB",L),CalcStat("InHealPRatPCap",L),N)
									end
								else
									Result = CalcStat("InDmgPRatPB",L)*CalcStat("InDmgPRatPC",L)
								end
							elseif SN < "INDMGPRATPB" then
								if SN > "INDMGPPRAT" then
									if SN == "INDMGPRATP" then
										Result = CalcPercAB(CalcStat("InDmgPRatPA",L),CalcStat("InDmgPRatPB",L),CalcStat("InDmgPRatPCap",L),N)
									elseif SN == "INDMGPRATPA" then
										Result = 1200.0
									end
								elseif SN == "ICPRT" then
									Result = EquSng(StatLinInter("PntMPICPR","TraitPntSVital","ProgBEnergy","",L,N))
								elseif SN == "INDMGPPRAT" then
									Result = CalcRatAB(CalcStat("InDmgPRatPA",L),CalcStat("InDmgPRatPB",L),CalcStat("InDmgPRatPCapR",L),N)
								end
							else
								Result = CalcStat("BRatStandard",L)
							end
						else
							Result = EquSng(StatLinInter("PntMPICMR","TraitPntSVital","ProgBHealth","",L,N,3))
						end
					elseif SN < "HUNTERCDBASEMORALE" then
						if SN < "HOBBITRDTRAITNCMR" then
							if SN > "HIGHELFRDTRAITFATE" then
								if SN > "HIGHELFRDTRAITNCMR" then
									if SN == "HIGHELFRDTRAITWILL" then
										Result = CalcStat("HElfSorrowUndyingWill",L)
									elseif SN == "HOBBITRDTRAITMIGHT" then
										Result = CalcStat("HobSmallSizeMight",L)
									end
								elseif SN == "HIGHELFRDTRAITMORALE" then
									Result = CalcStat("HElfPeaceEldarMorale",L)
								elseif SN == "HIGHELFRDTRAITNCMR" then
									Result = CalcStat("HElfPeaceEldarNCMR",L)
								end
							elseif SN < "HIGHELFRDTRAITFATE" then
								if SN > "HELFPEACEELDARMORALE" then
									if SN == "HELFPEACEELDARNCMR" then
										Result = CalcStat("NCMRT",L,0.6)
									elseif SN == "HELFSORROWUNDYINGWILL" then
										Result = -CalcStat("WillT",L,0.4)
									end
								elseif SN == "HELFFADINGFIRSTBORNFATE" then
									Result = -CalcStat("FateT",L,0.4)
								elseif SN == "HELFPEACEELDARMORALE" then
									Result = CalcStat("MoraleT",L,1.0)
								end
							else
								Result = CalcStat("HElfFadingFirstbornFate",L)
							end
						elseif SN > "HOBBITRDTRAITNCMR" then
							if SN > "HUNTERCDARMOURTYPE" then
								if SN < "HUNTERCDBASEICMR" then
									if SN == "HUNTERCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityH",L)
									elseif SN == "HUNTERCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									end
								elseif SN > "HUNTERCDBASEICMR" then
									if SN == "HUNTERCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									elseif SN == "HUNTERCDBASEMIGHT" then
										Result = CalcStat("ClassBaseMightM",L)
									end
								else
									Result = CalcStat("ClassBaseICMRM",L)
								end
							elseif SN < "HUNTERCDARMOURTYPE" then
								if SN > "HOBHOBBITTOUGHNVITALITY" then
									if SN == "HOBRAPIDRECOVERYNCMR" then
										Result = CalcStat("NCMRT",L,0.6)
									elseif SN == "HOBSMALLSIZEMIGHT" then
										Result = -CalcStat("MightT",L,0.4)
									end
								elseif SN == "HOBBITRDTRAITVITALITY" then
									Result = CalcStat("HobHobbitToughnVitality",L)
								elseif SN == "HOBHOBBITTOUGHNVITALITY" then
									Result = CalcStat("VitalityT",L,1.0)
								end
							else
								Result = 2
							end
						else
							Result = CalcStat("HobRapidRecoveryNCMR",L)
						end
					else
						Result = CalcStat("ClassBaseMorale",L)
					end
				elseif SN > "INHEALPRATPA" then
					if SN > "LOREMASTERCDBASEVITALITY" then
						if SN < "MANEASILYINSPINHEALP" then
							if SN > "LOREMASTERCDHASPOWER" then
								if SN < "LVLTOILVL" then
									if SN == "LVLEXPCOST" then
										if L <= 1 then
											Result = 0
										elseif L <= 5 then
											Result = RoundDbl(12.5*L*L+12.5666666666667*L+24.8666666666667)
										elseif L <= 10 then
											Result = RoundDbl(33.8*L*L-179.48*L+452.6)
										elseif L <= 15 then
											Result = RoundDbl(55.05*L*L-583.77*L+2370.5)
										elseif L <= 20 then
											Result = RoundDbl(76.2*L*L-1196.96*L+6809)
										elseif L <= 25 then
											Result = RoundDbl(97.4*L*L-2023*L+14849.8)
										elseif L <= 30 then
											Result = RoundDbl(118.7*L*L-3066.02 *L+27612.8)
										elseif L <= 35 then
											Result = RoundDbl(139.95*L*L-4319.23*L+46084.1)
										elseif L <= 40 then
											Result = RoundDbl(161.2*L*L-5785.04*L+71356.2)
										elseif L <= 45 then
											Result = RoundDbl(182.5*L*L-7467.38*L+104569.8)
										elseif L <= 50 then
											Result = RoundDbl(203.8*L*L-9363.48*L+146761.8)
										elseif L <= 55 then
											Result = RoundDbl(225.05*L*L-11467.77*L+198851.3)
										elseif L <= 60 then
											Result = RoundDbl(246.3*L*L-13784.46*L+261988)
										elseif 61 <= L and L <= 70 then
											Result = RoundDbl(ExpFmod(CalcStat("LvlExpCost",60),61,5.071,L,3.485))
										elseif 71 <= L and L <= 75 then
											Result = RoundDbl(ExpFmod(CalcStat("LvlExpCost",70),71,5.072,L,-0.95))
										elseif 76 <= L then
											Result = RoundDbl(ExpFmod(CalcStat("LvlExpCost",75),76,5,L,-0.5,0))
										end
									elseif SN == "LVLEXPCOSTTOT" then
										if L <= 0 then
											Result = 0
										elseif 1 <= L then
											Result = CalcStat("LvlExpCostTot",L-1)+CalcStat("LvlExpCost",L)
										end
									end
								elseif SN > "LVLTOILVL" then
									if SN == "MAINT" then
										Result = RoundDblDown(StatLinInter("PntMPMain","TraitPntS","ProgBMain","AdjTraitMain",L,N,2),0)
									elseif SN == "MANDIMMANKINDWILL" then
										Result = -CalcStat("WillT",L,0.4)
									end
								else
									Result = RoundDbl(LinInter({{1,25,50,75,76,100,105,106,115,116,120,121,130,131,140,141,150,151,160,161,170},{1.0,25.0,50.0,79.0,80.0,200.0,225.0,300.0,349.0,350.0,399.0,400.0,449.0,450.0,499.0,500.0,549.0,550.0,599.0,600.0,649.0}},L))
								end
							elseif SN < "LOREMASTERCDHASPOWER" then
								if SN > "LOREMASTERCDCALCTYPECOMPHYMIT" then
									if SN == "LOREMASTERCDCALCTYPENONPHYMIT" then
										Result = 12
									elseif SN == "LOREMASTERCDCALCTYPETACMIT" then
										Result = 25
									end
								elseif SN == "LOREMASTERCDBASEWILL" then
									Result = CalcStat("ClassBaseWillH",L)
								elseif SN == "LOREMASTERCDCALCTYPECOMPHYMIT" then
									Result = 12
								end
							else
								Result = 1
							end
						elseif SN > "MANEASILYINSPINHEALP" then
							if SN > "MANRDTRAITWILL" then
								if SN < "MARINERCDBASEAGILITY" then
									if SN == "MANSTRONGMENMIGHT" then
										Result = CalcStat("MightT",L,1.0)
									elseif SN == "MARINERCDARMOURTYPE" then
										Result = 2
									end
								elseif SN > "MARINERCDBASEAGILITY" then
									if SN == "MARINERCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									elseif SN == "MARINERCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRL",L)
									end
								else
									Result = CalcStat("ClassBaseAgilityM",L)
								end
							elseif SN < "MANRDTRAITWILL" then
								if SN > "MANRDTRAITFATE" then
									if SN == "MANRDTRAITINHEALP" then
										Result = CalcStat("ManEasilyInspInHealP",L)
									elseif SN == "MANRDTRAITMIGHT" then
										Result = CalcStat("ManStrongMenMight",L)
									end
								elseif SN == "MANGIFTOFMENFATE" then
									Result = CalcStat("FateT",L,1.0)
								elseif SN == "MANRDTRAITFATE" then
									Result = CalcStat("ManGiftOfMenFate",L)
								end
							else
								Result = CalcStat("ManDimMankindWill",L)
							end
						else
							Result = 5.0
						end
					elseif SN < "LOREMASTERCDBASEVITALITY" then
						if SN < "LMANCIENTWISDOMWILL" then
							if SN > "LEVELCAP" then
								if SN > "LI2REFORGECOST" then
									if SN == "LI2REFORGECOSTSEG" then
										if L <= 0 then
											Result = 0.0
										elseif 1 <= L and L <= 12 then
											Result = DataTableValue({1.0,1.2,1.4,1.7,2.1,2.5,3.0,3.6,4.3,5.1,6.4,7.5},L)
										elseif L <= 22 then
											Result = LinFmod(1.0,8.0,35.0,13,22,L)
										elseif L <= 41 then
											Result = LinFmod(1.0,35.7,48.3,23,41,L)
										elseif L <= 43 then
											Result = LinFmod(1.0,50.0,60.0,42,43,L)
										elseif L <= 48 then
											Result = RoundDbl(LinFmod(1.0,62.6,77.0,44,48,L),0)
										elseif L <= 53 then
											Result = RoundDbl(LinFmod(1.0,80.0,99.0,49,53,L),0)
										elseif L <= 55 then
											Result = LinFmod(1.0,103.5,109.0,54,55,L)
										elseif L <= 61 then
											Result = RoundDbl(LinFmod(1.0,113.6,145.0,56,61,L),0)
										elseif L <= 120 then
											Result = LinFmod(1.0,153.0,849.0,62,120,L)
										elseif L <= 130 then
											Result = LinFmod(1.0,855.0,909.0,121,130,L)
										else
											Result = LinFmod(1.0,914.0,1009.0,131,150,L)
										end
									elseif SN == "LI2REFORGEILVL" then
										Result = RoundDbl(CalcRatAB(N+N,CalcStat("AwardILvlI",L),N,N))
									end
								elseif SN == "LI2ILVLCAP" then
									Result = 565
								elseif SN == "LI2REFORGECOST" then
									if L <= 150 then
										Result = RoundDbl(CalcStat("Li2ReforgeCostSeg",L)*(L+9)*12.5)
									else
										Result = CalcStat("Li2ReforgeCost",150)
									end
								end
							elseif SN < "LEVELCAP" then
								if SN > "INHEALPRATPC" then
									if SN == "INHEALPRATPCAP" then
										Result = 25.0
									elseif SN == "INHEALPRATPCAPR" then
										Result = CalcStat("InHealPRatPB",L)*CalcStat("InHealPRatPC",L)
									end
								elseif SN == "INHEALPRATPB" then
									Result = CalcStat("BRatStandard",L)
								elseif SN == "INHEALPRATPC" then
									Result = 0.5
								end
							else
								Result = 160
							end
						elseif SN > "LMANCIENTWISDOMWILL" then
							if SN > "LOREMASTERCDBASEICPR" then
								if SN < "LOREMASTERCDBASENCMR" then
									if SN == "LOREMASTERCDBASEMIGHT" then
										Result = CalcStat("ClassBaseMightM",L)
									elseif SN == "LOREMASTERCDBASEMORALE" then
										Result = CalcStat("ClassBaseMorale",L)
									end
								elseif SN > "LOREMASTERCDBASENCMR" then
									if SN == "LOREMASTERCDBASENCPR" then
										Result = CalcStat("ClassBaseNCPR",L)
									elseif SN == "LOREMASTERCDBASEPOWER" then
										Result = CalcStat("ClassBasePower",L)
									end
								else
									Result = CalcStat("ClassBaseNCMRL",L)
								end
							elseif SN < "LOREMASTERCDBASEICPR" then
								if SN > "LOREMASTERCDBASEAGILITY" then
									if SN == "LOREMASTERCDBASEFATE" then
										Result = CalcStat("ClassBaseFate",L)
									elseif SN == "LOREMASTERCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRL",L)
									end
								elseif SN == "LOREMASTERCDARMOURTYPE" then
									Result = 1
								elseif SN == "LOREMASTERCDBASEAGILITY" then
									Result = CalcStat("ClassBaseAgilityL",L)
								end
							else
								Result = CalcStat("ClassBaseICPR",L)
							end
						else
							Result = RoundDblUp(CalcStat("DirectRatings",L,969.95/1200.0))
						end
					else
						Result = CalcStat("ClassBaseVitality",L)
					end
				else
					Result = 75.0
				end
			elseif SN < "GUARDIANCDHASPOWER" then
				if SN < "DWARFUNWEARBATTLEICPR" then
					if SN > "DEVHITPRATPCAP" then
						if SN < "DWARFRDTRAITICPR" then
							if SN > "DIRECTRATINGSOLD" then
								if SN > "DWARFRDTRAITAGILITY" then
									if SN == "DWARFRDTRAITFATE" then
										Result = CalcStat("DwarfLostDwarfKdsFate",L)
									elseif SN == "DWARFRDTRAITICMR" then
										Result = CalcStat("DwarfUnwearBattleICMR",L)
									end
								elseif SN == "DWARFLOSTDWARFKDSFATE" then
									Result = -CalcStat("FateT",L,0.4)
								elseif SN == "DWARFRDTRAITAGILITY" then
									Result = CalcStat("DwarfStockyAgility",L)
								end
							elseif SN < "DIRECTRATINGSOLD" then
								if SN > "DIRECTENERGY" then
									if SN == "DIRECTHEALTH" then
										if L <= 95 then
											Result = CalcStat("ProgBHealth",L)*N
										else
											Result = RoundDbl(CalcStat("ProgBHealth",L),0)*N
										end
									elseif SN == "DIRECTRATINGS" then
										if L <= 95 then
											Result = CalcStat("ProgBMain",L)*N
										else
											Result = RoundDbl(CalcStat("ProgBMain",L),0)*N
										end
									end
								elseif SN == "DEVHITPRATPCAPR" then
									Result = CalcStat("DevHitPRatPB",L)*CalcStat("DevHitPRatPC",L)
								elseif SN == "DIRECTENERGY" then
									if L <= 95 then
										Result = CalcStat("ProgBEnergy",L)*N
									else
										Result = RoundDbl(CalcStat("ProgBEnergy",L),1)*N
									end
								end
							else
								if L <= 95 then
									Result = CalcStat("ProgBMainOld",L)*N
								else
									Result = RoundDbl(CalcStat("ProgBMainOld",L),0)*N
								end
							end
						elseif SN > "DWARFRDTRAITICPR" then
							if SN > "DWARFRDTRAITVITALITY" then
								if SN < "DWARFSTURDINESSPHYMITP" then
									if SN == "DWARFSTOCKYAGILITY" then
										Result = -CalcStat("AgilityT",L,0.4)
									elseif SN == "DWARFSTURDINESSMIGHT" then
										Result = CalcStat("MightT",L,1.0)
									end
								elseif SN > "DWARFSTURDINESSPHYMITP" then
									if SN == "DWARFSTURDINESSVITALITY" then
										Result = CalcStat("VitalityT",L,1.0)
									elseif SN == "DWARFUNWEARBATTLEICMR" then
										Result = CalcStat("ICMRT",L,0.6)
									end
								else
									Result = 1.0
								end
							elseif SN < "DWARFRDTRAITVITALITY" then
								if SN > "DWARFRDTRAITNCMR" then
									if SN == "DWARFRDTRAITNCPR" then
										Result = CalcStat("DwarfUnwearBattleNCPR",L)
									elseif SN == "DWARFRDTRAITPHYMITP" then
										Result = CalcStat("DwarfSturdinessPhyMitP",L)
									end
								elseif SN == "DWARFRDTRAITMIGHT" then
									Result = CalcStat("DwarfSturdinessMight",L)
								elseif SN == "DWARFRDTRAITNCMR" then
									Result = CalcStat("DwarfUnwearBattleNCMR",L)
								end
							else
								Result = CalcStat("DwarfSturdinessVitality",L)
							end
						else
							Result = CalcStat("DwarfUnwearBattleICPR",L)
						end
					elseif SN < "DEVHITPRATPCAP" then
						if SN < "CRITMAGNPRATPCAPR" then
							if SN > "CRITMAGNPRATP" then
								if SN > "CRITMAGNPRATPB" then
									if SN == "CRITMAGNPRATPC" then
										Result = 0.5
									elseif SN == "CRITMAGNPRATPCAP" then
										Result = 75.0
									end
								elseif SN == "CRITMAGNPRATPA" then
									Result = 225.0
								elseif SN == "CRITMAGNPRATPB" then
									Result = CalcStat("BRatCritMagn",L)
								end
							elseif SN < "CRITMAGNPRATP" then
								if SN > "CRITHITPRATPCAP" then
									if SN == "CRITHITPRATPCAPR" then
										Result = CalcStat("CritHitPRatPB",L)*CalcStat("CritHitPRatPC",L)
									elseif SN == "CRITMAGNPPRAT" then
										Result = CalcRatAB(CalcStat("CritMagnPRatPA",L),CalcStat("CritMagnPRatPB",L),CalcStat("CritMagnPRatPCapR",L),N)
									end
								elseif SN == "CRITHITPRATPC" then
									Result = 0.5
								elseif SN == "CRITHITPRATPCAP" then
									Result = 25.0
								end
							else
								Result = CalcPercAB(CalcStat("CritMagnPRatPA",L),CalcStat("CritMagnPRatPB",L),CalcStat("CritMagnPRatPCap",L),N)
							end
						elseif SN > "CRITMAGNPRATPCAPR" then
							if SN > "DEFILERCDHASPOWER" then
								if SN < "DEVHITPRATPA" then
									if SN == "DEVHITPPRAT" then
										Result = CalcRatAB(CalcStat("DevHitPRatPA",L),CalcStat("DevHitPRatPB",L),CalcStat("DevHitPRatPCapR",L),N)
									elseif SN == "DEVHITPRATP" then
										Result = CalcPercAB(CalcStat("DevHitPRatPA",L),CalcStat("DevHitPRatPB",L),CalcStat("DevHitPRatPCap",L),N)
									end
								elseif SN > "DEVHITPRATPA" then
									if SN == "DEVHITPRATPB" then
										Result = CalcStat("BRatDevHit",L)
									elseif SN == "DEVHITPRATPC" then
										Result = 0.5
									end
								else
									Result = 30.0
								end
							elseif SN < "DEFILERCDHASPOWER" then
								if SN > "DEFILERCDCALCTYPECOMPHYMIT" then
									if SN == "DEFILERCDCALCTYPENONPHYMIT" then
										Result = 14
									elseif SN == "DEFILERCDCALCTYPETACMIT" then
										Result = 27
									end
								elseif SN == "DEFILERCANBLOCK" then
									Result = 1
								elseif SN == "DEFILERCDCALCTYPECOMPHYMIT" then
									Result = 13
								end
							else
								Result = 1
							end
						else
							Result = CalcStat("CritMagnPRatPB",L)*CalcStat("CritMagnPRatPC",L)
						end
					else
						Result = 10.0
					end
				elseif SN > "DWARFUNWEARBATTLEICPR" then
					if SN > "FINESSEPRATP" then
						if SN < "GUARDIANCDBASEMIGHT" then
							if SN > "FINESSEPRATPCAPR" then
								if SN < "GUARDIANCDBASEFATE" then
									if SN == "GUARDIANCDARMOURTYPE" then
										Result = 3
									elseif SN == "GUARDIANCDBASEAGILITY" then
										Result = CalcStat("ClassBaseAgilityM",L)
									end
								elseif SN > "GUARDIANCDBASEFATE" then
									if SN == "GUARDIANCDBASEICMR" then
										Result = CalcStat("ClassBaseICMRH",L)
									elseif SN == "GUARDIANCDBASEICPR" then
										Result = CalcStat("ClassBaseICPR",L)
									end
								else
									Result = CalcStat("ClassBaseFate",L)
								end
							elseif SN < "FINESSEPRATPCAPR" then
								if SN > "FINESSEPRATPB" then
									if SN == "FINESSEPRATPC" then
										Result = 0.5
									elseif SN == "FINESSEPRATPCAP" then
										Result = 50.0
									end
								elseif SN == "FINESSEPRATPA" then
									Result = 150.0
								elseif SN == "FINESSEPRATPB" then
									Result = CalcStat("BRatStandard",L)
								end
							else
								Result = CalcStat("FinessePRatPB",L)*CalcStat("FinessePRatPC",L)
							end
						elseif SN > "GUARDIANCDBASEMIGHT" then
							if SN > "GUARDIANCDBASEVITALITY" then
								if SN < "GUARDIANCDCALCTYPENONPHYMIT" then
									if SN == "GUARDIANCDBASEWILL" then
										Result = CalcStat("ClassBaseWillL",L)
									elseif SN == "GUARDIANCDCALCTYPECOMPHYMIT" then
										Result = 14
									end
								elseif SN > "GUARDIANCDCALCTYPENONPHYMIT" then
									if SN == "GUARDIANCDCALCTYPETACMIT" then
										Result = 27
									elseif SN == "GUARDIANCDCANBLOCK" then
										Result = 1
									end
								else
									Result = 14
								end
							elseif SN < "GUARDIANCDBASEVITALITY" then
								if SN > "GUARDIANCDBASENCMR" then
									if SN == "GUARDIANCDBASENCPR" then
										Result = CalcStat("ClassBaseNCPR",L)
									elseif SN == "GUARDIANCDBASEPOWER" then
										Result = CalcStat("ClassBasePower",L)
									end
								elseif SN == "GUARDIANCDBASEMORALE" then
									Result = CalcStat("ClassBaseMorale",L)
								elseif SN == "GUARDIANCDBASENCMR" then
									Result = CalcStat("ClassBaseNCMRH",L)
								end
							else
								Result = CalcStat("ClassBaseVitality",L)
							end
						else
							Result = CalcStat("ClassBaseMightH",L)
						end
					elseif SN < "FINESSEPRATP" then
						if SN < "ELFSORROWFIRSTBORNNCMR" then
							if SN > "ELFRDTRAITAGILITY" then
								if SN > "ELFRDTRAITMORALE" then
									if SN == "ELFRDTRAITNCMR" then
										Result = CalcStat("ElfSorrowFirstbornNCMR",L)
									elseif SN == "ELFSORROWFIRSTBORNMORALE" then
										Result = -CalcStat("MoraleT",L,0.4)
									end
								elseif SN == "ELFRDTRAITFATE" then
									Result = CalcStat("ElfFadingFirstbornFate",L)
								elseif SN == "ELFRDTRAITMORALE" then
									Result = CalcStat("ElfSorrowFirstbornMorale",L)
								end
							elseif SN < "ELFRDTRAITAGILITY" then
								if SN > "DWARFUNWEARBATTLENCPR" then
									if SN == "ELFAGILITYWOODSAGILITY" then
										Result = CalcStat("AgilityT",L,1.0)
									elseif SN == "ELFFADINGFIRSTBORNFATE" then
										Result = -CalcStat("FateT",L,0.4)
									end
								elseif SN == "DWARFUNWEARBATTLENCMR" then
									Result = -CalcStat("NCMRT",L,0.4)
								elseif SN == "DWARFUNWEARBATTLENCPR" then
									Result = -CalcStat("NCPRT",L,0.4)
								end
							else
								Result = CalcStat("ElfAgilityWoodsAgility",L)
							end
						elseif SN > "ELFSORROWFIRSTBORNNCMR" then
							if SN > "EVADEPRATPB" then
								if SN < "EVADEPRATPCAPR" then
									if SN == "EVADEPRATPC" then
										Result = CalcStat("BPEPRatPC",L)
									elseif SN == "EVADEPRATPCAP" then
										Result = CalcStat("BPEPRatPCap",L)
									end
								elseif SN > "EVADEPRATPCAPR" then
									if SN == "FATET" then
										Result = RoundDblDown(StatLinInter("PntMPFate","TraitPntSVital","ProgBEnergy","",L,N,2),0)
									elseif SN == "FINESSEPPRAT" then
										Result = CalcRatAB(CalcStat("FinessePRatPA",L),CalcStat("FinessePRatPB",L),CalcStat("FinessePRatPCapR",L),N)
									end
								else
									Result = CalcStat("BPEPRatPCapR",L)
								end
							elseif SN < "EVADEPRATPB" then
								if SN > "EVADEPPRAT" then
									if SN == "EVADEPRATP" then
										Result = CalcStat("BPEPRatP",L,N)
									elseif SN == "EVADEPRATPA" then
										Result = CalcStat("BPEPRatPA",L)
									end
								elseif SN == "EVADEPBONUS" then
									Result = CalcStat("BPEPBonus",L)
								elseif SN == "EVADEPPRAT" then
									Result = CalcStat("BPEPPRat",L,N)
								end
							else
								Result = CalcStat("BPEPRatPB",L)
							end
						else
							Result = -CalcStat("NCMRT",L,0.4)
						end
					else
						Result = CalcPercAB(CalcStat("FinessePRatPA",L),CalcStat("FinessePRatPB",L),CalcStat("FinessePRatPCap",L),N)
					end
				else
					Result = CalcStat("ICPRT",L,0.6)
				end
			else
				Result = 1
			end
		else
			Result = CalcStat("BRatExtra",L)
		end
	else
		Result = CalcStat("ClassBaseICPR",L)
	end

	return Result
end

-- to be used by other modules
-- misc.
p.trim = trim
-- floating point / rounding functions
p.DblCalcDev = DblCalcDev
p.RoundDbl = RoundDbl
p.RoundDblDown = RoundDblDown
p.RoundDblUp = RoundDblUp
p.RoundDblLotro = RoundDblLotro
p.RoundDblMorReg = RoundDblMorReg
p.RoundDblProg = RoundDblProg
p.EquSng = EquSng
p.DecSng = DecSng
-- calculation type functions
p.DataTableValue = DataTableValue
p.ExpFmod = ExpFmod
p.LinInter = LinInter
p.CalcPercAB = CalcPercAB
p.CalcRatAB = CalcRatAB
p.StatLinInter = StatLinInter
p.LinFmod = LinFmod
-- main function
p.CalcStat = CalcStat

-- ******************************* End CalcStat *******************************
