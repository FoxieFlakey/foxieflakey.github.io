local OUTPUT_DIR<const> = os.getenv("output_dir")
local INPUT_DIR<const> = os.getenv("input_dir")
local SITE_HOST_ROOT<const> = os.getenv("site_host_root")
if not OUTPUT_DIR or not SITE_HOST_ROOT or not INPUT_DIR then
  error("Environment variable 'output_dir', 'input_dir' or 'site_host_root' not defined, this should be only executed by project's Makefile")
end

local GALLERY_ROOT_DIR<const> = "/gallery"
assert(arg[1], "Output filename must be given");
assert(arg[2], "Drawings filename must be given");

local output = assert(io.open(arg[1], "w"))
local DRAWINGS<const> = dofile(arg[2])

function writeLine(str)
  output:write(str)
  output:write("\n")
end

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

writeLine("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")
writeLine("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" xmlns:image=\"http://www.google.com/schemas/sitemap-image/1.1\">")
for _, drawing in ipairs(DRAWINGS) do
  checkArg("drawing.date[1]", drawing.date[1], "number")
  checkArg("drawing.date[2]", drawing.date[2], "number")
  checkArg("drawing.date[3]", drawing.date[3], "number")
  checkArg("drawing.page_basename", drawing.page_basename, "string")
  
  local year = drawing.date[1]
  local month = drawing.date[2]
  local day  = drawing.date[3]
  local basename = drawing.page_basename
  
  local url_to_drawing = SITE_HOST_ROOT.."/"..year.."/"..month.."/"..basename..".html";
  local url_to_image = SITE_HOST_ROOT.."/"..drawing.image_url;
  
  writeLine("  <url>")
  writeLine("    <loc>"..url_to_drawing.."</loc>")
  writeLine("    <lastmod>"..year.."-"..month.."-"..day.."</lastmod>")
  writeLine("    <image:image>")
  writeLine("      <image:loc>"..url_to_image.."</image:loc>")
  writeLine("    </image:image>")
  writeLine("  </url>")
end
writeLine("</urlset>")


