-- test woodworker 16=category 299 (index=266)
--NOTE: ItemCategory KEYS do not correspond to Turbine category codes. ItemCategory VALUES correspond toTurbine category codes.
-- to get the index from a turbine category code, use ItemCategoryIndex, i.e. catIndex=ItemCategoryIndex[turbineCode]
-- also, since ItemCategory gets sorted by current language string the index values are MEANINGLESS between runs and SHOULD NOT BE STORED
-- this table is used to resolve the accountItemQty[name].SortCategory values

--ItemCategory[ItemCategoryIndex]={itemCategoryValue,{ENstring,FRstring,DEstring}}
--strings are added at the end of the file from ItemCategoryString[language]

-- AIItemData[category] should have category keys for all of the in-use items, thus all of the in-use categories
-- log any categories in AIItemData that are not in ItemCategory so we can figure out what they should be...
logMissingItemCategory=function()
	Turbine.Engine.ScriptLog("*** Start Missing Categories ***")
	for cat,data in pairs(AIItemData) do
		local found=false
		for k,v in pairs(ItemCategory) do
			if v[1]==cat then
				found=true
				break
			end
		end
		if not found then
			-- there is no name. for now, just send it to the script log
			Turbine.Engine.ScriptLog("cat:"..tostring(cat))
		end
	end
	Turbine.Engine.ScriptLog("*** End Missing Categories ***")
end

-- add missing categories to Turbine.Gameplay.ItemCategory enumeration
if Turbine.Gameplay~=nil and Turbine.Gameplay.ItemCategory~=nil then

	if Turbine.Gameplay.ItemCategory.SkillManual==nil then Turbine.Gameplay.ItemCategory.SkillManual=186 end
	if Turbine.Gameplay.ItemCategory.TaskItem==nil then Turbine.Gameplay.ItemCategory.TaskItem=207 end
	if Turbine.Gameplay.ItemCategory.EstemnetMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetMetalsmithScroll=208 end
	if Turbine.Gameplay.ItemCategory.EstemnetJewellerScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetJewellerScroll=209 end
	if Turbine.Gameplay.ItemCategory.EstemnetWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetWoodworkerScroll=210 end
	if Turbine.Gameplay.ItemCategory.EstemnetWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetWeaponsmithScroll=211 end
	if Turbine.Gameplay.ItemCategory.EstemnetTailorScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetTailorScroll=212 end
	if Turbine.Gameplay.ItemCategory.EstemnetFarmerScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetFarmerScroll=213 end
	if Turbine.Gameplay.ItemCategory.EstemnetForesterScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetForesterScroll=214 end
	if Turbine.Gameplay.ItemCategory.EstemnetProspectorScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetProspectorScroll=215 end
	if Turbine.Gameplay.ItemCategory.EstemnetCookScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetCookScroll=216 end
	if Turbine.Gameplay.ItemCategory.EstemnetScholarScroll==nil then Turbine.Gameplay.ItemCategory.EstemnetScholarScroll=217 end
	if Turbine.Gameplay.ItemCategory.Bridle==nil then Turbine.Gameplay.ItemCategory.Bridle=218 end
	if Turbine.Gameplay.ItemCategory.WestemnetScholarScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetScholarScroll=220 end
	if Turbine.Gameplay.ItemCategory.WestemnetMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetMetalsmithScroll=221 end
	if Turbine.Gameplay.ItemCategory.WestemnetJewellerScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetJewellerScroll=222 end
	if Turbine.Gameplay.ItemCategory.WestemnetWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetWoodworkerScroll=223 end
	if Turbine.Gameplay.ItemCategory.WestemnetWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetWeaponsmithScroll=224 end
	if Turbine.Gameplay.ItemCategory.WestemnetTailorScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetTailorScroll=225 end
	if Turbine.Gameplay.ItemCategory.WestemnetFarmerScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetFarmerScroll=226 end
	if Turbine.Gameplay.ItemCategory.WestemnetForesterScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetForesterScroll=227 end
	if Turbine.Gameplay.ItemCategory.WestemnetProspectorScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetProspectorScroll=228 end
	if Turbine.Gameplay.ItemCategory.WestemnetCookScroll==nil then Turbine.Gameplay.ItemCategory.WestemnetCookScroll=229 end
	if Turbine.Gameplay.ItemCategory.BoxMediumArmour==nil then Turbine.Gameplay.ItemCategory.BoxMediumArmour=230 end
	if Turbine.Gameplay.ItemCategory.BoxHeavyArmour==nil then Turbine.Gameplay.ItemCategory.BoxHeavyArmour=231 end
	if Turbine.Gameplay.ItemCategory.BoxLightArmour==nil then Turbine.Gameplay.ItemCategory.BoxLightArmour=232 end
	if Turbine.Gameplay.ItemCategory.Essence==nil then Turbine.Gameplay.ItemCategory.Essence=235 end
	if Turbine.Gameplay.ItemCategory.DoomfoldCookScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldCookScroll=236 end
	if Turbine.Gameplay.ItemCategory.DoomfoldJewellerScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldJewellerScroll=237 end
	if Turbine.Gameplay.ItemCategory.DoomfoldTailorScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldTailorScroll=238 end
	if Turbine.Gameplay.ItemCategory.DoomfoldWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldWoodworkerScroll=239 end
	if Turbine.Gameplay.ItemCategory.DoomfoldWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldWeaponsmithScroll=241 end
	if Turbine.Gameplay.ItemCategory.DoomfoldScholarScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldScholarScroll=242 end
	if Turbine.Gameplay.ItemCategory.DoomfoldMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.DoomfoldMetalsmithScroll=243 end
	if Turbine.Gameplay.ItemCategory.IronfoldMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldMetalsmithScroll=244 end
	if Turbine.Gameplay.ItemCategory.IronfoldJewellerScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldJewellerScroll=245 end
	if Turbine.Gameplay.ItemCategory.IronfoldTailorScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldTailorScroll=246 end
	if Turbine.Gameplay.ItemCategory.IronfoldWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldWoodworkerScroll=247 end
	if Turbine.Gameplay.ItemCategory.IronfoldCookScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldCookScroll=248 end
	if Turbine.Gameplay.ItemCategory.IronfoldWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldWeaponsmithScroll=249 end
	if Turbine.Gameplay.ItemCategory.IronfoldScholarScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldScholarScroll=250 end
	if Turbine.Gameplay.ItemCategory.Lootbox==nil then Turbine.Gameplay.ItemCategory.Lootbox=251 end
	if Turbine.Gameplay.ItemCategory.IronfoldFarmerScroll==nil then Turbine.Gameplay.ItemCategory.IronfoldFarmerScroll=252 end
	if Turbine.Gameplay.ItemCategory.WeaponAura==nil then Turbine.Gameplay.ItemCategory.WeaponAura=253 end
	if Turbine.Gameplay.ItemCategory.MinasIthilMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilMetalsmithScroll=254 end
	if Turbine.Gameplay.ItemCategory.GundabadMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.GundabadMetalsmithScroll=255 end
	if Turbine.Gameplay.ItemCategory.MinasIthilJewellerScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilJewellerScroll=256 end
	if Turbine.Gameplay.ItemCategory.MinasIthilTailorScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilTailorScroll=258 end
	if Turbine.Gameplay.ItemCategory.GundabadJewellerScroll==nil then Turbine.Gameplay.ItemCategory.GundabadJewellerScroll=259 end
	if Turbine.Gameplay.ItemCategory.MinasIthilWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilWoodworkerScroll=260 end
	if Turbine.Gameplay.ItemCategory.GundabadTailorScroll==nil then Turbine.Gameplay.ItemCategory.GundabadTailorScroll=261 end
	if Turbine.Gameplay.ItemCategory.GundabadWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.GundabadWoodworkerScroll=264 end
	if Turbine.Gameplay.ItemCategory.MinasIthilWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilWeaponsmithScroll=270 end
	if Turbine.Gameplay.ItemCategory.GundabadWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.GundabadWeaponsmithScroll=271 end
	if Turbine.Gameplay.ItemCategory.MinasIthilScholarScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilScholarScroll=273 end
	if Turbine.Gameplay.ItemCategory.GundabadScholarScroll==nil then Turbine.Gameplay.ItemCategory.GundabadScholarScroll=274 end
	if Turbine.Gameplay.ItemCategory.MinasIthilFarmerScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilFarmerScroll=276 end
	if Turbine.Gameplay.ItemCategory.Carryall==nil then Turbine.Gameplay.ItemCategory.Carryall=279 end
	if Turbine.Gameplay.ItemCategory.MinasIthilCookScroll==nil then Turbine.Gameplay.ItemCategory.MinasIthilCookScroll=280 end
	if Turbine.Gameplay.ItemCategory.GundabadCookScroll==nil then Turbine.Gameplay.ItemCategory.GundabadCookScroll=281 end
	if Turbine.Gameplay.ItemCategory.DecorationEnvironmentAndLight==nil then Turbine.Gameplay.ItemCategory.DecorationEnvironmentAndLight=285 end
	if Turbine.Gameplay.ItemCategory.DecorationHitchPost==nil then Turbine.Gameplay.ItemCategory.DecorationHitchPost=286 end
	if Turbine.Gameplay.ItemCategory.Trophy2==nil then Turbine.Gameplay.ItemCategory.Trophy2=287 end
	if Turbine.Gameplay.ItemCategory.BrawlerBattlegauntlets==nil then Turbine.Gameplay.ItemCategory.BrawlerBattlegauntlets=290 end
	if Turbine.Gameplay.ItemCategory.HousingDecoration==nil then Turbine.Gameplay.ItemCategory.HousingDecoration=291 end
	if Turbine.Gameplay.ItemCategory.DraughtsAndSalves==nil then Turbine.Gameplay.ItemCategory.DraughtsAndSalves=292 end
	if Turbine.Gameplay.ItemCategory.Instrument2==nil then Turbine.Gameplay.ItemCategory.Instrument2=293 end
	if Turbine.Gameplay.ItemCategory.EttenmoorQuest==nil then Turbine.Gameplay.ItemCategory.EttenmoorQuest=295 end
	if Turbine.Gameplay.ItemCategory.UmbariMetalsmithScroll==nil then Turbine.Gameplay.ItemCategory.UmbariMetalsmithScroll=257 end
	if Turbine.Gameplay.ItemCategory.UmbariJewellerScroll==nil then Turbine.Gameplay.ItemCategory.UmbariJewellerScroll=262 end
	if Turbine.Gameplay.ItemCategory.UmbariTailorScroll==nil then Turbine.Gameplay.ItemCategory.UmbariTailorScroll=265 end
	if Turbine.Gameplay.ItemCategory.UmbariWoodworkerScroll==nil then Turbine.Gameplay.ItemCategory.UmbariWoodworkerScroll=267 end
	if Turbine.Gameplay.ItemCategory.UmbariWeaponsmithScroll==nil then Turbine.Gameplay.ItemCategory.UmbariWeaponsmithScroll=272 end
	if Turbine.Gameplay.ItemCategory.UmbariScholarScroll==nil then Turbine.Gameplay.ItemCategory.UmbariScholarScroll=275 end
	if Turbine.Gameplay.ItemCategory.UmbariCookScroll==nil then Turbine.Gameplay.ItemCategory.UmbariCookScroll=282 end

	if Turbine.Gameplay.ItemCategory.SulMadashCookScroll==nil then Turbine.Gameplay.ItemCategory.SulMadashCookScroll=300 end
end

ItemCategory={}

ItemCategory[1]={Turbine.Gameplay.ItemCategory.Undefined};
ItemCategory[2]={Turbine.Gameplay.ItemCategory.Bow};
ItemCategory[3]={Turbine.Gameplay.ItemCategory.Treasure};
ItemCategory[4]={Turbine.Gameplay.ItemCategory.Chest};
ItemCategory[5]={Turbine.Gameplay.ItemCategory.Minstrel};
ItemCategory[6]={Turbine.Gameplay.ItemCategory.Hands};
ItemCategory[7]={Turbine.Gameplay.ItemCategory.Shoulders};
ItemCategory[8]={Turbine.Gameplay.ItemCategory.Head};
ItemCategory[9]={Turbine.Gameplay.ItemCategory.Mounts};
ItemCategory[10]={Turbine.Gameplay.ItemCategory.Pennant};
ItemCategory[11]={Turbine.Gameplay.ItemCategory.Dagger};
ItemCategory[12]={Turbine.Gameplay.ItemCategory.Instrument};
ItemCategory[13]={Turbine.Gameplay.ItemCategory.Axe};
ItemCategory[14]={Turbine.Gameplay.ItemCategory.Captain};
ItemCategory[15]={Turbine.Gameplay.ItemCategory.Oil};
ItemCategory[16]={Turbine.Gameplay.ItemCategory.Legs};
ItemCategory[17]={Turbine.Gameplay.ItemCategory.Effect};
ItemCategory[18]={Turbine.Gameplay.ItemCategory.Hunter};
ItemCategory[19]={Turbine.Gameplay.ItemCategory.Armor};
ItemCategory[20]={Turbine.Gameplay.ItemCategory.Loremaster};
ItemCategory[21]={Turbine.Gameplay.ItemCategory.Trap};
ItemCategory[22]={Turbine.Gameplay.ItemCategory.Decoration};
ItemCategory[23]={Turbine.Gameplay.ItemCategory.Champion};
ItemCategory[24]={Turbine.Gameplay.ItemCategory.Feet};
ItemCategory[25]={Turbine.Gameplay.ItemCategory.Hammer};
ItemCategory[26]={Turbine.Gameplay.ItemCategory.Healing};
ItemCategory[27]={Turbine.Gameplay.ItemCategory.Guardian};
ItemCategory[28]={Turbine.Gameplay.ItemCategory.Quest};
ItemCategory[29]={Turbine.Gameplay.ItemCategory.Potion};
ItemCategory[30]={Turbine.Gameplay.ItemCategory.Crossbow};
ItemCategory[31]={Turbine.Gameplay.ItemCategory.Mace};
ItemCategory[32]={Turbine.Gameplay.ItemCategory.Key};
ItemCategory[33]={Turbine.Gameplay.ItemCategory.Tool};
ItemCategory[34]={Turbine.Gameplay.ItemCategory.Shield};
ItemCategory[35]={Turbine.Gameplay.ItemCategory.JourneymanProspectingScroll};
ItemCategory[36]={Turbine.Gameplay.ItemCategory.Staff};
ItemCategory[37]={Turbine.Gameplay.ItemCategory.Crafting};
ItemCategory[38]={Turbine.Gameplay.ItemCategory.Halberd};
ItemCategory[39]={Turbine.Gameplay.ItemCategory.Component};
ItemCategory[40]={Turbine.Gameplay.ItemCategory.Ingredient};
ItemCategory[41]={Turbine.Gameplay.ItemCategory.Clothing};
ItemCategory[42]={Turbine.Gameplay.ItemCategory.Club};
ItemCategory[43]={Turbine.Gameplay.ItemCategory.Dye};
ItemCategory[44]={Turbine.Gameplay.ItemCategory.JourneymanMetalworkScroll};
ItemCategory[45]={Turbine.Gameplay.ItemCategory.Weapon};
ItemCategory[46]={Turbine.Gameplay.ItemCategory.KinshipCharter};
ItemCategory[47]={Turbine.Gameplay.ItemCategory.JourneymanJewellerScroll};
ItemCategory[48]={Turbine.Gameplay.ItemCategory.Sword};
ItemCategory[49]={Turbine.Gameplay.ItemCategory.Back};
ItemCategory[50]={Turbine.Gameplay.ItemCategory.Spear};
ItemCategory[51]={Turbine.Gameplay.ItemCategory.Trophy};
ItemCategory[52]={Turbine.Gameplay.ItemCategory.Burglar};
ItemCategory[53]={Turbine.Gameplay.ItemCategory.Jewelry};
ItemCategory[54]={Turbine.Gameplay.ItemCategory.Device};
ItemCategory[55]={Turbine.Gameplay.ItemCategory.Relic};
ItemCategory[56]={Turbine.Gameplay.ItemCategory.Book};
ItemCategory[57]={Turbine.Gameplay.ItemCategory.NonInventory};
ItemCategory[58]={Turbine.Gameplay.ItemCategory.Thrown};
ItemCategory[59]={Turbine.Gameplay.ItemCategory.Food};
ItemCategory[60]={Turbine.Gameplay.ItemCategory.Resource};
ItemCategory[61]={Turbine.Gameplay.ItemCategory.Scroll};
ItemCategory[62]={Turbine.Gameplay.ItemCategory.ApprenticeTailorScroll};
ItemCategory[63]={Turbine.Gameplay.ItemCategory.ApprenticeJewellerScroll};
ItemCategory[64]={Turbine.Gameplay.ItemCategory.ApprenticeProspectingScroll};
ItemCategory[65]={Turbine.Gameplay.ItemCategory.ApprenticeFarmerScroll};
ItemCategory[66]={Turbine.Gameplay.ItemCategory.ApprenticeScholarScroll};
ItemCategory[67]={Turbine.Gameplay.ItemCategory.ApprenticeForestryScroll};
ItemCategory[68]={Turbine.Gameplay.ItemCategory.ApprenticeMetalworkScroll};
ItemCategory[69]={Turbine.Gameplay.ItemCategory.ApprenticeCookScroll};
ItemCategory[70]={Turbine.Gameplay.ItemCategory.ApprenticeWeaponsmithScroll};
ItemCategory[71]={Turbine.Gameplay.ItemCategory.ApprenticeWoodworkScroll};
ItemCategory[72]={Turbine.Gameplay.ItemCategory.MusicDecoration};
ItemCategory[73]={Turbine.Gameplay.ItemCategory.SurfacePaintDecoration};
ItemCategory[74]={Turbine.Gameplay.ItemCategory.TrophyDecoration};
ItemCategory[75]={Turbine.Gameplay.ItemCategory.FurnitureDecoration};
ItemCategory[76]={Turbine.Gameplay.ItemCategory.FloorDecoration};
ItemCategory[77]={Turbine.Gameplay.ItemCategory.YardDecoration};
ItemCategory[78]={Turbine.Gameplay.ItemCategory.WallDecoration};
ItemCategory[79]={Turbine.Gameplay.ItemCategory.Implement};
ItemCategory[80]={Turbine.Gameplay.ItemCategory.CosmeticHeld};
ItemCategory[81]={Turbine.Gameplay.ItemCategory.CosmeticBack};
ItemCategory[82]={Turbine.Gameplay.ItemCategory.SpecialDecoration};
ItemCategory[83]={Turbine.Gameplay.ItemCategory.CeilingDecoration};
ItemCategory[84]={Turbine.Gameplay.ItemCategory.FishingBait};
ItemCategory[85]={Turbine.Gameplay.ItemCategory.FishingOther};
ItemCategory[86]={Turbine.Gameplay.ItemCategory.Fish};
ItemCategory[87]={Turbine.Gameplay.ItemCategory.FishingPole};
ItemCategory[88]={Turbine.Gameplay.ItemCategory.Orb};
ItemCategory[89]={Turbine.Gameplay.ItemCategory.Warden};
ItemCategory[90]={Turbine.Gameplay.ItemCategory.Runekeeper};
ItemCategory[91]={Turbine.Gameplay.ItemCategory.LegendaryWeaponExperience};
ItemCategory[92]={Turbine.Gameplay.ItemCategory.LegendaryWeaponReset};
ItemCategory[93]={Turbine.Gameplay.ItemCategory.Deconstructable};
ItemCategory[94]={Turbine.Gameplay.ItemCategory.Javelin};
ItemCategory[95]={Turbine.Gameplay.ItemCategory.GuardianBelt};
ItemCategory[96]={Turbine.Gameplay.ItemCategory.JourneymanCookScroll};
ItemCategory[97]={Turbine.Gameplay.ItemCategory.ExpertCookScroll};
ItemCategory[98]={Turbine.Gameplay.ItemCategory.ArtisanCookScroll};
ItemCategory[99]={Turbine.Gameplay.ItemCategory.MasterCookScroll};
ItemCategory[100]={Turbine.Gameplay.ItemCategory.SupremeCookScroll};
ItemCategory[101]={Turbine.Gameplay.ItemCategory.MasterWoodworkScroll};
ItemCategory[102]={Turbine.Gameplay.ItemCategory.MasterWeaponsmithScroll};
ItemCategory[103]={Turbine.Gameplay.ItemCategory.ArtisanTailorScroll};
ItemCategory[104]={Turbine.Gameplay.ItemCategory.ExpertFarmerScroll};
ItemCategory[105]={Turbine.Gameplay.ItemCategory.JourneymanForestryScroll};
ItemCategory[106]={Turbine.Gameplay.ItemCategory.SupremeWoodworkScroll};
ItemCategory[107]={Turbine.Gameplay.ItemCategory.SupremeWeaponsmithScroll};
ItemCategory[108]={Turbine.Gameplay.ItemCategory.MasterTailorScroll};
ItemCategory[109]={Turbine.Gameplay.ItemCategory.ArtisanFarmerScroll};
ItemCategory[110]={Turbine.Gameplay.ItemCategory.ExpertForestryScroll};
ItemCategory[111]={Turbine.Gameplay.ItemCategory.SupremeTailorScroll};
ItemCategory[112]={Turbine.Gameplay.ItemCategory.MasterFarmerScroll};
ItemCategory[113]={Turbine.Gameplay.ItemCategory.ArtisanForestryScroll};
ItemCategory[114]={Turbine.Gameplay.ItemCategory.SupremeFarmerScroll};
ItemCategory[115]={Turbine.Gameplay.ItemCategory.MasterForestryScroll};
ItemCategory[116]={Turbine.Gameplay.ItemCategory.SupremeForestryScroll};
ItemCategory[117]={Turbine.Gameplay.ItemCategory.ExpertProspectingScroll};
ItemCategory[118]={Turbine.Gameplay.ItemCategory.ArtisanProspectingScroll};
ItemCategory[119]={Turbine.Gameplay.ItemCategory.MasterProspectingScroll};
ItemCategory[120]={Turbine.Gameplay.ItemCategory.SupremeProspectingScroll};
ItemCategory[121]={Turbine.Gameplay.ItemCategory.JourneymanScholarScroll};
ItemCategory[122]={Turbine.Gameplay.ItemCategory.ExpertScholarScroll};
ItemCategory[123]={Turbine.Gameplay.ItemCategory.ArtisanScholarScroll};
ItemCategory[124]={Turbine.Gameplay.ItemCategory.ExpertMetalworkScroll};
ItemCategory[125]={Turbine.Gameplay.ItemCategory.MasterScholarScroll};
ItemCategory[126]={Turbine.Gameplay.ItemCategory.ExpertJewellerScroll};
ItemCategory[127]={Turbine.Gameplay.ItemCategory.JourneymanWoodworkScroll};
ItemCategory[128]={Turbine.Gameplay.ItemCategory.JourneymanWeaponsmithScroll};
ItemCategory[129]={Turbine.Gameplay.ItemCategory.ArtisanMetalworkScroll};
ItemCategory[130]={Turbine.Gameplay.ItemCategory.SupremeScholarScroll};
ItemCategory[131]={Turbine.Gameplay.ItemCategory.ArtisanJewellerScroll};
ItemCategory[132]={Turbine.Gameplay.ItemCategory.ExpertWoodworkScroll};
ItemCategory[133]={Turbine.Gameplay.ItemCategory.ExpertWeaponsmithScroll};
ItemCategory[134]={Turbine.Gameplay.ItemCategory.MasterMetalworkScroll};
ItemCategory[135]={Turbine.Gameplay.ItemCategory.JourneymanTailorScroll};
ItemCategory[136]={Turbine.Gameplay.ItemCategory.MasterJewellerScroll};
ItemCategory[137]={Turbine.Gameplay.ItemCategory.ArtisanWoodworkScroll};
ItemCategory[138]={Turbine.Gameplay.ItemCategory.ArtisanWeaponsmithScroll};
ItemCategory[139]={Turbine.Gameplay.ItemCategory.SupremeMetalworkScroll};
ItemCategory[140]={Turbine.Gameplay.ItemCategory.ExpertTailorScroll};
ItemCategory[141]={Turbine.Gameplay.ItemCategory.SupremeJewellerScroll};
ItemCategory[142]={Turbine.Gameplay.ItemCategory.JourneymanFarmerScroll};
ItemCategory[143]={Turbine.Gameplay.ItemCategory.MinstrelBook};
ItemCategory[144]={Turbine.Gameplay.ItemCategory.Misc};
ItemCategory[145]={Turbine.Gameplay.ItemCategory.LegendaryWeaponReplaceLegacy};
ItemCategory[146]={Turbine.Gameplay.ItemCategory.LegendaryWeaponIncreaseMaxLevel};
ItemCategory[147]={Turbine.Gameplay.ItemCategory.LegendaryWeaponUpgradeLegacy};
ItemCategory[148]={Turbine.Gameplay.ItemCategory.CraftingTrophy};
-- loremaster food is labelled "Loremaster Class Item" but is indeed a distinct category, thus the possibility of multiple entries with the same name
ItemCategory[149]={Turbine.Gameplay.ItemCategory.LoremasterFood};
ItemCategory[150]={Turbine.Gameplay.ItemCategory.SpecialTrophy};
ItemCategory[151]={Turbine.Gameplay.ItemCategory.Reputation};
ItemCategory[152]={Turbine.Gameplay.ItemCategory.Skirmish};
ItemCategory[153]={Turbine.Gameplay.ItemCategory.Barter};
ItemCategory[154]={Turbine.Gameplay.ItemCategory.CosmeticChest};
ItemCategory[155]={Turbine.Gameplay.ItemCategory.CosmeticHead};
ItemCategory[156]={Turbine.Gameplay.ItemCategory.SkillManual}; -- items that teach skills
ItemCategory[157]={Turbine.Gameplay.ItemCategory.OptionalIngredient};
ItemCategory[158]={Turbine.Gameplay.ItemCategory.Perk};
ItemCategory[159]={Turbine.Gameplay.ItemCategory.Travel};
ItemCategory[160]={Turbine.Gameplay.ItemCategory.Special};
ItemCategory[161]={Turbine.Gameplay.ItemCategory.CosmeticFeet};
ItemCategory[162]={Turbine.Gameplay.ItemCategory.Horn};
ItemCategory[163]={Turbine.Gameplay.ItemCategory.Shieldspike};
ItemCategory[164]={Turbine.Gameplay.ItemCategory.TaskItem};
ItemCategory[165]={Turbine.Gameplay.ItemCategory.WestfoldTailorScroll}; --199
ItemCategory[166]={Turbine.Gameplay.ItemCategory.WestfoldJewellerScroll}; -- 196
ItemCategory[167]={Turbine.Gameplay.ItemCategory.WestfoldProspectingScroll}; -- 202
ItemCategory[168]={Turbine.Gameplay.ItemCategory.WestfoldFarmerScroll}; -- 200
ItemCategory[169]={Turbine.Gameplay.ItemCategory.WestfoldScholarScroll}; -- 204
ItemCategory[170]={Turbine.Gameplay.ItemCategory.WestfoldForestryScroll}; --201
ItemCategory[171]={Turbine.Gameplay.ItemCategory.WestfoldMetalworkScroll}; -- 195
ItemCategory[172]={Turbine.Gameplay.ItemCategory.WestfoldCookScroll}; -- 203
ItemCategory[173]={Turbine.Gameplay.ItemCategory.WestfoldWeaponsmithScroll}; --198
ItemCategory[174]={Turbine.Gameplay.ItemCategory.WestfoldWoodworkScroll}; -- 197
ItemCategory[175]={Turbine.Gameplay.ItemCategory.Tome};
ItemCategory[176]={Turbine.Gameplay.ItemCategory.CosmeticHands};
ItemCategory[177]={Turbine.Gameplay.ItemCategory.LegendaryWeaponIncreaseItemLevel};
ItemCategory[178]={Turbine.Gameplay.ItemCategory.Map}; -- 175
ItemCategory[179]={Turbine.Gameplay.ItemCategory.Social};
ItemCategory[180]={Turbine.Gameplay.ItemCategory.CosmeticLegs};
ItemCategory[181]={Turbine.Gameplay.ItemCategory.CosmeticClass};
ItemCategory[182]={Turbine.Gameplay.ItemCategory.LegendaryWeaponLegacyReveal};
ItemCategory[183]={Turbine.Gameplay.ItemCategory.LegendaryWeaponUnslotRelics};
ItemCategory[184]={Turbine.Gameplay.ItemCategory.Festival}; --205
ItemCategory[185]={Turbine.Gameplay.ItemCategory.LegendaryWeaponAddLegacy}; --206
-- 207 was highest in RoI

ItemCategory[186]={208}
ItemCategory[187]={209}
ItemCategory[188]={210}
ItemCategory[189]={211}
ItemCategory[190]={212}
ItemCategory[191]={213}
ItemCategory[192]={214}
ItemCategory[193]={215}
ItemCategory[194]={216}
ItemCategory[195]={217}
ItemCategory[196]={218}
-- found in U37 update
ItemCategory[197]={Turbine.Gameplay.ItemCategory.Beorning} -- 233
ItemCategory[198]={Turbine.Gameplay.ItemCategory.Brawler} -- 288
ItemCategory[199]={Turbine.Gameplay.ItemCategory.Mariner} -- 294
ItemCategory[200]={Turbine.Gameplay.ItemCategory.CosmeticShoulders} -- 181

ItemCategory[201]={220}
ItemCategory[202]={221}
ItemCategory[203]={222}
ItemCategory[204]={223}
ItemCategory[205]={224}
ItemCategory[206]={225}
ItemCategory[207]={226}
ItemCategory[208]={227}
ItemCategory[209]={228}
ItemCategory[210]={229}

ItemCategory[211]={235}
ItemCategory[212]={236}
ItemCategory[213]={237}
ItemCategory[214]={238}
ItemCategory[215]={239}
ItemCategory[216]={241}
ItemCategory[217]={242}
ItemCategory[218]={243}
ItemCategory[219]={244}
ItemCategory[220]={245}
ItemCategory[221]={246}
ItemCategory[222]={247}
ItemCategory[223]={248}
ItemCategory[224]={249}
ItemCategory[225]={250}

ItemCategory[226]={252}
ItemCategory[227]={253}
ItemCategory[228]={254}
ItemCategory[229]={255}
ItemCategory[230]={256}
ItemCategory[231]={258}
ItemCategory[232]={259}
ItemCategory[233]={260}
ItemCategory[234]={261}
ItemCategory[235]={264}
ItemCategory[236]={270}
ItemCategory[237]={271}
ItemCategory[238]={273}
ItemCategory[239]={274}
ItemCategory[240]={276}
ItemCategory[241]={279}
ItemCategory[242]={280}
ItemCategory[243]={281}
ItemCategory[244]={285}
ItemCategory[245]={286}

ItemCategory[246]={230}
ItemCategory[247]={231}
ItemCategory[248]={232}
ItemCategory[249]={251}
ItemCategory[250]={287}
ItemCategory[251]={290}
ItemCategory[252]={291}
ItemCategory[253]={292}
ItemCategory[254]={293}
ItemCategory[255]={295}
ItemCategory[256]={257}
ItemCategory[257]={262}
ItemCategory[258]={265}
ItemCategory[259]={267}
ItemCategory[260]={272}
ItemCategory[261]={275}
ItemCategory[262]={282}

ItemCategory[263]={296} -- Sul Madash Metalsmith recipe
ItemCategory[264]={297} -- Sul Madash Jweller recipe
ItemCategory[265]={298} -- Sul Madash Tailor recipe
ItemCategory[266]={299} -- Sul Madash Woodworker recipe
ItemCategory[267]={300} -- Sul Madash Cook recipe
ItemCategory[268]={301} -- Sul Madash Weaponsmith recipe
ItemCategory[269]={302} -- Sul Madash Scholar recipe

-- language specific resource strings
if ItemCategoryString==nil then
ItemCategoryString={};
end
-- need to get translations for all of the built in categories :(
-- Categories NOT found in enumeration but known to be in use:
ItemCategoryString[1]={}
ItemCategoryString[1][1]="Undefined"; --Turbine.Gameplay.ItemCategory.Undefined
ItemCategoryString[1][2]="Bows"; --Turbine.Gameplay.ItemCategory.Bow
ItemCategoryString[1][3]="Treasure"; --	self.ItemCategory[3]={Turbine.Gameplay.ItemCategory.Treasure,"Treasure"};
ItemCategoryString[1][4]="Armour: Upperbody"; --Turbine.Gameplay.ItemCategory.Chest;
ItemCategoryString[1][5]="Minstrel Class Items"; --Turbine.Gameplay.ItemCategory.Minstrel
ItemCategoryString[1][6]="Armour: Hands"; --Turbine.Gameplay.ItemCategory.Hands
ItemCategoryString[1][7]="Armour: Shoulders"; --Turbine.Gameplay.ItemCategory.Shoulders
ItemCategoryString[1][8]="Armour: Head"; --Turbine.Gameplay.ItemCategory.Head
ItemCategoryString[1][9]="Mounts"; --Turbine.Gameplay.ItemCategory.Mounts
ItemCategoryString[1][10]="Pennant"; --	self.ItemCategory[10]={Turbine.Gameplay.ItemCategory.Pennant,"Pennant"};
ItemCategoryString[1][11]="Daggers"; --Turbine.Gameplay.ItemCategory.Dagger
ItemCategoryString[1][12]="Instrument"; --Turbine.Gameplay.ItemCategory.Instrument
ItemCategoryString[1][13]="Axes"; --Turbine.Gameplay.ItemCategory.Axe
ItemCategoryString[1][14]="Captain Class Items"; --Turbine.Gameplay.ItemCategory.Captain
ItemCategoryString[1][15]="Weapon Oils"; --Turbine.Gameplay.ItemCategory.Oil
ItemCategoryString[1][16]="Armour: Legs"; --Turbine.Gameplay.ItemCategory.Legs
ItemCategoryString[1][17]="Pipeweed"; --Turbine.Gameplay.ItemCategory.Effect
ItemCategoryString[1][18]="Hunter Class Items"; --Turbine.Gameplay.ItemCategory.Hunter
ItemCategoryString[1][19]="Armour"; --	self.ItemCategory[19]={Turbine.Gameplay.ItemCategory.Armor,"Armour"};
ItemCategoryString[1][20]="Loremaster Class Items"; --Turbine.Gameplay.ItemCategory.Loremaster
ItemCategoryString[1][21]="Trap"; --	self.ItemCategory[21]={Turbine.Gameplay.ItemCategory.Trap,"Trap"};
ItemCategoryString[1][22]="Decoration"; --	self.ItemCategory[22]={Turbine.Gameplay.ItemCategory.Decoration,"Decoration"};
ItemCategoryString[1][23]="Champion Class Items"; --	self.ItemCategory[23]={Turbine.Gameplay.ItemCategory.Champion,"Champion"};
ItemCategoryString[1][24]="Armour: Feet"; --Turbine.Gameplay.ItemCategory.Feet
ItemCategoryString[1][25]="Hammers"; --Turbine.Gameplay.ItemCategory.Hammer
ItemCategoryString[1][26]="Healing"; --	self.ItemCategory[26]={Turbine.Gameplay.ItemCategory.Healing,"Healing"};
ItemCategoryString[1][27]="Guardian Class Items"; --	self.ItemCategory[27]={Turbine.Gameplay.ItemCategory.Guardian,"Guardian"};
ItemCategoryString[1][28]="Quest"; --Turbine.Gameplay.ItemCategory.Quest
ItemCategoryString[1][29]="Potion"; --Turbine.Gameplay.ItemCategory.Potion
ItemCategoryString[1][30]="Crossbow"; --	self.ItemCategory[30]={Turbine.Gameplay.ItemCategory.Crossbow,"Crossbow"};
ItemCategoryString[1][31]="Maces"; --Turbine.Gameplay.ItemCategory.Mace
ItemCategoryString[1][32]="Keys"; --Turbine.Gameplay.ItemCategory.Key
ItemCategoryString[1][33]="Tool"; --Turbine.Gameplay.ItemCategory.Tool
ItemCategoryString[1][34]="Armour: Shield"; --Turbine.Gameplay.ItemCategory.Shield
ItemCategoryString[1][35]="Recipe: Prospecting Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanProspectingScroll
ItemCategoryString[1][36]="Staves"; --Turbine.Gameplay.ItemCategory.Staff
ItemCategoryString[1][37]="Crafting"; --	self.ItemCategory[37]={Turbine.Gameplay.ItemCategory.Crafting,"Crafting"};
ItemCategoryString[1][38]="Halberd"; --	self.ItemCategory[38]={Turbine.Gameplay.ItemCategory.Halberd,"Halberd"};
ItemCategoryString[1][39]="Craft: Component"; --Turbine.Gameplay.ItemCategory.Component
ItemCategoryString[1][40]="Craft: Ingredient"; --Turbine.Gameplay.ItemCategory.Ingredient
ItemCategoryString[1][41]="Clothing"; --	self.ItemCategory[41]={Turbine.Gameplay.ItemCategory.Clothing,"Clothing"};
ItemCategoryString[1][42]="Clubs"; --Turbine.Gameplay.ItemCategory.Club
ItemCategoryString[1][43]="Dyes"; --Turbine.Gameplay.ItemCategory.Dye
ItemCategoryString[1][44]="Recipe: Metalwork Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanMetalworkScroll
ItemCategoryString[1][45]="Weapon"; --	self.ItemCategory[45]={Turbine.Gameplay.ItemCategory.Weapon,"Weapon"};
ItemCategoryString[1][46]="Kinship Charter"; --Turbine.Gameplay.ItemCategory.KinshipCharter
ItemCategoryString[1][47]="Recipe: Jeweller Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanJewellerScroll
ItemCategoryString[1][48]="Sword"; --Turbine.Gameplay.ItemCategory.Sword
ItemCategoryString[1][49]="Armour: Back"; --Turbine.Gameplay.ItemCategory.Back
ItemCategoryString[1][50]="Spears"; --Turbine.Gameplay.ItemCategory.Spear
ItemCategoryString[1][51]="Trophy"; --	self.ItemCategory[51]={Turbine.Gameplay.ItemCategory.Trophy,"Trophy"};
ItemCategoryString[1][52]="Burglar Class Items"; --	self.ItemCategory[52]={Turbine.Gameplay.ItemCategory.Burglar,"Burglar"};
ItemCategoryString[1][53]="Jewelry"; --	self.ItemCategory[53]={Turbine.Gameplay.ItemCategory.Jewelry,"Jewelry"};
ItemCategoryString[1][54]="Device"; --Turbine.Gameplay.ItemCategory.Device
ItemCategoryString[1][55]="Relic"; --	self.ItemCategory[55]={Turbine.Gameplay.ItemCategory.Relic,"Relic"};
ItemCategoryString[1][56]="Book"; --	self.ItemCategory[56]={Turbine.Gameplay.ItemCategory.Book,"Book"};
ItemCategoryString[1][57]="Non-Inventory"; --	self.ItemCategory[57]={Turbine.Gameplay.ItemCategory.NonInventory,"Non-Inventory"};
ItemCategoryString[1][58]="Thrown"; --	self.ItemCategory[58]={Turbine.Gameplay.ItemCategory.Thrown,"Thrown"};
ItemCategoryString[1][59]="Food"; --Turbine.Gameplay.ItemCategory.Food
ItemCategoryString[1][60]="Craft: Resource"; --Turbine.Gameplay.ItemCategory.Resource
ItemCategoryString[1][61]="Scroll"; --Turbine.Gameplay.ItemCategory.Scroll
ItemCategoryString[1][62]="Recipe: Tailor Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeTailorScroll
ItemCategoryString[1][63]="Recipe: Jeweller Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeJewellerScroll
ItemCategoryString[1][64]="Recipe: Prospecting Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeProspectingScroll
ItemCategoryString[1][65]="Recipe: Farmer Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeFarmerScroll
ItemCategoryString[1][66]="Recipe: Scholar Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeScholarScroll
ItemCategoryString[1][67]="Recipe: Forestry Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeForestryScroll
ItemCategoryString[1][68]="Recipe: Metalwork Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeMetalworkScroll
ItemCategoryString[1][69]="Recipe: Cook Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeCookScroll
ItemCategoryString[1][70]="Recipe: Weaponsmith Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeWeaponsmithScroll
ItemCategoryString[1][71]="Recipe: Woodwork Tier 1"; --Turbine.Gameplay.ItemCategory.ApprenticeWoodworkScroll
ItemCategoryString[1][72]="Home: Music"; --Turbine.Gameplay.ItemCategory.MusicDecoration
ItemCategoryString[1][73]="Home: Paints & Surfaces"; --Turbine.Gameplay.ItemCategory.SurfacePaintDecoration
ItemCategoryString[1][74]="Home: Trophy"; --Turbine.Gameplay.ItemCategory.TrophyDecoration
ItemCategoryString[1][75]="Home: Furniture"; --Turbine.Gameplay.ItemCategory.FurnitureDecoration
ItemCategoryString[1][76]="Home: Floor"; --Turbine.Gameplay.ItemCategory.FloorDecoration
ItemCategoryString[1][77]="Home: Yard"; --Turbine.Gameplay.ItemCategory.YardDecoration
ItemCategoryString[1][78]="Home: Wall"; --Turbine.Gameplay.ItemCategory.WallDecoration
ItemCategoryString[1][79]="Implement"; --	self.ItemCategory[79]={Turbine.Gameplay.ItemCategory.Implement,"Implement"};
ItemCategoryString[1][80]="Held"; --	self.ItemCategory[80]={Turbine.Gameplay.ItemCategory.CosmeticHeld,"Cosmetic-Held"};
ItemCategoryString[1][81]="Outfit: Back"; --Turbine.Gameplay.ItemCategory.CosmeticBack
ItemCategoryString[1][82]="Home: Special"; --Turbine.Gameplay.ItemCategory.SpecialDecoration
ItemCategoryString[1][83]="Home: Ceiling"; --Turbine.Gameplay.ItemCategory.CeilingDecoration
ItemCategoryString[1][84]="Fishing-Bait"; --	self.ItemCategory[84]={Turbine.Gameplay.ItemCategory.FishingBait,"Fishing-Bait"};
ItemCategoryString[1][85]="Fishing-Other"; --	self.ItemCategory[85]={Turbine.Gameplay.ItemCategory.FishingOther,"Fishing-Other"};
ItemCategoryString[1][86]="Fish"; --	self.ItemCategory[86]={Turbine.Gameplay.ItemCategory.Fish,"Fish"};
ItemCategoryString[1][87]="Poles"; --Turbine.Gameplay.ItemCategory.FishingPole
ItemCategoryString[1][88]="Rune-stones"; --Turbine.Gameplay.ItemCategory.Orb
ItemCategoryString[1][89]="Warden Class Items"; --	self.ItemCategory[89]={Turbine.Gameplay.ItemCategory.Warden,"Warden"};
ItemCategoryString[1][90]="Runekeeper Class Items"; --	self.ItemCategory[90]={Turbine.Gameplay.ItemCategory.Runekeeper,"Runekeeper"};
ItemCategoryString[1][91]="Legendary Item XP"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponExperience
ItemCategoryString[1][92]="Legendary Item Legendary Points Reset"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponReset
ItemCategoryString[1][93]="Deconstructables"; --Turbine.Gameplay.ItemCategory.Deconstructable
ItemCategoryString[1][94]="Javelin"; --	self.ItemCategory[94]={Turbine.Gameplay.ItemCategory.Javelin,"Javelin"};
ItemCategoryString[1][95]="Guardian Class Items"; --	self.ItemCategory[95]={Turbine.Gameplay.ItemCategory.GuardianBelt,"Guardian Belt"};
ItemCategoryString[1][96]="Recipe: Cook Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanCookScroll
ItemCategoryString[1][97]="Recipe: Cook Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertCookScroll
ItemCategoryString[1][98]="Recipe: Cook Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanCookScroll
ItemCategoryString[1][99]="Recipe: Cook Tier 5"; --Turbine.Gameplay.ItemCategory.MasterCookScroll
ItemCategoryString[1][100]="Recipe: Cook Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeCookScroll
ItemCategoryString[1][101]="Recipe: Woodwork Tier 5"; --Turbine.Gameplay.ItemCategory.MasterWoodworkScroll
ItemCategoryString[1][102]="Recipe: Weaponsmith Tier 5"; --Turbine.Gameplay.ItemCategory.MasterWeaponsmithScroll
ItemCategoryString[1][103]="Recipe: Tailor Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanTailorScroll
ItemCategoryString[1][104]="Recipe: Farmer Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertFarmerScroll
ItemCategoryString[1][105]="Recipe: Forestry Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanForestryScroll
ItemCategoryString[1][106]="Recipe: Woodwork Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeWoodworkScroll
ItemCategoryString[1][107]="Recipe: Weaponsmith Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeWeaponsmithScroll
ItemCategoryString[1][108]="Recipe: Tailor Tier 5"; --Turbine.Gameplay.ItemCategory.MasterTailorScroll
ItemCategoryString[1][109]="Recipe: Farmer Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanFarmerScroll
ItemCategoryString[1][110]="Recipe: Forestry Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertForestryScroll
ItemCategoryString[1][111]="Recipe: Tailor Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeTailorScroll
ItemCategoryString[1][112]="Recipe: Farmer Tier 5"; --Turbine.Gameplay.ItemCategory.MasterFarmerScroll
ItemCategoryString[1][113]="Recipe: Forestry Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanForestryScroll
ItemCategoryString[1][114]="Recipe: Farmer Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeFarmerScroll
ItemCategoryString[1][115]="Recipe: Forestry Tier 5"; --Turbine.Gameplay.ItemCategory.MasterForestryScroll
ItemCategoryString[1][116]="Recipe: Forestry Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeForestryScroll
ItemCategoryString[1][117]="Recipe: Prospecting Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertProspectingScroll
ItemCategoryString[1][118]="Recipe: Prospecting Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanProspectingScroll
ItemCategoryString[1][119]="Recipe: Prospecting Tier 5"; --Turbine.Gameplay.ItemCategory.MasterProspectingScroll
ItemCategoryString[1][120]="Recipe: Prospecting Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeProspectingScroll
ItemCategoryString[1][121]="Recipe: Scholar Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanScholarScroll
ItemCategoryString[1][122]="Recipe: Scholar Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertScholarScroll
ItemCategoryString[1][123]="Recipe: Scholar Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanScholarScroll
ItemCategoryString[1][124]="Recipe: Metalwork Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertMetalworkScroll
ItemCategoryString[1][125]="Recipe: Scholar Tier 5"; --Turbine.Gameplay.ItemCategory.MasterScholarScroll
ItemCategoryString[1][126]="Recipe: Jeweller Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertJewellerScroll
ItemCategoryString[1][127]="Recipe: Woodwork Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanWoodworkScroll
ItemCategoryString[1][128]="Recipe: Weaponsmith Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanWeaponsmithScroll
ItemCategoryString[1][129]="Recipe: Metalwork Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanMetalworkScroll
ItemCategoryString[1][130]="Recipe: Scholar Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeScholarScroll
ItemCategoryString[1][131]="Recipe: Jeweller Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanJewellerScroll
ItemCategoryString[1][132]="Recipe: Woodwork Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertWoodworkScroll
ItemCategoryString[1][133]="Recipe: Weaponsmith Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertWeaponsmithScroll
ItemCategoryString[1][134]="Recipe: Metalwork Tier 5"; --Turbine.Gameplay.ItemCategory.MasterMetalworkScroll
ItemCategoryString[1][135]="Recipe: Tailor Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanTailorScroll
ItemCategoryString[1][136]="Recipe: Jeweller Tier 5"; --Turbine.Gameplay.ItemCategory.MasterJewellerScroll
ItemCategoryString[1][137]="Recipe: Woodwork Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanWoodworkScroll
ItemCategoryString[1][138]="Recipe: Weaponsmith Tier 4"; --Turbine.Gameplay.ItemCategory.ArtisanWeaponsmithScroll
ItemCategoryString[1][139]="Recipe: Metalwork Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeMetalworkScroll
ItemCategoryString[1][140]="Recipe: Tailor Tier 3"; --Turbine.Gameplay.ItemCategory.ExpertTailorScroll
ItemCategoryString[1][141]="Recipe: Jeweller Tier 6"; --Turbine.Gameplay.ItemCategory.SupremeJewellerScroll
ItemCategoryString[1][142]="Recipe: Farmer Tier 2"; --Turbine.Gameplay.ItemCategory.JourneymanFarmerScroll
ItemCategoryString[1][143]="Minstrel Class Item"; --	self.ItemCategory[143]={Turbine.Gameplay.ItemCategory.MinstrelBook,"Minstrel Book"};
ItemCategoryString[1][144]="Misc."; --	self.ItemCategory[144]={Turbine.Gameplay.ItemCategory.Misc,"Misc"};
ItemCategoryString[1][145]="Legendary Item Replace Legacy"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponReplaceLegacy
ItemCategoryString[1][146]="Legendary Item Increase Max Level"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponIncreaseMaxLevel
ItemCategoryString[1][147]="Legendary Item Upgrade Legacy"; --	self.ItemCategory[147]={Turbine.Gameplay.ItemCategory.LegendaryWeaponUpgradeLegacy,"Legendary Weapon Upgrade Legacy"};
ItemCategoryString[1][148]="Crafting Trophy"; --	self.ItemCategory[148]={Turbine.Gameplay.ItemCategory.CraftingTrophy,"Crafting Trophy"};
ItemCategoryString[1][149]="Loremaster Class Items"; --	self.ItemCategory[149]={Turbine.Gameplay.ItemCategory.LoremasterFood,"Loremaster Food"};
ItemCategoryString[1][150]="Special Trophy"; --	self.ItemCategory[150]={Turbine.Gameplay.ItemCategory.SpecialTrophy,"Special Trophy"};;
ItemCategoryString[1][151]="Barter: Reputation";
ItemCategoryString[1][152]="Barter: Skirmish"; --177
ItemCategoryString[1][153]="Barter"; --178
ItemCategoryString[1][154]="Outfit: Upperbody"; --182
ItemCategoryString[1][155]="Outfit: Head"; --183
ItemCategoryString[1][156]="Skill Manuals"; --186
ItemCategoryString[1][157]="Craft: Optional Ingredient"; --188
ItemCategoryString[1][158]="Perks"; --189
ItemCategoryString[1][159]="Travel & Maps"; --191
ItemCategoryString[1][160]="Special"; --173
ItemCategoryString[1][161]="Outfit: Feet"; --180

ItemCategoryString[1][162]="Champion Horns";
ItemCategoryString[1][163]="Shield-spikes";

ItemCategoryString[1][164]="Task Items";
ItemCategoryString[1][165]="Recipe: Tailor Tier 7";
ItemCategoryString[1][166]="Recipe: Jeweller Tier 7";
ItemCategoryString[1][167]="Recipe: Prospecting Tier 7";
ItemCategoryString[1][168]="Recipe: Farmer Tier 7";
ItemCategoryString[1][169]="Recipe: Scholar Tier 7";
ItemCategoryString[1][170]="Recipe: Forestry Tier 7";
ItemCategoryString[1][171]="Recipe: Metalwork Tier 7";
ItemCategoryString[1][172]="Recipe: Cook Tier 7";
ItemCategoryString[1][173]="Recipe: Weaponsmith Tier 7";
ItemCategoryString[1][174]="Recipe: Woodworker Tier 7";
ItemCategoryString[1][175]="Tomes";
ItemCategoryString[1][176]="Outfit: Hands";
ItemCategoryString[1][177]="Legendary Weapon Increase Item Level";
ItemCategoryString[1][178]="Map";
ItemCategoryString[1][179]="Social"; -- text may be incorrect, take from category enumeration
ItemCategoryString[1][180]="Outfit: Legs";
ItemCategoryString[1][181]="Outfit: Class";
ItemCategoryString[1][182]="Legendary Weapon Legacy Reveal";
ItemCategoryString[1][183]="Relic Removal Scroll";
ItemCategoryString[1][184]="Festival";
ItemCategoryString[1][185]="Legendary Weapon Add Legacy";
ItemCategoryString[1][186]="Recipe: Metalwork Tier 8";--{208} --Turbine.Gameplay.ItemCategory.WestfoldMetalworkScroll};
ItemCategoryString[1][187]="Recipe: Jeweller Tier 8";--{209} --Turbine.Gameplay.ItemCategory.EstemnetJewellerScroll};
ItemCategoryString[1][188]="Recipe: Woodworker Tier 8";--{210} --Turbine.Gameplay.ItemCategory.WestfoldWoodworkScroll};
ItemCategoryString[1][189]="Recipe: Weaponsmith Tier 8";--{211} --Turbine.Gameplay.ItemCategory.EstemnetWeaponsmithScroll};
ItemCategoryString[1][190]="Recipe: Tailor Tier 8";--{212} --Turbine.Gameplay.ItemCategory.EstemnetTailorScroll};
ItemCategoryString[1][191]="Recipe: Farmer Tier 8";--{213} --Turbine.Gameplay.ItemCategory.EstemnetFarmerScroll};
ItemCategoryString[1][192]="Recipe: Forestry Tier 8";--{214} --Turbine.Gameplay.ItemCategory.EstemnetForestryScroll};
ItemCategoryString[1][193]="Recipe: Prospecting Tier 8";--{215} --Turbine.Gameplay.ItemCategory.EstemnetProspectingScroll};
ItemCategoryString[1][194]="Recipe: Cook Tier 8" -- (216) Turbine.Gameplay.ItemCategory.EstemnetCookScroll};
ItemCategoryString[1][195]="Recipe: Scholar Tier 8" -- (217) Turbine.Gameplay.ItemCategory.EstemnetScholarScroll};
ItemCategoryString[1][196]="Bridle" -- (218) Turbine.Gameplay.ItemCategory.Bridle
ItemCategoryString[1][197]="Beorning"--{Turbine.Gameplay.ItemCategory.Beorning} -- 233
ItemCategoryString[1][198]="Brawler"--{Turbine.Gameplay.ItemCategory.Brawler} -- 288
ItemCategoryString[1][199]="Mariner"--{Turbine.Gameplay.ItemCategory.Mariner} -- 294
ItemCategoryString[1][200]="Cosmetic Shoulders"--{Turbine.Gameplay.ItemCategory.CosmeticShoulders} -- 181
ItemCategoryString[1][201]="Recipe: Scholar Tier 9"--{220} --Turbine.Gameplay.ItemCategory.WestemnetScholarScroll}
ItemCategoryString[1][202]="Recipe: Metalwork Tier 9"--{221} --Turbine.Gameplay.ItemCategory.WestemnetMetalworkScroll}
ItemCategoryString[1][203]="Recipe: Jeweller Tier 9"--{222} --Turbine.Gameplay.ItemCategory.WestemnetJewellerScroll}
ItemCategoryString[1][204]="Recipe: Woodworker Tier 9"--{223} --Turbine.Gameplay.ItemCategory.WestemnetWoodworkScroll}
ItemCategoryString[1][205]="Recipe: Weaponsmith Tier 9"--{224} --Turbine.Gameplay.ItemCategory.WestemnetWeaponsmithScroll}
ItemCategoryString[1][206]="Recipe: Tailor Tier 9"--{225} --Turbine.Gameplay.ItemCategory.WestemnetTailorScroll}
ItemCategoryString[1][207]="Recipe: Farmer Tier 9"--{226} --Turbine.Gameplay.ItemCategory.WestemnetFarmerScroll}
ItemCategoryString[1][208]="Recipe: Forestry Tier 9"--{227} --Turbine.Gameplay.ItemCategory.WestemnetForestryScroll}
ItemCategoryString[1][209]="Recipe: Prospector Tier 9"--{228} --Turbine.Gameplay.ItemCategory.ProspectorCookScroll}
ItemCategoryString[1][210]="Recipe: Cook Tier 9"--{229} --Turbine.Gameplay.ItemCategory.WestemnetCookScroll}
ItemCategoryString[1][211]="Essence" --{235} -- Turbine.Gameplay.ItemCategory.ItemUpgrade
-- looks like Tier 10 (Anorien) shares a category with Tier 9
ItemCategoryString[1][212]="Recipe: Cook Tier 11" --{236} -- Turbine.Gameplay.ItemCategory.DoomfoldCookScroll
ItemCategoryString[1][213]="Recipe: Jeweller Tier 11" --{237} -- Turbine.Gameplay.ItemCategory.DoomfoldJewellerScroll
ItemCategoryString[1][214]="Recipe: Tailor Tier 11" --{238} -- Turbine.Gameplay.ItemCategory.DoomfoldTailorScroll
ItemCategoryString[1][215]="Recipe: Woodworker Tier 11" --{239} -- Turbine.Gameplay.ItemCategory.DoomfoldWoodworkerScroll
ItemCategoryString[1][216]="Recipe: Weaponsmith Tier 11" --{241} -- Turbine.Gameplay.ItemCategory.DoomfoldWeaponsmithScroll
ItemCategoryString[1][217]="Recipe: Scholar Tier 11" --{242} -- Turbine.Gameplay.ItemCategory.DoomfoldScholarScroll
ItemCategoryString[1][218]="Recipe: Metalwork Tier 11" --{243} -- Turbine.Gameplay.ItemCategory.DoomfoldMetalsmithScroll
ItemCategoryString[1][219]="Recipe: Metalwork Tier 12" --{244} -- Turbine.Gameplay.ItemCategory.IronfoldMetalsmithScroll
ItemCategoryString[1][220]="Recipe: Jeweller Tier 12" --{245} -- Turbine.Gameplay.ItemCategory.IronfoldJewellerScroll
ItemCategoryString[1][221]="Recipe: Tailor Tier 12" --{246} -- Turbine.Gameplay.ItemCategory.IronfoldTailorScroll
ItemCategoryString[1][222]="Recipe: Woodworker Tier 12" --{247} -- Turbine.Gameplay.ItemCategory.IronfoldWoodworkerScroll
ItemCategoryString[1][223]="Recipe: Cook Tier 12" --{248} -- Turbine.Gameplay.ItemCategory.IronfoldCookScroll
ItemCategoryString[1][224]="Recipe: Weaponsmith Tier 12" --{249} -- Turbine.Gameplay.ItemCategory.IronfoldWeaponsmithScroll
ItemCategoryString[1][225]="Recipe: Scholar Tier 12" --{250} -- Turbine.Gameplay.ItemCategory.IronfoldScholarScroll
ItemCategoryString[1][226]="Recipe: Farmer Tier 12" --k:252 exampleID:1879390611 -- 0x70053993 Turbine.Gameplay.ItemCategory.Ironfold Farmer
ItemCategoryString[1][227]="Weapon Aura" -- k:253 exampleID:1879402060 -- 0x7005664c Turbine.Gameplay.ItemCategory.Weapon Aura
ItemCategoryString[1][228]="Recipe: Metalsmith Tier 13" --k:254 exampleID:1879397135 -- 0x7005530f Turbine.Gameplay.ItemCategory.MinasIthil Metalsmith -- seems minas ithil and gundabad are intermixed
ItemCategoryString[1][229]="Recipe: Metalsmith Tier 14" --k:255 exampleID:1879440391 -- 0x7005fc07 Turbine.Gameplay.ItemCategory.Gundabad Metalsmith
ItemCategoryString[1][230]="Recipe: Jeweller Tier 13" --k:256 exampleID:1879394283 -- 0x700547eb Turbine.Gameplay.ItemCategory.MinasIthil Jeweller --13
ItemCategoryString[1][231]="Recipe: Tailor Tier 13" --k:258 exampleID:1879397012 -- 0x70055294 Turbine.Gameplay.ItemCategory.MinasIthil Tailor
ItemCategoryString[1][232]="Recipe: Jeweller Tier 14" --k:259 exampleID:1879440032 -- 0x7005faa0 Turbine.Gameplay.ItemCategory.Gundabad Jeweller --14
ItemCategoryString[1][233]="Recipe: Woodworker Tier 13" --k:260 exampleID:1879397117 -- 0x700552fd Turbine.Gameplay.ItemCategory.MinasIthil Woodworker
ItemCategoryString[1][234]="Recipe: Tailor Tier 14" --k:261 exampleID:1879440041 -- 0x7005faa9 Turbine.Gameplay.ItemCategory.Gundabad Tailor
ItemCategoryString[1][235]="Recipe: Woodworker Tier 14" --k:264 exampleID:1879445506 -- 0x70061002 Turbine.Gameplay.ItemCategory.Gundabad Woodworker
ItemCategoryString[1][236]="Recipe: Weaponsmith Tier 13" --k:270 exampleID:1879397122 -- 0x70055302 Turbine.Gameplay.ItemCategory.MinasIthil Weaponsmith
ItemCategoryString[1][237]="Recipe: Weaponsmith Tier 14" --k:271 exampleID:1879440423 -- 0x7005fc27 Turbine.Gameplay.ItemCategory.Gundabad Weaponsmith
ItemCategoryString[1][238]="Recipe: Scholar Tier 13" --k:273 exampleID:1879397139 -- 0x70055313 Turbine.Gameplay.ItemCategory.MinasIthil Scholar
ItemCategoryString[1][239]="Recipe: Scholar Tier 14" --k:274 exampleID:1879448683 -- 0x70061c6b Turbine.Gameplay.ItemCategory.Gundabad Scholar
ItemCategoryString[1][240]="Recipe: Farmer Tier 13" --k:276 exampleID:1879406708 -- 0x70057874 Turbine.Gameplay.ItemCategory.MinasIthil Farmer
ItemCategoryString[1][241]="Carry-all" --k:279 exampleID:1879402646 -- 0x70056896 Turbine.Gameplay.ItemCategory.Carryall
ItemCategoryString[1][242]="Recipe: Cook Tier 13" --k:280 exampleID:1879397132 -- 0x7005530c Turbine.Gameplay.ItemCategory.MinasIthil Cook
ItemCategoryString[1][243]="Recipe: Cook Tier 14" --k:281 exampleID:1879448678 -- 0x70061c66 Turbine.Gameplay.ItemCategory.Gundabad Cook
ItemCategoryString[1][244]="Light Decoration" --k:285 exampleID:1879404322 -- 0x70056f22 Turbine.Gameplay.ItemCategory.LightDecoration
ItemCategoryString[1][245]="Mount/Pet Decoration" --k:286 exampleID:1879404951 -- 0x70057197 Turbine.Gameplay.ItemCategory.MountDecoration
ItemCategoryString[1][246]="Box of Medium Armour"
ItemCategoryString[1][247]="Box of Heavy Armour"
ItemCategoryString[1][248]="Box of Light Armour"
ItemCategoryString[1][249]="Lootbox"
ItemCategoryString[1][250]="Trophy 2"
ItemCategoryString[1][251]="Battle-gauntlets"
ItemCategoryString[1][252]="Wall Decoration"
ItemCategoryString[1][253]="Potion 2"
ItemCategoryString[1][254]="Instruments"
ItemCategoryString[1][255]="Quest Trophy"
ItemCategoryString[1][256]="Recipe: Metalsmith Tier 15" --umbari metalmisth recipe
ItemCategoryString[1][257]="Recipe: Jeweller Tier 15" --umbari jeweller recipe
ItemCategoryString[1][258]="Recipe: Tailor Tier 15" --umbari tailor recipe
ItemCategoryString[1][259]="Recipe: Woodworker Tier 15" --umbari woodworker recipe
ItemCategoryString[1][260]="Recipe: Weaponsmith Tier 15" --umbari weaponsmith recipe
ItemCategoryString[1][261]="Recipe: Scholar Tier 15" --umbari scholar recipe
ItemCategoryString[1][262]="Recipe: Cook Tier 15" --umbari cook recipe
ItemCategoryString[1][263]="Recipe: Metalsmith Tier 16" -- sul madash metalsmith recipe
ItemCategoryString[1][264]="Recipe: Jeweller Tier 16" --jeweller recipe
ItemCategoryString[1][265]="Recipe: Tailor Tier 16" --tailor recipe
ItemCategoryString[1][266]="Recipe: Woodworker Tier 16" --woodworker recipe
ItemCategoryString[1][267]="Recipe: Cook Tier 16" --cook recipe
ItemCategoryString[1][268]="Recipe: Weaponsmith Tier 16" --weaponsmith recipe
ItemCategoryString[1][269]="Recipe: Scholar Tier 16" --scholar recipe

-- categories
ItemCategoryString[2]={}
ItemCategoryString[2][1]="Ind\195\169fini"; -- undefined
ItemCategoryString[2][2]="Arc"; -- Bows
ItemCategoryString[2][3]="Tr\195\169sor"; -- Treasure
ItemCategoryString[2][4]="Armure: Protection des Membres Sup\195\169rieurs"; -- verified
ItemCategoryString[2][5]="M\195\169nestrels"; -- verified
ItemCategoryString[2][6]="Armure: Protection des Mains"; -- verified
ItemCategoryString[2][7]="Armure: Spalli\195\168res"; -- verified
ItemCategoryString[2][8]="Armure: Casque"; -- verified
ItemCategoryString[2][9]="Montures"; -- verified
ItemCategoryString[2][10]="Fanion"; -- Pennant
ItemCategoryString[2][11]="Dagues"; -- verified
ItemCategoryString[2][12]="Instruments"; -- verified
ItemCategoryString[2][13]="Haches"; -- verified
ItemCategoryString[2][14]="Captaines"; -- verified
ItemCategoryString[2][15]="Huiles pour Arme"; -- verified
ItemCategoryString[2][16]="Armure: Jambi\195\168res"; -- verified
ItemCategoryString[2][17]="Herb \195\161 Pipe"; -- verified
ItemCategoryString[2][18]="Chasseurs"; -- verified
ItemCategoryString[2][19]="Armure"; -- armour
ItemCategoryString[2][20]="Ma\195\174tres du Savoir"; -- verified
ItemCategoryString[2][21]="Pi\195\168ge"; -- verified
ItemCategoryString[2][22]="Maison"; -- decoration (housing)
ItemCategoryString[2][23]="Champions"; -- verified
ItemCategoryString[2][24]="Armure: Soleret"; -- verified ?? not sure what this word IS. would have figured Pieds
ItemCategoryString[2][25]="Marteaux"; -- verified
--*** ??
ItemCategoryString[2][26]="Healing";
ItemCategoryString[2][27]="Guardiens"; -- verified
ItemCategoryString[2][28]="Objet de Qu\195\170te"; -- verified
ItemCategoryString[2][29]="Potions"; --verified
ItemCategoryString[2][30]="Arbal\195\168te";--crossbow
ItemCategoryString[2][31]="Masses"; -- verified
ItemCategoryString[2][32]="Cl\195\169s"; -- verified
ItemCategoryString[2][33]="Outil"; -- verified
ItemCategoryString[2][34]="Armure: Bouclier"; -- verified
ItemCategoryString[2][35]="M\195\169thode: Prospecting Niveau 2";
ItemCategoryString[2][36]="B\195\162tons"; -- verified
ItemCategoryString[2][37]="Artisanat"; -- Crafting
ItemCategoryString[2][38]="Hallebardes"; -- verified
ItemCategoryString[2][39]="Artisanat: Composant"; -- verified
ItemCategoryString[2][40]="Artisanat: \195\137l\195\169ment"; -- verified
ItemCategoryString[2][41]="V\195\170tements"; -- clothing
ItemCategoryString[2][42]="Gourdins"; -- verified
ItemCategoryString[2][43]="Teintures"; -- verified
ItemCategoryString[2][44]="M\195\169thode: Travail M\195\169taux Niveau 2"; -- verified
ItemCategoryString[2][45]="Arme"; -- weapon
ItemCategoryString[2][46]="Charte Confr\195\169rie"; -- verified
ItemCategoryString[2][47]="M\195\169thode: Bijouterie Niveau 2"; -- verified
ItemCategoryString[2][48]="\195\137p\195\169e"; -- verified
ItemCategoryString[2][49]="Armure: Dossiere"; -- verified
ItemCategoryString[2][50]="Lances"; -- verified
ItemCategoryString[2][51]="Troph\195\169e"; -- trophy
ItemCategoryString[2][52]="Cambrioleurs"; -- verified
ItemCategoryString[2][53]="Bijoux"; -- verified
ItemCategoryString[2][54]="Appareils"; -- verified
ItemCategoryString[2][55]="Relique"; -- relic
--*** LM or Mini ?
ItemCategoryString[2][56]="Livre"; -- book
--*** ??
ItemCategoryString[2][57]="Non-Inventory";
ItemCategoryString[2][58]="De jet"; -- verified
ItemCategoryString[2][59]="Nourriture"; -- verified
ItemCategoryString[2][60]="Artissanat: Ressource"; -- verified
ItemCategoryString[2][61]="Parchemin"; -- verified
ItemCategoryString[2][62]="M\195\169thode: Couture Niveau 1"; -- verified
ItemCategoryString[2][63]="M\195\169thode: Bijouterie Niveau 1"; -- verified
--*** prospector
ItemCategoryString[2][64]="M\195\169thode: Prospecting Niveau 1";
ItemCategoryString[2][65]="M\195\169thode: Agriculture Niveau 1"; -- verified
ItemCategoryString[2][66]="M\195\169thode: \195\137rudition Niveau 1"; -- verified
ItemCategoryString[2][67]="M\195\169thode: Foresterie Niveau 1";
ItemCategoryString[2][68]="M\195\169thode: Travail M\195\169taux Niveau 1"; -- verified
ItemCategoryString[2][69]="M\195\169thode: Cuisine Niveau 1"; -- verified
ItemCategoryString[2][70]="M\195\169thode: Fabr. Armes Niveau 1"; -- verified
ItemCategoryString[2][71]="M\195\169thode: Menuiserie Niveau 1"; -- verified
ItemCategoryString[2][72]="Maison: Musique"; -- verified
ItemCategoryString[2][73]="Maison: Peintures et Surfaces"; -- verified;
ItemCategoryString[2][74]="Maison: Troph\195\169e"; -- home: trophy
ItemCategoryString[2][75]="Maison: Meubles"; -- verified
ItemCategoryString[2][76]="Maison: Etage"; -- home: floor
ItemCategoryString[2][77]="Maison: Jardin"; -- verified
ItemCategoryString[2][78]="Maison: Mur"; -- verified
ItemCategoryString[2][79]="Outil"; --Implement -- how is this different than Tool?
--*** letters of endearment, etc
ItemCategoryString[2][80]="Cosmétique, tenu";
ItemCategoryString[2][81]="\195\137quipement: Dos"; -- verified
ItemCategoryString[2][82]="Maison: Sp\195\169ciale"; -- home: special
ItemCategoryString[2][83]="Maison: Plafond"; -- home: ceiling
ItemCategoryString[2][84]="De P\195\170che: App\195\162t"; -- bait was removed from game, not sure what this category is for
ItemCategoryString[2][85]="De P\195\170che: Autre"; -- not sure what this category represents either
ItemCategoryString[2][86]="Poisons"; -- verified
ItemCategoryString[2][87]="Cannes"; -- verified
ItemCategoryString[2][88]="Pierres Runique"; -- verified
ItemCategoryString[2][89]="Sentinelles"; -- verified
ItemCategoryString[2][90]="Guardiens des Runes"; -- verified
--*** need LI...
ItemCategoryString[2][91]="XP objet l\195\169gendaire";
ItemCategoryString[2][92]="Legendary Item Legendary Points Reset";
--*** sealed runes
ItemCategoryString[2][93]="Deconstructables";
ItemCategoryString[2][94]="Javelots"; -- javelin (warden javelins)
--*** guard
ItemCategoryString[2][95]="Guardiens"; -- verified
ItemCategoryString[2][96]="M\195\169thode: Cuisine Niveau 2"; -- verified
ItemCategoryString[2][97]="M\195\169thode: Cuisine Niveau 3"; -- verified
ItemCategoryString[2][98]="M\195\169thode: Cuisine Niveau 4"; -- verified
ItemCategoryString[2][99]="M\195\169thode: Cuisine Niveau 5"; -- verified
ItemCategoryString[2][100]="M\195\169thode: Cuisine Niveau 6"; -- verified
ItemCategoryString[2][101]="M\195\169thode: iserie Niveau 5"; -- verified
ItemCategoryString[2][102]="M\195\169thode: Fabr. Armes Niveau 5"; -- verified
ItemCategoryString[2][103]="M\195\169thode: Couture Niveau 4"; -- verified
ItemCategoryString[2][104]="M\195\169thode: Agriculture Niveau 3"; -- verified
--*** forester
ItemCategoryString[2][105]="M\195\169thode: Foresterie Niveau 2";
ItemCategoryString[2][106]="M\195\169thode: Menuiserie Niveau 6"; -- verified
ItemCategoryString[2][107]="M\195\169thode: Fabr. Armes Niveau 6"; -- verified
ItemCategoryString[2][108]="M\195\169thode: Couture Niveau 5"; -- verified
ItemCategoryString[2][109]="M\195\169thode: Agriculture Niveau 4"; -- verified
ItemCategoryString[2][110]="M\195\169thode: Foresterie Niveau 3"; -- verified
ItemCategoryString[2][111]="M\195\169thode: Couture Niveau 6"; -- verified
ItemCategoryString[2][112]="M\195\169thode: Agriculture Niveau 5"; -- verified
--*** forester
ItemCategoryString[2][113]="M\195\169thode: Foresterie Niveau 4";
ItemCategoryString[2][114]="M\195\169thode: Agriculture Niveau 6"; -- verified
--*** forester
ItemCategoryString[2][115]="M\195\169thode: Foresterie Niveau 5";
ItemCategoryString[2][116]="M\195\169thode: Foresterie Niveau 6";
--*** prospector
ItemCategoryString[2][117]="M\195\169thode: Prospecting Niveau 3";
ItemCategoryString[2][118]="M\195\169thode: Prospecting Niveau 4";
ItemCategoryString[2][119]="M\195\169thode: Prospecting Niveau 5";
ItemCategoryString[2][120]="M\195\169thode: Prospecting Niveau 6";
ItemCategoryString[2][121]="M\195\169thode: \195\137rudition Niveau 2"; -- verified
ItemCategoryString[2][122]="M\195\169thode: \195\137rudition Niveau 3"; -- verified
ItemCategoryString[2][123]="M\195\169thode: \195\137rudition Niveau 4"; -- verified
ItemCategoryString[2][124]="M\195\169thode: Travail M\195\169taux Niveau 3"; -- verified
ItemCategoryString[2][125]="M\195\169thode: \195\137rudition Niveau 5"; -- verified
ItemCategoryString[2][126]="M\195\169thode: Bijouterie Niveau 3"; -- verified
ItemCategoryString[2][127]="M\195\169thode: Menuiserie Niveau 2"; -- verified
ItemCategoryString[2][128]="M\195\169thode: Fabr. Armes Niveau 2"; -- verified
ItemCategoryString[2][129]="M\195\169thode: Travail M\195\169taux Niveau 4"; -- verified
ItemCategoryString[2][130]="M\195\169thode: \195\137rudition Niveau 6"; -- verified
ItemCategoryString[2][131]="M\195\169thode: Bijouterie Niveau 4"; -- verified
ItemCategoryString[2][132]="M\195\169thode: Menuiserie Niveau 3"; -- verified
ItemCategoryString[2][133]="M\195\169thode: Fabr. Armes Niveau 3"; -- verified
ItemCategoryString[2][134]="M\195\169thode: Travail M\195\169taux Niveau 5"; -- verified
ItemCategoryString[2][135]="M\195\169thode: Couture Niveau 2"; -- verified
ItemCategoryString[2][136]="M\195\169thode: Bijouterie Niveau 5"; -- verified
ItemCategoryString[2][137]="M\195\169thode: Menuiserie Niveau 4"; -- verified
ItemCategoryString[2][138]="M\195\169thode: Fabr. Armes Niveau 4"; -- verified
ItemCategoryString[2][139]="M\195\169thode: Travail M\195\169taux Niveau 6"; -- verified
ItemCategoryString[2][140]="M\195\169thode: Couture Niveau 3"; -- verified
ItemCategoryString[2][141]="M\195\169thode: Bijouterie Niveau 6"; -- verified
ItemCategoryString[2][142]="M\195\169thode: Agriculture Niveau 2"; -- verified
--*** minstrel
ItemCategoryString[2][143]="M\195\169nestrels"; -- verified
ItemCategoryString[2][144]="Divers"; -- verified
--*** LIs
ItemCategoryString[2][145]="Objet L\195\169gendaire - Remplace Legs";
--ItemCategoryString[2][146]="Legendary Item Increase Max Level"; -- pretty sure this category is deprecated
ItemCategoryString[2][147]="Objet L\195\169gendaire - augmenter mise \195\160 niveau";
ItemCategoryString[2][148]="Artisanat Troph\195\169e"; -- crafting trophy
--*** scholar
ItemCategoryString[2][149]="Ma\195\174tres du Savoir";
ItemCategoryString[2][150]="Troph\195\169e"; --verified
--*** barter items...
ItemCategoryString[2][151]="N\195\169gociation: R\195\169putation"; -- verified
ItemCategoryString[2][152]="N\195\169gociation: Escarmouche";
ItemCategoryString[2][153]="N\195\169gociation";
ItemCategoryString[2][154]="\195\137quipement: Membres Sup\195\169rieurs"; -- verified
ItemCategoryString[2][155]="\195\137quipement: T\195\170te"; -- outfit: head
ItemCategoryString[2][156]="Parchemins de Compétence"; -- verified
ItemCategoryString[2][157]="Artisanat: \195\137l\195\169ment Facultatif"; -- verified
ItemCategoryString[2][158]="Avantages"; -- Perks
ItemCategoryString[2][159]="Voyager & Cartes"; -- Travel & Maps
ItemCategoryString[2][160]="Sp\195\169cial"; -- Special
ItemCategoryString[2][161]="\195\137quipement: Pieds"; -- verified
ItemCategoryString[2][162]="Cors de Champion"; -- verified
ItemCategoryString[2][163]="Pointes de Bouclier"; -- verified

ItemCategoryString[2][164]="Objets de T\195\162che";
ItemCategoryString[2][165]="M\195\169thode: Couture Niveau 7";
ItemCategoryString[2][166]="M\195\169thode: Bijouterie Niveau 7";
ItemCategoryString[2][167]="M\195\169thode: Prospecting Niveau 7";
ItemCategoryString[2][168]="M\195\169thode: Agriculture Niveau 7";
ItemCategoryString[2][169]="M\195\169thode: \195\137rudition Niveau 7";
ItemCategoryString[2][170]="M\195\169thode: Foresterie Niveau 7";
ItemCategoryString[2][171]="M\195\169thode: Travail M\195\169taux Niveau 7";
ItemCategoryString[2][172]="M\195\169thode: Cuisine Niveau 7";
ItemCategoryString[2][173]="M\195\169thode: Fabr. Armes Niveau 7";
ItemCategoryString[2][174]="M\195\169thode: Menuiserie Niveau 7";
ItemCategoryString[2][175]="Tomes";
ItemCategoryString[2][176]="\195\137quipement: Mains";
ItemCategoryString[2][177]="Arme légendaire: Niveau d'augmentation";
ItemCategoryString[2][178]="Carte";
ItemCategoryString[2][179]="Social"; -- this is problematic in french as it is gender dependant
ItemCategoryString[2][180]="\195\137quipement: Jambes";
ItemCategoryString[2][181]="\195\137quipement: Classe";
ItemCategoryString[2][182]="Arme légendaire: Révélation de l'héritage";
ItemCategoryString[2][183]="Parchemin de suppression de reliques";
ItemCategoryString[2][184]="Festival";
ItemCategoryString[2][185]="Objet L\195\169gendaire - ajoute legs";

ItemCategoryString[2][186]="M\195\169thode: Travail M\195\169taux Niveau 8";--{208} --Turbine.Gameplay.ItemCategory.WestfoldMetalworkScroll};
ItemCategoryString[2][187]="M\195\169thode: Bijouterie Niveau 8";--{209} --Turbine.Gameplay.ItemCategory.EstemnetJewellerScroll};
ItemCategoryString[2][188]="M\195\169thode: Menuiserie Niveau 8";--{210} --Turbine.Gameplay.ItemCategory.WestfoldWoodworkScroll};
ItemCategoryString[2][189]="M\195\169thode: Fabr. Armes Niveau 8";--{211} --Turbine.Gameplay.ItemCategory.EstemnetWeaponsmithScroll};
ItemCategoryString[2][190]="M\195\169thode: Couture Niveau 8";--{212} --Turbine.Gameplay.ItemCategory.EstemnetTailorScroll};
ItemCategoryString[2][191]="M\195\169thode: Agriculture Niveau 8";--{213} --Turbine.Gameplay.ItemCategory.EstemnetFarmerScroll};
ItemCategoryString[2][192]="M\195\169thode: Foresterie Niveau 8";--{214} --Turbine.Gameplay.ItemCategory.EstemnetForestryScroll};
ItemCategoryString[2][193]="M\195\169thode: Prospecting Niveau 8";--{215} --Turbine.Gameplay.ItemCategory.EstemnetProspectingScroll};
ItemCategoryString[2][194]="M\195\169thode: Cuisine Niveau 8";
ItemCategoryString[2][195]="M\195\169thode: \195\137rudition Niveau 8";
ItemCategoryString[2][196]="Bride";

ItemCategoryString[2][197]="Beornide" --Beorninger
ItemCategoryString[2][198]="Bagarreur"
ItemCategoryString[2][199]="Marin"
ItemCategoryString[2][200]="Cosmétique, Épaules"
ItemCategoryString[2][201]="M\195\169thode: \195\137rudition Niveau 9"--{220} --Turbine.Gameplay.ItemCategory.WestemnetScholarScroll}
ItemCategoryString[2][202]="M\195\169thode: Travail M\195\169taux Niveau 9"--{221} --Turbine.Gameplay.ItemCategory.WestemnetMetalworkScroll}
ItemCategoryString[2][203]="M\195\169thode: Bijouterie Niveau 9"--{222} --Turbine.Gameplay.ItemCategory.WestemnetJewellerScroll}
ItemCategoryString[2][204]="M\195\169thode: Menuiserie Niveau 9"--{223} --Turbine.Gameplay.ItemCategory.WestemnetWoodworkScroll}
ItemCategoryString[2][205]="M\195\169thode: Fabr. Armes Niveau 9"--{224} --Turbine.Gameplay.ItemCategory.WestemnetWeaponsmithScroll}
ItemCategoryString[2][206]="M\195\169thode: Couture Niveau 9"--{225} --Turbine.Gameplay.ItemCategory.WestemnetTailorScroll}
ItemCategoryString[2][207]="M\195\169thode: Agriculture Niveau 9"--{226} --Turbine.Gameplay.ItemCategory.WestemnetFarmerScroll}
ItemCategoryString[2][208]="M\195\169thode: Foresterie Niveau 9"--{227} --Turbine.Gameplay.ItemCategory.WestemnetForestryScroll}
ItemCategoryString[2][209]="M\195\169thode: Prospecting Niveau 9"--{228} --Turbine.Gameplay.ItemCategory.WestemnetProspectorScroll}
ItemCategoryString[2][210]="M\195\169thode: Cuisine Niveau 9"--{229} --Turbine.Gameplay.ItemCategory.WestemnetCookScroll}
ItemCategoryString[2][211]="Essence" --{235} -- Turbine.Gameplay.ItemCategory.Essence
-- looks like Tier 10 (Anorien) shares a category with Tier 9
ItemCategoryString[2][212]="M\195\169thode: Cuisine Niveau 11" --{236} -- Turbine.Gameplay.ItemCategory.DoomfoldCookScroll
ItemCategoryString[2][213]="M\195\169thode: Bijouterie Niveau 11" --{237} -- Turbine.Gameplay.ItemCategory.DoomfoldJewellerScroll
ItemCategoryString[2][214]="M\195\169thode: Couture Niveau 11" --{238} -- Turbine.Gameplay.ItemCategory.DoomfoldTailorScroll
ItemCategoryString[2][215]="M\195\169thode: Menuiserie Niveau 11" --{239} -- Turbine.Gameplay.ItemCategory.DoomfoldWoodworkerScroll
ItemCategoryString[2][216]="M\195\169thode: Fabr. Armes Niveau 11" --{241} -- Turbine.Gameplay.ItemCategory.DoomfoldWeaponsmithScroll
ItemCategoryString[2][217]="M\195\169thode: \195\137rudition Niveau 11" --{242} -- Turbine.Gameplay.ItemCategory.DoomfoldScholarScroll
ItemCategoryString[2][218]="M\195\169thode: Travail M\195\169taux Niveau 11" --{243} -- Turbine.Gameplay.ItemCategory.DoomfoldMetalsmithScroll
ItemCategoryString[2][219]="M\195\169thode: Travail M\195\169taux Niveau 12" --{244} -- Turbine.Gameplay.ItemCategory.IronfoldMetalsmithScroll
ItemCategoryString[2][220]="M\195\169thode: Bijouterie Niveau 12" --{245} -- Turbine.Gameplay.ItemCategory.IronfoldJewellerScroll
ItemCategoryString[2][221]="M\195\169thode: Couture Niveau 12" --{246} -- Turbine.Gameplay.ItemCategory.IronfoldTailorScroll
ItemCategoryString[2][222]="M\195\169thode: Menuiserie Niveau 12" --{247} -- Turbine.Gameplay.ItemCategory.IronfoldWoodworkerScroll
ItemCategoryString[2][223]="M\195\169thode: Cuisine Niveau 12" --{248} -- Turbine.Gameplay.ItemCategory.IronfoldCookScroll
ItemCategoryString[2][224]="M\195\169thode: Fabr. Armes Niveau 12" --{249} -- Turbine.Gameplay.ItemCategory.IronfoldWeaponsmithScroll
ItemCategoryString[2][225]="M\195\169thode: \195\137rudition Niveau 12" --{250} -- Turbine.Gameplay.ItemCategory.IronfoldScholarScroll
ItemCategoryString[2][226]="M\195\169thode: Agriculture Niveau 12" --k:252 exampleID:1879390611 -- 0x70053993 Turbine.Gameplay.ItemCategory.Ironfold Farmer
ItemCategoryString[2][227]="Aura d'arme" -- k:253 exampleID:1879402060 -- 0x7005664c Turbine.Gameplay.ItemCategory.Weapon Aura
ItemCategoryString[2][228]="M\195\169thode: Metalsmith Niveau 13" --k:254 exampleID:1879397135 -- 0x7005530f Turbine.Gameplay.ItemCategory.MinasIthil Metalsmith -- seems minas ithil and gundabad are intermixed
ItemCategoryString[2][229]="M\195\169thode: Metalsmith Niveau 14" --k:255 exampleID:1879440391 -- 0x7005fc07 Turbine.Gameplay.ItemCategory.Gundabad Metalsmith
ItemCategoryString[2][230]="M\195\169thode: Bijouterie Niveau 13" --k:256 exampleID:1879394283 -- 0x700547eb Turbine.Gameplay.ItemCategory.MinasIthil Jeweller --13
ItemCategoryString[2][231]="M\195\169thode: Couture Niveau 13" --k:258 exampleID:1879397012 -- 0x70055294 Turbine.Gameplay.ItemCategory.MinasIthil Tailor
ItemCategoryString[2][232]="M\195\169thode: Bijouterie Niveau 14" --k:259 exampleID:1879440032 -- 0x7005faa0 Turbine.Gameplay.ItemCategory.Gundabad Jeweller --14
ItemCategoryString[2][233]="M\195\169thode: Menuiserie Niveau 13" --k:260 exampleID:1879397117 -- 0x700552fd Turbine.Gameplay.ItemCategory.MinasIthil Woodworker
ItemCategoryString[2][234]="M\195\169thode: Couture Niveau 14" --k:261 exampleID:1879440041 -- 0x7005faa9 Turbine.Gameplay.ItemCategory.Gundabad Tailor
ItemCategoryString[2][235]="M\195\169thode: Menuiserie Niveau 14" --k:264 exampleID:1879445506 -- 0x70061002 Turbine.Gameplay.ItemCategory.Gundabad Woodworker
ItemCategoryString[2][236]="M\195\169thode: Fabr. Armes Niveau 13" --k:270 exampleID:1879397122 -- 0x70055302 Turbine.Gameplay.ItemCategory.MinasIthil Weaponsmith
ItemCategoryString[2][237]="M\195\169thode: Fabr. Armes Niveau 14" --k:271 exampleID:1879440423 -- 0x7005fc27 Turbine.Gameplay.ItemCategory.Gundabad Weaponsmith
ItemCategoryString[2][238]="M\195\169thode: \195\137rudition Niveau 13" --k:273 exampleID:1879397139 -- 0x70055313 Turbine.Gameplay.ItemCategory.MinasIthil Scholar
ItemCategoryString[2][239]="M\195\169thode: \195\137rudition Niveau 14" --k:274 exampleID:1879448683 -- 0x70061c6b Turbine.Gameplay.ItemCategory.Gundabad Scholar
ItemCategoryString[2][240]="M\195\169thode: Agriculture Niveau 13" --k:276 exampleID:1879406708 -- 0x70057874 Turbine.Gameplay.ItemCategory.MinasIthil Farmer
ItemCategoryString[2][241]="Fourre-tout" --k:279 exampleID:1879402646 -- 0x70056896 Turbine.Gameplay.ItemCategory.Carryall
ItemCategoryString[2][242]="M\195\169thode: Cuisine Niveau 13" --k:280 exampleID:1879397132 -- 0x7005530c Turbine.Gameplay.ItemCategory.MinasIthil Cook
ItemCategoryString[2][243]="M\195\169thode: Cuisine Niveau 14" --k:281 exampleID:1879448678 -- 0x70061c66 Turbine.Gameplay.ItemCategory.Gundabad Cook
ItemCategoryString[2][244]="Décoration : lumière" --k:285 exampleID:1879404322 -- 0x70056f22 Turbine.Gameplay.ItemCategory.LightDecoration
ItemCategoryString[2][245]="Décoration : Monture/Animal" --k:286 exampleID:1879404951 -- 0x70057197 Turbine.Gameplay.ItemCategory.MountDecoration
ItemCategoryString[2][246]="Boîte d'armure moyenne"
ItemCategoryString[2][247]="Boîte d'armure lourde"
ItemCategoryString[2][248]="Boîte d'armure légère"
ItemCategoryString[2][249]="Boîte à butin"
ItemCategoryString[2][250]="Trophée 2"
ItemCategoryString[2][251]="Gantelets de bataille"
ItemCategoryString[2][252]="Décoration : mur"
ItemCategoryString[2][253]="Potion 2"
ItemCategoryString[2][254]="Instruments 2"
ItemCategoryString[2][255]="Trophée de quête"
ItemCategoryString[2][256]="M\195\169thode: Travail M\195\169taux Niveau 15" --umbari metalmisth recipe
ItemCategoryString[2][257]="M\195\169thode: Bijouterie Niveau 15" --umbari jeweller recipe
ItemCategoryString[2][258]="M\195\169thode: Couture Niveau 15" --umbari tailor recipe
ItemCategoryString[2][259]="M\195\169thode: Menuiserie Niveau 15" --umbari woodworker recipe
ItemCategoryString[2][260]="M\195\169thode:  Fabr. Armes Niveau 15" --umbari weaponsmith recipe
ItemCategoryString[2][261]="M\195\169thode: \195\137rudition Niveau 15" --umbari scholar recipe
ItemCategoryString[2][262]="M\195\169thode: Cuisine Niveau 15" --umbari cook recipe
ItemCategoryString[2][263]="M\195\169thode: Cuisine Niveau 16" -- sul madash metalsmith recipe
ItemCategoryString[2][264]="M\195\169thode: Bijouterie Niveau 16" --jeweller recipe
ItemCategoryString[2][265]="M\195\169thode: Couture Niveau 16" --tailor recipe
ItemCategoryString[2][266]="M\195\169thode: Menuiserie Niveau 16" --woodworker recipe
ItemCategoryString[2][267]="M\195\169thode: Cuisine Niveau 16" --cook recipe
ItemCategoryString[2][268]="M\195\169thode:  Fabr. Armes Niveau 16" --weaponsmith recipe
ItemCategoryString[2][269]="M\195\169thode: \195\137rudition Niveau 16" --scholar recipe

-- German Deutsch
-- need codes for special characters:  , , , , , , 
-- =#195##132#
-- =#195##164#
-- =#195##156#
-- =#195##188#
-- =#195##150#
-- =#195##182#
-- =#195##159#
ItemCategoryString[3]={}
ItemCategoryString[3][1]="Undefiniert"; --Undefined
ItemCategoryString[3][2]="Bogen"; --verified
ItemCategoryString[3][3]="Schatz"; --treasure
ItemCategoryString[3][4]="R\195\188stung: Oberk\195\182rper"; --verified (R stung: Oberk rper)
ItemCategoryString[3][5]="Barden-Gegenst\195\164nde"; --verified (Barden-Gegenst nde)
ItemCategoryString[3][6]="R\195\188stung: H\195\164nde"; --verfied (R stung: H nde)
ItemCategoryString[3][7]="R\195\188stung: Schultern"; --verified (R stung: Schultern)
ItemCategoryString[3][8]="R\195\188stung: Kopf"; --verfied (R stung: Kopf)
ItemCategoryString[3][9]="Reittiere"; --verified
ItemCategoryString[3][10]="Wimpel"; --pennant
ItemCategoryString[3][11]="Dolche"; --verified
ItemCategoryString[3][12]="Instrument"; --verified
ItemCategoryString[3][13]="\195\132xes"; --ItemCategory.Axe ( xte)
ItemCategoryString[3][14]="Hauptmann-Gegenst\195\164nde"; --verified (Hauptmann-Gegenst nde)
ItemCategoryString[3][15]="Waffen\195\182le"; --	Oil (Waffen le)
ItemCategoryString[3][16]="R\195\188stung: Beine"; --verfied (R stung: Beine)
ItemCategoryString[3][17]="Pfeifenkraut"; --verified
ItemCategoryString[3][18]="J\195\164ger-Gegenst\195\164nde"; --verified
ItemCategoryString[3][19]="R\195\188stung"; --Armor (R stung)
ItemCategoryString[3][20]="Kundigen-Gegenst\195\164nde"; --verified
ItemCategoryString[3][21]="Falle"; --verified
ItemCategoryString[3][22]="Dekoration"; --Decoration
ItemCategoryString[3][23]="Waffenmeister-Gegenst\195\164nde"; --verified
ItemCategoryString[3][24]="R\195\188stung: Fusse"; --verified (R stung: Fusse)
ItemCategoryString[3][25]="Hammers"; --Turbine.Gameplay.ItemCategory.Hammer
ItemCategoryString[3][26]="Heilend"; --Healing
ItemCategoryString[3][27]="W\195\164chter-Gegenst\195\164nde"; --verified
ItemCategoryString[3][28]="Aufgabengegenstand"; --Turbine.Gameplay.ItemCategory.Quest
ItemCategoryString[3][29]="Trank"; --verified
ItemCategoryString[3][30]="Crossbow"; --	self.ItemCategory[30]={Turbine.Gameplay.ItemCategory.Crossbow,"Crossbow"};
ItemCategoryString[3][31]="Streitkolben"; --Maces
ItemCategoryString[3][32]="Schl\195\188ssel"; --verified (Schl ssel)
ItemCategoryString[3][33]="Werkzeug"; --verified
ItemCategoryString[3][34]="R\195\188stung: Schild"; --verified (R stung: Schild)
ItemCategoryString[3][35]="Rezept: Prospektion Stufe 2"; --JourneymanProspectingScroll
ItemCategoryString[3][36]="St\195\164be"; --verified (St be)
ItemCategoryString[3][37]="Handwerk";
ItemCategoryString[3][38]="Hellebarde"; --Halberd
ItemCategoryString[3][39]="Handwerk: Komponente"; --verified
ItemCategoryString[3][40]="Handwerk: Zutat"; --verified
ItemCategoryString[3][41]="Clothing"; --	self.ItemCategory[41]={Turbine.Gameplay.ItemCategory.Clothing,"Clothing"};
ItemCategoryString[3][42]="Keulen"; --verified
ItemCategoryString[3][43]="Farbe"; --verfied
ItemCategoryString[3][44]="Rezept: Metallverarbeitung Stufe 2"; --verified
ItemCategoryString[3][45]="Waffen"; --verified
ItemCategoryString[3][46]="Sippen-Satzung"; --	self.ItemCategory[46]={Turbine.Gameplay.ItemCategory.KinshipCharter,"Kinship Charter"};
ItemCategoryString[3][47]="Rezept: Goldschmied Stufe 2"; --verified
ItemCategoryString[3][48]="Klingen"; --verified
ItemCategoryString[3][49]="R\195\188stung: R\195\188cken"; --verified (R stung: R cken)
ItemCategoryString[3][50]="Speere"; --verified
ItemCategoryString[3][51]="Troph\195\164e"; --Trophy (Troph e)
ItemCategoryString[3][52]="Schurken-Gegenst\195\164nde"; --verified
ItemCategoryString[3][53]="Geschmeide"; --verified
ItemCategoryString[3][54]="Hilfsmittel"; --verified
ItemCategoryString[3][55]="Reliquie"; --Relic
ItemCategoryString[3][56]="Buch"; --Book
ItemCategoryString[3][57]="Non-Inventory"; --	self.ItemCategory[57]={Turbine.Gameplay.ItemCategory.NonInventory,"Non-Inventory"};
ItemCategoryString[3][58]="Wurfwaffe"; --verified
ItemCategoryString[3][59]="Nahrungsmittel"; --verified
ItemCategoryString[3][60]="Handwerk: Rohstoff"; --verfied
ItemCategoryString[3][61]="Schriftrolle"; --verified
ItemCategoryString[3][62]="Rezept: Schneider Stufe 1"; --verified
ItemCategoryString[3][63]="Rezept: Goldschmied Stufe 1"; --verified
ItemCategoryString[3][64]="Rezept: Prospektion Stufe 1"; --ApprenticeProspectingScroll
ItemCategoryString[3][65]="Rezept: Landwirtschaft Stufe 1"; --verified
ItemCategoryString[3][66]="Rezept: Gelehrter Stufe 1"; --verified
ItemCategoryString[3][67]="Rezept: Forstwirtschaft Stufe 1"; --ApprenticeForestryScroll
ItemCategoryString[3][68]="Rezept: Metallverarbeitung Stufe 1"; --verified
ItemCategoryString[3][69]="Rezept: Kochen Stufe 1"; --verified
ItemCategoryString[3][70]="Rezept: Waffenschmiedekunst Stufe 1"; --verified
ItemCategoryString[3][71]="Rezept: Drechseln Stufe 1"; --verified
ItemCategoryString[3][72]="Heim: Musik"; --verified
ItemCategoryString[3][73]="Heim: Farben & Bel\195\164ge"; --verified (Heim: Farben & Bel ge)
ItemCategoryString[3][74]="Heim: Troph\195\164e"; --Turbine.Gameplay.ItemCategory.TrophyDecoration (Troph e)
ItemCategoryString[3][75]="Heim: M\195\182bel"; --verified (Heim: M bel)
ItemCategoryString[3][76]="Heim: Boden"; --Turbine.Gameplay.ItemCategory.FloorDecoration
ItemCategoryString[3][77]="Heim: Hof"; --verified
ItemCategoryString[3][78]="Heim: Wand"; --verified
ItemCategoryString[3][79]="Implement"; --	self.ItemCategory[79]={Turbine.Gameplay.ItemCategory.Implement,"Implement"};
ItemCategoryString[3][80]="Ausstattung: Gehalten"; --	self.ItemCategory[80]={Turbine.Gameplay.ItemCategory.CosmeticHeld,"Cosmetic-Held"};
ItemCategoryString[3][81]="Ausstattung: Rücken"; --verified
ItemCategoryString[3][82]="Heim: Spezielle"; --Turbine.Gameplay.ItemCategory.SpecialDecoration
ItemCategoryString[3][83]="Heim: Decke"; --Turbine.Gameplay.ItemCategory.CeilingDecoration
ItemCategoryString[3][84]="Fishing-Bait"; --	self.ItemCategory[84]={Turbine.Gameplay.ItemCategory.FishingBait,"Fishing-Bait"};
ItemCategoryString[3][85]="Fishing-Other"; --	self.ItemCategory[85]={Turbine.Gameplay.ItemCategory.FishingOther,"Fishing-Other"};
ItemCategoryString[3][86]="Fisch"; --verified
ItemCategoryString[3][87]="Ruten"; --verified
ItemCategoryString[3][88]="Runensteine"; --verified
ItemCategoryString[3][89]="H\195\188ter-Gegenst\195\164nde"; --verified
ItemCategoryString[3][90]="Runenbewahrer-Gegenst\195\164nde"; --verified
ItemCategoryString[3][91]="Legend\195\164rer Gegenstand: Erfahrung"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponExperience
ItemCategoryString[3][92]="Legend\195\164rer Gegenstand: Legend\195\164r-Punkte Zur\195\188ckgesetzt"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponReset
ItemCategoryString[3][93]="Deconstructables"; --Turbine.Gameplay.ItemCategory.Deconstructable
ItemCategoryString[3][94]="Wurfspie\195\159"; --self.ItemCategory[94]={Turbine.Gameplay.ItemCategory.Javelin,"Javelin"};(Wurfspie )
ItemCategoryString[3][95]="W\195\164chter-Gegenst\195\164nde"; --verified
ItemCategoryString[3][96]="Rezept: Kochen Stufe 2"; --verified
ItemCategoryString[3][97]="Rezept: Kochen Stufe 3"; --verified
ItemCategoryString[3][98]="Rezept: Kochen Stufe 4"; --verified
ItemCategoryString[3][99]="Rezept: Kochen Stufe 5"; --verified
ItemCategoryString[3][100]="Rezept: Kochen Stufe 6"; --verified
ItemCategoryString[3][101]="Rezept: Drechseln Stufe 5"; --verified
ItemCategoryString[3][102]="Rezept: Waffenschmiedekunst Stufe 5"; --verified
ItemCategoryString[3][103]="Rezept: Schneider Stufe 4"; --verified
ItemCategoryString[3][104]="Rezept: Landwirtschaft Stufe 3"; --verified
ItemCategoryString[3][105]="Rezept: Forstwirtschaft Stufe 2"; --JourneymanForestryScroll
ItemCategoryString[3][106]="Rezept: Drechseln Stufe 6"; --verified
ItemCategoryString[3][107]="Rezept: Waffenschmiedekunst Stufe 6"; --verified
ItemCategoryString[3][108]="Rezept: Schneider Stufe 5"; --verified
ItemCategoryString[3][109]="Rezept: Landwirtschaft Stufe 4"; --verified
ItemCategoryString[3][110]="Rezept: Forstwirtschaft Stufe 3"; --ExpertForestryScroll
ItemCategoryString[3][111]="Rezept: Schneider Stufe 6"; --verified
ItemCategoryString[3][112]="Rezept: Landwirtschaft Stufe 5"; --verified
ItemCategoryString[3][113]="Rezept: Forstwirtschaft Stufe 4"; --ArtisanForestryScroll
ItemCategoryString[3][114]="Rezept: Landwirtschaft Stufe 6"; --verified
ItemCategoryString[3][115]="Rezept: Forstwirtschaft Stufe 5"; --MasterForestryScroll
ItemCategoryString[3][116]="Rezept: Forstwirtschaft Stufe 6"; --SupremeForestryScroll
ItemCategoryString[3][117]="Rezept: Prospektion Stufe 3"; --ExpertProspectingScroll
ItemCategoryString[3][118]="Rezept: Prospektion Stufe 4"; --ArtisanProspectingScroll
ItemCategoryString[3][119]="Rezept: Prospektion Stufe 5"; --MasterProspectingScroll
ItemCategoryString[3][120]="Rezept: Prospektion Stufe 6"; --SupremeProspectingScroll
ItemCategoryString[3][121]="Rezept: Gelehrter Stufe 2"; --Turbine.Gameplay.ItemCategory.JourneymanScholarScroll
ItemCategoryString[3][122]="Rezept: Gelehrter Stufe 3"; --Turbine.Gameplay.ItemCategory.ExpertScholarScroll
ItemCategoryString[3][123]="Rezept: Gelehrter Stufe 4"; --Turbine.Gameplay.ItemCategory.ArtisanScholarScroll
ItemCategoryString[3][124]="Rezept: Metallverarbeitung Stufe 3"; --verified
ItemCategoryString[3][125]="Rezept: Gelehrter Stufe 5"; --Turbine.Gameplay.ItemCategory.MasterScholarScroll
ItemCategoryString[3][126]="Rezept: Goldschmied Stufe 3"; --verified
ItemCategoryString[3][127]="Rezept: Drechseln Stufe 2"; --verified
ItemCategoryString[3][128]="Rezept: Waffenschmiedekunst Stufe 2"; --verified
ItemCategoryString[3][129]="Rezept: Metallverarbeitung Stufe 4"; --verified
ItemCategoryString[3][130]="Rezept: Gelehrter Stufe 6"; --Turbine.Gameplay.ItemCategory.SupremeScholarScroll
ItemCategoryString[3][131]="Rezept: Goldschmied Stufe 4"; --verified
ItemCategoryString[3][132]="Rezept: Drechseln Stufe 3"; --verified
ItemCategoryString[3][133]="Rezept: Waffenschmiedekunst Stufe 3"; --verified
ItemCategoryString[3][134]="Rezept: Metallverarbeitung Stufe 5"; --verified
ItemCategoryString[3][135]="Rezept: Schneider Stufe 2"; --verified
ItemCategoryString[3][136]="Rezept: Goldschmied Stufe 5"; --verified
ItemCategoryString[3][137]="Rezept: Drechseln Stufe 4"; --verified
ItemCategoryString[3][138]="Rezept: Waffenschmiedekunst Stufe 4"; --verified
ItemCategoryString[3][139]="Rezept: Metallverarbeitung Stufe 6"; --verified
ItemCategoryString[3][140]="Rezept: Schneider Stufe 3"; --verified
ItemCategoryString[3][141]="Rezept: Goldschmied Stufe 6"; --verified
ItemCategoryString[3][142]="Rezept: Landwirtschaft Stufe 2"; --verified
ItemCategoryString[3][143]="Barden-Gegenst\195\164nde"; --verified (Barden-Gegenst nde)
ItemCategoryString[3][144]="Verschiedenes"; --verified
ItemCategoryString[3][145]="Legend\195\164rer Gegenstand: Attribute Ersetzen"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponReplaceLegacy
ItemCategoryString[3][146]="Legendary Item Increase Max Level"; --Turbine.Gameplay.ItemCategory.LegendaryWeaponIncreaseMaxLevel
ItemCategoryString[3][147]="Legend\195\164rer Gegenstand: Attribute Verbessern"; --	self.ItemCategory[147]={Turbine.Gameplay.ItemCategory.LegendaryWeaponUpgradeLegacy,"Legendary Weapon Upgrade Legacy"};
ItemCategoryString[3][148]="Handwerk Troph\195\164e"; --CraftingTrophy (Handwerk Troph e)
ItemCategoryString[3][149]="Kundigen-Gegenst\195\164nde"; --verified
ItemCategoryString[3][150]="Troph\195\164e"; --verified (Spezielle Troph e)
ItemCategoryString[3][151]="Tauschen: Ruf"; --verified
ItemCategoryString[3][152]="Tauschen: Scharm\195\188tzel"; --177 (Tauschen: Scharm tzel)
ItemCategoryString[3][153]="Tauschen"; --178
ItemCategoryString[3][154]="Ausstattung: Oberk\195\182rper"; --verified (Ausstattung: Oberk rper)
ItemCategoryString[3][155]="Ausstattung: Kopf"; --183
ItemCategoryString[3][156]="Fertigkeitshandbuch"; --186
ItemCategoryString[3][157]="Handwerk: Optionale Zutat"; -- verified
ItemCategoryString[3][158]="Vorteile"; --verified
ItemCategoryString[3][159]="Reisen/Karten"; --verfieid
ItemCategoryString[3][160]="Spezial"; --173 verified
ItemCategoryString[3][161]="Ausstattung: Fusse"; --180
ItemCategoryString[3][162]="Waffenmeister H\195\150rner"; --verified
ItemCategoryString[3][163]="Schilddornen";

ItemCategoryString[3][164]="Auftragsgegenst\195\164nde";
ItemCategoryString[3][165]="Rezept: Schneider Tier 7";
ItemCategoryString[3][166]="Rezept: Goldschmied Tier 7";
ItemCategoryString[3][167]="Rezept: Prospektion Tier 7";
ItemCategoryString[3][168]="Rezept: Landwirtschaft Tier 7";
ItemCategoryString[3][169]="Rezept: Gelehrter Tier 7";
ItemCategoryString[3][170]="Rezept: Forstwirtschaft Tier 7";
ItemCategoryString[3][171]="Rezept: Metallverarbeitung Tier 7";
ItemCategoryString[3][172]="Rezept: Kochen Tier 7";
ItemCategoryString[3][173]="Rezept: Waffenschmiedekunst Tier 7";
ItemCategoryString[3][174]="Rezept: Drechseln Tier 7";
ItemCategoryString[3][175]="Schriften"; --175 verified
ItemCategoryString[3][176]="Ausrüstung: Hände";
ItemCategoryString[3][177]="Legendäre Gegenstand: Stufe erhöhen";
ItemCategoryString[3][178]="Karte";
ItemCategoryString[3][179]="Soziale"
ItemCategoryString[3][180]="Ausrüstung: Beine";
ItemCategoryString[3][181]="Ausrüstung: Klasse";
ItemCategoryString[3][182]="Legendäre Gegenstand: Vermächtnis enthüllt";
ItemCategoryString[3][183]="Schriftrolle der Reliktentfernung";
ItemCategoryString[3][184]="Festival";
ItemCategoryString[3][185]="Legendärer Gegenstand: Attribut Hinz\195\188fugen"; -- Weapon Add Legacy";
ItemCategoryString[3][186]="Rezept: Metallverarbeitung Tier 8";
ItemCategoryString[3][187]="Rezept: Goldschmied Tier 8";
ItemCategoryString[3][188]="Rezept: Drechseln Tier 8";
ItemCategoryString[3][189]="Rezept: Waffenschmiedekunst Tier 8";
ItemCategoryString[3][190]="Rezept: Schneider Tier 8";
ItemCategoryString[3][191]="Rezept: Landwirtschaft Tier 8";
ItemCategoryString[3][192]="Rezept: Forstwirtschaft Tier 8";
ItemCategoryString[3][193]="Rezept: Prospektion Tier 8";
ItemCategoryString[3][194]="Rezept: Kochen Tier 8";
ItemCategoryString[3][195]="Rezept: Gelehrter Tier 8";
ItemCategoryString[3][196]="Zaumzeug";
ItemCategoryString[3][197]="Beorninger"
ItemCategoryString[3][198]="Schläger"
ItemCategoryString[3][199]="Seefahrer"
ItemCategoryString[3][200]="Ausstattung: Schulter"
ItemCategoryString[3][201]="Rezept: Gelehrter Tier 9"--{220} --Turbine.Gameplay.ItemCategory.WestemnetScholarScroll}
ItemCategoryString[3][202]="Rezept: Metallverarbeitung Tier 9"--{221} --Turbine.Gameplay.ItemCategory.WestemnetMetalworkScroll}
ItemCategoryString[3][203]="Rezept: Goldschmied Tier 9"--{222} --Turbine.Gameplay.ItemCategory.WestemnetJewellerScroll}
ItemCategoryString[3][204]="Rezept: Drechseln Tier 9"--{223} --Turbine.Gameplay.ItemCategory.WestemnetWoodworkScroll}
ItemCategoryString[3][205]="Rezept: Waffenschmiedekunst Tier 9"--{224} --Turbine.Gameplay.ItemCategory.WestemnetWeaponsmithScroll}
ItemCategoryString[3][206]="Rezept: Schneider Tier 9"--{225} --Turbine.Gameplay.ItemCategory.WestemnetTailorScroll}
ItemCategoryString[3][207]="Rezept: Landwirtschaft Tier 9"--{226} --Turbine.Gameplay.ItemCategory.WestemnetFarmerScroll}
ItemCategoryString[3][208]="Rezept: Forstwirtschaft Tier 9"--{227} --Turbine.Gameplay.ItemCategory.WestemnetForestryScroll}
ItemCategoryString[3][209]="Rezept: Prospektion Tier 9"--{228} --Turbine.Gameplay.ItemCategory.WestemnetProspectorScroll}
ItemCategoryString[3][210]="Rezept: Kochen Tier 9"--{229} --Turbine.Gameplay.ItemCategory.WestemnetCookScroll}
ItemCategoryString[3][211]="Essenz" --{235} -- Turbine.Gameplay.ItemCategory.ItemUpgrade
-- looks like Tier 10 (Anorien) shares a category with Tier 9
ItemCategoryString[3][212]="Rezept: Kochen Tier 11" --{236} -- Turbine.Gameplay.ItemCategory.DoomfoldCookScroll
ItemCategoryString[3][213]="Rezept: Goldschmied Tier 11" --{237} -- Turbine.Gameplay.ItemCategory.DoomfoldJewellerScroll
ItemCategoryString[3][214]="Rezept: Schneider Tier 11" --{238} -- Turbine.Gameplay.ItemCategory.DoomfoldTailorScroll
ItemCategoryString[3][215]="Rezept: Drechseln Tier 11" --{239} -- Turbine.Gameplay.ItemCategory.DoomfoldWoodworkerScroll
ItemCategoryString[3][216]="Rezept: Waffenschmiedekunst Tier 11" --{241} -- Turbine.Gameplay.ItemCategory.DoomfoldWeaponsmithScroll
ItemCategoryString[3][217]="Rezept: Gelehrter Tier 11" --{242} -- Turbine.Gameplay.ItemCategory.DoomfoldScholarScroll
ItemCategoryString[3][218]="Rezept: Metallverarbeitung Tier 11" --{243} -- Turbine.Gameplay.ItemCategory.DoomfoldMetalsmithScroll
ItemCategoryString[3][219]="Rezept: Metallverarbeitung Tier 12" --{244} -- Turbine.Gameplay.ItemCategory.IronfoldMetalsmithScroll
ItemCategoryString[3][220]="Rezept: Goldschmied Tier 12" --{245} -- Turbine.Gameplay.ItemCategory.IronfoldJewellerScroll
ItemCategoryString[3][221]="Rezept: Schneider Tier 12" --{246} -- Turbine.Gameplay.ItemCategory.IronfoldTailorScroll
ItemCategoryString[3][222]="Rezept: Drechseln Tier 12" --{247} -- Turbine.Gameplay.ItemCategory.IronfoldWoodworkerScroll
ItemCategoryString[3][223]="Rezept: Kochen Tier 12" --{248} -- Turbine.Gameplay.ItemCategory.IronfoldCookScroll
ItemCategoryString[3][224]="Rezept: Waffenschmiedekunst Tier 12" --{249} -- Turbine.Gameplay.ItemCategory.IronfoldWeaponsmithScroll
ItemCategoryString[3][225]="Rezept: Gelehrter Tier 12" --{250} -- Turbine.Gameplay.ItemCategory.IronfoldScholarScroll
ItemCategoryString[3][226]="Rezept: Landwirtschaft Tier 12" --k:252 exampleID:1879390611 -- 0x70053993 Turbine.Gameplay.ItemCategory.Ironfold Farmer
ItemCategoryString[3][227]="Waffenaura" -- k:253 exampleID:1879402060 -- 0x7005664c Turbine.Gameplay.ItemCategory.Weapon Aura
ItemCategoryString[3][228]="Rezept: Metallverarbeitung Tier 13" --k:254 exampleID:1879397135 -- 0x7005530f Turbine.Gameplay.ItemCategory.MinasIthil Metalsmith -- seems minas ithil and gundabad are intermixed
ItemCategoryString[3][229]="Rezept: Metallverarbeitung Tier 14" --k:255 exampleID:1879440391 -- 0x7005fc07 Turbine.Gameplay.ItemCategory.Gundabad Metalsmith
ItemCategoryString[3][230]="Rezept: Goldschmied Tier 13" --k:256 exampleID:1879394283 -- 0x700547eb Turbine.Gameplay.ItemCategory.MinasIthil Jeweller --13
ItemCategoryString[3][231]="Rezept: Schneider Tier 13" --k:258 exampleID:1879397012 -- 0x70055294 Turbine.Gameplay.ItemCategory.MinasIthil Tailor
ItemCategoryString[3][232]="Rezept: Goldschmied Tier 14" --k:259 exampleID:1879440032 -- 0x7005faa0 Turbine.Gameplay.ItemCategory.Gundabad Jeweller --14
ItemCategoryString[3][233]="Rezept: Drechseln Tier 13" --k:260 exampleID:1879397117 -- 0x700552fd Turbine.Gameplay.ItemCategory.MinasIthil Woodworker
ItemCategoryString[3][234]="Rezept: Schneider Tier 14" --k:261 exampleID:1879440041 -- 0x7005faa9 Turbine.Gameplay.ItemCategory.Gundabad Tailor
ItemCategoryString[3][235]="Rezept: Drechseln Tier 14" --k:264 exampleID:1879445506 -- 0x70061002 Turbine.Gameplay.ItemCategory.Gundabad Woodworker
ItemCategoryString[3][236]="Rezept: Waffenschmiedekunst Tier 13" --k:270 exampleID:1879397122 -- 0x70055302 Turbine.Gameplay.ItemCategory.MinasIthil Weaponsmith
ItemCategoryString[3][237]="Rezept: Waffenschmiedekunst Tier 14" --k:271 exampleID:1879440423 -- 0x7005fc27 Turbine.Gameplay.ItemCategory.Gundabad Weaponsmith
ItemCategoryString[3][238]="Rezept: Gelehrter Tier 13" --k:273 exampleID:1879397139 -- 0x70055313 Turbine.Gameplay.ItemCategory.MinasIthil Scholar
ItemCategoryString[3][239]="Rezept: Gelehrter Tier 14" --k:274 exampleID:1879448683 -- 0x70061c6b Turbine.Gameplay.ItemCategory.Gundabad Scholar
ItemCategoryString[3][240]="Rezept: Landwirtschaft Tier 13" --k:276 exampleID:1879406708 -- 0x70057874 Turbine.Gameplay.ItemCategory.MinasIthil Farmer
ItemCategoryString[3][241]="Tragetasche" --k:279 exampleID:1879402646 -- 0x70056896 Turbine.Gameplay.ItemCategory.Carryall
ItemCategoryString[3][242]="Rezept: Kochen Tier 13" --k:280 exampleID:1879397132 -- 0x7005530c Turbine.Gameplay.ItemCategory.MinasIthil Cook
ItemCategoryString[3][243]="Rezept: Kochen Tier 14" --k:281 exampleID:1879448678 -- 0x70061c66 Turbine.Gameplay.ItemCategory.Gundabad Cook
ItemCategoryString[3][244]="Dekoration: Licht" --k:285 exampleID:1879404322 -- 0x70056f22 Turbine.Gameplay.ItemCategory.LightDecoration
ItemCategoryString[3][245]="Dekoration: Reittier/Haustier" --k:286 exampleID:1879404951 -- 0x70057197 Turbine.Gameplay.ItemCategory.MountDecoration
ItemCategoryString[3][246]="Kiste mit mittlerer Rüstung"
ItemCategoryString[3][247]="Kiste mit schwerer Rüstung"
ItemCategoryString[3][248]="Kiste mit leichter Rüstung"
ItemCategoryString[3][249]="Lootbox"
ItemCategoryString[3][250]="Trophäe 2"
ItemCategoryString[3][251]="Faustkampf Handschuhe"
ItemCategoryString[3][252]="Dekoration: Wand"
ItemCategoryString[3][253]="Trank 2"
ItemCategoryString[3][254]="Instruments"
ItemCategoryString[3][255]="Quest-Trophäe"
ItemCategoryString[3][256]="Rezept: Metallverarbeitung Tier 15" --umbari metalmisth recipe
ItemCategoryString[3][257]="Rezept: Goldschmied Tier 15" --umbari jeweller recipe
ItemCategoryString[3][258]="Rezept: Schneider Tier 15" --umbari tailor recipe
ItemCategoryString[3][259]="Rezept: Drechseln Tier 15" --umbari woodworker recipe
ItemCategoryString[3][260]="Rezept: Waffenschmiedekunst Tier 15" --umbari weaponsmith recipe
ItemCategoryString[3][261]="Rezept: Gelehrter Tier 15" --umbari scholar recipe
ItemCategoryString[3][262]="Rezept: Kochen Tier 15" --umbari cook recipe
ItemCategoryString[3][263]="Rezept: Metallverarbeitung Tier 16" -- sul madash metalsmith recipe
ItemCategoryString[3][264]="Rezept: Goldschmied Tier 16" --jeweller recipe
ItemCategoryString[3][265]="Rezept: Schneider Tier 16" --tailor recipe
ItemCategoryString[3][266]="Rezept: Drechseln Tier 16" --woodworker recipe
ItemCategoryString[3][267]="Rezept: Kochen Tier 16" --cook recipe
ItemCategoryString[3][268]="Rezept: Waffenschmiedekunst Tier 16" --weaponsmith recipe
ItemCategoryString[3][269]="Rezept: Gelehrter Tier 16" --scholar recipe


-- fill any missing translations with the english equivalent
local tmpIndex;
for tmpIndex=1,#ItemCategoryString[1] do
	if ItemCategoryString[2][tmpIndex]==nil then ItemCategoryString[2][tmpIndex]=ItemCategoryString[1][tmpIndex] end
	if ItemCategoryString[3][tmpIndex]==nil then ItemCategoryString[3][tmpIndex]=ItemCategoryString[1][tmpIndex] end
end

-- assign the strings to ItemCategory
for tmpIndex, data in ipairs(ItemCategory) do
	data[2]={}
	for language=1,3 do
		data[2][language]=ItemCategoryString[language][tmpIndex];
	end
end

--sort the categories by the current language
success,result=pcall(table.sort,ItemCategory,function(arg1,arg2) return(arg1[2][language]<arg2[2][language]) end)
categorySortOrder={} -- this is populated in inventoryWindow.lua after we load and sort the category list alphabetically
undefinedCategoryIndex=1 -- this is also overridden in inventoryWindow.lua
-- build ItemCategoryIndex - we need an index that is ordered by categoryID, not alphabetically
rebuildItemCategoryIndex=function()
	ItemCategoryIndex={}
	for k,v in pairs(ItemCategory) do
		table.insert(ItemCategoryIndex,{v[1],k})
	end
	table.sort(ItemCategoryIndex,function(a,b) if a[1]<b[1] then return true else if a[1]==b[1] and a[2]<b[2] then return true end end end)
end
rebuildItemCategoryIndex()