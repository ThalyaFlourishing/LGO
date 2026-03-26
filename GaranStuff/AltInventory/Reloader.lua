import "Turbine";
import "Turbine.UI";
Reloader=Turbine.UI.Control()
Reloader.step=0
Reloader.cmdList={}
Reloader.Update=function()
	Turbine.PluginManager.UnloadScriptState("AltInventory")
	-- now reload altinventory
	Turbine.PluginManager.LoadPlugin("AltInventory");
	Reloader:SetWantsUpdates(false);
end
do
	local AltInventoryIsLoaded=false
	local tmpPlugins=Turbine.PluginManager.GetLoadedPlugins();
	for pluginIndex=1,#tmpPlugins do
		if tmpPlugins[pluginIndex].Name=="AltInventory" then
			AltInventoryIsLoaded=true;
			break;
		end
	end

	if AltInventoryIsLoaded then
		Reloader:SetWantsUpdates(true);
	else
		if Turbine.Shell.IsCommand("aide") then
			error("N'ex\195\169cutez PAS AltInventoryReloader. Ce plugin est utilis\195\169 en interne par AltInventory et ne doit pas \195\170tre d\195\169marr\195\169 automatiquement ou par l'utilisateur.",0)
		elseif Turbine.Shell.IsCommand("hilfe") then
			error("F\195\188hren Sie AltInventoryReloader NICHT aus. Dieses Plugin wird intern von AltInventory verwendet und sollte nicht automatisch oder vom Benutzer gestartet werden.",0)
		else
			error("Do NOT run AltInvnetoryReloader. This plugin is used internally by AltInventory and should not be started automatically or by the user.",0)
		end
	end
end
