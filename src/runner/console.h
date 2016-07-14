#pragma once

#include <vector>

#include <annis/db.h>
#include <annis/DBCache.h>
#include <annis/util/helper.h>
#include <annis/json/jsonqueryparser.h>

#include <boost/algorithm/string.hpp>
#include <boost/filesystem.hpp>


namespace annis
{

class Console
{
public:
  Console();

  bool execute(const std::string& cmd, const std::vector<std::string>& args);

private:
  // our main database
  boost::filesystem::path currentDBPath;
  annis::DBCache dbCache;

  std::weak_ptr<annis::DB> dbPtr = dbCache.get(currentDBPath.string());
};

}

