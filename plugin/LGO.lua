-- LGO: LotRO Gear Optimizer Plugin
-- Exports current equipment and character stats for use with the lgo CLI optimizer.
--
-- Usage: /lgo export   => prints equipped gear to chat as JSON
--        /lgo help     => shows help text

import "Turbine";
import "Turbine.Gameplay";
import "Turbine.UI";
import "Turbine.UI.Lotro";

-- ── Plugin setup ─────────────────────────────────────────────────────────────

local plugin = Turbine.Plugin.GetCurrent();
plugin:SetName("LGO");

-- ── Helpers ───────────────────────────────────────────────────────────────────

--- Map from EquipmentSlot integer to slot name matching lgo CLI snake_case keys.
local SLOT_NAMES = {
    [1]  = "head",
    [2]  = "chest",
    [3]  = "legs",
    [4]  = "hands",
    [5]  = "feet",
    [6]  = "shoulders",
    [7]  = "back",
    [8]  = "neck",
    [9]  = "ear1",
    [10] = "ear2",
    [11] = "finger1",
    [12] = "finger2",
    [13] = "wrist1",
    [14] = "wrist2",
    [15] = "main_hand",
    [16] = "off_hand",
    [17] = "pocketed",
};

--- Print a string to the default chat panel.
local function Print(msg)
    Turbine.Shell.WriteLine("[LGO] " .. tostring(msg));
end

--- Escape a string for use inside a JSON double-quoted value.
local function JsonEscape(s)
    s = tostring(s);
    s = s:gsub('\\', '\\\\');
    s = s:gsub('"',  '\\"');
    s = s:gsub('\n', '\\n');
    s = s:gsub('\r', '\\r');
    s = s:gsub('\t', '\\t');
    return s;
end

-- ── Gear export ───────────────────────────────────────────────────────────────

--- Collect equipped items from the local player and return a JSON string.
local function ExportGear()
    local player = Turbine.Gameplay.LocalPlayer.GetInstance();
    if player == nil then
        Print("Could not access local player.");
        return;
    end

    local equipment = player:GetEquipment();
    if equipment == nil then
        Print("Could not access player equipment.");
        return;
    end

    local items = {};
    local slotCount = equipment:GetSize();

    for slot = 1, slotCount do
        local item = equipment:GetItem(slot);
        if item ~= nil then
            local info = item:GetItemInfo();
            if info ~= nil then
                local slotName = SLOT_NAMES[slot] or ("slot_" .. slot);
                local itemLevel = info:GetItemLevel and info:GetItemLevel() or 0;
                local name      = info:GetName and info:GetName() or "Unknown";

                -- Build a minimal stat block from item quality/ilvl.
                -- Full stat introspection requires LOTRO's GetAttributes which
                -- is only available for the player's own attributes, not items.
                -- This export captures name/slot/ilvl as a starting point.
                local entry = string.format(
                    '  {"name":"%s","slot":"%s","item_level":%d,"stats":{}}',
                    JsonEscape(name), slotName, itemLevel
                );
                table.insert(items, entry);
            end
        end
    end

    if #items == 0 then
        Print("No equipped items found.");
        return;
    end

    Print("Copy the JSON below into data/gear_set.json and run:  lgo score");
    Turbine.Shell.WriteLine("[");
    for i, entry in ipairs(items) do
        local comma = (i < #items) and "," or "";
        Turbine.Shell.WriteLine(entry .. comma);
    end
    Turbine.Shell.WriteLine("]");
end

--- Print current player primary stats using FreePeopleAttributes.
local function ExportStats()
    local player = Turbine.Gameplay.LocalPlayer.GetInstance();
    if player == nil then
        Print("Could not access local player.");
        return;
    end

    local attrs = player:GetAttributes();
    if attrs == nil then
        Print("Could not access player attributes.");
        return;
    end

    -- Cast to FreePeopleAttributes to access primary stats.
    local fp = attrs;

    Print("Character Stats:");
    local function TryPrint(label, fn)
        local ok, val = pcall(fn);
        if ok then
            Turbine.Shell.WriteLine(string.format("  %-22s %d", label .. ":", val));
        end
    end

    TryPrint("Might",      function() return fp:GetMight()    end);
    TryPrint("Agility",    function() return fp:GetAgility()  end);
    TryPrint("Vitality",   function() return fp:GetVitality() end);
    TryPrint("Will",       function() return fp:GetWill()     end);
    TryPrint("Fate",       function() return fp:GetFate()     end);
    TryPrint("Armor",      function() return fp:GetArmor()    end);
    TryPrint("Resistance", function() return fp:GetResistance() end);
    TryPrint("Morale",     function() return player:GetMorale()   end);
    TryPrint("Power",      function() return player:GetPower()    end);
end

-- ── Shell command ─────────────────────────────────────────────────────────────

local shellCommand = Turbine.ShellCommand();

function shellCommand:Execute(cmd, args)
    local action = (args or ""):match("^%s*(%S*)");
    action = (action or ""):lower();

    if action == "export" then
        ExportGear();
    elseif action == "stats" then
        ExportStats();
    else
        Print("LotRO Gear Optimizer  v" .. (plugin:GetVersion() or "0.1"));
        Turbine.Shell.WriteLine("  /lgo export   — print equipped items as JSON for the optimizer");
        Turbine.Shell.WriteLine("  /lgo stats    — show current character primary stats");
        Turbine.Shell.WriteLine("  /lgo help     — show this help message");
        Turbine.Shell.WriteLine("");
        Turbine.Shell.WriteLine("  Paste the /lgo export output into data/gear_set.json, then run:");
        Turbine.Shell.WriteLine("    lgo score      (evaluate the set)");
        Turbine.Shell.WriteLine("    lgo optimize   (find best-in-slot from data/items.json)");
    end
end

Turbine.Shell.AddCommand("lgo", shellCommand);

Print("loaded — type /lgo help for usage.");
