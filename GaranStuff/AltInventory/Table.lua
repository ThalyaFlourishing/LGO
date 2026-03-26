
--import "Turbine.Utils";

Table = class();

local function DumpTable( t, indentation, seen )
	if ( t == nil ) then
		Turbine.Shell.WriteLine( indentation .. "(nil)" );
		return;
	end
	seen[t] = true;
	local s= {};
	local n = 0;
	for k in pairs(t) do
		n = n + 1;
		s[n] = k;
	end
	table.sort(s, function(a,b) return tostring(a) < tostring(b) end);
	for k,v in pairs(s) do
		Turbine.Shell.WriteLine( indentation .. tostring( v ) .. ": " .. tostring( t[v] ) );
		if type( t[v] ) == "table" and not seen[t[v]] then
			DumpTable( t[v], indentation .. "  ", seen );
		end
	end
end

Table.Dump = function( t, indentation )
	local seen = {};
	DumpTable( t, indentation or "  ", seen );
end
Table.ShallowCopy=function(src)
	local ret
	if type(src)=="table" then
		ret={}
		for k,v in pairs(src) do
			ret[k]=Table.ShallowCopy(v)
		end
	else
		ret=src
	end
	return ret
end
Table.Copy=function(object)
    local lookup_table = {}
    local function _copy(object)
        if type(object) ~= "table" then
            return object
        elseif lookup_table[object] then
            return lookup_table[object]
        end
        local new_table = {}
        lookup_table[object] = new_table
        for index, value in pairs(object) do
            new_table[_copy(index)] = _copy(value)
        end
--        return setmetatable(new_table, getmetatable(object))
        return setmetatable(new_table, _copy(getmetatable(object)))
    end
    return _copy(object)
end

Table.GetInsertIndex=function(srcTable,value)
	-- will return the index of the position to insert a new element in a sorted table - note, it will return the position of the value if it already exists
	local min=1
	local max=#srcTable
	local index=1
	if value>srcTable[max] then
		min=max+1
	else
		while min<max do
			index=math.floor((min+max)/2)
			if srcTable[index]<value then
				min=index+1
			else
				max=index
			end
		end
	end
	return min
end
Table.Contains=function(srcTable,value,useQuickSearch,allowPatterns,allowPartialMatch)
	-- there is a problem if the value being looked for is numeric
	-- only set useQuickSearch=true if the srcTable is sorted and you know that the srcTable uses integer indexes and has no holes in the indexes
	if allowPatterns==nil then allowPatterns=false end
	if allowPartialMatch==nil then allowPartialMatch=false end -- default to full element match only
	--note, useQuickSearch will invalidate allowPartialMatch
	local retVal=false
	if srcTable~=nil and value~=nil and type(value)~="table" then
		if (not allowPatterns) and useQuickSearch then
			-- use a simple binary search
			local max=#srcTable
			if max~=nil and max>0 then
				local min=1
				local index=1
				while min<max do
					index=math.floor((min+max)/2)
					if srcTable[index]<value then
						min=index+1
					else
						max=index
					end
				end
				if srcTable[min]==value then
					retVal=min
				end
			end
		else
			if type(value)=="string" and allowPartialMatch then
				for k,v in pairs(srcTable) do
					if allowPatterns then
						if string.find(v,value)~=nil then
							retVal=k
							break
						end
					else
						if string.find(v,value,nil,true) then
							retVal=k
							break
						end
					end
				end
			else
				for k,v in pairs(srcTable) do
					if v==value then
						retVal=k
						break
					end
				end
			end
		end
	end
	return retVal
end