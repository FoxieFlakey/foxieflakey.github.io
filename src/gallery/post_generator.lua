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

if arg[1] == nil then
  error("Path to .post is not given")
end

assert(arg[1], "Input filename must be given");
assert(arg[2], "Output filename must be given");

print("Input "..arg[1])
print("Output "..arg[2])

-- Output would be looked like
-- $(output_dir)/intermediate/<n>/gallery/2025/10/micro_foxie.html.unpreprocessed
local year, month, page_basename = arg[2]:match("([0-9]+)/([0-9]+)/([^/]+)$")
page_basename = page_basename:match("^([^.]+)")
year = assert(tonumber(year))
month = assert(tonumber(month))

local DRAWINGS<const> = dofile(arg[1])
local foundIdx, foundDrawing

for idx, drawing in ipairs(DRAWINGS) do
  checkArg("drawing.date[1]", drawing.date[1], "number")
  checkArg("drawing.date[2]", drawing.date[2], "number")
  checkArg("drawing.page_basename", drawing.page_basename, "string")
  
  if drawing.date[1] == year and drawing.date[2] == month and drawing.page_basename == page_basename then
    foundIdx = idx
    foundDrawing = drawing
    break
  end
end

if not foundDrawing then
  error("Cannot find drawing for Year: "..year.." Month: "..month.." Page basename: "..page_basename)
end

local nextDraw = DRAWINGS[foundIdx - 1]
local prevDraw = DRAWINGS[foundIdx + 1]

local output = io.open(arg[2], "a")
writePage(foundDrawing, function(str)
  output:write(str)
end, nextDraw, prevDraw)

