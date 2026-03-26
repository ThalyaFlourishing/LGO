-- lgo: LotRO Gear Optimizer Plugin
--
-- Direction:
--   - Candidate gear comes from Shared Storage chest named "lgo"
--   - Ignore inventory bags for selection (bags/rows can be rearranged)
--
-- Commands:
--   /lgo
--     -> help
--   /lgo probe
--     -> write API probe data
--   /lgo ss
--     -> export ALL items in Shared Storage (debug)
--   /lgo ss chest <name>
--     -> export items in Shared Storage chest <name>
--   /lgo ss chestindex <n>
--     -> export items in Shared Storage chest index <n>
--   /lgo equip
--     -> export equipped items
--   /lgo export
--     -> export (equipped) + (shared storage chest 'lgo')
--   /lgo export chest <name>
--     -> export (equipped) + (shared storage chest <name>)
--
-- Data is written via Turbine.PluginData.Save(Turbine.DataScope.Account, key, table)

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

local function SaveAccount(prefix, data)
  local key = prefix .. "_" .. CharacterName() .. "_" .. NowKeySuffix();
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

local function ExtractItemRecord(item, indexOrSlot)
  local rec = {
    slot = indexOrSlot, -- "slot" is used for both equipment slots and storage indices
  };

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

  if item ~= nil and type(item.GetQuantity) == "function" then
    local ok, q = pcall(function() return item:GetQuantity(); end);
    if ok then rec.quantity = q end
  end

  if item ~= nil and type(item.GetChest) == "function" then
    local ok, c = pcall(function() return item:GetChest(); end);
    if ok then rec.chest = c end
  end

  if info ~= nil then
    rec.itemInfo = {};
    local fields = {
      "GetCategory",
      "GetQuality",
      "GetDescription",
      "GetLevel",
      "GetRequiredLevel",
      "GetItemClass",
    };
    for _, methodName in ipairs(fields) do
      local v, existed, ok = TryCall0(info, methodName);
      if existed and ok and v ~= nil then
        rec.itemInfo[methodName] = tostring(v);
      end
    end
  end

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
      return nil, "Shared Storage not available (open it in-game first)";
    end
  end

  return ss, nil;
end

local function GetSharedStorageChestName(ss, chestIndex)
  if ss == nil then return nil end
  if type(ss.GetChestName) ~= "function" then return nil end
  local ok, n = pcall(function() return ss:GetChestName(chestIndex); end);
  if ok and n ~= nil and n ~= "" then return n end
  return nil
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

  local okCount, count = Try("sharedStorage:GetCount()", function() return ss:GetCount(); end);
  if not okCount or type(count) ~= "number" then
    return nil, "sharedStorage:GetCount() did not return a number";
  end

  local cap = nil;
  if type(ss.GetCapacity) == "function" then
    local okCap, c = Try("sharedStorage:GetCapacity()", function() return ss:GetCapacity(); end);
    if okCap then cap = c end
  end

  local out = {
    version = "shared-storage-export-1",
    character = CharacterName(),
    storage = { count = count, capacity = cap },
    items = {},
    chestsSeen = {}, -- [chestIndex] = chestName
  };

  for i = 1, count do
    local okItem, item = Try("sharedStorage:GetItem(" .. tostring(i) .. ")", function()
      return ss:GetItem(i);
    end);

    if okItem and item ~= nil then
      local rec = ExtractItemRecord(item, i);

      if rec.chest ~= nil then
        local chestName = GetSharedStorageChestName(ss, rec.chest);
        if chestName ~= nil then
          rec.chestName = chestName;
          out.chestsSeen[tostring(rec.chest)] = chestName;
        end
      end

      if rec.name ~= nil then
        if filterFn == nil or filterFn(rec) then
          table.insert(out.items, rec);
        end
      end
    end
  end

  return out, nil;
end

local function ExportSharedStorageAll()
  local data, err = EnumerateSharedStorageItems(nil);
  if err ~= nil then
    Print("ss: ERROR: " .. tostring(err));
    return;
  end
  SaveAccount("lgo_ss", data);
  Print("ss: exported " .. tostring(#data.items) .. " items (with names)");
end

local function ExportSharedStorageChestName(chestName)
  chestName = Trim(chestName);
  if chestName == "" then
    Print("ss chest: please provide a chest name, e.g. /lgo ss chest lgo");
    return;
  end

  local data, err = EnumerateSharedStorageItems(function(rec)
    return rec.chestName ~= nil and Lower(rec.chestName) == Lower(chestName);
  end);

  if err ~= nil then
    Print("ss chest: ERROR: " .. tostring(err));
    return;
  end

  data.filter = { type = "sharedStorageChestName", value = chestName };
  SaveAccount("lgo_ss_chest", data);
  Print("ss chest: exported " .. tostring(#data.items) .. " items in chest '" .. chestName .. "'");
end

local function ExportSharedStorageChestIndex(chestIndexStr)
  chestIndexStr = Trim(chestIndexStr);
  local chestIndex = tonumber(chestIndexStr);
  if chestIndex == nil then
    Print("ss chestindex: please provide a number, e.g. /lgo ss chestindex 10");
    return;
  end

  local data, err = EnumerateSharedStorageItems(function(rec)
    return rec.chest ~= nil and tonumber(rec.chest) == chestIndex;
  end);

  if err ~= nil then
    Print("ss chestindex: ERROR: " .. tostring(err));
    return;
  end

  data.filter = { type = "sharedStorageChestIndex", value = chestIndex };
  SaveAccount("lgo_ss_chestindex", data);
  Print("ss chestindex: exported " .. tostring(#data.items) .. " items in chest index " .. tostring(chestIndex));
end

-- ── Equipment enumeration ───────────────────────────────────────────────────

local function GetEquipment()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  if player == nil then return nil, "LocalPlayer nil" end
  if type(player.GetEquipment) ~= "function" then
    return nil, "player.GetEquipment is not a function";
  end
  local ok, eq = Try("player:GetEquipment()", function() return player:GetEquipment(); end);
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
    version = "equip-export-1",
    character = CharacterName(),
    count = count,
    items = {},
  };

  local function addSlot(slot)
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
    -- Conservative scan if we can't discover count. Stop after long nil streak.
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
        if nilStreak >= nilStreakStop then
          break;
        end
      end
    end
    out._note = "Equipment count unavailable; scanned slots 1.." .. tostring(maxSlot);
  end

  return out, nil;
end

local function ExportEquipped()
  local data, err = EnumerateEquippedItems();
  if err ~= nil then
    Print("equip: ERROR: " .. tostring(err));
    return;
  end
  SaveAccount("lgo_equip", data);
  Print("equip: exported " .. tostring(#data.items) .. " equipped items (with names)");
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

  local ss, errS = EnumerateSharedStorageItems(function(rec)
    return rec.chestName ~= nil and Lower(rec.chestName) == Lower(sharedChestName);
  end);
  if errS ~= nil then
    Print("export: shared storage ERROR: " .. tostring(errS));
    return;
  end

  local out = {
    version = "lgo-export-1",
    character = CharacterName(),
    selectedSharedStorageChestName = sharedChestName,
    equipped = equip,
    sharedStorage = ss,
  };

  SaveAccount("lgo_export", out);
  Print("export: equipped=" .. tostring(#equip.items) .. " + sharedStorage('" .. sharedChestName .. "')=" .. tostring(#ss.items));
end

-- ── Probe (API surface) ──────────────────────────────────────────────────────

local function ProbeAPIs()
  local player = Turbine.Gameplay.LocalPlayer.GetInstance();
  local backpack = nil;
  local sharedStorage = nil;
  local vault = nil;
  local equipment = nil;

  if player ~= nil then
    if player.GetBackpack ~= nil then backpack = player:GetBackpack(); end
    if player.GetSharedStorage ~= nil then
      local ok, ss = pcall(function() return player:GetSharedStorage(); end);
      if ok then sharedStorage = ss end
    end
    if player.GetVault ~= nil then
      local ok, v = pcall(function() return player:GetVault(); end);
      if ok then vault = v end
    end
    if player.GetEquipment ~= nil then
      local ok, e = pcall(function() return player:GetEquipment(); end);
      if ok then equipment = e end
    end
  end

  local probe = {
    version = "probe-3",
    character = CharacterName(),
    notes = {
      "Probe tests a fixed list of methods (userdata cannot be enumerated reliably).",
      "Candidate gear is expected from Shared Storage chest name 'lgo'.",
    },
    player = {},
    sharedStorage = {},
    vault = {},
    backpack = {},
    equipment = {},
    sampleSharedStorage = {
      count = nil,
      capacity = nil,
      firstItem = {},
      firstItemInfo = {},
    },
  };

  local function ProbeMethods(obj, methodNames)
    local out = {};
    if obj == nil then
      out._isNil = true;
      return out;
    end
    for _, m in ipairs(methodNames) do
      out[m] = tostring(obj[m]);
    end
    return out;
  end

  probe.player = ProbeMethods(player, {
    "GetBackpack",
    "GetEquipment",
    "GetVault",
    "GetSharedStorage",
    "GetSharedVault",
    "GetInventory",
  });

  probe.backpack = ProbeMethods(backpack, {
    "GetSize",
    "GetCapacity",
    "GetItem",
    "GetBag",
    "GetContainer",
  });

  probe.equipment = ProbeMethods(equipment, {
    "GetItem",
    "GetCount",
    "GetSize",
  });

  probe.sharedStorage = ProbeMethods(sharedStorage, {
    "IsAvailable",
    "GetCount",
    "GetCapacity",
    "GetItem",
    "GetChestName",
    "GetChestCount",
  });

  probe.vault = ProbeMethods(vault, {
    "IsAvailable",
    "GetCount",
    "GetCapacity",
    "GetItem",
    "GetChestName",
    "GetChestCount",
  });

  -- sample: shared storage item 1 (if available)
  if sharedStorage ~= nil and type(sharedStorage.IsAvailable) == "function" then
    local okAvail, avail = pcall(function() return sharedStorage:IsAvailable(); end);
    if okAvail and avail and type(sharedStorage.GetCount) == "function" then
      local okC, c = pcall(function() return sharedStorage:GetCount(); end);
      if okC then probe.sampleSharedStorage.count = c end
      if type(sharedStorage.GetCapacity) == "function" then
        local okCap, cap = pcall(function() return sharedStorage:GetCapacity(); end);
        if okCap then probe.sampleSharedStorage.capacity = cap end
      end

      if type(sharedStorage.GetItem) == "function" and type(probe.sampleSharedStorage.count) == "number" and probe.sampleSharedStorage.count > 0 then
        local okI, item = pcall(function() return sharedStorage:GetItem(1); end);
        if okI and item ~= nil then
          probe.sampleSharedStorage.firstItem = ProbeMethods(item, {
            "GetName",
            "GetItemInfo",
            "GetQuantity",
            "GetChest",
          });

          local info = GetItemInfoSafe(item);
          probe.sampleSharedStorage.firstItemInfo = ProbeMethods(info, {
            "GetName",
            "GetCategory",
            "GetQuality",
            "GetDescription",
          });

          if type(item.GetChest) == "function" and type(sharedStorage.GetChestName) == "function" then
            local okChest, chest = pcall(function() return item:GetChest(); end);
            if okChest and chest ~= nil then
              local okCN, cn = pcall(function() return sharedStorage:GetChestName(chest); end);
              if okCN then
                probe.sampleSharedStorage.firstItemResolvedChestName = cn;
                probe.sampleSharedStorage.firstItemChest = chest;
              end
            end
          end
        end
      end
    end
  end

  SaveAccount("lgo_probe", probe);
  Print("probe: wrote probe file (check PluginData)");
end

-- ── Shell command ────────────────────────────────────────────────────────────

Thalya.lgo.Command = Thalya.lgo.Command or Turbine.ShellCommand();

function Thalya.lgo.Command:Execute(command, arguments)
  arguments = Trim(arguments);

  if arguments == "" then
    Print("Commands:");
    Print("  /lgo probe");
    Print("  /lgo ss");
    Print("  /lgo ss chest <name>");
    Print("  /lgo ss chestindex <n>");
    Print("  /lgo equip");
    Print("  /lgo export");
    Print("  /lgo export chest <name>");
    Print("");
    Print("Workflow:");
    Print("  1) Put candidate items in Shared Storage chest named 'lgo'");
    Print("  2) Run: /lgo export");
    return;
  end

  local action, rest = arguments:match("^(%S+)%s*(.*)$");
  action = Lower(action);
  rest = rest or "";

  if action == "probe" then
    ProbeAPIs();
    return;
  end

  if action == "ss" then
    rest = Trim(rest);
    if rest == "" then
      ExportSharedStorageAll();
      return;
    end

    local sub, subrest = rest:match("^(%S+)%s*(.*)$");
    sub = Lower(sub);
    subrest = subrest or "";

    if sub == "chest" then
      ExportSharedStorageChestName(subrest);
      return;
    end

    if sub == "chestindex" then
      ExportSharedStorageChestIndex(subrest);
      return;
    end

    Print("ss: unknown subcommand '" .. tostring(sub) .. "'");
    Print("Try: /lgo ss  OR  /lgo ss chest <name>  OR  /lgo ss chestindex <n>");
    return;
  end

  if action == "equip" then
    ExportEquipped();
    return;
  end

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