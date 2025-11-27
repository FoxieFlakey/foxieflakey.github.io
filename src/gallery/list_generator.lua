local OUTPUT_DIR<const> = os.getenv("output_dir")
local INPUT_DIR<const> = os.getenv("input_dir")
local SITE_ROOT<const> = os.getenv("site_root")
if not OUTPUT_DIR or not SITE_ROOT or not INPUT_DIR then
  error("Environment variable 'output_dir', 'input_dir' or 'site_root' not defined, this should be only executed by project's Makefile")
end

local GALLERY_ROOT_DIR<const> = "/gallery"
local PAGE_TEMPLATE_START_PATH<const> = INPUT_DIR.."/gallery/template/page_start.html"
local PAGE_TEMPLATE_END_PATH<const> = INPUT_DIR.."/gallery/template/page_end.html"

-- Shamelessy copied from https://github.com/MightyPirates/OpenComputers/blob/571482db88080d56329e8f8cf0db2a90825bf1d7/src/main/resources/assets/opencomputers/lua/machine.lua#L68
function checkArg(n, have, ...)
  have = type(have)
  local function check(want, ...)
    if not want then
      return false
    else
      return have == want or check(...)
    end
  end
  if not check(...) then
    local msg = string.format("bad argument %s (%s expected, got %s)",
                              totstring(n), table.concat({...}, " or "), have)
    error(msg)
  end
end

function createUrl(post)
  return SITE_ROOT..GALLERY_ROOT_DIR.."/"..post.date[1].."/"..post.date[2].."/"..post.page_basename..".html"
end

function writePage(post, writer, optionalNext, optionalPrev)
  checkArg("post.date[1]", post.date[1], "number")
  checkArg("post.date[2]", post.date[2], "number")
  checkArg("post.date[3]", post.date[3], "number")
  checkArg("post.page_basename", post.page_basename, "string")
  checkArg("post.title", post.title, "string")
  checkArg("post.image_url", post.image_url, "string")
  checkArg("post.width", post.width, "number")
  checkArg("post.height", post.height, "number")
  checkArg("post.short_description", post.short_description, "string")
  checkArg("post.description", post.description, "string")
  
  writer("#define POST_EMBEDDING\n")
  writer("#define POST_YEAR "..post.date[1].."\n")
  writer("#define POST_MONTH "..post.date[2].."\n")
  writer("#define POST_DAY "..post.date[3].."\n")
  writer("#define POST_TITLE "..post.title.."\n")
  writer("#define POST_IMG_URL \""..SITE_ROOT..post.image_url.."\"\n")
  writer("#define POST_IMG_WIDTH \""..post.width.."\"\n")
  writer("#define POST_IMG_HEIGHT \""..post.height.."\"\n")
  writer("#define POST_SHORT_DESCRIPTION "..post.short_description.."\n")
  
  if optionalNext then
    writer("#define POST_NEXT_LINK \""..createUrl(optionalNext).."\"\n")
  end
  
  if optionalPrev then
    writer("#define POST_PREV_LINK \""..createUrl(optionalPrev).."\"\n")
  end
  
  writer("#include \""..PAGE_TEMPLATE_START_PATH.."\"\n")
  writer(post.description.."\n")
  writer("#include \""..PAGE_TEMPLATE_END_PATH.."\"\n")
end

assert(arg[1], "Input filename must be given");
assert(arg[2], "Output filename must be given");

-- Output would be looked like
-- $(output_dir)/intermediate/<n>/gallery/2025/10/index.html.unpreprocessed
local year, month = arg[2]:match("([0-9]+)/([0-9]+)/index.html.unpreprocessed$")
local isYearly = false
if not month then
  -- Might be yearly list!
  -- loking like
  -- $(output_dir)/intermediate/<n>/gallery/2025/10/index.html.unpreprocessed
  year = arg[2]:match("([0-9]+)/index.html.unpreprocessed$")
  isYearly = true
else
  month = assert(tonumber(month))
end

year = assert(tonumber(year))

local output = io.open(arg[2], "w")
local DRAWINGS<const> = dofile(arg[1])
local foundIdx

for idx, drawing in ipairs(DRAWINGS) do
  checkArg("drawing.date[1]", drawing.date[1], "number")
  checkArg("drawing.date[2]", drawing.date[2], "number")
  checkArg("drawing.page_basename", drawing.page_basename, "string")
  
  if drawing.date[1] == year then
    if isYearly then
      foundIdx = idx
      break
    end
    
    if drawing.date[2] == month then
      foundIdx = idx
      break
    end
  end
end

if not foundIdx then
  if isYearly then
    error("Cannot find list of drawings for Year: "..year)
  else
    error("Cannot find list of drawings for Year: "..year.." Month: "..month)
  end
end

output:write("#define LIST_YEAR "..year.."\n")
if not isYearly then
  output:write("#define LIST_MONTH "..month.."\n")
end

function monthly(startIdx, month)
  output:write("#define LIST_YEAR "..year.."\n")
  output:write("#define LIST_MONTH "..month.."\n")
  output:write([[
  #include "gallery/template/listing_start.html"

  #pragma push_macro("POST_EMBEDDING")
  #pragma push_macro("LIST_EMBEDDING")
  #define POST_EMBEDDING
  #define LIST_EMBEDDING

  LIST_BEGIN()
  ]])
  
  local current = startIdx
  while DRAWINGS[current] do
    local drawing = DRAWINGS[current]
    checkArg("drawing.date[1]", drawing.date[1], "number")
    checkArg("drawing.date[2]", drawing.date[2], "number")
    if drawing.date[1] ~= year or drawing.date[2] ~= month then
      break
    end
    
    output:write("LIST_ITEM_START("..drawing.title..", \""..createUrl(drawing).."\")\n")
    output:write("\n")
    writePage(drawing, function(str)
      output:write(str)
    end)
    output:write("\n")
    output:write("LIST_ITEM_END()\n")
    
    current = current + 1
  end

  output:write([[
  LIST_END()  

  #pragma pop_macro("POST_EMBEDDING")
  #pragma pop_macro("LIST_EMBEDDING")

  #include "gallery/template/listing_end.html"
  ]])
end

if isYearly then
  output:write("#define LIST_YEAR "..year.."\n")
  output:write([[
  #include "gallery/template/listing_start.html"

  #pragma push_macro("POST_EMBEDDING")
  #pragma push_macro("LIST_EMBEDDING")
  #define POST_EMBEDDING
  #define LIST_EMBEDDING

  LIST_BEGIN()
  ]])
  
  local MONTHS_LOOKUP = {
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December"
  }
  
  local current = foundIdx
  while DRAWINGS[current] do
    local drawing = DRAWINGS[current]
    checkArg("drawing.date[1]", drawing.date[1], "number")
    checkArg("drawing.date[2]", drawing.date[2], "number")
    if drawing.date[1] ~= year then
      break
    end
    
    output:write("LIST_ITEM_START("..MONTHS_LOOKUP[drawing.date[2]]..", \""..SITE_ROOT..GALLERY_ROOT_DIR.."/"..year.."/"..drawing.date[2].."\")\n")
    monthly(current, drawing.date[2])
    output:write("LIST_ITEM_END()\n")
    
    while DRAWINGS[current] and DRAWINGS[current].date[2] == drawing.date[2] do
      current = current + 1
    end
  end
  
  output:write([[
  LIST_END()  

  #pragma pop_macro("POST_EMBEDDING")
  #pragma pop_macro("LIST_EMBEDDING")

  #include "gallery/template/listing_end.html"
  ]])
else
  monthly(foundIdx, month)
end

