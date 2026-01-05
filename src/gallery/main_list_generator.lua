local OUTPUT_DIR<const> = os.getenv("output_dir")
local INPUT_DIR<const> = os.getenv("input_dir")
local SITE_ROOT<const> = os.getenv("site_root")
if not OUTPUT_DIR or not SITE_ROOT or not INPUT_DIR then
  error("Environment variable 'output_dir', 'input_dir' or 'site_root' not defined, this should be only executed by project's Makefile")
end

local GALLERY_ROOT_DIR<const> = "/gallery"
local PAGE_TEMPLATE_START_PATH<const> = INPUT_DIR.."/gallery/template/page_start.html"
local PAGE_TEMPLATE_END_PATH<const> = INPUT_DIR.."/gallery/template/page_end.html"

assert(arg[1], "Input filename must be given");
assert(arg[2], "Output filename must be given");
assert(arg[3], "Drawings filename must be given");

local template = assert(io.open(arg[1], "r"):read("*a"))
local output = assert(io.open(arg[2], "w"))
local DRAWINGS<const> = dofile(arg[3])

local YEARLY_CATEGORY = "<ul>"

local current = 1
local currentYear
while DRAWINGS[current] do
  local drawing = DRAWINGS[current]
  
  YEARLY_CATEGORY = YEARLY_CATEGORY.."<li><a href=\""..SITE_ROOT..GALLERY_ROOT_DIR.."/"..drawing.date[1].."\">"..drawing.date[1].."</a></li>"
  
  while DRAWINGS[current] and DRAWINGS[current].date[1] == drawing.date[1] do
    current = current + 1
  end
end

YEARLY_CATEGORY = YEARLY_CATEGORY.."</ul>"

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

function writePage(post, writer)
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
  
  writer("#include \""..PAGE_TEMPLATE_START_PATH.."\"\n")
  writer(post.description.."\n")
  writer("#include \""..PAGE_TEMPLATE_END_PATH.."\"\n")
end

local ALL_TIME_LIST = "<div id=\"all_drawings_list\">"

for _, drawing in ipairs(DRAWINGS) do
  ALL_TIME_LIST = ALL_TIME_LIST.."ITEM_START("..drawing.date[3].." "..MONTHS_LOOKUP[drawing.date[2]].." "..drawing.date[1].." - "..drawing.title..", \""..createUrl(drawing).."\")\n"
  writePage(drawing, function(str) ALL_TIME_LIST = ALL_TIME_LIST..str end)
  ALL_TIME_LIST = ALL_TIME_LIST.."ITEM_END()\n"
end

ALL_TIME_LIST = ALL_TIME_LIST.."</div>"

template = template:gsub("[$]INSERT_YEAR_CATEGORY_HERE[$]", YEARLY_CATEGORY)
template = template:gsub("[$]INSERT_ALL_DRAWINGS_HERE[$]", ALL_TIME_LIST)
output:write(template)


