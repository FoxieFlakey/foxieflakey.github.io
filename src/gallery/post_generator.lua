local OUTPUT_DIR<const> = os.getenv("output_dir")
local INPUT_DIR<const> = os.getenv("input_dir")
local SITE_ROOT<const> = os.getenv("site_root")
if not OUTPUT_DIR or not SITE_ROOT or not INPUT_DIR then
  error("Environment variable 'output_dir', 'input_dir' or 'site_root' not defined, this should be only executed by project's Makefile")
end

local GALLERY_ROOT_DIR<const> = "/src/gallery"
local PAGE_TEMPLATE_START_PATH<const> = INPUT_DIR.."/src/allery/template/page_start.html"
local PAGE_TEMPLATE_END_PATH<const> = INPUT_DIR.."/src/allery/template/page_end.html"

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

function writePage(post, writer)
  checkArg("post.date[1]", post.date[1], "number")
  checkArg("post.date[2]", post.date[2], "number")
  checkArg("post.date[3]", post.date[3], "number")
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
  
  writer("#include \"gallery/template/page_start.html\"\n")
  writer(post.description.."\n")
  writer("#include \"gallery/template/page_end.html\"\n")
end

if arg[1] == nil then
  error("Path to .post is not given")
end

assert(arg[1], "Input filename must be given");
assert(arg[2], "Output filename must be given");

local drawing<const> = dofile(arg[1])
local output = io.open(arg[2], "a")
writePage(drawing, function(str)
  output:write(str)
end)

