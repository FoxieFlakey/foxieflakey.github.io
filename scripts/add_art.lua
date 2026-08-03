local data_file = "./web/src/config/arts/data.rs"

function readAll(path)
  local fp<close> = assert(io.open(data_file, "r"))
  return assert(fp:read("*a"))
end

function writeAll(path, data)
  local fp<close> = assert(io.open(data_file, "w"))
  assert(fp:write(data))
end

function ask(question)
  io.write(question..": ")
  io.flush()
  return io.read()
end

function askMulti(question)
  io.write(question..": ")
  io.flush()
  
  local buffer = {}
  while true do
    local c, err = io.read(1)
    if c == nil and err == nil then
      -- EOF reached
      break
    end
    
    table.insert(buffer, assert(c))
  end
  print()
  return table.concat(buffer)
end

local year = ask("Year of art post")
local month = ask("Month of art post")
local day = ask("Day of art post")
local filename = ask("Art filename in data dir")
local title = ask("Title of art")
local page_id = ask("Page id of art")
local description = askMulti("Description. EOF/Ctrl+D to end")

local source = readAll(data_file)
local transformed = {}

for line in source:gmatch("([^\n]*)\n?") do
    local art_count = line:match("^pub static ARTS: [[]Art; ([0-9]+)[]] = [[]$")
    if art_count then
      art_count = tonumber(art_count) + 1
      table.insert(transformed, "pub static ARTS: [Art; "..art_count.."] = [")
      table.insert(transformed, ([[    art! {
        posted_on: NaiveDate::from_ymd_opt(%s, %s, %s).unwrap(),
        data: include_bytes!("data/%s"),
        title: "%s",
        page_id: "%s",
        description_long: "%s",
    },]]):format(year, month, day, filename, title, page_id, description))
    elseif line ~= "" or source:sub(-1) == "\n" then -- handles empty trailing checks if needed
        table.insert(transformed, line)
    end
end

transformed = table.concat(transformed, "\n")
print("Added new art")
writeAll(data_file, transformed)















