-- lgo: LotRO Gear Optimizer Plugin
--
-- Direction:
--   - Candidate gear comes from Shared Storage chest named "lgo"
--   - Ignore inventory bags for selection (bags/rows can be rearranged)
--   - Shared Storage panel must be opened at least once before use
--
-- Commands:
--   /lgo
--     -> help
--   /lgo export
--     -> export item names from (equipped, excl. craft tool & bridle) + (shared storage chest 'lgo')
--   /lgo export chest <name>
--     -> export item names from (equipped, excl. craft tool & bridle) + (shared storage chest <name>)
--
-- Data is written via Turbine.PluginData.Save(Turbine.DataScope.Account, key, table)
--   - lgo_<character>_gearNames_<timestamp>.plugindata
--   - top-level shape:
--       {
--         version = "lgo-gearlist-1",
--         character = "...",
--         class = "...",
--         baseStats = { GetBaseMight=..., GetBaseAgility=..., ... },
--         names = { [1.000000]="...", [2.000000]="...", ... },
--       }
--
-- Notes on gear stats:
--   The LotRO plugin API does not expose numeric stat values.
--   GetDescription() returns an unserializable engine token.
--   GetLevel/GetRequiredLevel/GetItemClass are absent on this API version.
--   Per-item data exported: item names only (for wiki lookup).
--   The Rust optimizer looks up stats externally by item name.

import "Turbine";
import "Turbine.Gameplay";
import "Turbine.UI";
import "Turbine.UI.Lotro";

-- Keep objects alive (prevents GC from breaking slash commands)
Thalya = Thalya or {};
Thalya.lgo = Thalya.lgo or {};

-- ── Helpers ──────────────────────────────────────────────────────────────────

local function Print(msg)
  Turbine.Shell.WriteLine("[lgo] " .. tostring(msg));
end

local function Try(label, fn)
  local ok, a, b, c, d = pcall(fn);
  if not ok then
    Print(label .. " => ERROR: " .. tostring(a));
    return false, nil, nil, nil, nil;
  end
  return true, a, b, c, d;
end

local function Trim(s)
  return (tostring(s or ""):gsub("^%s+", ""):gsub("%s+$", ""));
end

local function Lower(s)
  return string.lower(tostring(s or ""));
end

local function z2(n)
  n = tonumber(n) or 0;
  if n < 10 then return "0" .. tostring(n) end
  return tostring(n);
end

local function z4(n)
  n = tonumber(n) or 0;
  if n < 10 then return "000" .. n end
  if n < 100 then return "00" .. n end
  if n < 1000 then return "0" .. n end
  return tostring(n);
end

local function NowKeySuffix()
  local d = Turbine.Engine.GetDate();
  return string.format(
    "%s%s%s_%s%s%s",
    z4(d.Year), z2(d.Month), z2(d.Day),
    z2(d.Hour), z2(d.Minute), z2(d.Second)
  );
end

local function CharacterName()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  if player == nil or player.GetName == nil then return "Unknown" end
  local ok, n = pcall(function() return player:GetName(); end);
  if ok and n and n ~= "" then return n end
  return "Unknown";
end

local function ShouldSkipEquippedSlot(slot)
  -- Probe-confirmed exclusions:
  -- Users commonly keep these equipped, but they are not real optimizer candidates:
  --   19 = craft tool
  --   21 = bridle
  return slot == 19 or slot == 21
end

-- ── Character class ────────────────────────────────────────────────────────────

local function CharacterClass()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  if player == nil or type(player.GetClass) ~= "function" then return "Unknown" end
  local ok, cls = pcall(function() return player:GetClass(); end);
  if not ok or cls == nil then return "Unknown" end

  -- Build map from Turbine.Gameplay.Class enum constants (wrapped in pcall
  -- so that missing entries on older clients fail gracefully).
  local map = {};
  pcall(function()
    map[Turbine.Gameplay.Class.Burglar]     = "Burglar";
    map[Turbine.Gameplay.Class.Captain]     = "Captain";
    map[Turbine.Gameplay.Class.Champion]    = "Champion";
    map[Turbine.Gameplay.Class.Guardian]    = "Guardian";
    map[Turbine.Gameplay.Class.Hunter]      = "Hunter";
    map[Turbine.Gameplay.Class.LoreMaster]  = "Lore-master";
    map[Turbine.Gameplay.Class.Minstrel]    = "Minstrel";
    map[Turbine.Gameplay.Class.RuneKeeper]  = "Rune-keeper";
    map[Turbine.Gameplay.Class.Warden]      = "Warden";
    map[Turbine.Gameplay.Class.Beorning]    = "Beorning";
    map[Turbine.Gameplay.Class.Brawler]     = "Brawler";
    map[Turbine.Gameplay.Class.Mariner]     = "Mariner";
  end);

  return map[cls] or ("Class_" .. tostring(cls));
end

-- Saves account-scoped plugin data as:
--   lgo_<character>_<kind>_<timestamp>.plugindata
local function SaveAccount(kind, data)
  local key = "lgo_" .. CharacterName() .. "_" .. kind .. "_" .. NowKeySuffix();
  Turbine.PluginData.Save(Turbine.DataScope.Account, key, data);
  Print("Saved: " .. key .. " (Account scope)");
  return key;
end

-- ── Item extraction ──────────────────────────────────────────────────────────

local function GetItemInfoSafe(item)
  if item == nil or item.GetItemInfo == nil then return nil end
  local ok, info = pcall(function() return item:GetItemInfo(); end);
  if ok then return info end
  return nil
end

local function GetNameFromItemInfo(info)
  if info == nil or info.GetName == nil then return nil end
  local ok, n = pcall(function() return info:GetName(); end);
  if ok and n and n ~= "" then return n end
  return nil
end

local function TryCall0(obj, methodName)
  if obj == nil then return nil, false, false end
  local m = obj[methodName];
  if type(m) ~= "function" then return nil, false, false end
  local ok, val = pcall(function() return m(obj); end);
  return val, true, ok
end

-- ── Base primary stats ─────────────────────────────────────────────────────────

local function GetBaseStats()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  if player == nil or type(player.GetAttributes) ~= "function" then return nil end
  local ok, attrs = pcall(function() return player:GetAttributes(); end);
  if not ok or attrs == nil then return nil end

  local stats = {};
  local methods = {
    "GetBaseMight",
    "GetBaseAgility",
    "GetBaseVitality",
    "GetBaseWill",
    "GetBaseFate",
  };
  for _, m in ipairs(methods) do
    local v, existed, sok = TryCall0(attrs, m);
    if existed and sok and v ~= nil then
      stats[m] = tonumber(v);
    end
  end
  return stats;
end



local function ExtractItemRecord(item, indexOrSlot)
  local rec = {};

  -- Item-level name (may be a custom rename; falls back to info name)
  local name = nil;
  if item ~= nil and type(item.GetName) == "function" then
    local ok, n = pcall(function() return item:GetName(); end);
    if ok and n and n ~= "" then name = n end
  end

  local info = GetItemInfoSafe(item);
  local infoName = GetNameFromItemInfo(info);
  if name == nil then name = infoName end

  rec.name = name;
  rec.infoName = infoName;

  return rec;
end

-- ── Shared Storage access / enumeration ──────────────────────────────────────

local function GetSharedStorage()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  if player == nil then return nil, "LocalPlayer nil" end

  if type(player.GetSharedStorage) ~= "function" then
    return nil, "player.GetSharedStorage is not a function on this client";
  end

  local ok, ss = Try("player:GetSharedStorage()", function()
    return player:GetSharedStorage();
  end);

  if not ok or ss == nil then
    return nil, "player:GetSharedStorage() returned nil";
  end

  if type(ss.IsAvailable) == "function" then
    local ok2, available = Try("sharedStorage:IsAvailable()", function()
      return ss:IsAvailable();
    end);
    if ok2 and available == false then
      return nil, "Shared Storage not available (open the panel in-game first)";
    end
  end

  return ss, nil;
end

local function EnumerateSharedStorageItems(filterFn)
  local ss, err = GetSharedStorage();
  if err ~= nil then return nil, err end

  if type(ss.GetCount) ~= "function" then
    return nil, "sharedStorage.GetCount is not a function";
  end
  if type(ss.GetItem) ~= "function" then
    return nil, "sharedStorage.GetItem is not a function";
  end

  local okCount, count = Try("sharedStorage:GetCount()", function()
    return ss:GetCount();
  end);
  if not okCount or type(count) ~= "number" then
    return nil, "sharedStorage:GetCount() did not return a number";
  end

  local out = { items = {} };

  for i = 1, count do
    local okItem, item = Try("sharedStorage:GetItem(" .. tostring(i) .. ")", function()
      return ss:GetItem(i);
    end);

    if okItem and item ~= nil then
      local rec = ExtractItemRecord(item, i);

      local chestIndex = nil;
      if type(item.GetChest) == "function" then
        local okChest, c = pcall(function() return item:GetChest(); end);
        if okChest then chestIndex = c end
      end

      local chestName = nil;
      if chestIndex ~= nil and type(ss.GetChestName) == "function" then
        local okName, n = pcall(function() return ss:GetChestName(chestIndex); end);
        if okName and n ~= nil and n ~= "" then chestName = n end
      end

      if rec.name ~= nil then
        if filterFn == nil or filterFn(chestName, chestIndex, rec) then
          table.insert(out.items, rec);
        end
      end
    end
  end

  return out, nil;
end

-- ── Equipment enumeration ───────────────────────────────────────────────────

local function GetEquipment()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  if player == nil then return nil, "LocalPlayer nil" end
  if type(player.GetEquipment) ~= "function" then
    return nil, "player.GetEquipment is not a function";
  end
  local ok, eq = Try("player:GetEquipment()", function()
    return player:GetEquipment();
  end);
  if not ok or eq == nil then
    return nil, "player:GetEquipment() returned nil";
  end
  return eq, nil
end

local function GetEquipmentCount(eq)
  if eq == nil then return nil end
  if type(eq.GetCount) == "function" then
    local ok, c = pcall(function() return eq:GetCount(); end);
    if ok and type(c) == "number" then return c end
  end
  if type(eq.GetSize) == "function" then
    local ok, c = pcall(function() return eq:GetSize(); end);
    if ok and type(c) == "number" then return c end
  end
  return nil
end

local function EnumerateEquippedItems()
  local eq, err = GetEquipment();
  if err ~= nil then return nil, err end

  if type(eq.GetItem) ~= "function" then
    return nil, "equipment.GetItem is not a function";
  end

  local count = GetEquipmentCount(eq);

  local out = {
    version = "equip-export-3",
    character = CharacterName(),
    count = count,
    items = {},
  };

  local function addSlot(slot)
    if ShouldSkipEquippedSlot(slot) then
      return
    end

    local ok, item = pcall(function() return eq:GetItem(slot); end);
    if ok and item ~= nil then
      local rec = ExtractItemRecord(item, slot);
      if rec.name ~= nil then
        table.insert(out.items, rec);
      end
    end
  end

  if type(count) == "number" then
    for slot = 1, count do
      addSlot(slot);
    end
  else
    -- Count unavailable: scan conservatively, stop after a long nil streak
    local maxSlot = 40;
    local nilStreak = 0;
    local nilStreakStop = 20;

    for slot = 1, maxSlot do
      local ok, item = pcall(function() return eq:GetItem(slot); end);
      if ok and item ~= nil then
        nilStreak = 0;
        local rec = ExtractItemRecord(item, slot);
        if rec.name ~= nil then
          table.insert(out.items, rec);
        end
      else
        nilStreak = nilStreak + 1;
        if nilStreak >= nilStreakStop then break end
      end
    end
    out._note = "Equipment count unavailable; scanned slots 1.." .. tostring(maxSlot);
  end

  return out, nil;
end

local function CollectItemNames(equip, ss)
  local names = {};
  local function addName(rec)
    local n = rec.infoName or rec.name;
    if n ~= nil and n ~= "" then
      table.insert(names, n);
    end
  end
  if equip ~= nil and equip.items ~= nil then
    for _, rec in ipairs(equip.items) do addName(rec) end
  end
  if ss ~= nil and ss.items ~= nil then
    for _, rec in ipairs(ss.items) do addName(rec) end
  end
  return names;
end

-- ── Combined export (equipped + shared storage chest) ───────────────────────

local function ExportCombined(sharedChestName)
  sharedChestName = Trim(sharedChestName or "lgo");
  if sharedChestName == "" then sharedChestName = "lgo" end

  local equip, errE = EnumerateEquippedItems();
  if errE ~= nil then
    Print("export: equip ERROR: " .. tostring(errE));
    return;
  end

  local ss, errS = EnumerateSharedStorageItems(function(chestName)
    return chestName ~= nil and Lower(chestName) == Lower(sharedChestName);
  end);
  if errS ~= nil then
    Print("export: shared storage ERROR: " .. tostring(errS));
    return;
  end

  local out = {
    version = "lgo-gearlist-1",
    character = CharacterName(),
    class = CharacterClass(),
    baseStats = GetBaseStats(),
    names = CollectItemNames(equip, ss),
  };

  Print("export: equipped=" .. tostring(#equip.items) ..
    " + sharedStorage('" .. sharedChestName .. "')=" .. tostring(#ss.items));
  SaveAccount("gearNames", out);
  Print("export: saved " .. tostring(#out.names) .. " owned item instances");
end

-- ── Shell command ─���──────────────────────────────────────────────────────────

Thalya.lgo.Command = Thalya.lgo.Command or Turbine.ShellCommand();

function Thalya.lgo.Command:Execute(command, arguments)
  arguments = Trim(arguments);

  if arguments == "" then
    Print("Commands:");
    Print("  /lgo export");
    Print("  /lgo export chest <name>");
    Print("");
    Print("Workflow:");
    Print("  1) Open Shared Storage panel at least once");
    Print("  2) Put candidate items in chest named 'lgo'");
    Print("  3) Run: /lgo export");
    return;
  end

  local action, rest = arguments:match("^(%S+)%s*(.*)$");
  action = Lower(action);
  rest = rest or "";

  if action == "export" then
    rest = Trim(rest);
    if rest == "" then
      ExportCombined("lgo");
      return;
    end

    local sub, subrest = rest:match("^(%S+)%s*(.*)$");
    sub = Lower(sub);
    subrest = subrest or "";

    if sub == "chest" then
      ExportCombined(subrest);
      return;
    end

    Print("export: unknown subcommand '" .. tostring(sub) .. "'");
    Print("Try: /lgo export  OR  /lgo export chest <name>");
    return;
  end

  Print("Unknown subcommand: " .. tostring(action));
end

Turbine.Shell.AddCommand("lgo;LGO", Thalya.lgo.Command);

Print("loaded — type /lgo for help.");