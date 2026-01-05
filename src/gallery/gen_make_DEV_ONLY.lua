local drawings = dofile("drawings.lua")
local year_printed = {}
local month_printed = {}

for i=1,#drawings do
  local drawing = drawings[i];
  if year_printed[drawing.date[1]] ~= true then
    print("\tgallery/"..drawing.date[1].."/index.html \\")
    print("\t\\")
  end
  year_printed[drawing.date[1]] = true;
  
  if month_printed[drawing.date[1] * 100 + drawing.date[2]] ~= true then
    print("\tgallery/"..drawing.date[1].."/"..drawing.date[2].."/index.html \\")
    print("\t\\")
  end
  month_printed[drawing.date[1] * 100 + drawing.date[2]] = true;
  
  print("\tgallery/"..drawing.date[1].."/"..drawing.date[2].."/"..drawing.page_basename..".html \\")
  print("\tgallery/"..drawing.date[1].."/"..drawing.date[2].."/"..drawing.page_basename..".png \\")
end

