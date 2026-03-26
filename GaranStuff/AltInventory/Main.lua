-- need to translate itemExplorer Quality, Durability, and Sort lists to FR and DE

-- bug - in zoom>1, with display mode 1 (text only), scrolling will cause the icons to show back up
-- bug - in zoom>1, clicking a display mode other than text only will cause the icons to display outside the main window - they aren't getting clipped/cropped until scrolled
-- bug - undocked windows in zoom>1, display mode 1(text only) icons are always showing :(
-- bug - undocked windows, clicking a display mode does not relayout panel (all display modes, all zooms)
-- a zoomed icon is showing up in the top left of the main screen, seems to be the first item equipped (even from regular inventory window) when zoom>1 
-- -- only seems to affect items that are defined for an undocked window
-- -- perhaps a test control for sizing?

-- finish testing displayMode with undocked windows
-- -- cycling through display modes by toggle doesn't resize itemlist properly (number of columns gets out of wack? in icon only mode?)
-- something odd happened with zoomed icons with new displayMode, needs testing
-- undocked windows are not updating correctly when showing alt bags - example, create a floating window for ore, it won't show the selected chars ore (and that ore will still show in the main window). getDisplayTabIndexFromName not working when char~=player?

-- ToDo...

-- add indicator that inventory is not current bag? perhaps change border color from gold to something else on undocked windows.

-- apply text size to panel headers
-- apply text size to inventoryPanel labels and fields - gonna be challenging since there are some fixed height graphics
-- apply text size to minimumWindow title bar - will change the way minimumWindow positions relative to inventoryPanel...

importPath=string.gsub(getfenv(1)._.Name,"%.Main","").."."
resourcePath=string.gsub(importPath,"%.","/").."Resources/"

--*****************************************************************************************************
-- ToDo
-- verify that new panels are using Settings - note, default panel back color is kind of ugly

-- allow user to change font face/size. Major overhaul... not sure how changing the font size of ItemEntry objects will impact the pane.ItemList listboxes
-- -- will need to account for font size in minimum width of entries

-- need to update Equipped Items mannequin when equipped items change AND the current player EI panel is currently displayed (it gets refreshed when selected)
-- -- perhaps based on a timer? there doesn't seem to be a WearStateChanged() event for items

-- carry-all messages
--  Gathered [2 Chunks of Minas Ithil Skarn] into the Large Crafting Carry-all.
-- Item removed from Large Crafting Carry-all: [2 Chunks of Minas Ithil Skarn].
-- would be nice if ALL adding/removing caused messages

-- should add events for capacity changes but the next refresh will automatically adjust anyway
--*****************************************************************************************************

-- this file contains the initialization, inventory maintenance and event handling, settings and data save/load functionality
-- I may at some point remerge the itemtracker functionality since 75% or more of the functionality is actually replicated here
-- the display functionality is all handled in the inventoryWindow.lua file

-- the quantities are stored in the accountItemQty structure:
-- -- NOTE itemName is the ItemName, not ItemInfoName so a custom named item will have its own inventory
-- -- it would be MUCH better if we could use ItemID as the key, but, alas, that is not programatically available :(
-- -- accountItemQty[itemName].InfoName = generic name from ItemInfo - this will be the same as the key unless the item has a custom name (the key)
-- -- accountItemQty[itemName].ItemGID = generic itemID (0x00000000) -- currently not available - now that the ItemExplorer database is updated to U37, who knows... unfortunately, matching on item InfoName is not ideal as it is not unique :(
-- -- accountItemQty[itemName].ItemUID = server unique itemID (0x0000000000000000) -- currently not available
-- -- accountItemQty[itemName].Total = total for this item
-- -- accountItemQty[itemName].Category = the category code for this item, used for filtering
-- -- accountItemQty[itemName].SortCategory = the category code for this item, used for sorting in case the Category code is not yet defined
-- -- accountItemQty[itemName].Quality = the quality code for this item, used for sorting
-- -- accountItemQty[itemName].BackgroundImageID = image resource ID for the icon background. used to display an icon for items not currently accessible by the current character
-- -- accountItemQty[itemName].IconImageID = image resource ID for the icon foreground. used to display an icon for items not currently accessible by the current character
-- -- accountItemQty[itemName].ListItem[storage] = handle to an entry in inventoryPanel.ItemList representing this item
-- -- accountItemQty[itemName].ListItem[charName][slot] = handle to an entry in inventoryPanel.ItemList representing this item
-- -- accountItemQty[name].Qty[storage].Subtotal = quantity in the specified storage
-- -- accountItemQty[name].Qty[storage][chest/bag] = detail quantity in the specified chest/bag of the specified storage
-- -- accountItemQty[name].Qty[charName][slot] = quantity in the specified slot of the specified player bags

-- ?????????????????????????????????? how to deal with enhancement runes that all have the same name but different qualities=different items
-- each time we encounter an item, test the quality against the existing item quality and if the quality is different, then set the qty by quality flag to true
-- -- accountItemQty[name].QtyByQuality=[true|false] flag whether quantity is tracked by quality - this is needed for items like enhancement runes
-- -- accountItemQty[name].Q[quality].Total = quantity in the specified storage
-- -- accountItemQty[name].Q[quality].Qty[storage].Subtotal = quantity in the specified storage
-- -- accountItemQty[name].Q[quality].Qty[storage][chest/bag] = detail quantity in the specified chest/bag of the specified storage
-- -- accountItemQty[name].Q[quality].Qty[charName][slot] = quantity in the specified slot of the specified player bags

-- Equipped Items data
-- -- equippedItems[charName][slot] - this uses a seperate table so as not to ever corrupt the accountItemQty table if SSG ever breaks EquippedItems again
-- -- equippedItems[charName][slot].Name=item name
-- -- equippedItems[charName][slot].WearState=item wear state (if available, used for mannequin)
-- -- equippedItems[charName][slot].BackgroundImageID = image resource ID for the icon background. used to display an icon for items not currently accessible by the current character
-- -- equippedItems[charName][slot].IconImageID = image resource ID for the icon foreground. used to display an icon for items not currently accessible by the current character
-- use Settings.enableEI to disable equipped items in case SSG ever breaks it again

-- turbine client imports
import "Turbine";
import "Turbine.Gameplay";
import "Turbine.UI";
import "Turbine.UI.Lotro";
-- need localPlayer to test for session play
localPlayer=Turbine.Gameplay.LocalPlayer.GetInstance();
charName=localPlayer:GetName()
charVaultName=charName.." (Vault)"
charEIName=charName.." (EI)"

-- immediately test for session play BEFORE importing any other files so that we can safely bail without loading any data and without any handlers
if string.sub(charName,1,1)=="~" then
	-- do not load if the character is a session play character
	Turbine.Shell.WriteLine("Alt Inventory does not support session play characters.");
	error("Alt Inventory does not support session play characters.",2)
	return
end

string.ltrim=function(str)
	return (string.gsub(str, "^%s*(.-)", "%1"))
end
string.rtrim=function(str)
	return (string.gsub(str, "^(.-)%s*$", "%1"))
end
string.trim=function(str)
	return (string.gsub(str, "^%s*(.-)%s*$", "%1"))
end
replaceStr={[128]="A",[129]="A",[130]="A",[131]="A",[132]="A",[133]="A",[134]="AE",[135]="C",[136]="E",[137]="E",[138]="E",[139]="E",[140]="I",[141]="I",[142]="I",[143]="I",[144]="",[145]="N",[146]="O",[147]="O",[148]="O",[149]="O",[150]="O",[151]="",[152]="O",[153]="U",[154]="U",[155]="U",[156]="U",[157]="Y",[158]="Y",[159]="sz",[160]="a",[161]="a",[162]="a",[163]="a",[164]="a",[165]="a",[166]="ae",[167]="c",[168]="e",[169]="e",[170]="e",[171]="e",[172]="i",[173]="i",[174]="i",[175]="i",[176]="o",[177]="n",[178]="o",[179]="o",[180]="o",[181]="o",[182]="o",[183]="",[184]="o",[185]="u",[186]="u",[187]="u",[188]="u",[189]="y",[190]="y",[191]="y"}
string.stripaccent=function(str)
	local ret = "";
	local replace=false
	for i,v in ipairs({str:byte(1,-1)}) do
		if replace then
			replace=false
			if replaceStr[v]~=nil then
				ret=ret..replaceStr[v]
			end
		else
			if (v==195) then
				replace=true
			else
				ret = ret .. string.char(v);
			end
		end
	end
	return ret;
end
string.isPattern=function(str)
	str=string.stripaccent(str) -- need to ignore accented chars
	local ret=false
	for i,v in ipairs({str:byte(1,-1)}) do
		if v~=32 and (v<65 or v>90) and (v<97 or v>122) then
			ret=true
			break
		end
	end
	return ret
end
function getContrastedBackgroundColor(backPercent)
	if backPercent==nil then backPercent=.9 end
	if backPercent>1 then backPercent=1 end
	if backPercent<0 then backPercent=0 end
	local forePercent=1-backPercent
	-- calculates a usable background color by blending a % of the text color (default 10%) with a % of the backcolor.(default 90%) this yields a control background that is very similar to the background but with enough contrast that borderless controls are clearly visible
	local A=Settings.backColor.A*backPercent+Settings.fontColor.A*forePercent
	local R=Settings.backColor.R*backPercent+Settings.fontColor.R*forePercent
	local G=Settings.backColor.G*backPercent+Settings.fontColor.G*forePercent
	local B=Settings.backColor.B*backPercent+Settings.fontColor.B*forePercent
	return Turbine.UI.Color(A,R,G,B)
end
import (importPath.."Class")
import (importPath.."VindarPatch")
-- This is the Load/Save patch which accounts for European formats, written by Vindar
-- load resource strings - this MUST occur before any windows that use strings get loaded or there will be no defaults
import (importPath.."ColorPicker")
import (importPath.."Strings")
import (importPath.."Table")
import (importPath.."RadioButtonGroup")
import (importPath.."DropDownList")
import (importPath.."DebugWindow")
import (importPath.."FontSupport")
import (importPath.."PopUpDialog")
import (importPath.."ScalableButton")
language=1;
euroFormat=(tonumber("1,000")==1);
if (tonumber("1,000")==1) then
	function euroNormalize(value)
		local ret
		if value~=nil then
			ret=tonumber((string.gsub(value,"%.",",")));
		end
		return ret
	end
else
	function euroNormalize(value)
		local ret
		if value~=nil then
			ret=tonumber((string.gsub(value,",",".")))
		end
		return ret
	end
end

locale = "en";
if Turbine.Shell.IsCommand("hilfe") then
	locale = "de";
	language=3;
elseif Turbine.Shell.IsCommand("aide") then
	locale = "fr";
	language=2;
end

groupMaintVisible=false
setupVisible=false;
inventoryVisible=false;
minimalVisible=false;
iconVisible=false;
itemExplorerVisible=false
itemInfoDetailVisible=false
hudVisible=true; -- assume hud is visible when loading plugins -- I hate this assumption, but Turbine has failed to provide a legitimate means of testing the hud state
moveToggle=false;
displayWidth=Turbine.UI.Display:GetWidth();
displayHeight=Turbine.UI.Display:GetHeight();

Settings=PatchDataLoad(Turbine.DataScope.Character, "AltInventorySettings");
if Settings==nil then
	Settings={}
end
if Settings.showIcon==nil then
	Settings.showIcon=1
else
	if type(Settings.showIcon)=="boolean" then
		if Settings.showIcon then
			Settings.showIcon=1
		else
			Settings.showIcon=0
		end
	end
end
if Settings.enableEI==nil then Settings.enableEI=true end -- for now, default to true
if Settings.itemExplorerSearchThrottle==nil then Settings.itemExplorerSearchThrottle=100 end
if Settings.zoom==nil then Settings.zoom=1 end
--*** temporarilly disable zoom
if Settings.zoom>1 then
--	PopUpDialog(Resource[language][55],Resource[language][98],1,Resource[language][61],nil,nil,false)
end
--Settings.zoom=1 -- remove this once zoom is fixed
clientLanguage=language -- need to track the actual client language for system generated chat messages
if Settings.language==nil then Settings.language=0 end
if Settings.language>0 then language=Settings.language end
if Settings.debug==nil then Settings.debug=false end;
Settings.debug=false; -- don't allow persistent debugging (to debug initialization this has to be temporarily changed to true)
if Settings.fontColor==nil then
	Settings.fontColor=Turbine.UI.Color(1,.9,.5)
else
	Settings.fontColor=Turbine.UI.Color(euroNormalize(Settings.fontColor.R),euroNormalize(Settings.fontColor.G),euroNormalize(Settings.fontColor.B))
end
if Settings.errorColor==nil then
	Settings.errorColor=Turbine.UI.Color(1,0,0)
else
	Settings.errorColor=Turbine.UI.Color(euroNormalize(Settings.errorColor.R),euroNormalize(Settings.errorColor.G),euroNormalize(Settings.errorColor.B))
end

if Settings.trimColor==nil then
	Settings.trimColor=Turbine.UI.Color(.4,.4,.5)
else
	Settings.trimColor=Turbine.UI.Color(euroNormalize(Settings.trimColor.R),euroNormalize(Settings.trimColor.G),euroNormalize(Settings.trimColor.B))
end
if Settings.backColor==nil then
	Settings.backColor=Turbine.UI.Color(.05,.05,.05)
else
	Settings.backColor=Turbine.UI.Color(euroNormalize(Settings.backColor.R),euroNormalize(Settings.backColor.G),euroNormalize(Settings.backColor.B))
end
if Settings.panelBackColor==nil then
	Settings.panelBackColor=Turbine.UI.Color(.25,.75,.45)
else
	Settings.panelBackColor=Turbine.UI.Color(euroNormalize(Settings.panelBackColor.R),euroNormalize(Settings.panelBackColor.G),euroNormalize(Settings.panelBackColor.B))
end
if Settings.headingsColor==nil then
	Settings.headingsColor=Turbine.UI.Color(1,1,1)
else
	Settings.headingsColor=Turbine.UI.Color(euroNormalize(Settings.headingsColor.R),euroNormalize(Settings.headingsColor.G),euroNormalize(Settings.headingsColor.B))
end
if Settings.listTextColor==nil then
	Settings.listTextColor=Turbine.UI.Color(1,1,1)
else
	Settings.listTextColor=Turbine.UI.Color(euroNormalize(Settings.listTextColor.R),euroNormalize(Settings.listTextColor.G),euroNormalize(Settings.listTextColor.B))
end
if Settings.critColor==nil then
	Settings.critColor=Turbine.UI.Color(.9,0,0)
else
	Settings.critColor=Turbine.UI.Color(euroNormalize(Settings.critColor.R),euroNormalize(Settings.critColor.G),euroNormalize(Settings.critColor.B))
end
if Settings.lowColor==nil then
	Settings.lowColor=Turbine.UI.Color(.8,.5,0)
else
	Settings.lowColor=Turbine.UI.Color(euroNormalize(Settings.lowColor.R),euroNormalize(Settings.lowColor.G),euroNormalize(Settings.lowColor.B))
end
if Settings.highColor==nil then
	Settings.highColor=Turbine.UI.Color(0,.8,0)
else
	Settings.highColor=Turbine.UI.Color(euroNormalize(Settings.highColor.R),euroNormalize(Settings.highColor.G),euroNormalize(Settings.highColor.B))
end

if Settings.fontFace==nil then Settings.fontFace=Turbine.UI.Lotro.Font.Verdana20 end
if Settings.opacity==nil then Settings.opacity=1 end
if Settings.lowQty==nil then Settings.lowQty=10 end
if Settings.critQty==nil then Settings.critQty=5 end
if Settings.highQty==nil then Settings.highQty=100 end
if Settings.lowShow==nil then Settings.lowShow=true end
if Settings.critShow==nil then Settings.critShow=true end
if Settings.normalShow==nil then Settings.normalShow=true end
if Settings.highShow==nil then Settings.highShow=true end
if Settings.showTotals==nil then Settings.showTotals=true end
if Settings.showCurrent==nil then Settings.showCurrent=true end
if Settings.outlineQtyText==nil then Settings.outlineQtyText=true end
if Settings.loadMinimized==nil then Settings.loadMinimized=false end
if Settings.useMinimalHeader==nil then Settings.useMinimalHeader=false end
if Settings.bagSeparator then Settings.bagSeparator=false end
if Settings.panelViewMode==nil then Settings.panelViewMode=2 end
if Settings.useMiniIcon==nil then Settings.useMiniIcon=false end
if Settings.defaultToAllView==nil then Settings.defaultToAllView=false end

if Settings.panelWidth==nil then
	Settings.panelWidth=406/displayWidth;
else
	Settings.panelWidth=euroNormalize(Settings.panelWidth);
	if Settings.panelWidth+20/displayWidth>1 then Settings.panelWidth=1-20/displayWidth end
end

if Settings.panelHeight==nil then
	Settings.panelHeight=114/displayHeight;
else
	Settings.panelHeight=euroNormalize(Settings.panelHeight);
	if Settings.panelHeight+20/displayHeight>1 then Settings.panelHeight=1-20/displayHeight end
end
Settings.panelMinWidth=406/displayWidth;
Settings.panelMinHeight=114/displayHeight;

if Settings.panelLeft==nil then
	Settings.panelLeft=.5-Settings.panelWidth/2;
else
	Settings.panelLeft=euroNormalize(Settings.panelLeft);
	if Settings.panelLeft+Settings.panelWidth>1 then Settings.panelLeft=1-Settings.panelWidth end
	if Settings.panelLeft<0 then Settings.panelLeft=0 end -- don't allow panel off left or top of screen
end
if Settings.panelTop==nil then
	Settings.panelTop=.5-Settings.panelHeight/2;
else
	Settings.panelTop=euroNormalize(Settings.panelTop);
	if Settings.panelTop+Settings.panelHeight>1 then Settings.panelTop=1-Settings.panelHeight end
	if Settings.panelTop<0 then Settings.panelTop=0 end
end

if Settings.replaceBags==nil then Settings.replaceBags=false end
if Settings.replaceBags then
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack1, false );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack2, false );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack3, false );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack4, false );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack5, false );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack6, false );
end
if Settings.totalsOnly==nil then Settings.totalsOnly=false end


CharList=PatchDataLoad(Turbine.DataScope.Server, "AltInventoryCharList");
if CharList==nil then
	CharList={};
	CharList["Shared Storage"]={};
end
if CharList[charName]==nil then
	CharList[charName]={};
end
if CharList[charVaultName]==nil then
	CharList[charVaultName]={};
end
if Settings.enableEI then
	if CharList[charEIName]==nil then
		CharList[charEIName]={}
	end
end
-- now, check for orphaned EI records and remove them - NOTE, this does NOT affect inventory as EI entries are not counted toward totals
for k,v in pairs(CharList) do
	if string.find(k,"(EI)")~=nil then
		local tmpName=string.match(k,"(.*) %(EI%)")
--		Turbine.Shell.WriteLine("k:"..tostring(k).." Name:"..tostring(tmpName))
		if CharList[tmpName]==nil then
			CharList[k]=nil
		end
	end
end

colorGold=Turbine.UI.Color(.80,.60,.1);
colorSilver=Turbine.UI.Color(.75,.75,.75);
colorDarkGrey=Turbine.UI.Color(.1,.1,.1);
Tab1TabTrimColor=Turbine.UI.Color(.5,.5,.8);
Tab1TabBackColor=Turbine.UI.Color(.1,.1,.15);
Tab2TabTrimColor=Turbine.UI.Color(.8,.5,.5);
Tab2TabBackColor=Turbine.UI.Color(.15,.1,.1);
Tab3TabTrimColor=Turbine.UI.Color(.8,.8,.85);
Tab3TabBackColor=Turbine.UI.Color(.15,.15,.15);
errorColor=Turbine.UI.Color(1,.2,.2);
successColor=Turbine.UI.Color(.2,1,.2);
fontMetric=FontMetric();
fontMetric:SetFont(Settings.fontFace);

-- the Turbine Quality enumeration is NOT ordered correctly :(
function GetQualityCode(quality)
	local tmpVal=quality
	if tmpVal==2 then
		tmpVal=3
	elseif tmpVal==3 then
		tmpVal=2
	end
	return tmpVal;
end
function GetQualityColor(qualityCode)
	local tmpColor=Turbine.UI.Color.White
	if qualityCode==1 then
		tmpColor=Turbine.UI.Color.Gold
	elseif qualityCode==2 then
		tmpColor=Turbine.UI.Color.Aqua
	elseif qualityCode==3 then
		tmpColor=Turbine.UI.Color.DarkMagenta
	elseif qualityCode==4 then
		tmpColor=Turbine.UI.Color.Yellow
	end
	return tmpColor		
end
function GetCategoryCode(category)
	-- for now just return the Turbine category
	return category;
end

-- This is the callback mechanism provided by Pengoros, slightly modified to guarantee uniqueness
function AddCallback(object, event, callback)
	if (object[event] == nil) then
		object[event] = callback;
	else
		if (type(object[event]) == "table") then
			local exists=false;
			local k,v;
			for k,v in ipairs(object[event]) do
				if v==callback then
					exists=true;
					break;
				end
			end
			if not exists then
				table.insert(object[event], callback);
			end
		else
			if object[event]~=callback then
				object[event] = {object[event], callback};
			end
		end
	end
	return callback;
end

-- safely remove a callback without clobbering any extras
function RemoveCallback(object, event, callback)
    if (object[event] == callback) then
        object[event] = nil;
    else
        if (type(object[event]) == "table") then
            local size = table.getn(object[event]);
            for i = 1, size do
                if (object[event][i] == callback) then
                    table.remove(object[event], i);
                    break;
                end
            end
        end
    end
end

defaultTabs=PatchDataLoad(Turbine.DataScope.Account, "AltInventoryDefaultTabs")

displayTabs=PatchDataLoad(Turbine.DataScope.Character, "AltInventoryDisplayTabs")
if displayTabs==nil then
	-- create default tab
	displayTabs={}
end
-- create any missing container definitions
if displayTabs.bags==nil then
	displayTabs.bags={}
	displayTabs.bags[1]={}
	displayTabs.bags[1].name="Main"
	displayTabs.bags[1].isMain=true -- indicates that this is the main "catchall" and can NOT be deleted and has no defined criteria
	displayTabs.bags[1].docked=true
	displayTabs.bags[1].criteria={}
	displayTabs.bags[1].criteria.UIDValues={}
	displayTabs.bags[1].criteria.GIDValues={}
	displayTabs.bags[1].criteria.ItemNames={}
	displayTabs.bags[1].criteria.ItemInfoNames={}
	displayTabs.bags[1].criteria.CIDValues={}
end
if displayTabs.all==nil then
	displayTabs.all={}
	displayTabs.all.useBagTags=false -- if set to true then the rest of the displayTabs.all subtable is ignored and displayTabs.bags is used
	displayTabs.all[1]={}
	displayTabs.all[1].name="Main"
	displayTabs.all[1].isMain=true -- indicates that this is the main "catchall" and can NOT be deleted and has no defined criteria
	displayTabs.all[1].docked=true
	displayTabs.all[1].criteria={}
	displayTabs.all[1].criteria.UIDValues={}
	displayTabs.all[1].criteria.GIDValues={}
	displayTabs.all[1].criteria.ItemNames={}
	displayTabs.all[1].criteria.ItemInfoNames={}
	displayTabs.all[1].criteria.CIDValues={}
end
if displayTabs.vault==nil then
	displayTabs.vault={}
	displayTabs.vault.useBagTags=false -- if set to true then the rest of the displayTabs.vault subtable is ignored and displayTabs.bags is used
	displayTabs.vault[1]={}
	displayTabs.vault[1].name="Main"
	displayTabs.vault[1].isMain=true -- indicates that this is the main "catchall" and can NOT be deleted and has no defined criteria
	displayTabs.vault[1].docked=true
	displayTabs.vault[1].criteria={}
	displayTabs.vault[1].criteria.UIDValues={}
	displayTabs.vault[1].criteria.GIDValues={}
	displayTabs.vault[1].criteria.ItemNames={}
	displayTabs.vault[1].criteria.ItemInfoNames={}
	displayTabs.vault[1].criteria.CIDValues={}
end
if displayTabs.shared==nil then
	displayTabs.shared={}
	displayTabs.shared.useBagTags=false -- if set to true then the rest of the displayTabs.shared subtable is ignored and displayTabs.bags is used
	displayTabs.shared[1]={}
	displayTabs.shared[1].name="Main"
	displayTabs.shared[1].isMain=true -- indicates that this is the main "catchall" and can NOT be deleted and has no defined criteria
	displayTabs.shared[1].docked=true
	displayTabs.shared[1].criteria={}
	displayTabs.shared[1].criteria.UIDValues={}
	displayTabs.shared[1].criteria.GIDValues={}
	displayTabs.shared[1].criteria.ItemNames={}
	displayTabs.shared[1].criteria.ItemInfoNames={}
	displayTabs.shared[1].criteria.CIDValues={}
end
-- verify displayTabs criteria integrity - each unique itemID can only exist in one tab per container, each global itemID on one tab per container, each category id on one tab per container, etc
for tmpContainerType,tmpContainer in pairs(displayTabs) do
	if tmpContainerType=="bags" or tmpContainerType=="vault" or tmpContainerType=="shared" or tmpContainerType=="all" then
		local mainIndex=nil
		-- first make sure that "main" is unique and sort the criteria values in each group - they should always be sorted anyway, but this makes sure
		for tmpGroupIndex,tmpGroup in pairs(tmpContainer) do
			if type(tmpGroup)=="table" then
				if tmpGroup.isMain then
					if mainIndex==nil then
						mainIndex=tmpGroupIndex
						tmpGroup.criteria.ItemNames={}
						tmpGroup.criteria.ItemInfoNames={}
						tmpGroup.criteria.UIDValues={}
						tmpGroup.criteria.GIDValues={}
						tmpGroup.criteria.CIDValues={}
					else
						-- we have multiple Groups flagged as "Main", convert all additional Groups to not main
						tmpGroup.isMain=false
						if tmpGroup.criteria.ItemNames==nil then tmpGroup.criteria.ItemNames={} end
						if tmpGroup.criteria.ItemInfoNames==nil then tmpGroup.criteria.ItemInfoNames={} end
						if tmpGroup.criteria.UIDValues==nil then tmpGroup.criteria.UIDValues={} end
						if tmpGroup.criteria.GIDValues==nil then tmpGroup.criteria.GIDValues={} end
						if tmpGroup.criteria.CIDValues==nil then tmpGroup.criteria.CIDValues={} end
						table.sort(tmpGroup.criteria.ItemNames)
						table.sort(tmpGroup.criteria.ItemInfoNames)
						table.sort(tmpGroup.criteria.UIDValues)
						table.sort(tmpGroup.criteria.GIDValues)
						table.sort(tmpGroup.criteria.CIDValues)
					end
				else
					if tmpGroup.criteria==nil then tmpGroup.criteria={} end
					if tmpGroup.criteria.ItemNames==nil then tmpGroup.criteria.ItemNames={} end
					if tmpGroup.criteria.ItemInfoNames==nil then tmpGroup.criteria.ItemInfoNames={} end
					if tmpGroup.criteria.UIDValues==nil then tmpGroup.criteria.UIDValues={} end
					if tmpGroup.criteria.GIDValues==nil then tmpGroup.criteria.GIDValues={} end
					if tmpGroup.criteria.CIDValues==nil then tmpGroup.criteria.CIDValues={} end
					table.sort(tmpGroup.criteria.ItemNames)
					table.sort(tmpGroup.criteria.ItemInfoNames)
					table.sort(tmpGroup.criteria.UIDValues)
					table.sort(tmpGroup.criteria.GIDValues)
					table.sort(tmpGroup.criteria.CIDValues)
				end
			end
		end
		if mainIndex==nil then
			-- Yikes! we have no "Main" tab, assign the first entry as "Main" (or create one if no entries)
			if #tmpContainer==0 then
				tmpContainer[1]={}
				tmpContainer[1].name="Main"
			end
			tmpContainer[1].isMain=true
			tmpContainer[1].docked=true
			tmpContainer[1].criteria={}
			tmpContainer[1].criteria.ItemNames={}
			tmpContainer[1].criteria.ItemInfoNames={}
			tmpContainer[1].criteria.UIDValues={}
			tmpContainer[1].criteria.GIDValues={}
			tmpContainer[1].criteria.CIDValues={}
		end
		-- now make sure they are unique
		for tmpGroupIndex,tmpGroup in ipairs(tmpContainer) do
			if type(tmpGroup)=="table" then
				if not tmpGroup.isMain then
					for key,tmpCriteria in pairs(tmpGroup.criteria) do
						if type(tmpCriteria)=="table" then
							for tmpValIndex,tmpValue in ipairs(tmpCriteria) do
								-- first make sure it is unique within this critera list
								local k=tmpValIndex+1
								local numVals=#tmpCriteria -- # is inherently inefficient, so prevent multiple table scans by grabbing the count once
								while k<=numVals do
									if tmpCriteria[k]>tmpValue then
										break
									elseif tmpCriteria[k]==tmpValue then
										-- remove the duplicate
										table.remove(tmpCriteria,k)
										numVals=numVals-1
									else
										k=k+1
									end
								end
								-- now make sure it does not exist in any other criteria list
								for i=tmpGroupIndex+1, #tmpContainer do
									if tmpContainer[i].criteria~=nil then
										local tmpCrit=tmpContainer[i].criteria[key]
										if tmpCrit~=nil then
											k=1
											numVals=#tmpCrit
											while k<=numVals do
												if tmpCrit[k]>tmpValue then
													break
												elseif tmpCrit[k]==tmpValue then
													-- remove the duplicate
													table.remove(tmpCrit,k)
													numVals=numVals-1
												else
													k=k+1
												end
											end
										end
									end
								end
							end
						end
					end
				end
			end
		end
	end
end
-- displayTabs is now guaranteed to exist, has a "Main" entry for each type and all criteria are unique and sorted
displayTabXref={} -- this is the xref that assigns items to a tab and is recreated each time the plugin loads. itemName is the key, tabindex is the value

import (importPath.."ItemCategory");
import (importPath.."ItemData")
if Settings.itemDataMaxItem==nil then Settings.itemDataMaxItem=maxAIItemData end
maxAIItemData=nil
-- load any knownMissingItems
knownMissingItems=PatchDataLoad(Turbine.DataScope.Account, "AltInventoryKnownMissingItems")
if knownMissingItems==nil then knownMissingItems={} end
-- if not reloading, check default data for knownMissingItems? this could take a while but if we don't do it, knownMissingItems will just keep growing
if Settings.reloading~=true then
	local tmpRemove={}
	for tmpName,_ in pairs(knownMissingItems) do
		for cat,items in pairs(AIItemData) do
			for k,v in pairs(items) do
				if v.name==tmpName then
					tmpRemove[tmpName]=1
				end
			end
		end
	end
	for k,v in pairs(tmpRemove) do
		knownMissingItems[k]=nil
	end
end
-- load any foundItems
discoveredItemData=PatchDataLoad(Turbine.DataScope.Account, "AltInventoryDiscoveredItemData")
if discoveredItemData==nil then discoveredItemData={} end
dicoveredItemCategory=PatchDataLoad(Turbine.DataScope.Account, "AltInventoryDiscoveredItemCategory")
if discoveredItemCategory==nil then discoveredItemCategory={} end

-- now merge any discovered items with the default itemData - discovered items should be a small table but can accomodate as many as needed to be future proof until a new ItemData.lua file can be produced
for k,v in pairs(discoveredItemCategory) do
	local removeItems={}
	local addCat=true
	for _,cat in pairs(ItemCategory) do
		if cat[1]==v then
			addCat=false
			break
		end
	end
	if addCat then
		table.insert(ItemCategory,{v})
		local newIndex=#ItemCategory
		if ItemCategoryString[1][newIndex]==nil then ItemCategoryString[1][newIndex]="Unknown Category ("..tostring(v)..")" end
		if ItemCategoryString[2][newIndex]==nil then ItemCategoryString[2][newIndex]="Unknown Category ("..tostring(v)..")" end
		if ItemCategoryString[3][newIndex]==nil then ItemCategoryString[3][newIndex]="Unknown Category ("..tostring(v)..")" end
		ItemCategory[newIndex][2]={ItemCategoryString[1][newIndex],ItemCategoryString[2][newIndex],ItemCategoryString[3][newIndex]}
	else
		removeItems[k]=1
	end
	for k,v in pairs(removeItems) do
		discoveredItemCategory[k]=nil
	end
end
for category,itemData in pairs(discoveredItemData) do
	-- need to account for categories that were missed in U46 update
	local addCat=true
	for k,v in pairs(ItemCategory) do
		if v[1]==category then
			addCat=false
			break
		end
	end
	if addCat then
		--make sure it isn't already there
		for k,v in pairs(discoveredItemCategory) do
			if v==category then
				addCat=false
				break
			end
		end
	end
	if addCat then
		table.insert(discoveredItemCategory,category) -- this is a temporary table for tracking discovered categories so we don't add the strings
		table.insert(ItemCategory,{category})
		local newIndex=#ItemCategory
		if ItemCategoryString[1][newIndex]==nil then ItemCategoryString[1][newIndex]="Unknown Category ("..tostring(category)..")" end
		if ItemCategoryString[2][newIndex]==nil then ItemCategoryString[2][newIndex]="Unknown Category ("..tostring(category)..")" end
		if ItemCategoryString[3][newIndex]==nil then ItemCategoryString[3][newIndex]="Unknown Category ("..tostring(category)..")" end
		ItemCategory[newIndex][2]={ItemCategoryString[1][newIndex],ItemCategoryString[2][newIndex],ItemCategoryString[3][newIndex]}
	end

	local removeItems={}
	for k,v in pairs(itemData) do
		if k>Settings.itemDataMaxItem then Settings.itemDataMaxItem=k end
		if AIItemData[category]==nil then AIItemData[category]={} end
		if AIItemData[category][k]==nil then
			AIItemData[category][k]={}
		end
		if AIItemData[category][k].name==v.name then
			-- flag for removal from discoveredItemData since it is now in the default data
			removeItems[k]=1
		else
			-- currently we only track name, but we might want to track other info in the future so each item entry is a table with one entry, "name"
			AIItemData[category][k].name=v.name
		end
		AIItemData[category][k].backgroundImageID=v.backgroundImageID
		-- if for somereason we need the description, we can always generate an ItemInfo container from the itemID
--		AIItemData[category][k].description=v.description
		AIItemData[category][k].durability=v.durability
		AIItemData[category][k].iconImageID=v.iconImageID
		AIItemData[category][k].maxQuantity=v.maxQuantity
		AIItemData[category][k].maxStackSize=v.maxStackSize
		AIItemData[category][k].quality=v.quality
		AIItemData[category][k].qualityImageID=v.qualityImageID
		AIItemData[category][k].shadowImageID=v.shadowImageID
		AIItemData[category][k].underlayImageID=v.underlayImageID
		AIItemData[category][k].isMagic=v.isMagic
		AIItemData[category][k].isUnique=v.isUnique
	end
	for k,v in pairs(removeItems) do
		discoveredItemData[category][k]=nil
	end
end
--sort the categories by the current language
success,result=pcall(table.sort,ItemCategory,function(arg1,arg2) return(arg1[2][language]<arg2[2][language]) end)
-- now, rebuild the ItemCategoryIndex table
rebuildItemCategoryIndex()

-- create hidden window and controls for update handler and test QS
itemDataWindow=Turbine.UI.Window()
itemDataWindow:SetSize(0,0)
itemDataScan=Turbine.UI.Control()
itemDataScan:SetParent(itemDataWindow)
itemDataTestQS=Turbine.UI.Lotro.Quickslot()
itemDataTestQS:SetParent(itemDataWindow)
itemDataScan.Update=function()
	if Settings.itemDataNeedsScan then
		if itemExplorer.Magnifier.State==nil then itemExplorer.Magnifier.State=0 end
		if itemExplorer.Magnifier.Timeout==nil then itemExplorer.Magnifier.Timeout=Turbine.Engine:GetGameTime()+.125 end
		if Turbine.Engine:GetGameTime()>itemExplorer.Magnifier.Timeout then
			itemExplorer.Magnifier.Timeout=Turbine.Engine:GetGameTime()+.125
			itemExplorer.Magnifier.State=itemExplorer.Magnifier.State+1
			if itemExplorer.Magnifier.State>8 then itemExplorer.Magnifier.State=1 end
			itemExplorer.Magnifier:SetBackground(resourcePath.."itemExplorerMagnifier"..tostring(itemExplorer.Magnifier.State)..".tga")
		end
		itemExplorer.MagnifierBack:SetVisible(true)
	else
		itemExplorer.MagnifierBack:SetVisible(false)
	end
	if Settings.itemDataCurrentScanItem==nil then Settings.itemDataCurrentScanItem=Settings.itemDataMaxItem end
	if Settings.itemDataCurrentScanStart==nil then Settings.itemDataCurrentScanStart=Settings.itemDataCurrentScanItem end
	-- this function will scan for new items in the background using the Update handler to scan 100 items per frame refresh
	-- this function is automatically enabled whenever an item is found with an itemInfo name that is not in AIItemData but is not already in the knownMissingItems table
	-- by using the update handler and an automatic loop mechanism, it will persist after reloads but will automatically terminate once a full scan has been completed
	for count=1,100 do
		if Settings.itemDataNeedsScan then -- only continue iterations if we weren't already done
			-- check if Settings.itemDataCurrentScanItem is in AIItemData
			local tmpFound=false
			for cat,items in pairs(AIItemData) do
				if items[Settings.itemDataCurrentScanItem]~=nil then
					tmpFound=true
					break
				end
			end
			if not tmpFound then
				-- test if it is a valid item
				local sc=Turbine.UI.Lotro.Shortcut(Turbine.UI.Lotro.ShortcutType.Item,"0x0,"..string.format("0x%x",Settings.itemDataCurrentScanItem))
				local success, results=pcall(Turbine.UI.Lotro.Quickslot.SetShortcut,itemDataTestQS,sc)
				if success then
					local tmpItem=sc:GetItem()
					if tmpItem~=nil then
						local tmpItemInfo=tmpItem:GetItemInfo()					
						if tmpItemInfo~=nil then
							local tmpCategory=tmpItemInfo:GetCategory()
							-- add to AIItemData
							if AIItemData[tmpCategory]==nil then AIItemData[tmpCategory]={} end
							if AIItemData[tmpCategory][Settings.itemDataCurrentScanItem]==nil then AIItemData[tmpCategory][Settings.itemDataCurrentScanItem]={} end
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].name=tmpItemInfo:GetName()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].backgroundImageID=tmpItemInfo:GetBackgroundImageID()
--							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].description=tmpItemInfo:GetDescription()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].durability=tmpItemInfo:GetDurability()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].iconImageID=tmpItemInfo:GetIconImageID()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].maxQuantity=tmpItemInfo:GetMaxQuantity()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].maxStackSize=tmpItemInfo:GetMaxStackSize()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].quality=tmpItemInfo:GetQuality()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].qualityImageID=tmpItemInfo:GetQualityImageID()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].shadowImageID=tmpItemInfo:GetShadowImageID()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].underlayImageID=tmpItemInfo:GetUnderlayImageID()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].isMagic=tmpItemInfo:IsMagic()
							AIItemData[tmpCategory][Settings.itemDataCurrentScanItem].isUnique=tmpItemInfo:IsUnique()
							-- add to discoveredItemData
							if discoveredItemData[tmpCategory]==nil then discoveredItemData[tmpCategory]={} end
							if discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem]==nil then discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem]={} end
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].name=tmpItemInfo:GetName()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].backgroundImageID=tmpItemInfo:GetBackgroundImageID()
--							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].description=tmpItemInfo:GetDescription()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].durability=tmpItemInfo:GetDurability()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].iconImageID=tmpItemInfo:GetIconImageID()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].maxQuantity=tmpItemInfo:GetMaxQuantity()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].maxStackSize=tmpItemInfo:GetMaxStackSize()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].quality=tmpItemInfo:GetQuality()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].qualityImageID=tmpItemInfo:GetQualityImageID()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].shadowImageID=tmpItemInfo:GetShadowImageID()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].underlayImageID=tmpItemInfo:GetUnderlayImageID()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].isMagic=tmpItemInfo:IsMagic()
							discoveredItemData[tmpCategory][Settings.itemDataCurrentScanItem].isUnique=tmpItemInfo:IsUnique()
							-- if category was previously unknown, add it to discoveredItemCategory
							local addCat=true
							for k,v in pairs(ItemCategory) do
								if v[1]==tmpCategory then
									addCat=false
									break
								end
							end
							if addCat then
								--make sure it isn't already there
								for k,v in pairs(discoveredItemCategory) do
									if v==tmpCategory then
										addCat=false
										break
									end
								end
							end
							if addCat then
								table.insert(discoveredItemCategory,tmpCategory)
								local newIndex=#discoveredItemCategory
								ItemCategoryString[1][newIndex]="Unknown Category ("..tostring(tmpCategory)..")"
								ItemCategoryString[2][newIndex]="Unknown Category ("..tostring(tmpCategory)..")"
								ItemCategoryString[3][newIndex]="Unknown Category ("..tostring(tmpCategory)..")"
								
							end
							-- if it exists in knownMissingItems, remove it
							if knownMissingItems[name]~=nil then knownMissingItems[name]=nil end
						end
					end
					if Settings.itemDataCurrentScanItem>Settings.itemDataMaxItem then Settings.itemDataMaxItem=Settings.itemDataCurrentScanItem end
				end
			end
			-- now increment
			Settings.itemDataCurrentScanItem=Settings.itemDataCurrentScanItem+1
			if Settings.itemDataScanLooped==true and Settings.itemDataCurrentScanItem>Settings.itemDataCurrentScanStart then
				-- we are done
				Settings.itemDataCurrentScanStart=nil
				Settings.itemDataScanLooped=false
				Settings.itemDataCurrentScanItem=nil
				Settings.itemDataNeedsScan=false
				itemDataScan:SetWantsUpdates(false)
				itemExplorer.MagnifierBack:SetVisible(false)
				-- need to rebuild AIItemDataIndex
				if itemExplorer:IsVisible() then
					rebuildAIItemDataIndex()
				else
					AIItemDataIndex=nil -- will force rebuild when the user shows itemExplorer - this way it only rebuilds if it is actually going to be used
				end
			else
				if Settings.itemDataCurrentScanItem> Settings.itemDataMaxItem+4096 then
					Settings.itemDataScanLooped=true
					Settings.itemDataCurrentScanItem=0x70000000
				end
			end
		end
	end
end
if Settings.itemDataNeedsScan==nil then Settings.itemDataNeedsScan=false end
if Settings.itemDataScanLooped==nil then Settings.itemDataScanLooped=false end
if Settings.itemDataCurrentScanItem==nil then Settings.itemDataCurrentScanItem=0x70000000 end -- afaik, this is the first valid item ID
itemDataScan:SetWantsUpdates(Settings.itemDataNeedsScan)

function getItemID(itemName,category)
	local itemID=nil
	if AIItemData~=nil then
		if category==nil then
			for tmpCategory,tmpItems in pairs(AIItemData) do
				if itemID==nil then
					for k,v in pairs(tmpItems) do
						if v["name"]==itemName then
							itemID=k
							break
						end
					end
				end
			end
		else
			for k,v in pairs(AIItemData[category]) do
				if v["name"]==itemName then
					itemID=k
					break
				end
			end
		end
	end
	if itemID==nil then
		-- unable to resolve the item, check knownMissingItems
		if knownMissingItems[itemName]==nil then
			knownMissingItems[itemName]=1 -- doesn't matter what value we use, just need to make an entry so the key exists so it doesn't keep triggering rescans
			-- now trigger a rescan
			Settings.itemDataCurrentScanStart=Settings.itemDataCurrentScanItem -- just in case we were already in a scan
			Settings.itemDataScanLooped=false -- also in case we were already in a scan, force it to check again
			Settings.itemDataNeedsScan=true
			itemDataScan:SetWantsUpdates(true)
		end
	end
	return itemID
end

import (importPath.."groupMaint")
import (importPath.."InventoryWindow")
import (importPath.."IconWindow")
import (importPath.."ItemExplorer")

function UnloadPlugin()
	SaveData();

	-- have to remove all shell commands before unloading or we get a crash to desktop :(
	if setupWindow~=nil then
		Turbine.Shell.RemoveCommand(setupWindow.shellCmd)
	end

	-- safely remove all callbacks
	local index;
	for index=1,backPack:GetSize() do
		if backpackImage[index]~=nil and backpackImage[index].Item~=nil then
			if backpackImage[index].Item.QuantityChanged~=nil then
				RemoveCallback(backpackImage[index].Item,"QuantityChanged",quantityChanged)
			end
		end
	end
	for index=1,#sharedStorageImage do
		if sharedStorageImage[index].Item~=nil then
			if sharedStorageImage[index].Item.QuantityChanged~=nil then
				RemoveCallback(sharedStorageImage[index].Item,"QuantityChanged",sharedStorageQuantityChanged)
				RemoveCallback(sharedStorageImage[index].Item,"ChestChanged",sharedStorageChestChanged)
			end
		end
	end
	for index=1,#vaultImage do
		if vaultImage[index].Item~=nil then
			if vaultImage[index].Item.QuantityChanged~=nil then
				RemoveCallback(vaultImage[index].Item,"QuantityChanged",vaultQuantityChanged)
				RemoveCallback(vaultImage[index].Item,"ChestChanged",vaultChestChanged)
			end
		end
	end

	RemoveCallback(backPack,"ItemAdded",itemAdded)
	RemoveCallback(backPack,"ItemRemoved",itemRemoved)
	RemoveCallback(backPack,"ItemMoved",itemMoved)
	RemoveCallback(sharedStorage,"ItemAdded",sharedStorageItemAdded);
	RemoveCallback(sharedStorage,"ItemRemoved",sharedStorageItemRemoved);
	RemoveCallback(sharedStorage,"IsAvailableChanged",refreshSharedStorage);
	RemoveCallback(sharedStorage,"ItemsRefreshed",refreshSharedStorage);
	RemoveCallback(vault,"ItemAdded",vaultItemAdded);
	RemoveCallback(vault,"ItemRemoved",vaultItemRemoved);
	RemoveCallback(vault,"IsAvailableChanged",refreshVault);
	RemoveCallback(vault,"ItemsRefreshed",refreshVault);

	-- reenable built-in bags
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack1, true );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack2, true );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack3, true );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack4, true );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack5, true );
	Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack6, true );
end
function SaveData()
	CharList[charName].VaultChestNames={};
	for index=1,vault:GetChestCount() do
		CharList[charName].VaultChestNames[index]=vault:GetChestName(index-1);
	end
	PatchDataSave(Turbine.DataScope.Server,"AltInventoryCharList",CharList);
	PatchDataSave(Turbine.DataScope.Account, "AltInventoryKnownMissingItems",knownMissingItems)
	PatchDataSave(Turbine.DataScope.Account, "AltInventoryDiscoveredItemData",discoveredItemData)
	PatchDataSave(Turbine.DataScope.Account, "AltInventoryDiscoveredItemCategory",discoveredItemCategory)

	-- save the tooltip settings
	local tmpIndex;
	Settings.version=Plugins.AltInventory:GetVersion();
	Settings.SWTop=setupWindow:GetTop();
	Settings.SWLeft=setupWindow:GetLeft();
	local parent=inventoryPanel:GetParent();
	Settings.panelLeft=(parent:GetLeft()+inventoryPanel:GetLeft()*Settings.zoom)/displayWidth;
	Settings.panelTop=(parent:GetTop()+inventoryPanel:GetTop()*Settings.zoom)/displayHeight;
	Settings.itemExplorerLeft=itemExplorer:GetLeft()/displayWidth
	Settings.itemExplorerTop=itemExplorer:GetTop()/displayHeight
	Settings.itemExplorerWidth=itemExplorer:GetWidth()/displayWidth
	Settings.itemExplorerHeight=itemExplorer:GetHeight()/displayHeight
	Settings.iconTop=iconWindow:GetTop()/displayHeight;
	Settings.iconLeft=iconWindow:GetLeft()/displayWidth;
	if Settings.snapShot==nil then Settings.snapShot=0 end
	Settings.snapShot=Settings.snapShot+1
	if Settings.snapShot>10 then Settings.snapShot=1 end
	PatchDataSave(Turbine.DataScope.Character, "AltInventorySettings", Settings);

	if accountItemQty==nil then accountItemQty={} end

	-- remove the listItem link and itemInfo values and any entries with qty=0 since they are irrelevant
	for k,v in pairs(accountItemQty) do
		local total=0;
		v.ListItem=nil;
		v.ItemInfo=nil;
		for char,qty in pairs(v.Qty) do
			if (type(qty)=="table" and qty.Subtotal==0) or (type(qty)=="number" and qty==0) then
				v.Qty[char]=nil;
			else
				if char==charName then
					local bagTotal={};
					local subtotal=0;
					-- rollup the current char bag inventory - only need to store the qty per bag, not each stack
					for slot, slotQty in pairs(qty) do
						if slot~="Subtotal" then
							local bagIndex=math.floor((slot+14)/15)
							if bagTotal[bagIndex]==nil then
								bagTotal[bagIndex]=slotQty
							else
								bagTotal[bagIndex]=bagTotal[bagIndex]+slotQty
							end
								subtotal=subtotal+slotQty
						end
					end
					if subtotal==0 then
						accountItemQty[k].Qty[charName]=nil;
					else
						accountItemQty[k].Qty[charName]={["Subtotal"]=subtotal}
						for bag,bagQty in pairs(bagTotal) do
							accountItemQty[k].Qty[charName][bag]=bagQty;
						end
					end
				end
				if type(qty)=="table" then
					total=total+qty.Subtotal;
				else
					total=total+qty;
				end
			end
		end
		if total==0 then
			accountItemQty[k]=nil;
		end
	end

	PatchDataSave(Turbine.DataScope.Server, "AltInventoryData", accountItemQty);
	PatchDataSave(Turbine.DataScope.Server, "AltInventoryData_SnapShot"..tostring(Settings.snapShot), accountItemQty);
	if Settings.enableEI then
		refreshEI() -- refresh the current character data
		PatchDataSave(Turbine.DataScope.Server, "AltInventoryEI", equippedItems);
	end
	-- strip out the temporary window and listbox control references from displayTabs - they get dynamically recreated when displayed
	for tmpIndex,tmpContainer in pairs(displayTabs) do
		if type(tmpContainer)=="table" then
			for k,v in ipairs(tmpContainer) do
				if type(v)=="table" then
					v.window=nil
					v.panel=nil
				end
			end
		end
	end
	PatchDataSave(Turbine.DataScope.Character, "AltInventoryDisplayTabs", displayTabs);
	if defaultTabs~=nil then
		PatchDataSave(Turbine.DataScope.Account, "AltInventoryDefaultTabs", defaultTabs)
	end
end

import (importPath.."SetupWindow")

backpackImage={}

backPack=localPlayer:GetBackpack()
CharList[charName].capacity=backPack:GetSize();

function quantityChanged(sender,args)
-- uses the brute force method of scanning the whole backpack. this will automatically resync any backpack quantity that gets out of sync due to odd sequences in event firing
-- I have to revisit this and determine whether the event firing order has been fixed in which case this can be significantly optimised to improve performance
	if sender~=nil and sender.GetItemInfo~=nil then
		local itemInfo=sender:GetItemInfo();
		if itemInfo~=nil then
			local name=sender:GetItemInfo():GetName() -- supports custom names
			local infoName=sender:GetItemInfo():GetName()
			if name==nil then name=infoName end
			if name~=nil then
				local index=sender.Index
				local qty=sender:GetQuantity();
				if index==nil then
					if Settings.debug then
						Turbine.Shell.WriteLine("quantityChanged: index is nil")
					end
				else
					if accountItemQty[name]==nil then
						accountItemQty[name]={}
						accountItemQty[name].InfoName=infoName
						accountItemQty[name].Total=0;
						accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
						accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
						accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
						accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
						accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
						accountItemQty[name].Qty={};
						accountItemQty[name].ListItem={};
						local found=false;
						for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
							if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
								if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charName then
									inventoryPanel.FilterList:ShowEntry(filterIndex);
								end
								found=true;
								break;
							end
						end
						if not found then
							accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
							if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charName then
								inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
							end
						end
					end
					if accountItemQty[name].ItemInfo==nil then
						accountItemQty[name].ItemInfo=itemInfo;
						if accountItemQty[name].ListItem[charName]~=nil then
							if Settings.totalsOnly then
								accountItemQty[name].ListItem[charName].ItemInfo:SetItemInfo(itemInfo);
								accountItemQty[name].ListItem[charName].IconBorder:SetVisible(false);
							else
								if accountItemQty[name].ListItem[charName][index]~=nil then
									accountItemQty[name].ListItem[charName][index].ItemInfo:SetItemInfo(itemInfo);
									accountItemQty[name].ListItem[charName][index].IconBorder:SetVisible(false);
								end
							end
						end
					end
					local oldQty=0;
					if accountItemQty[name].Qty[charName][index]~=nil then
						oldQty=accountItemQty[name].Qty[charName][index];
					end
					accountItemQty[name].Qty[charName][index]=qty;

					accountItemQty[name].Qty[charName].Subtotal=accountItemQty[name].Qty[charName].Subtotal-oldQty+qty;
					accountItemQty[name].Total=accountItemQty[name].Total-oldQty+qty;
					-- now update the display element
					if inventoryWindow.CharList:GetText()==charName then
						if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
						if Settings.totalsOnly then
							if accountItemQty[name].ListItem[charName]==nil then
								-- create a new entry, set height by filter and position it according to the sort
								local tmpEntry=ItemEntry(name,charName,accountItemQty[name].Qty[charName].Subtotal,itemInfo)
								if not Settings.totalsOnly then
									tmpEntry.Item:SetItem(backPack:GetItem(index))
									tmpEntry.Item.Index=index;
								end
								accountItemQty[name].ListItem[charName]=tmpEntry;
--***
--								inventoryPanel.ItemList:AddItem(tmpEntry)
								tmpEntry.panel.ItemList:AddItem(tmpEntry)
								inventoryPanel.SortList:SelectedIndexChanged();
							else
								accountItemQty[name].ListItem[charName].Qty:SetText(accountItemQty[name].Qty[charName].Subtotal);
							end
						else
							if accountItemQty[name].ListItem[charName]==nil then accountItemQty[name].ListItem[charName]={} end
							if accountItemQty[name].ListItem[charName][index]==nil then
								-- create a new entry, set height by filter and position it according to the sort
								local tmpEntry=ItemEntry(name,charName,qty,itemInfo)
								if not Settings.totalsOnly then
									tmpEntry.Item:SetItem(backPack:GetItem(index))
									tmpEntry.Item.Index=index;
								end
								accountItemQty[name].ListItem[charName][index]=tmpEntry;
--***
--								inventoryPanel.ItemList:AddItem(tmpEntry)
								tmpEntry.panel.ItemList:AddItem(tmpEntry)
								inventoryPanel.SortList:SelectedIndexChanged();
							else
								accountItemQty[name].ListItem[charName][index].Qty:SetText(qty);
							end
						end
					elseif inventoryWindow.CharList:GetSelectedIndex()==1 then
						if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
						if accountItemQty[name].ListItem["ALL"]==nil then
							-- create a new entry, set height by filter and position it according to the sort
							local tmpEntry=ItemEntry(name,"ALL",accountItemQty[name].Total,itemInfo)
							accountItemQty[name].ListItem["ALL"]=tmpEntry;
--***
--							inventoryPanel.ItemList:AddItem(tmpEntry)
							tmpEntry.panel.ItemList:AddItem(tmpEntry)
							inventoryPanel.SortList:SelectedIndexChanged();
						else
							accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
						end
					end
				end
			end
		end
	end
end

-- Shared Storage
sharedStorage=localPlayer:GetSharedStorage();
function sharedStorageChestChanged(sender)
	-- sender.Chest== old chest
	local oldChest=sender.Chest;
	local newChest=sender:GetChest();
	local qty=sender:GetQuantity();
	local name=sender:GetName()
	local infoName=sender:GetItemInfo():GetName()
	if name==nil then name=infoName end
	accountItemQty[name].Qty["Shared Storage"][oldChest]=accountItemQty[name].Qty["Shared Storage"][oldChest]-qty;
	if accountItemQty[name].Qty["Shared Storage"][oldChest]==0 then accountItemQty[name].Qty["Shared Storage"][oldChest]=nil end
	if accountItemQty[name].Qty["Shared Storage"][newChest]==nil then
		accountItemQty[name].Qty["Shared Storage"][newChest]=qty;
	else
		accountItemQty[name].Qty["Shared Storage"][newChest]=accountItemQty[name].Qty["Shared Storage"][newChest]+qty;
	end
	sender.Chest=newChest;
end
function sharedStorageQuantityChanged(sender,args)
	if Settings.debug then
		Turbine.Shell.WriteLine("sharedStorageQuantityChanged")
		Turbine.Shell.WriteLine("sender:"..tostring(sender))
	end
	local name=sender:GetName()
	local infoName=sender:GetItemInfo():GetName()
	if name==nil then name=infoName end

	local index;

	-- scan the shared storage, recalc the total for this name and then update accountItemQty
	local total=0;
	local containerQty={};
	for index=1,sharedStorage:GetCount() do
		local item=sharedStorage:GetItem(index);
		if item~=nil and item:GetItemInfo():GetName()==name then
			local tmpQty=item:GetQuantity();
			local bag=item:GetChest();
			total=total+tmpQty;
			if containerQty[bag]==nil then
				containerQty[bag]=tmpQty;
			else
				containerQty[bag]=containerQty[bag]+tmpQty;
			end
		end
	end
	local oldTotal=accountItemQty[name].Qty["Shared Storage"].Subtotal;
	accountItemQty[name].Total=accountItemQty[name].Total+total-oldTotal;
	accountItemQty[name].Qty["Shared Storage"]={["Subtotal"]=total};
	for k,v in pairs(containerQty) do
		accountItemQty[name].Qty["Shared Storage"][k]=v;
	end
	-- now update the display
	if inventoryWindow.CharList:GetSelectedIndex()==2 then
		if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
		if accountItemQty[name].ListItem["Shared Storage"]==nil then
			-- create a new entry, set height by filter and position it according to the sort
			local tmpEntry=ItemEntry(name,"Shared Storage",accountItemQty[name].Qty["Shared Storage"].Subtotal,itemInfo)
			accountItemQty[name].ListItem["Shared Storage"]=tmpEntry;
--***
--			inventoryPanel.ItemList:AddItem(tmpEntry)
			tmpEntry.panel.ItemList:AddItem(tmpEntry)
			inventoryPanel.SortList:SelectedIndexChanged();
		else
			accountItemQty[name].ListItem["Shared Storage"].Qty:SetText(accountItemQty[name].Qty["Shared Storage"].Subtotal);
		end
	elseif inventoryWindow.CharList:GetSelectedIndex()==1 then
		if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
		if accountItemQty[name].ListItem["ALL"]==nil then
			-- create a new entry, set height by filter and position it according to the sort
			local tmpEntry=ItemEntry(name,"ALL",accountItemQty[name].Total,itemInfo)
			accountItemQty[name].ListItem["ALL"]=tmpEntry;
--***
--			inventoryPanel.ItemList:AddItem(tmpEntry)
			tmpEntry.panel.ItemList:AddItem(tmpEntry)
			inventoryPanel.SortList:SelectedIndexChanged();
		else
			accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
		end
	end
end
sharedStorageImage={};
function refreshSharedStorage()
	if Settings.debug then
		Turbine.Shell.WriteLine("refreshSharedStorage")
	end
	if sharedStorage:IsAvailable() then
		local tmpItemFlag={}
		-- clear out any accountItemQty record for "Shared Storage" and recalc the total for any item that had a "Shared Storage" qty

		for k,v in pairs(sharedStorageImage) do
			if v.Item~=nil then
				-- safely remove old callbacks
				if v.Item.QuantityChanged~=nil then
					RemoveCallback(v.Item,"QuantityChanged",sharedStorageQuantityChanged);
					RemoveCallback(v.Item,"ChestChanged",sharedStorageChestChanged);
				end
			end
		end
		sharedStorageImage={};
		-- set flags for items that were in the old vault
		for k,v in pairs(accountItemQty) do
			local total=0;
			for name,qty in pairs(v.Qty) do
				if name=="Shared Storage" then
					if type(qty)~="table" then
						tmpItemFlag[k]=qty
						accountItemQty[k].Qty["Shared Storage"]={["Subtotal"]=qty};
					else
						tmpItemFlag[k]=qty.Subtotal;
						accountItemQty[k].Qty["Shared Storage"]={["Subtotal"]=qty.Subtotal};
					end
				end
			end
		end
		-- process the current Shared Storage, adding an accountItemQty record and updating total
		local size=sharedStorage:GetCount();
		CharList["Shared Storage"].used=size;
		CharList["Shared Storage"].capacity=sharedStorage:GetCapacity();
		if inventoryWindow.CharList:GetSelectedIndex()==2 then
			inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList["Shared Storage"].used).."/"..tostring(CharList["Shared Storage"].capacity)..")")
		end

		local tmpIndex,tmpItem;
		for tmpIndex=1,size do
			tmpItem=sharedStorage:GetItem(tmpIndex);
			sharedStorageImage[tmpIndex]={}
			sharedStorageImage[tmpIndex].Item=tmpItem;
			if tmpItem~=nil then
				local itemInfo=tmpItem:GetItemInfo();
				local name=tmpItem:GetName()
				local infoName=tmpItem:GetItemInfo():GetName()
				if name==nil then name=infoName end
				local qty=tmpItem:GetQuantity();
				sharedStorageImage[tmpIndex].Index=tmpIndex;
				AddCallback(sharedStorageImage[tmpIndex].Item,"QuantityChanged",sharedStorageQuantityChanged);
				AddCallback(sharedStorageImage[tmpIndex].Item,"ChestChanged",sharedStorageChestChanged);
				sharedStorageImage[tmpIndex].Item.Chest=sharedStorageImage[tmpIndex].Item:GetChest();
				sharedStorageImage[tmpIndex].ItemName=name;
				if accountItemQty[name]==nil then
					accountItemQty[name]={}
					accountItemQty[name].InfoName=infoName
					accountItemQty[name].Total=0;
					accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
					accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
					accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
					accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
					accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
					accountItemQty[name].Qty={};
					accountItemQty[name].ListItem={};
					local found=false;
					for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
						if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
							if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetSelectedIndex()==2 then
								inventoryPanel.FilterList:ShowEntry(filterIndex);
							end
							found=true;
							break;
						end
					end
					if not found then
						accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
						if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetSelectedIndex()==2 then
							inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
						end
					end
				end
				if accountItemQty[name].ItemInfo==nil then
					accountItemQty[name].ItemInfo=itemInfo;
				end
				if inventoryWindow.CharList:GetSelectedIndex()==2 and accountItemQty[name].ListItem["Shared Storage"]==nil then
					local tmpEntry=ItemEntry(name,"Shared Storage",qty,itemInfo);					
--***
--					inventoryPanel.ItemList:AddItem(tmpEntry)
					tmpEntry.panel.ItemList:AddItem(tmpEntry)
					accountItemQty[name].ListItem["Shared Storage"]=tmpEntry;
				end
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[name].ListItem["ALL"]==nil then
					local tmpEntry=ItemEntry(name,"ALL",qty,itemInfo);					
--***
--					inventoryPanel.ItemList:AddItem(tmpEntry)
					tmpEntry.panel.ItemList:AddItem(tmpEntry)
					accountItemQty[name].ListItem["ALL"]=tmpEntry;
				end
				if accountItemQty[name].Qty["Shared Storage"]==nil then
					accountItemQty[name].Qty["Shared Storage"]={["Subtotal"]=qty};
				else
					if tmpItemFlag[name]==nil then
						accountItemQty[name].Qty["Shared Storage"].Subtotal=accountItemQty[name].Qty["Shared Storage"].Subtotal+qty;
					else
						accountItemQty[name].Total=accountItemQty[name].Total-tmpItemFlag[name];
						tmpItemFlag[name]=nil;
						accountItemQty[name].Qty["Shared Storage"].Subtotal=qty;
					end
				end
				local bag=tmpItem:GetChest();
				if accountItemQty[name].Qty["Shared Storage"][bag]==nil then
					accountItemQty[name].Qty["Shared Storage"][bag]=qty;
				else
					accountItemQty[name].Qty["Shared Storage"][bag]=accountItemQty[name].Qty["Shared Storage"][bag]+qty;
				end
				if inventoryWindow.CharList:GetSelectedIndex()==2 and accountItemQty[name].ListItem["Shared Storage"].ItemInfo~=nil then
					accountItemQty[name].ListItem["Shared Storage"].ItemInfo:SetItemInfo(itemInfo);
					accountItemQty[name].ListItem["Shared Storage"].IconBorder:SetVisible(false);
					accountItemQty[name].ListItem["Shared Storage"].Qty:SetText(accountItemQty[name].Qty["Shared Storage"].Subtotal);
				end
				accountItemQty[name].Total=accountItemQty[name].Total+qty;
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[name].ListItem["ALL"]~=nil and accountItemQty[name].ListItem["ALL"].ItemInfo~=nil then
					accountItemQty[name].ListItem["ALL"].ItemInfo:SetItemInfo(itemInfo);
					accountItemQty[name].ListItem["ALL"].IconBorder:SetVisible(false);
					accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
				end
			end
		end
		for tmpItem,val in pairs(tmpItemFlag) do
			if inventoryWindow.CharList:GetSelectedIndex()==2 and accountItemQty[tmpItem].ListItem["Shared Storage"]~=nil then
				-- remove the linked display entry
--***
--				local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem["Shared Storage"])
				local tmpEntryPanel=getCurrentItemPanel(tmpItem)
				local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem["Shared Storage"])
--***
--				if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end
				if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end
				accountItemQty[tmpItem].ListItem["Shared Storage"]=nil;
			end

			-- these items existed previously and now have to be accounted for
			-- recalc the total instead of adjusting just in case the discrepancy was due to a character rename, data corruption or playing the account on another computer
			accountItemQty[tmpItem].Qty["Shared Storage"]=nil;
			local total=0;
			for char,qty in pairs(accountItemQty[tmpItem].Qty) do
				total=total+qty.Subtotal;
			end
			if total==0 then
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[tmpItem].ListItem~=nil and accountItemQty[tmpItem].ListItem["ALL"]~=nil then
					-- remove the "ALL" linked display entry
--***
--					local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem["ALL"])
					local tmpEntryPanel=getCurrentItemPanel(tmpItem)
					local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem["ALL"])
--***
--					if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					accountItemQty[tmpItem].ListItem["ALL"]=nil;
				end
				accountItemQty[tmpItem]=nil;
			else
				accountItemQty[tmpItem].Total=total;
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[tmpItem].ListItem~=nil and accountItemQty[tmpItem].ListItem["ALL"]~=nil then
					accountItemQty[tmpItem].ListItem["ALL"].Qty:SetText(total);
				end
			end
		end
		-- now reapply the sort
		if inventoryWindow.CharList:GetSelectedIndex()==2 or inventoryWindow.CharList:GetSelectedIndex()==1 then
			inventoryPanel.SortList:SelectedIndexChanged();
		end
	end
end
AddCallback(sharedStorage,"IsAvailableChanged",refreshSharedStorage);
AddCallback(sharedStorage,"ItemsRefreshed",refreshSharedStorage);
sharedStorageItemAdded=function(sender,args)
	-- Index, Item
	if Settings.debug then
		Turbine.Shell.WriteLine("sharedStorageItemAdded");
	end
	local index=args.Index;
	local item=args.Item;
	table.insert(sharedStorageImage,index,{});

	CharList["Shared Storage"].used=sharedStorage:GetCount();
	if inventoryWindow.CharList:GetSelectedIndex()==2 then
		inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList["Shared Storage"].used).."/"..tostring(CharList["Shared Storage"].capacity)..")")
	end
	sharedStorageImage[index].Item=item;
	if item~=nil then
		local itemInfo=item:GetItemInfo();
		local name=itemInfo:GetName();
		local name=item:GetName()
		local infoName=itemInfo:GetName()
		if name==nil then name=infoName end
		local qty=item:GetQuantity();
		sharedStorageImage[index].Index=tmpIndex;
		AddCallback(sharedStorageImage[index].Item,"QuantityChanged",sharedStorageQuantityChanged);
		AddCallback(sharedStorageImage[index].Item,"ChestChanged",sharedStorageChestChanged);
		sharedStorageImage[index].Item.Chest=sharedStorageImage[index].Item:GetChest();
		sharedStorageImage[index].ItemName=name;
		if accountItemQty[name]==nil then
			accountItemQty[name]={}
			accountItemQty[name].InfoName=infoName
			accountItemQty[name].Total=0;
			accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
			accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
			accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
			accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
			accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
			accountItemQty[name].Qty={};
			accountItemQty[name].ListItem={};
			local found=false;
			for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
				if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
					if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetSelectedIndex()==2 then
						inventoryPanel.FilterList:ShowEntry(filterIndex);
					end
					found=true;
					break;
				end
			end
			if not found then
				accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
				if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetSelectedIndex()==2 then
					inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
				end
			end
		end
		if accountItemQty[name].ItemInfo==nil then
			accountItemQty[name].ItemInfo=itemInfo;
			if accountItemQty[name].ListItem["Shared Storage"]~=nil then
				accountItemQty[name].ListItem["Shared Storage"].ItemInfo:SetItemInfo(itemInfo);
				accountItemQty[name].ListItem["Shared Storage"].IconBorder:SetVisible(false);
			end
		end
		if accountItemQty[name].Qty["Shared Storage"]==nil then
			accountItemQty[name].Qty["Shared Storage"]={["Subtotal"]=qty};
		else
			accountItemQty[name].Qty["Shared Storage"].Subtotal=accountItemQty[name].Qty["Shared Storage"].Subtotal+qty;
		end
		local bag=item:GetChest();
		if accountItemQty[name].Qty["Shared Storage"][bag]==nil then
			accountItemQty[name].Qty["Shared Storage"][bag]=qty;
		else
			accountItemQty[name].Qty["Shared Storage"][bag]=accountItemQty[name].Qty["Shared Storage"][bag]+qty;
		end
		accountItemQty[name].Total=accountItemQty[name].Total+qty;
		-- now update the display element
		if inventoryWindow.CharList:GetSelectedIndex()==2 then
			if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
			if accountItemQty[name].ListItem["Shared Storage"]==nil then
				-- create a new entry, set height by filter and position it according to the sort
				local tmpEntry=ItemEntry(name,"Shared Storage",accountItemQty[name].Qty["Shared Storage"].Subtotal,itemInfo)
				accountItemQty[name].ListItem["Shared Storage"]=tmpEntry;
--***
--				inventoryPanel.ItemList:AddItem(tmpEntry)
				tmpEntry.panel.ItemList:AddItem(tmpEntry)
				if tmpEntry:GetHeight()>0 then
					inventoryPanel.SortList:SelectedIndexChanged();
				end
			else
				accountItemQty[name].ListItem["Shared Storage"].Qty:SetText(accountItemQty[name].Qty["Shared Storage"].Subtotal);
			end
		elseif inventoryWindow.CharList:GetSelectedIndex()==1 then
			if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
			if accountItemQty[name].ListItem["ALL"]==nil then
				-- create a new entry, set height by filter and position it according to the sort
				local tmpEntry=ItemEntry(name,"ALL",accountItemQty[name].Total,itemInfo)
				accountItemQty[name].ListItem["ALL"]=tmpEntry;
--***
--				inventoryPanel.ItemList:AddItem(tmpEntry)
				tmpEntry.panel.ItemList:AddItem(tmpEntry)
				inventoryPanel.SortList:SelectedIndexChanged();
			else
				accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
			end
		end
		-- now reapply the sort
		if inventoryWindow.CharList:GetSelectedIndex()==2 or inventoryWindow.CharList:GetSelectedIndex()==1 then
			inventoryPanel.SortList:SelectedIndexChanged();
		end
	end
end
sharedStorageItemRemoved=function(sender,args)
-- Index, Item
	if Settings.debug then
		Turbine.Shell.WriteLine("sharedStorageItemRemoved");
	end
	local index=args.Index;

	local name=args.Item:GetName()
	local infoName=args.Item:GetItemInfo():GetName()
	if name==nil then name=infoName end

	local item=args.Item;
	local qty=0;
	if item~=nil then
		qty=item:GetQuantity();
		if item.QuantityChanged~=nil then
			RemoveCallback(item,"QuantityChanged",sharedStorageQuantityChanged);
			RemoveCallback(item,"ChestChanged",sharedStorageChestChanged);
		end
	end
	table.remove(sharedStorageImage,index);
	local accountItem=accountItemQty[name];
	if accountItem~=nil then
		local tmpEntryPanel=getCurrentItemPanel(name)
		accountItem.Total=accountItem.Total-qty;
		if accountItem.Qty["Shared Storage"]~=nil then
			local totQty=accountItem.Qty["Shared Storage"].Subtotal-qty
			if totQty==0 then
				accountItem.Qty["Shared Storage"]=nil
				-- remove the display entry
				if inventoryWindow.CharList:GetSelectedIndex()==2 and accountItem.ListItem["Shared Storage"]~=nil then
					-- remove the linked display entry
--***
--					local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem["Shared Storage"])
					local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem["Shared Storage"])
--***
--					if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					accountItem.ListItem["Shared Storage"]=nil;
				end
			else
				local bag=item:GetChest();
				accountItem.Qty["Shared Storage"][bag]=accountItem.Qty["Shared Storage"][bag]-qty;
				if accountItem.Qty["Shared Storage"][bag]==0 then accountItem.Qty["Shared Storage"][bag]=nil end
				accountItem.Qty["Shared Storage"].Subtotal=totQty
				if inventoryWindow.CharList:GetSelectedIndex()==2 and accountItem.ListItem["Shared Storage"]~=nil then
					-- update the linked display entry
					accountItem.ListItem["Shared Storage"].Qty:SetText(totQty);
				end
			end
		end
		if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItem.ListItem["ALL"]~=nil then
			if accountItem.Total==0 then
				-- remove the linked display entry
--***
--				local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem["ALL"])
				local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem["ALL"])
--***
--				if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
				if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
				accountItem.ListItem["ALL"]=nil;
			else
				accountItem.ListItem["ALL"].Qty:SetText(accountItem.Total);
			end
		end
	end
	CharList["Shared Storage"].used=CharList["Shared Storage"].used-1; -- can't use the GetCount since this event is fired BEFORE the item is actually removed
	if inventoryWindow.CharList:GetSelectedIndex()==2 then
		inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList["Shared Storage"].used).."/"..tostring(CharList["Shared Storage"].capacity)..")")
	end
end
AddCallback(sharedStorage,"ItemAdded",sharedStorageItemAdded);

AddCallback(sharedStorage,"ItemRemoved",sharedStorageItemRemoved);

-- Equipped Items
function refreshEI()
	-- update the equippedItems[charEIName] table
-- only really need to do this when unloading since the display uses EquipmentSlot controls for current character data
	local ei=localPlayer:GetEquipment()
--	if equippedItems==nil then equippedItems={} end
	equippedItems[charName]={}
	local tmpItem
	for tmpIndex=1,20 do
		tmpItem=ei:GetItem(tmpIndex)
		if tmpItem~=nil then
			local tmpInfo=tmpItem:GetItemInfo()
			equippedItems[charName][tmpIndex]={}
			equippedItems[charName][tmpIndex].Name=tmpItem:GetName()
			equippedItems[charName][tmpIndex].InfoName=tmpInfo:GetName()
			equippedItems[charName][tmpIndex].Durability=tmpInfo:GetDurability()
			equippedItems[charName][tmpIndex].Quality=tmpItem:GetQuality()
			equippedItems[charName][tmpIndex].Category=tmpItem:GetCategory()
			equippedItems[charName][tmpIndex].WearState=tmpItem:GetWearState()
			equippedItems[charName][tmpIndex].BackgroundImageID=tmpInfo:GetBackgroundImageID()
			equippedItems[charName][tmpIndex].IconImageID=tmpInfo:GetIconImageID()
			equippedItems[charName][tmpIndex].QualityImageID=tmpInfo:GetQualityImageID()
			equippedItems[charName][tmpIndex].ShadowImageID=tmpInfo:GetShadowImageID()
			equippedItems[charName][tmpIndex].UnderlayImageID=tmpInfo:GetUnderlayImageID()
		end
	end
end

-- Vault
vault=localPlayer:GetVault();
function vaultChestChanged(sender)
	-- sender.Chest== old chest
	local oldChest=sender.Chest;
	local newChest=sender:GetChest();
	local qty=sender:GetQuantity();

	local name=sender:GetName()
	local infoName=sender:GetItemInfo():GetName()
	if name==nil then name=infoName end

	accountItemQty[name].Qty[charVaultName][oldChest]=accountItemQty[name].Qty[charVaultName][oldChest]-qty;
	if accountItemQty[name].Qty[charVaultName][oldChest]==0 then accountItemQty[name].Qty[charVaultName][oldChest]=nil end
	if accountItemQty[name].Qty[charVaultName][newChest]==nil then
		accountItemQty[name].Qty[charVaultName][newChest]=qty;
	else
		accountItemQty[name].Qty[charVaultName][newChest]=accountItemQty[name].Qty[charVaultName][newChest]+qty;
	end
	sender.Chest=newChest;
end
function vaultQuantityChanged(sender,args)
	if Settings.debug then
		Turbine.Shell.WriteLine("vaultQuantityChanged")
		Turbine.Shell.WriteLine("sender:"..tostring(sender))
	end

	local name=sender:GetName()
	local infoName=sender:GetItemInfo():GetName()
	if name==nil then name=infoName end

	local index;

	-- scan the vault, recalc the total for this name and then update accountItemQty
	local total=0;
	local containerQty={};
	for index=1,vault:GetCount() do
		local item=vault:GetItem(index);
		if item~=nil and item:GetItemInfo():GetName()==name then
			local tmpQty=item:GetQuantity();
			local bag=item:GetChest();
			total=total+tmpQty;
			if containerQty[bag]==nil then
				containerQty[bag]=tmpQty;
			else
				containerQty[bag]=containerQty[bag]+tmpQty;
			end
		end
	end
	local oldTotal=accountItemQty[name].Qty[charVaultName].Subtotal;
	accountItemQty[name].Total=accountItemQty[name].Total+total-oldTotal;
	accountItemQty[name].Qty[charVaultName]={["Subtotal"]=total};
	for k,v in pairs(containerQty) do
		accountItemQty[name].Qty[charVaultName][k]=v;
	end
	-- now update the display
	if inventoryWindow.CharList:GetText()==charVaultName then
		if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
		if accountItemQty[name].ListItem[charVaultName]==nil then
			-- create a new entry, set height by filter and position it according to the sort
			local tmpEntry=ItemEntry(name,charVaultName,accountItemQty[name].Qty[charVaultName].Subtotal,itemInfo)
			accountItemQty[name].ListItem[charVaultName]=tmpEntry;
--***
--			inventoryPanel.ItemList:AddItem(tmpEntry)
			tmpEntry.panel.ItemList:AddItem(tmpEntry)
			inventoryPanel.SortList:SelectedIndexChanged();
		else
			accountItemQty[name].ListItem[charVaultName].Qty:SetText(accountItemQty[name].Qty[charVaultName].Subtotal);
		end
	elseif inventoryWindow.CharList:GetSelectedIndex()==1 then
		if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
		if accountItemQty[name].ListItem["ALL"]==nil then
			-- create a new entry, set height by filter and position it according to the sort
			local tmpEntry=ItemEntry(name,"ALL",accountItemQty[name].Total,itemInfo)
			accountItemQty[name].ListItem["ALL"]=tmpEntry;
--***
--			inventoryPanel.ItemList:AddItem(tmpEntry)
			tmpEntry.panel.ItemList:AddItem(tmpEntry)
			inventoryPanel.SortList:SelectedIndexChanged();
		else
			accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
		end
	end
end
vaultImage={};
function vaultAvailbleChanged()
--	Turbine.Shell.WriteLine("vault available changed:"..tostring(vault:IsAvailable()))
	refreshVault()
end
function refreshVault()
	if Settings.debug then
		Turbine.Shell.WriteLine("refreshVault")
	end
	if vault:IsAvailable() then
		local tmpItemFlag={}
		-- clear out any accountItemQty record for charVaultName and recalc the total for any item that had a charVaultName qty

		for k,v in pairs(vaultImage) do
			if v.Item~=nil then
				-- safely remove old callbacks
				if v.Item.QuantityChanged~=nil then
					RemoveCallback(v.Item,"QuantityChanged",vaultQuantityChanged);
					RemoveCallback(v.Item,"ChestChanged",vaultChestChanged);
				end
			end
		end
		vaultImage={};
		-- set flags for items that were in the old vault
		for k,v in pairs(accountItemQty) do
			local total=0;
			for name,qty in pairs(v.Qty) do
				if name==charVaultName then
					tmpItemFlag[k]=qty.Subtotal;
					accountItemQty[k].Qty[charVaultName]={["Subtotal"]=qty.Subtotal};
				end
			end
		end
		-- process the current Shared Storage, adding an accountItemQty record and updating total
		local size=vault:GetCount();
		CharList[charVaultName].used=size;
		CharList[charVaultName].capacity=vault:GetCapacity();
		if inventoryWindow.CharList:GetText()==charVaultName then
			inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[charVaultName].used).."/"..tostring(CharList[charVaultName].capacity)..")")
		end

		local tmpIndex,tmpItem;
		for tmpIndex=1,size do
			tmpItem=vault:GetItem(tmpIndex);
			vaultImage[tmpIndex]={}
			vaultImage[tmpIndex].Item=tmpItem;
			if tmpItem~=nil then
				local itemInfo=tmpItem:GetItemInfo();

				local name=tmpItem:GetName()
				local infoName=itemInfo:GetName()
				if name==nil then name=infoName end

				local qty=tmpItem:GetQuantity();
				vaultImage[tmpIndex].Index=tmpIndex;
				AddCallback(vaultImage[tmpIndex].Item,"QuantityChanged",vaultQuantityChanged);
				AddCallback(vaultImage[tmpIndex].Item,"ChestChanged",vaultChestChanged);
				vaultImage[tmpIndex].Item.Chest=vaultImage[tmpIndex].Item:GetChest();
				vaultImage[tmpIndex].ItemName=name;
				if accountItemQty[name]==nil then
					accountItemQty[name]={}
					accountItemQty[name].InfoName=infoName
					accountItemQty[name].Total=0;
					accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
					accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
					accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
					accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
					accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
					accountItemQty[name].Qty={};
					accountItemQty[name].ListItem={};
					local found=false;
					for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
						if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
							if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charVaultName then
								inventoryPanel.FilterList:ShowEntry(filterIndex);
							end
							found=true;
							break;
						end
					end
					if not found then
						accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
						if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charVaultName then
							inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
						end
					end
				end
				if accountItemQty[name].ItemInfo==nil then
					accountItemQty[name].ItemInfo=itemInfo;
				end
				if inventoryWindow.CharList:GetText()==charVaultName and accountItemQty[name].ListItem[charVaultName]==nil then
					local tmpEntry=ItemEntry(name,charVaultName,qty,itemInfo);					
--***
--					inventoryPanel.ItemList:AddItem(tmpEntry)
					tmpEntry.panel.ItemList:AddItem(tmpEntry)
					accountItemQty[name].ListItem[charVaultName]=tmpEntry;
				end
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[name].ListItem["ALL"]==nil then
					local tmpEntry=ItemEntry(name,"ALL",qty,itemInfo);					
--***
--					inventoryPanel.ItemList:AddItem(tmpEntry)
					tmpEntry.panel.ItemList:AddItem(tmpEntry)
					accountItemQty[name].ListItem["ALL"]=tmpEntry;
				end
				if accountItemQty[name].Qty[charVaultName]==nil then
					accountItemQty[name].Qty[charVaultName]={["Subtotal"]=qty};
				else
					if tmpItemFlag[name]==nil then
						accountItemQty[name].Qty[charVaultName].Subtotal=accountItemQty[name].Qty[charVaultName].Subtotal+qty;
					else
						accountItemQty[name].Total=accountItemQty[name].Total-tmpItemFlag[name];
						tmpItemFlag[name]=nil;
						accountItemQty[name].Qty[charVaultName].Subtotal=qty;
					end
				end
				local bag=tmpItem:GetChest();
				if accountItemQty[name].Qty[charVaultName][bag]==nil then
					accountItemQty[name].Qty[charVaultName][bag]=qty;
				else
					accountItemQty[name].Qty[charVaultName][bag]=accountItemQty[name].Qty[charVaultName][bag]+qty;
				end
				if inventoryWindow.CharList:GetText()==charVaultName and accountItemQty[name].ListItem[charVaultName].ItemInfo~=nil then
					accountItemQty[name].ListItem[charVaultName].ItemInfo:SetItemInfo(itemInfo);
					accountItemQty[name].ListItem[charVaultName].IconBorder:SetVisible(false);
					accountItemQty[name].ListItem[charVaultName].Qty:SetText(accountItemQty[name].Qty[charVaultName].Subtotal);
				end
				accountItemQty[name].Total=accountItemQty[name].Total+qty;
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[name].ListItem["ALL"]~=nil and accountItemQty[name].ListItem["ALL"].ItemInfo~=nil then
					accountItemQty[name].ListItem["ALL"].ItemInfo:SetItemInfo(itemInfo);
					accountItemQty[name].ListItem["ALL"].IconBorder:SetVisible(false);
					accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
				end
			end
		end
		for tmpItem,val in pairs(tmpItemFlag) do
			local tmpEntryPanel=getCurrentItemPanel(tmpItem)
			if inventoryWindow.CharList:GetText()==charVaultName and accountItemQty[tmpItem].ListItem[charVaultName]~=nil then
				-- remove the linked display entry
--***
--				local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem[charVaultName])
				local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem[charVaultName])
--***
				if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
				accountItemQty[tmpItem].ListItem[charVaultName]=nil;
			end

			-- these items existed previously and now have to be accounted for
			-- recalc the total instead of adjusting just in case the discrepancy was due to a character rename, data corruption or playing the account on another computer
			accountItemQty[tmpItem].Qty[charVaultName]=nil;
			local total=0;
			for char,qty in pairs(accountItemQty[tmpItem].Qty) do
				total=total+qty.Subtotal;
			end
			if total==0 then
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[tmpItem].ListItem~=nil and accountItemQty[tmpItem].ListItem["ALL"]~=nil then
					-- remove the "ALL" linked display entry
--***
--					local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem["ALL"])
					local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItemQty[tmpItem].ListItem["ALL"])
--***
--					if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					accountItemQty[tmpItem].ListItem["ALL"]=nil;
				end
				accountItemQty[tmpItem]=nil;
			else
				accountItemQty[tmpItem].Total=total;
				if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItemQty[tmpItem].ListItem~=nil and accountItemQty[tmpItem].ListItem["ALL"]~=nil then
					accountItemQty[tmpItem].ListItem["ALL"].Qty:SetText(total);
				end
			end
		end
		-- now reapply the sort
		if inventoryWindow.CharList:GetText()==charVaultName or inventoryWindow.CharList:GetSelectedIndex()==1 then
			inventoryPanel.SortList:SelectedIndexChanged();
		end
	end
end
--AddCallback(vault,"IsAvailableChanged",refreshVault);
AddCallback(vault,"IsAvailableChanged",vaultAvailbleChanged);

AddCallback(vault,"ItemsRefreshed",refreshVault);
vaultItemAdded=function(sender,args)
	-- Index, Item
	if Settings.debug then
		Turbine.Shell.WriteLine("vaultItemAdded");
	end
	local index=args.Index;
	local item=args.Item;
	table.insert(vaultImage,index,{});

	CharList[charVaultName].used=vault:GetCount();
	if inventoryWindow.CharList:GetText()==charVaultName then
		inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[charVaultName].used).."/"..tostring(CharList[charVaultName].capacity)..")")
	end
	vaultImage[index].Item=item;
	if item~=nil then
		local itemInfo=item:GetItemInfo()

		local name=item:GetName()
		local infoName=itemInfo:GetName()
		if name==nil then name=infoName end

		local qty=item:GetQuantity();
		vaultImage[index].Index=tmpIndex;
		AddCallback(vaultImage[index].Item,"QuantityChanged",vaultQuantityChanged);
		AddCallback(vaultImage[index].Item,"ChestChanged",vaultChestChanged);
		vaultImage[index].Item.Chest=vaultImage[index].Item:GetChest();
		vaultImage[index].ItemName=name;
		if accountItemQty[name]==nil then
			accountItemQty[name]={}
			accountItemQty[name].InfoName=infoName
			accountItemQty[name].Total=0;
			accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
			accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
			accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
			accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
			accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
			accountItemQty[name].Qty={};
			accountItemQty[name].ListItem={};
			local found=false;
			for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
				if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
					if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charVaultName then
						inventoryPanel.FilterList:ShowEntry(filterIndex);
					end
					found=true;
					break;
				end
			end
			if not found then
				accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
				if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charVaultName then
					inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
				end
			end
		end
		if accountItemQty[name].ItemInfo==nil then
			accountItemQty[name].ItemInfo=itemInfo;
			if accountItemQty[name].ListItem[charVaultName]~=nil then
				accountItemQty[name].ListItem[charVaultName].ItemInfo:SetItemInfo(itemInfo);
				accountItemQty[name].ListItem[charVaultName].IconBorder:SetVisible(false);
			end
		end
		if accountItemQty[name].Qty[charVaultName]==nil then
			accountItemQty[name].Qty[charVaultName]={["Subtotal"]=qty};
		else
			accountItemQty[name].Qty[charVaultName].Subtotal=accountItemQty[name].Qty[charVaultName].Subtotal+qty;
		end
		local bag=item:GetChest();
		if accountItemQty[name].Qty[charVaultName][bag]==nil then
			accountItemQty[name].Qty[charVaultName][bag]=qty;
		else
			accountItemQty[name].Qty[charVaultName][bag]=accountItemQty[name].Qty[charVaultName][bag]+qty;
		end
		accountItemQty[name].Total=accountItemQty[name].Total+qty;
		-- now update the display element
		if inventoryWindow.CharList:GetText()==charVaultName then
			if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
			if accountItemQty[name].ListItem[charVaultName]==nil then
				-- create a new entry, set height by filter and position it according to the sort
				local tmpEntry=ItemEntry(name,charVaultName,accountItemQty[name].Qty[charVaultName].Subtotal,itemInfo)
				accountItemQty[name].ListItem[charVaultName]=tmpEntry;
--***
--				inventoryPanel.ItemList:AddItem(tmpEntry)
				tmpEntry.panel.ItemList:AddItem(tmpEntry)
				if tmpEntry:GetHeight()>0 then
					inventoryPanel.SortList:SelectedIndexChanged();
				end
			else
				accountItemQty[name].ListItem[charVaultName].Qty:SetText(accountItemQty[name].Qty[charVaultName].Subtotal);
			end
		elseif inventoryWindow.CharList:GetSelectedIndex()==1 then
			if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
			if accountItemQty[name].ListItem["ALL"]==nil then
				-- create a new entry, set height by filter and position it according to the sort
				local tmpEntry=ItemEntry(name,"ALL",accountItemQty[name].Total,itemInfo)
				accountItemQty[name].ListItem["ALL"]=tmpEntry;
--***
--				inventoryPanel.ItemList:AddItem(tmpEntry)
				tmpEntry.panel.ItemList:AddItem(tmpEntry)
				inventoryPanel.SortList:SelectedIndexChanged();
			else
				accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
			end
		end
		-- now reapply the sort
		if inventoryWindow.CharList:GetText()==charVaultName or inventoryWindow.CharList:GetSelectedIndex()==1 then
			inventoryPanel.SortList:SelectedIndexChanged();
		end
	end
end
vaultItemRemoved=function(sender,args)
-- Index, Item
	if Settings.debug then
		Turbine.Shell.WriteLine("vaultItemRemoved");
	end
	local index=args.Index;

	local name=args.Item:GetName()
	local infoName=args.Item:GetItemInfo():GetName()
	if name==nil then name=infoName end

	local item=args.Item;
	local qty=0;
	if item~=nil then
		qty=item:GetQuantity();
		if item.QuantityChanged~=nil then
			RemoveCallback(item,"QuantityChanged",vaultQuantityChanged);
			RemoveCallback(item,"ChestChanged",vaultChestChanged);
		end
	end
	table.remove(vaultImage,index);
	local accountItem=accountItemQty[name];
 	if accountItem~=nil then
		local tmpEntryPanel=getCurrentItemPanel(name)
		accountItem.Total=accountItem.Total-qty;
		if accountItem.Qty[charVaultName]~=nil then
			local totQty=accountItem.Qty[charVaultName].Subtotal-qty
			if totQty==0 then
				accountItem.Qty[charVaultName]=nil
				-- remove the display entry
				if inventoryWindow.CharList:GetText()==charVaultName and accountItem.ListItem[charVaultName]~=nil then
					-- remove the linked display entry
--***
--					local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem[charVaultName])
					local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem[charVaultName])
--***
--					if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					accountItem.ListItem[charVaultName]=nil;
				end
			else
				local bag=item:GetChest();
				accountItem.Qty[charVaultName][bag]=accountItem.Qty[charVaultName][bag]-qty;
				if accountItem.Qty[charVaultName][bag]==0 then accountItem.Qty[charVaultName][bag]=nil end
				accountItem.Qty[charVaultName].Subtotal=totQty;
				if inventoryWindow.CharList:GetText()==charVaultName and accountItem.ListItem[charVaultName]~=nil then
					-- update the linked display entry
					accountItem.ListItem[charVaultName].Qty:SetText(totQty);
				end
			end
		end
		if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItem.ListItem["ALL"]~=nil then
			if accountItem.Total==0 then
				-- remove the linked display entry
--***
--				local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem["ALL"])
				local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem["ALL"])
--***
--				if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
				if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
				accountItem.ListItem["ALL"]=nil;
			else
				accountItem.ListItem["ALL"].Qty:SetText(accountItem.Total);
			end
		end
	end
	CharList[charVaultName].used=CharList[charVaultName].used-1; -- can't use the GetCount since this event is fired BEFORE the item is actually removed
	if inventoryWindow.CharList:GetText()==charVaultName then
		inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[charVaultName].used).."/"..tostring(CharList[charVaultName].capacity)..")")
	end
end
AddCallback(vault,"ItemAdded",vaultItemAdded);
AddCallback(vault,"ItemRemoved",vaultItemRemoved);

function refreshBackPackImage()
	if Settings.debug then
		Turbine.Shell.WriteLine("refreshBackPackImage")
	end
	local used=0;
	local index
	for index=1,backPack:GetSize() do
		if backpackImage[index]~=nil and backpackImage[index].Item~=nil then
			-- safely remove old callbacks
			if backpackImage[index].Item.QuantityChanged~=nil then
				RemoveCallback(backpackImage[index].Item,"QuantityChanged",quantityChanged)
			end
		end
		backpackImage[index]={};
		backpackImage[index].Item=backPack:GetItem(index);
		if backpackImage[index].Item~=nil then
			used=used+1;
			backpackImage[index].Index=index;
			AddCallback(backpackImage[index].Item,"QuantityChanged",quantityChanged);
			backpackImage[index].Item.Index=index;
--			backpackImage[index].ItemName=backpackImage[index].Item:GetItemInfo():GetName()
			backpackImage[index].ItemName=backpackImage[index].Item:GetName()
			backpackImage[index].ItemInfoName=backpackImage[index].Item:GetItemInfo():GetName()
		end
	end
	CharList[charName].used=used;
	CharList[charName].capacity=backPack:GetSize()
	if inventoryWindow.CharList:GetText()==charName then
		inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[charName].used).."/"..tostring(CharList[charName].capacity)..")")
	end
end
refreshBackPackImage()
itemAdded=function(sender,args)
	if Settings.debug then
		Turbine.Shell.WriteLine("itemAdded")
		Table.Dump(args)
		Turbine.Shell.WriteLine("name:"..backPack:GetItem(args.Index):GetName())
	end
	-- Index, Item
	local index=args.Index;
	if index~=nil then
		if backpackImage[index]~=nil and backpackImage[index].Item~=nil then
			-- safely remove old callbacks
			if backpackImage[index].Item.QuantityChanged~=nil then
				RemoveCallback(backpackImage[index].Item,"QuantityChanged",quantityChanged)
			end
			backpackImage[index].ItemName=nil
			backpackImage[index].ItemInfoName=nil
			backpackImage[index].Item=nil;
			backpackImage[index].Index=nil;
			backpackImage[index]=nil;
		end
		backpackImage[index]={};
		backpackImage[index].Item=backPack:GetItem(index);
		if backpackImage[index].Item~=nil then
			CharList[charName].used=CharList[charName].used+1;
			if inventoryWindow.CharList:GetText()==charName then
				inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[charName].used).."/"..tostring(CharList[charName].capacity)..")")
			end
			backpackImage[index].Index=index;
			AddCallback(backpackImage[index].Item,"QuantityChanged",quantityChanged)
			backpackImage[index].Item.Index=index;
			-- we need to track the item name for itemRemoved events
			local itemInfo=backpackImage[index].Item:GetItemInfo()
			local name=backpackImage[index].Item:GetName()
			local infoName=itemInfo:GetName()
			if name==nil then name=infoName end
			local qty=backpackImage[index].Item:GetQuantity();
			if accountItemQty[name]==nil then
				accountItemQty[name]={}
				accountItemQty[name].InfoName=infoName
				accountItemQty[name].Total=0;
				accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
				accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
				accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
				accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
				accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
				accountItemQty[name].Qty={};
				accountItemQty[name].ListItem={};
				local found=false;
				for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
					if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
						if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charName then
							inventoryPanel.FilterList:ShowEntry(filterIndex);
						end
						found=true;
						break;
					end
				end
				if not found then
					accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
					if inventoryWindow.CharList:GetSelectedIndex()==1 or inventoryWindow.CharList:GetText()==charName then
						inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
					end
				end
			end
			if accountItemQty[name].ItemInfo==nil then
				accountItemQty[name].ItemInfo=itemInfo;
				if inventoryWindow.CharList:GetText()==charName and accountItemQty[name].ListItem[charName]~=nil then
					if Settings.totalsOnly then
						accountItemQty[name].ListItem[charName].ItemInfo:SetItemInfo(itemInfo);
						accountItemQty[name].ListItem[charName].IconBorder:SetVisible(false);
					else
						if accountItemQty[name].ListItem[charName][index]~=nil then
							accountItemQty[name].ListItem[charName][index].ItemInfo:SetItemInfo(itemInfo);
							accountItemQty[name].ListItem[charName][index].IconBorder:SetVisible(false);
						end
					end
				end
			end
			if accountItemQty[name].Qty[charName]==nil then
				accountItemQty[name].Qty[charName]={["Subtotal"]=qty};
			else
				accountItemQty[name].Qty[charName].Subtotal=accountItemQty[name].Qty[charName].Subtotal+qty;
			end
			accountItemQty[name].Qty[charName][index]=qty
			accountItemQty[name].Total=accountItemQty[name].Total+qty;
			backpackImage[index].ItemName=name
			-- update display
			if inventoryWindow.CharList:GetText()==charName then
				if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
				if Settings.totalsOnly then
					if accountItemQty[name].ListItem[charName]==nil then
						-- create a new entry, set height by filter and position it according to the sort
						local tmpEntry=ItemEntry(name,charName,accountItemQty[name].Qty[charName].Subtotal,itemInfo)
						if not Settings.totalsOnly then
							tmpEntry.Item:SetItem(backPack:GetItem(index))
							tmpEntry.Item.Index=index;
						end
						accountItemQty[name].ListItem[charName]=tmpEntry;
--***
--						inventoryPanel.ItemList:AddItem(tmpEntry)
						tmpEntry.panel.ItemList:AddItem(tmpEntry)
						if tmpEntry:GetHeight()>0 then
							inventoryPanel.SortList:SelectedIndexChanged();
						end
					else
						accountItemQty[name].ListItem[charName].Qty:SetText(accountItemQty[name].Qty[charName].Subtotal);
					end
				else
					if accountItemQty[name].ListItem[charName]==nil then accountItemQty[name].ListItem[charName]={} end
					if accountItemQty[name].ListItem[charName][index]==nil then
						-- create a new entry, set height by filter and position it according to the sort
						local tmpEntry=ItemEntry(name,charName,qty,itemInfo)
						if not Settings.totalsOnly then
							tmpEntry.Item:SetItem(backPack:GetItem(index))
							tmpEntry.Item.Index=index;
						end
						accountItemQty[name].ListItem[charName][index]=tmpEntry;
--***
--						inventoryPanel.ItemList:AddItem(tmpEntry)
						tmpEntry.panel.ItemList:AddItem(tmpEntry)
						if tmpEntry:GetHeight()>0 then
							inventoryPanel.SortList:SelectedIndexChanged();
						end
--Turbine.Shell.WriteLine("tmpEntry.IconBack:GetHeight():"..tostring(tmpEntry.IconBack:GetHeight()))
--Turbine.Shell.WriteLine("tmpEntry.ItemInfo:GetHeight():"..tostring(tmpEntry.ItemInfo:GetHeight()))
--Turbine.Shell.WriteLine("tmpEntry.Icon:GetHeight():"..tostring(tmpEntry.Icon:GetHeight()))
					else
						accountItemQty[name].ListItem[charName][index].Qty:SetText(qty);
					end
				end
			elseif inventoryWindow.CharList:GetSelectedIndex()==1 then
				if accountItemQty[name].ListItem==nil then accountItemQty[name].ListItem={} end
				if accountItemQty[name].ListItem["ALL"]==nil then
					-- create a new entry, set height by filter and position it according to the sort
					local tmpEntry=ItemEntry(name,"ALL",accountItemQty[name].Total,itemInfo)
					accountItemQty[name].ListItem["ALL"]=tmpEntry;
--***
--					inventoryPanel.ItemList:AddItem(tmpEntry)
					tmpEntry.panel.ItemList:AddItem(tmpEntry)
					inventoryPanel.SortList:SelectedIndexChanged();
				else
					accountItemQty[name].ListItem["ALL"].Qty:SetText(accountItemQty[name].Total);
				end
			end
			-- now reapply the sort
			if inventoryWindow.CharList:GetText()==charName or inventoryWindow.CharList:GetSelectedIndex()==1 then
				inventoryPanel.SortList:SelectedIndexChanged();
			end
		end
	end
end
itemRemoved=function(sender,args)
	if Settings.debug then
		Turbine.Shell.WriteLine("itemRemoved")
		Table.Dump(args)
		Turbine.Shell.WriteLine("name:"..backPack:GetItem(args.Index):GetName())
	end
	-- Index
	local index=args.Index;
	local bag=math.floor((index+14)/15);
	if index~=nil and backpackImage[index]~=nil and backpackImage[index].Item~=nil then
		-- safely remove old callbacks
		if backpackImage[index].Item.QuantityChanged~=nil then
			RemoveCallback(backpackImage[index].Item,"QuantityChanged",quantityChanged);
		end
		local name=backpackImage[index].ItemName;
		local item=backpackImage[index].Item;
		local qty=item:GetQuantity();
		local accountItem=accountItemQty[name];
		if accountItem~=nil then
			local tmpEntryPanel=getCurrentItemPanel(name)
			accountItem.Total=accountItem.Total-qty;
			if accountItem.Qty[charName]~=nil then
				totQty=accountItem.Qty[charName].Subtotal-qty;
				accountItem.Qty[charName][index]=nil;
				if inventoryWindow.CharList:GetText()==charName and accountItem.ListItem[charName]~=nil then
					-- remove the linked display entry
					if Settings.totalsOnly then
						if totQty==0 then
--***
--							local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem[charName])
							local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem[charName])
--***
--							if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
							if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
							accountItem.ListItem[charName]=nil;
						else
							accountItem.ListItem[charName].Qty:SetText(totQty);
						end
					else
						if accountItem.ListItem[charName][index]~=nil then
--***
--							local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem[charName][index])
							local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem[charName][index])
--***
--							if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
							if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end
							accountItem.ListItem[charName][index]=nil
						end
					end
				end
				if totQty==0 then
					accountItem.Qty[charName]=nil;
					accountItem.ListItem[charName]=nil;
				else
					accountItem.Qty[charName].Subtotal=totQty;
				end
			end
			if inventoryWindow.CharList:GetSelectedIndex()==1 and accountItem.ListItem["ALL"]~=nil then
				if accountItem.Total==0 then
					-- remove the linked display entry
--***
--					local tmpIndex=inventoryPanel.ItemList:IndexOfItem(accountItem.ListItem["ALL"])
					local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItem.ListItem["ALL"])
--***
--					if tmpIndex~=nil and tmpIndex>0 then inventoryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end;
					accountItem.ListItem["ALL"]=nil;
				else
					accountItem.ListItem["ALL"].Qty:SetText(accountItem.Total);
				end
			end
		end

		backpackImage[index].ItemName=nil;
		backpackImage[index].Item=nil;
		backpackImage[index].Index=nil;
		backpackImage[index]=nil;
	end
	if CharList[charName].used~=nil then
		CharList[charName].used=CharList[charName].used-1;
	end
	if inventoryWindow.CharList:GetText()==charName then
		inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[charName].used).."/"..tostring(CharList[charName].capacity)..")")
	end
end
itemMoved=function(sender,args)
	if Settings.debug then
		Turbine.Shell.WriteLine("itemMoved")
		Table.Dump(args)
		Turbine.Shell.WriteLine("new name:"..backPack:GetItem(args.NewIndex):GetName())
	end
	if args~=nil and args.Item~=nil then
		-- in U46, the itemMoved event started being fired without an item.
		-- Item, NewIndex, OldIndex
		local itemName=args.Item:GetName()
		local itemInfoName=args.Item:GetItemInfo():GetName()
		local newIndex=args.NewIndex
		local newBag=math.floor((newIndex+14)/15);
		local oldIndex=args.OldIndex
		local oldBag=math.floor((oldIndex+14)/15);
		local qty=args.Item:GetQuantity();
		if accountItemQty[itemName]==nil then accountItemQty[itemName]={} end
		if accountItemQty[itemName].Qty==nil then accountItemQty[itemName].Qty={} end
		if accountItemQty[itemName].Qty[charName]==nil then
			accountItemQty[itemName].Qty[charName]={}
		else
			accountItemQty[itemName].Qty[charName][oldIndex]=nil;
		end
		accountItemQty[itemName].Qty[charName][newIndex]=qty;
		if Settings.totalsOnly then
			-- nothing to do
		else
			local tmpEntryPanel=getCurrentItemPanel(itemName)
			if accountItemQty[itemName].ListItem[charName]~=nil and accountItemQty[itemName].ListItem[charName][oldIndex]~=nil then
				if accountItemQty[itemName].ListItem[charName][newIndex]~=nil then
					-- we are in the middle of a drag/drop and there has been a bogus "itemAdded" event fired
					-- remove the extraneous bogus element from the ItemList
--***
					local tmpIndex=tmpEntryPanel.ItemList:IndexOfItem(accountItemQty[itemName].ListItem[charName][newIndex])
--***
					if tmpIndex~=nil and tmpIndex>0 then tmpEntryPanel.ItemList:RemoveItemAt(tmpIndex) end
				end
				accountItemQty[itemName].ListItem[charName][newIndex]=accountItemQty[itemName].ListItem[charName][oldIndex];
				accountItemQty[itemName].ListItem[charName][newIndex].Item.Index=newIndex;
				accountItemQty[itemName].ListItem[charName][oldIndex]=nil;
			end
		end
		if backpackImage[oldIndex]~=nil and backpackImage[oldIndex].Item~=nil then
			-- safely remove old callbacks
			if backpackImage[oldIndex].Item.QuantityChanged~=nil then
				RemoveCallback(backpackImage[oldIndex].Item,"QuantityChanged",quantityChanged)
			end
			backpackImage[oldIndex].ItemName=nil;
			backpackImage[oldIndex].ItemInfoName=nil
			backpackImage[oldIndex].Item=nil;
			backpackImage[oldIndex].Index=nil;
			backpackImage[oldIndex]=nil;
		end
		if backpackImage[newIndex]~=nil and backpackImage[newIndex].Item~=nil then
			-- safely remove old callbacks
			if backpackImage[newIndex].Item.QuantityChanged~=nil then
				RemoveCallback(backpackImage[newIndex].Item,"QuantityChanged",quantityChanged)
			end
			backpackImage[newIndex].ItemName=nil
			backpackImage[newIndex].ItemInfoName=nil
			backpackImage[newIndex].Item=nil;
			backpackImage[newIndex].Index=nil;
			backpackImage[newIndex]=nil;
		end

		backpackImage[oldIndex]={};
		backpackImage[oldIndex].Item=backPack:GetItem(oldIndex);
		if backpackImage[oldIndex].Item~=nil then
			-- should never happen
			backpackImage[oldIndex].Index=oldIndex;
			AddCallback(backpackImage[oldIndex].Item,"QuantityChanged",quantityChanged);
			backpackImage[oldIndex].Item.Index=oldIndex;
			backpackImage[oldIndex].ItemName=backpackImage[oldIndex].Item:GetName();
			backpackImage[oldIndex].ItemInfoName=backpackImage[oldIndex].Item:GetItemInfo():GetName();
		end
		backpackImage[newIndex]={}
		backpackImage[newIndex].Item=backPack:GetItem(newIndex);
		if backpackImage[newIndex].Item~=nil then
			backpackImage[newIndex].Index=newIndex;
			AddCallback(backpackImage[newIndex].Item,"QuantityChanged",quantityChanged)
			backpackImage[newIndex].Item.Index=newIndex;
			backpackImage[newIndex].ItemName=backpackImage[newIndex].Item:GetName()
			backpackImage[newIndex].ItemInfoName=backpackImage[newIndex].Item:GetItemInfo():GetName()
		end
	end
end
AddCallback(backPack,"ItemAdded",itemAdded)
AddCallback(backPack,"ItemRemoved",itemRemoved)
AddCallback(backPack,"ItemMoved",itemMoved)

accountItemQty=PatchDataLoad(Turbine.DataScope.Server, "AltInventoryData");
equippedItems=PatchDataLoad(Turbine.DataScope.Server, "AltInventoryEI");
if equippedItems==nil then equippedItems={} end

function LoadData()
	-- now load all other data
	if accountItemQty==nil then accountItemQty={} end
	
	-- remove any data from current character
	for k,v in pairs(accountItemQty) do
		local qty=v.Qty[charName];
		local total=0;
		if qty~=nil then
			accountItemQty[k].Qty[charName]=nil;
		end
		for name,qty in pairs(v.Qty) do
			if name~=charName then
				if type(qty)=="table" then
					total=total+qty.Subtotal;
				else
					total=total+qty;
				end
			end
		end
		accountItemQty[k].Total=total;
		if accountItemQty[k].Total==0 then
			accountItemQty[k]=nil;
		else
			if accountItemQty[k].SortCategory==0 then
				-- attempt to resolve the sort category
				for index,cat in ipairs(ItemCategory) do
					if accountItemQty[k].Category==cat[1] then
						accountItemQty[k].SortCategory=accountItemQty[k].Category;
						break;
					end
				end
			end
		end
	end
	-- now re-add current character bag items
	local tmpIndex;
	for tmpIndex=1,backPack:GetSize() do
		local tmpItem=backPack:GetItem(tmpIndex);
		if tmpItem~=nil then
			local itemInfo=tmpItem:GetItemInfo();

			local name=tmpItem:GetName()
			local infoName=itemInfo:GetName()
			if name==nil then name=infoName end

			local qty=tmpItem:GetQuantity();
			if accountItemQty[name]==nil then
				accountItemQty[name]={}
				accountItemQty[name].InfoName=infoName
				accountItemQty[name].Total=0;
				accountItemQty[name].BackgroundImageID=itemInfo:GetBackgroundImageID();
				accountItemQty[name].IconImageID=itemInfo:GetIconImageID();
				accountItemQty[name].Quality=GetQualityCode(itemInfo:GetQuality());
				accountItemQty[name].Category=GetCategoryCode(itemInfo:GetCategory());
				accountItemQty[name].SortCategory=GetCategoryCode(itemInfo:GetCategory());
				accountItemQty[name].Qty={};
				accountItemQty[name].ListItem={};
				local found=false;
				for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
					if accountItemQty[name].Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue then
						inventoryPanel.FilterList:ShowEntry(filterIndex);
						found=true;
						break;
					end
				end
				if not found then
					accountItemQty[name].SortCategory=Turbine.Gameplay.ItemCategory.Undefined;
					inventoryPanel.FilterList:ShowEntry(undefinedCategoryIndex);
				end
			end
			if itemInfo~=nil then
				accountItemQty[name].ItemInfo=itemInfo
			end
--			if accountItemQty[name].ItemInfo==nil then
--				accountItemQty[name].ItemInfo=itemInfo;
--			end
			if accountItemQty[name].Qty[charName]==nil then
				accountItemQty[name].Qty[charName]={["Subtotal"]=qty};
			else
				accountItemQty[name].Qty[charName].Subtotal=accountItemQty[name].Qty[charName].Subtotal+qty;
			end
			accountItemQty[name].Qty[charName][tmpIndex]=qty;
			accountItemQty[name].Total=accountItemQty[name].Total+tmpItem:GetQuantity();
		end
	end

	-- initialize the character list
	local minMenu=minimalWindow.CharMenu:GetItems();
	function populateCharLists()
		inventoryWindow.CharList:AddItem(Resource[language][12]);
		minMenu:Add(Turbine.UI.MenuItem(Resource[language][12]));
		inventoryWindow.CharList:AddItem(Resource[language][9]);
		minMenu:Add(Turbine.UI.MenuItem(Resource[language][9]));
		inventoryWindow.CharList:AddItem(Resource[language][118]);
		minMenu:Add(Turbine.UI.MenuItem(Resource[language][118]));
		local selIndex=1;
		local tmpList={};
		for k,v in pairs(CharList) do
			if k~="Shared Storage" then
				table.insert(tmpList,k)
--				tmpList[#tmpList+1]=k
			end
		end
		table.sort(tmpList)
		for k,v in ipairs(tmpList) do
			if v==charName then
				selIndex=k+3
				inventoryWindow.currentCharBagsIndex=k+3
			end
			inventoryWindow.CharList:AddItem(v);
			minMenu:Add(Turbine.UI.MenuItem(v));
		end
		if Settings.defaultToAllView==true then
			inventoryWindow.CharList:SetSelectedIndex(1);
		else
			inventoryWindow.CharList:SetSelectedIndex(selIndex);
		end
	end
	populateCharLists()

	inventoryWindow.CharList:SelectedIndexChanged();
	for tmpIndex=1,minMenu:GetCount() do
		minMenu:Get(tmpIndex).Click=function(sender,args)
			inventoryWindow.CharList:SetSelectedIndex(tmpIndex);
			-- have to manually trigger the control to update otherwise the "Current" value doesn't update in time
			inventoryWindow.CharList.ListData:SelectedIndexChanged();
			inventoryWindow.CharList:SelectedIndexChanged();
		end
	end
end
detailMenu=Turbine.UI.ContextMenu()

-- handle reload condition (this is a temporary workaround for an issue with floating displays)
if Settings.reloading==true then
	Settings.reloading=false
	if Settings.showGroupMaint==true then
		Settings.showGroupMaint=false
		groupMaint:Show()
		if Settings.selectedGroupIndex~=nil then
			-- try to reselect the correct group before showing the window
			groupMaint.groupSelect:SetSelectedIndex(Settings.selectedGroupIndex)
		end
		if Settings.groupMaintMessage~=nil then
			setGroupMaintMessage(Settings.groupMaintMessage)
			Settings.groupMaintMessage=nil
		end
	end
	-- the reloader plugin gets unloaded in the setupWindow.Update handler - that allows the Reloader plugin to complete its loading state before being unceremoniously unloaded
end

function ReloadAltInventory()
	Settings.reloading=true
	if groupMaint:IsVisible() then
		Settings.showGroupMaint=true
		Settings.selectedGroupIndex=groupMaint.groupSelect:GetSelectedIndex()
		Settings.groupMaintMessage=groupMaint.message:GetText()
	end
	Turbine.PluginManager.LoadPlugin("AIReloader");	
end
function ExportItemData()
	-- function to export AIItemData to script log, used on each update to publish new data (used to come from ItemExplorer plugin, but no longer needed)
	tmpMaxAIItemData=0
	Turbine.Engine.ScriptLog("*** START ITEM DATA ***")
	Turbine.Engine.ScriptLog("AIItemData={}")
	for category,items in pairs(AIItemData) do
		Turbine.Engine.ScriptLog("AIItemData["..tostring(category).."]={}")
		for k,v in pairs(items) do
			if k>tmpMaxAIItemData then tmpMaxAIItemData=k end
			-- need to escape new lines, tabs, and carriage returns - probably embedded quotes too
			local tmpName=string.gsub(string.gsub(string.gsub(tostring(v.name),"\n","\\010"),"\t","\\009"),"\r","\\013")
			Turbine.Engine.ScriptLog("AIItemData["..tostring(category).."]["..tostring(k).."]={[\"name\"]=\""..tmpName.."\",[\"backgroundImageID\"]="..tostring(v.backgroundImageID)..",[\"durability\"]="..tostring(v.durability)..",[\"iconImageID\"]="..tostring(v.iconImageID)..",[\"maxQuantity\"]="..tostring(v.maxQuantity)..",[\"maxStackSize\"]="..tostring(v.maxStackSize)..",[\"quality\"]="..tostring(v.quality)..",[\"qualityImageID\"]="..tostring(v.qualityImageID)..",[\"shadowImageID\"]="..tostring(v.shadowImageID)..",[\"underlayImageID\"]="..tostring(v.underlayImageID)..",[\"isMagic\"]="..tostring(v.isMagic)..",[\"isUnique\"]="..tostring(v.isUnique).."}")
		end
	end
	Turbine.Engine.ScriptLog("maxAIItemData="..string.format("0x%x",tmpMaxAIItemData))
	Turbine.Engine.ScriptLog("*** END ITEM DATA ***")
end
function getItemInfo(genericItemID,uniqueItemID)
	-- returns a Turbine.Gameplay.ItemInfo entity for the given itemID or nil
	-- useful for determining if an itemID is a valid item as well as getting the item information, such as ItemInfo:GetDescription()
	local tmpItemInfo
	local sc
	if uniqueItemID==nil then
		sc=Turbine.UI.Lotro.Shortcut(Turbine.UI.Lotro.ShortcutType.Item,"0x0,"..string.format("0x%x",genericItemID))
	else
-- haven't tested with unique IDs yet
		sc=Turbine.UI.Lotro.Shortcut(Turbine.UI.Lotro.ShortcutType.Item,string.format("0x%x",uniqueItemID))
	end
	local success, results=pcall(Turbine.UI.Lotro.Quickslot.SetShortcut,itemDataTestQS,sc)
	if success then
		local tmpItem=sc:GetItem()
		if tmpItem~=nil then
			tmpItemInfo=tmpItem:GetItemInfo()					
		end
	end
	return tmpItemInfo
end
-- functions for debugging
function safeHexFormat(x)
	if x==nil then
		return nil
	else
		return string.format("0x%x",x)
	end
end
function flushItemData()
	AIItemData={}
	getItemID("Barrow-brie")
--	knownMissingItems={}
--	discoveredItemData={}
end

function findItemIDs(name,category,durability,quality,minID,maxID,tmpIndex,maxCount)
	-- uses AIItemDataIndex[category] to allow retrieving results in groups of 100 max to improve performance
	-- returns a maximum of 500 items. if more items are desired, call again with the minID one greater than the greatest ID of the prior group
	-- Note: category is REQUIRED if you need to iterate more than one category, do so from the calling routine
	local count=0
	-- convert name to lowercase and strip accent
	if name~=nil then
		name=string.stripaccent(string.lower(name))
	end
	local ret
	if tmpIndex==nil then tmpIndex=1 end
	if AIItemData~=nil then
		if AIItemData[category]~=nil and AIItemDataIndex[category]~=nil then
			local maxIndex=#AIItemDataIndex[category]
			if maxCount==nil then maxCount=maxIndex end
			while tmpIndex<=maxIndex and count<=maxCount do
				k=AIItemDataIndex[category][tmpIndex]
				v=AIItemData[category][k]
				if (minID==nil or k>=minID) and (maxID==nil or k<=maxID) then
					if (name==nil or string.find(string.stripaccent(string.lower(v.name)),name,nil,true)~=nil) and (durability==nil or v.durability==durability) and (quality==nil or v.quality==quality) and (iconImageID==nil or v.iconImageID==iconImageID) then
						if ret==nil then ret={} end
						table.insert(ret,{k,v.name,category})
						count=count+1
					end
				end
				tmpIndex=tmpIndex+1
			end
		end
	end
	return ret,tmpIndex
end
