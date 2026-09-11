import "Turbine"
import "Turbine.Gameplay"
import "Turbine.UI"
import "Turbine.UI.Lotro"

import "Giseldah.CalcStat.CSExpres"

local bStartUp = true

-- local functions are a little faster:

-- local used library functions
local mathmax = math.max
local tableconcat = table.concat
local tableinsert = table.insert

-- CalcStat module access
local csm = _G
-- CalcStat
local CalcStat = csm.CalcStat
-- Formatting/Display Support
local DisplayableError = csm.DisplayableError
-- CalcStat Expressions
local SetExpressionVariables = csm.SetExpressionVariables
local TranslateExpression = csm.TranslateExpression
local ExecuteExpression = csm.ExecuteExpression

local function ShowHelpOutput()
	-- will make it scroll to the top when text is set
	lblOutput:SetText("Result")
	ltbOutput:SetText(" ")
	ltbOutput:SetSelection(1,1)
	ltbOutput:SetSelectedText(
						"Field $L(evel): enter a number which will be assigned to variable $L (default: 1).\n"..
						"Field to: if you enter a number here then the expression will be run multiple times with a changing $L.\n"..
						"Field $N or $C: if you enter a number then this will be assigned to $N (default: 1) and otherwise it will be $C (default: \"\").\n"..
						"\n"..
						"See https://lotro-wiki.com/index.php/Module:CalcStat/doc#Expressions for some more information.\n"..
						"\n"..
						"List of all available Player variables:\n"..
						"-- Character --\n"..
						"$PlayerLevel\n"..
						"$PlayerName\n"..
						"$PlayerRace (Id)\n"..
						"$PlayerClass (Id)\n"..
						"$PlayerAlignment (Id)\n"..
						"$PlayerMorale, $PlayerPower / $PlayerWrath (current)\n"..
						"\n"..
						"-- Basic Stats --\n"..
						"$PlayerMaxMorale, $PlayerICMR, $PlayerNCMR\n"..
						"$PlayerMaxPower, $PlayerICPR, $PlayerNCPR\n"..
						"$PlayerArmour\n"..
						"$PlayerMight, $PlayerBaseMight\n"..
						"$PlayerAgility, $PlayerBaseAgility\n"..
						"$PlayerVitality, $PlayerBaseVitality\n"..
						"$PlayerWill, $PlayerBaseWill\n"..
						"$PlayerFate, $PlayerBaseFate\n"..
						"-- Offence --\n"..
						"$PlayerCritHit, $PlayerMelCritHit, $PlayerRngCritHit, $PlayerTacCritHit\n"..
						"$PlayerFinesse\n"..
						"$PlayerPhyMas, $PlayerMelOff, $PlayerRngOff\n"..
						"$PlayerTacMas, $PlayerTacOff, $PlayerOutHeal\n"..
						"-- Defence --\n"..
						"$PlayerResist, $PlayerDiseaseResist, $PlayerFearResist, $PlayerPoisonResist, $PlayerWoundResist\n"..
						"$PlayerCritDef, $PlayerMelCritDef, $PlayerRngCritDef, $PlayerTacCritDef\n"..
						"$PlayerInHeal\n"..
						"  -- Avoidance --\n"..
						"  $PlayerCanBlock, $PlayerBlock\n"..
						"  $PlayerCanParry, $PlayerParry\n"..
						"  $PlayerCanEvade, $PlayerEvade\n"..
						"  -- Mitigations --\n"..
						"    -- Damage Source --\n"..
						"    $PlayerMelDef\n"..
						"    $PlayerRngDef\n"..
						"    $PlayerTacDef\n"..
						"    -- Damage Type --\n"..
						"    $PlayerPhyMit, $PlayerOFMit\n"..
						"    $PlayerTacMit, $PlayerFireMit, $PlayerLightningMit, $PlayerFrostMit, $PlayerAcidMit, $PlayerShadowMit"
					)
	ltbOutput:SetSelectionLength(0)
end

local function ClearMyExpression()
	lblOutput:SetText("Result")
	ltbExpression:SetText("")
	ltbOutput:SetText("")
end

local function SetPlayerVariables(aTranslation)
	-- Character
	SetExpressionVariables(aTranslation,{
			PlayerAlignment = Player:GetAlignment(),
			PlayerClass = Player:GetClass(),
			PlayerLevel = Player:GetLevel(),
			PlayerMaxMorale = Player:GetMaxMorale(),
			PlayerMaxPower = Player:GetMaxPower(),
			PlayerMorale = Player:GetMorale(),
			PlayerName = Player:GetName(),
			PlayerPower = Player:GetPower(),
			PlayerRace = Player:GetRace(),
			PlayerWrath = Player:GetClass() == 214 and Player:GetClassAttributes():GetWrath() or 0
		})

	if Player:GetAlignment() == 1 then

	SetExpressionVariables(aTranslation,{
	-- Basic Stats
			PlayerICMR = PlayerAttrib:GetInCombatMoraleRegeneration(),
			PlayerNCMR = PlayerAttrib:GetOutOfCombatMoraleRegeneration(),
			PlayerICPR = PlayerAttrib:GetInCombatPowerRegeneration(),
			PlayerNCPR = PlayerAttrib:GetOutOfCombatPowerRegeneration(),
			PlayerArmour = PlayerAttrib:GetArmor(),
			PlayerMight = PlayerAttrib:GetMight(),
			PlayerAgility = PlayerAttrib:GetAgility(),
			PlayerVitality = PlayerAttrib:GetVitality(),
			PlayerWill = PlayerAttrib:GetWill(),
			PlayerFate = PlayerAttrib:GetFate(),
			PlayerBaseMight = PlayerAttrib:GetBaseMight(),
			PlayerBaseAgility = PlayerAttrib:GetBaseAgility(),
			PlayerBaseVitality = PlayerAttrib:GetBaseVitality(),
			PlayerBaseWill = PlayerAttrib:GetBaseWill(),
			PlayerBaseFate = PlayerAttrib:GetBaseFate(),

	-- Offence
			PlayerCritHit = PlayerAttrib:GetBaseCriticalHitChance(),
			PlayerMelCritHit = PlayerAttrib:GetMeleeCriticalHitChance(),
			PlayerRngCritHit = PlayerAttrib:GetRangeCriticalHitChance(),
			PlayerTacCritHit = PlayerAttrib:GetTacticalCriticalHitChance(),
			PlayerFinesse = PlayerAttrib:GetFinesse(),
			PlayerPhyMas = mathmax(PlayerAttrib:GetMeleeDamage(),PlayerAttrib:GetRangeDamage()),
			PlayerMelOff = PlayerAttrib:GetMeleeDamage(),
			PlayerRngOff = PlayerAttrib:GetRangeDamage(),
			PlayerTacMas = PlayerAttrib:GetTacticalDamage(),
			PlayerTacOff = PlayerAttrib:GetTacticalDamage(),
			PlayerOutHeal = PlayerAttrib:GetOutgoingHealing(),

	-- Defence
			PlayerResist = PlayerAttrib:GetBaseResistance(),
			PlayerDiseaseResist = PlayerAttrib:GetDiseaseResistance(),
			PlayerFearResist = PlayerAttrib:GetFearResistance(),
			PlayerPoisonResist = PlayerAttrib:GetPoisonResistance(),
			PlayerWoundResist = PlayerAttrib:GetWoundResistance(),
			PlayerCritDef = PlayerAttrib:GetBaseCriticalHitAvoidance(),
			PlayerMelCritDef = PlayerAttrib:GetMeleeCriticalHitAvoidance(),
			PlayerRngCritDef = PlayerAttrib:GetRangeCriticalHitAvoidance(),
			PlayerTacCritDef = PlayerAttrib:GetTacticalCriticalHitAvoidance(),
			PlayerInHeal = PlayerAttrib:GetIncomingHealing(),
	-- Avoidance
			PlayerCanBlock = PlayerAttrib:CanBlock(),
			PlayerCanParry = PlayerAttrib:CanParry(),
			PlayerCanEvade = PlayerAttrib:CanEvade(),
			PlayerBlock = PlayerAttrib:GetBlock(),
			PlayerParry = PlayerAttrib:GetParry(),
			PlayerEvade = PlayerAttrib:GetEvade(),
	-- Mitigations
	-- Damage Source
			PlayerMelDef = PlayerAttrib:GetMeleeDefence(),
			PlayerRngDef = PlayerAttrib:GetRangeDefence(),
			PlayerTacDef = PlayerAttrib:GetTacticalDefence(),
	-- Damage Type
			PlayerPhyMit = PlayerAttrib:GetCommonMitigation(),
			PlayerOFMit = PlayerAttrib:GetPhysicalMitigation(),
			PlayerTacMit = PlayerAttrib:GetTacticalMitigation(),
			PlayerFireMit = PlayerAttrib:GetFireMitigation(),
			PlayerLightningMit = PlayerAttrib:GetLightningMitigation(),
			PlayerFrostMit = PlayerAttrib:GetFrostMitigation(),
			PlayerAcidMit = PlayerAttrib:GetAcidMitigation(),
			PlayerShadowMit = PlayerAttrib:GetShadowMitigation()
		})
		
	end
end

local function ExecuteMyExpression()
	-- clear result output
	ltbOutput:SetText("")
	local sExpression = ltbExpression:GetText()

	-- get variables
	local nL = tonumber(ltbLevel:GetText())
	if nL == nil then
		nL = 1
	end
	local nToL = tonumber(ltbToLevel:GetText())
	if nToL == nil then
		nToL = nL
	end
	local nLvlStep = 1
	if nL > nToL then
		nLvlStep = -1
	end
	local nN = tonumber(ltbNorC:GetText())
	local sC = ""
	if nN == nil then
		nN = 1
		sC = ltbNorC:GetText()
	end

	local aResult = {}

	-- translate expression (during translation, processing requests limit will be default 10,000)
	local aTranslation, e = TranslateExpression(sExpression,{
			N = nN,
			C = sC
		})

	-- execute if no error encountered during translation
	if e == nil then
		SetPlayerVariables(aTranslation)

		local xResult

		for nCurrentL = nL, nToL, nLvlStep do
			-- execute expression (with processing requests limit set to 100,000)
			xResult, e = ExecuteExpression(aTranslation,{
					L = nCurrentL
				},100000)
			if e then
				break -- break off level loop on error
			end

			-- convert a boolean result to string
			if type(xResult) == "boolean" then
				if xResult then
					xResult = "True"
				else
					xResult = "False"
				end
			end

			-- add execute result
			tableinsert(aResult,xResult)
			if nCurrentL+nLvlStep <= nToL then
				tableinsert(aResult,"\n")
			end
		end
	end

	-- add error info
	if e then
		tableinsert(aResult,DisplayableError(e,"\n"))
	end

	-- output result
	ltbOutput:SetText(tableconcat(aResult))
end

local function SetLayout()
	local nCtrlStartX = 25
	local nCtrlStartY = 50
	local nCtrlCursorX = nCtrlStartX
	local nCtrlCursorY = nCtrlStartY
	local nCtrlSpacingX = 45
	local nCtrlSpacingY = 20

	local nExprOutpWidth = wndMain:GetWidth()-2*nCtrlStartX
	local nExprWidth = nExprOutpWidth
	local nOutpWidth = nExprOutpWidth
	local nTotalExprOutpHeight = wndMain:GetHeight()-242
	local nExprHeight = 0.35*nTotalExprOutpHeight
	local nOutpHeight = nTotalExprOutpHeight-nExprHeight

	lblLevel:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+lblLevel:GetWidth()
	ltbLevel:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+ltbLevel:GetWidth()
	lblToLevel:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+lblToLevel:GetWidth()
	ltbToLevel:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+ltbToLevel:GetWidth()+nCtrlSpacingX
	lblNorC:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+lblNorC:GetWidth()
	ltbNorC:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+ltbNorC:GetWidth()

	nCtrlCursorX = nCtrlStartX
	nCtrlCursorY = nCtrlCursorY+ltbLevel:GetHeight()+nCtrlSpacingY

	lblExpression:SetPosition(nCtrlCursorX,nCtrlCursorY)
	lblExpression:SetWidth(nExprWidth)
	nCtrlCursorY = nCtrlCursorY+lblExpression:GetHeight()
	cntExpression:SetPosition(nCtrlCursorX,nCtrlCursorY)
	cntExpression:SetSize(nExprWidth,nExprHeight)
	cnt2Expression:SetPosition(nCtrlCursorX+1,nCtrlCursorY)
	cnt2Expression:SetSize(nExprWidth-2,nExprHeight-1)
	ltbExpression:SetPosition(nCtrlCursorX+1,nCtrlCursorY)
	ltbExpression:SetSize(nExprWidth-11,nExprHeight-1)
    lsbExpression:SetPosition(nCtrlCursorX+nExprWidth-11,nCtrlCursorY)
	lsbExpression:SetHeight(nExprHeight)
	nCtrlCursorY = nCtrlCursorY+ltbExpression:GetHeight()+nCtrlSpacingY

	lbtnExecute:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+lbtnExecute:GetWidth()+nCtrlSpacingX
	lbtnClear:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+lbtnClear:GetWidth()+nCtrlSpacingX
	lbtnHelp:SetPosition(nCtrlCursorX,nCtrlCursorY)
	nCtrlCursorX = nCtrlCursorX+lbtnHelp:GetWidth()+nCtrlSpacingX

	nCtrlCursorX = nCtrlStartX
	nCtrlCursorY = nCtrlCursorY+lbtnExecute:GetHeight()+nCtrlSpacingY

	lblOutput:SetPosition(nCtrlCursorX,nCtrlCursorY)
	lblOutput:SetWidth(nOutpWidth)
	nCtrlCursorY = nCtrlCursorY+lblOutput:GetHeight()
	cntOutput:SetPosition(nCtrlCursorX,nCtrlCursorY)
	cntOutput:SetSize(nOutpWidth,nOutpHeight)
	cnt2Output:SetPosition(nCtrlCursorX+1,nCtrlCursorY)
	cnt2Output:SetSize(nOutpWidth-2,nOutpHeight-1)
	ltbOutput:SetPosition(nCtrlCursorX+1,nCtrlCursorY)
	ltbOutput:SetSize(nOutpWidth-11,nOutpHeight-1)
    lsbOutput:SetPosition(nCtrlCursorX+nOutpWidth-11,nCtrlCursorY)
	lsbOutput:SetHeight(nOutpHeight)
	nCtrlCursorY = nCtrlCursorY+ltbOutput:GetHeight()+nCtrlSpacingY
end

Player = Turbine.Gameplay.LocalPlayer.GetInstance()
PlayerAttrib = Player:GetAttributes()

-- create main window + controls
wndMain = Turbine.UI.Lotro.Window()
wndMain:SetSize(850,750)
local displayWidth, displayHeight = Turbine.UI.Display.GetSize()
local windowWidth, windowHeight = wndMain:GetSize()
wndMain:SetPosition((displayWidth-windowWidth)/2,(displayHeight-windowHeight)/2)
wndMain:SetText("CalcStat Expressions")

local nLabelFont = Turbine.UI.Lotro.Font.Verdana18
local nLabelHeight = 22
local nTextBoxFont = Turbine.UI.Lotro.Font.Verdana18
local nTextBoxHeight = 22

lblLevel = Turbine.UI.Label()
lblLevel:SetSize(75,nLabelHeight)
lblLevel:SetFont(nLabelFont)
lblLevel:SetText("$L(evel):")
lblLevel:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
lblLevel:SetParent(wndMain)
lblLevel:SetVisible(true)
ltbLevel = Turbine.UI.Lotro.TextBox()
ltbLevel:SetSize(40,nTextBoxHeight)
ltbLevel:SetFont(nTextBoxFont)
ltbLevel:SetMultiline(false)
ltbLevel:SetParent(wndMain)
ltbLevel:SetVisible(true)
function lblLevel:MouseClick(sender, args) ltbLevel:Focus() end
lblToLevel = Turbine.UI.Label()
lblToLevel:SetSize(40,nLabelHeight)
lblToLevel:SetFont(nLabelFont)
lblToLevel:SetText("to")
lblToLevel:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
lblToLevel:SetParent(wndMain)
lblToLevel:SetVisible(true)
ltbToLevel = Turbine.UI.Lotro.TextBox()
ltbToLevel:SetSize(40,nTextBoxHeight)
ltbToLevel:SetFont(nTextBoxFont)
ltbToLevel:SetMultiline(false)
ltbToLevel:SetParent(wndMain)
ltbToLevel:SetVisible(true)
function lblToLevel:MouseClick(sender, args) ltbToLevel:Focus() end

lblNorC = Turbine.UI.Label()
lblNorC:SetSize(82,nLabelHeight)
lblNorC:SetFont(nLabelFont)
lblNorC:SetText("$N or $C:")
lblNorC:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
lblNorC:SetParent(wndMain)
lblNorC:SetVisible(true)
ltbNorC = Turbine.UI.Lotro.TextBox()
ltbNorC:SetSize(200,nTextBoxHeight)
ltbNorC:SetFont(nTextBoxFont)
ltbNorC:SetMultiline(false)
ltbNorC:SetParent(wndMain)
ltbNorC:SetVisible(true)
function lblNorC:MouseClick(sender, args) ltbNorC:Focus() end

lblExpression = Turbine.UI.Label()
lblExpression:SetHeight(nLabelHeight+6)
lblExpression:SetFont(nLabelFont)
lblExpression:SetBackColor(Turbine.UI.Color(0.1,0.15,0.35))
lblExpression:SetText("Enter Expression")
lblExpression:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
lblExpression:SetParent(wndMain)
lblExpression:SetVisible(true)
cntExpression = Turbine.UI.Control()
cntExpression:SetBackColor(Turbine.UI.Color(0.1,0.15,0.35))
cntExpression:SetParent(wndMain)
cntExpression:SetMouseVisible(false)
cnt2Expression = Turbine.UI.Control()
cnt2Expression:SetBackColor(Turbine.UI.Color.Black)
cnt2Expression:SetParent(wndMain)
cnt2Expression:SetMouseVisible(false)
ltbExpression = Turbine.UI.TextBox()
ltbExpression:SetSelectable(true)
ltbExpression:SetFont(nTextBoxFont)
ltbExpression:SetParent(wndMain)
ltbExpression:SetVisible(true)
function lblExpression:MouseClick(sender, args) ltbExpression:Focus() end
lsbExpression = Turbine.UI.Lotro.ScrollBar()
lsbExpression:SetOrientation(Turbine.UI.Orientation.Vertical)
lsbExpression:SetParent(wndMain)
lsbExpression:SetVisible(true)
lsbExpression:SetWidth(10)
ltbExpression:SetVerticalScrollBar(lsbExpression)

lbtnExecute = Turbine.UI.Lotro.Button()
lbtnExecute:SetWidth(120)
lbtnExecute:SetText("Run Expression")
lbtnExecute:SetParent(wndMain)
lbtnExecute:SetVisible(true)
function lbtnExecute:MouseClick(sender, args) ExecuteMyExpression() end

lbtnClear = Turbine.UI.Lotro.Button()
lbtnClear:SetWidth(100)
lbtnClear:SetText("Clear")
lbtnClear:SetParent(wndMain)
lbtnClear:SetVisible(true)
function lbtnClear:MouseClick(sender, args) ClearMyExpression() end

lbtnHelp = Turbine.UI.Lotro.Button()
lbtnHelp:SetWidth(100)
lbtnHelp:SetText("Help")
lbtnHelp:SetParent(wndMain)
lbtnHelp:SetVisible(true)
function lbtnHelp:MouseClick(sender, args) ShowHelpOutput() end

lblOutput = Turbine.UI.Label()
lblOutput:SetHeight(nLabelHeight+6)
lblOutput:SetFont(nLabelFont)
lblOutput:SetBackColor(Turbine.UI.Color(0.1,0.15,0.35))
lblOutput:SetText("Result")
lblOutput:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
lblOutput:SetParent(wndMain)
lblOutput:SetVisible(true)
cntOutput = Turbine.UI.Control()
cntOutput:SetBackColor(Turbine.UI.Color(0.1,0.15,0.35))
cntOutput:SetParent(wndMain)
cntOutput:SetMouseVisible(false)
cnt2Output = Turbine.UI.Control()
cnt2Output:SetBackColor(Turbine.UI.Color.Black)
cnt2Output:SetParent(wndMain)
cnt2Output:SetMouseVisible(false)
ltbOutput = Turbine.UI.TextBox()
ltbOutput:SetSelectable(true)
ltbOutput:SetFont(nTextBoxFont)
ltbOutput:SetBackColor(Turbine.UI.Color.Black)
ltbOutput:SetParent(wndMain)
ltbOutput:SetText("This plugin doesn't need to be loaded for functioning calculation services to other plugins like TitanBar.")
ltbOutput:SetVisible(true)
function lblOutput:MouseClick(sender, args) ltbOutput:Focus() end
lsbOutput = Turbine.UI.Lotro.ScrollBar()
lsbOutput:SetOrientation(Turbine.UI.Orientation.Vertical)
lsbOutput:SetParent(wndMain)
lsbOutput:SetVisible(true)
lsbOutput:SetWidth(10)
ltbOutput:SetVerticalScrollBar(lsbOutput)

wndMain:SetResizable(true)
function wndMain:SizeChanged(sender, args) if not bStartUp then SetLayout() end end

SetLayout()

wndMain:SetVisible(true)

cmdCalcStat = Turbine.ShellCommand()
function cmdCalcStat:Execute(command, args)
	if type(args) == "string" then
		local sArgs = args:upper()
		if sArgs == "SHOW" then
			wndMain:SetVisible(true)
		elseif sArgs == "HIDE" then
			wndMain:SetVisible(false)
		else
			Turbine.Shell.WriteLine("no valid option provided")
			Turbine.Shell.WriteLine("usage: /calcstat show|hide")
		end
	end
end
Turbine.Shell.AddCommand("calcstat",cmdCalcStat)

Turbine.Shell.WriteLine("CalcStat version "..CalcStat("-Version",0).." loaded.")
Turbine.Shell.WriteLine("usage: /calcstat show|hide")

bStartUp = false