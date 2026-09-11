Stat definition file statdata.csv:

You can view this comma separated values file with something like a spreadsheet application. Upload it to Google Sheets for example or use LibreOffice Calc, Microsoft Excel etc.

Compilation of statdata.csv:

Execute statdcmp.vbs (double click) to generate the CalcStat script file. This will contain the major calculation function.
Output version "Percentage calculations only" supports ratings<->percentages and rating penetration calculations and will do for all known Lotro plugins at this time.

Installation/use:

Lotro Plugins:
Extract the zip file to your Lotro Plugins directory. After, you should have a directory like "C:\Users\UserName\Documents\The Lord of the Rings Online\Plugins\Giseldah\CalcStat"
Plugins can import this library using the statement: import "Giseldah.CalcStat"
The directory already contains a "Percentage calculations only" version by default, so you don't need to compile statdata.csv if this is all you require.

OpenOffice/LibreOffice:
Compile statdata.csv with StarBasic as output script type. This will generate the CalcStat.bas file with the CalcStat function.
Inside the Calc spreadsheet application, create a new CalcStat macro module, either under "My macros & Dialogs" for use globally in any spreadsheet or create the module in a specific spreadsheet.
Copy the content of the CalcStat.bas file to this new module.

VB-script (Windows):
Compile statdata.csv with VB-script as output script type. This will generate the CalcStat.vbs file with the CalcStat function.
You can include CalcStat.vbs conveniently in a script job by using a WSF file (see https://en.wikipedia.org/wiki/Windows_Script_File).
See the vbs_example directory for an example script.

PHP:
Compile statdata.csv with PHP as output script type. This will generate the calcstat.php file.
You can include this file in your PHP enabled web pages and start using the CalcStat function for your calculations.

JavaScript:
Compile statdata.csv with JavaScript as output script type. This will generate the calcstat.js file.
You can include this file in your JavaScript enabled web pages and start using the CalcStat function for your calculations.

Python:
Compile statdata.csv with Python as output script type. This will generate the calcstat.py file.
You can use something like this, to include the function in your script: from calcstat import CalcStat

Examples:
Most examples need a compiled CalcStat.? file in the same directory.