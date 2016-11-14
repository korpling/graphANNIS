#pragma once

#include <vector>

#include <annis/db.h>
#include <annis/dbcache.h>
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

  void import(const std::vector<std::string>& args);
  void save(const std::vector<std::string>& args);
  void load(const std::vector<std::string>& args);
  void info();
  void optimize();
  void count(const std::vector<std::string>& args);
  void find(const std::vector<std::string>& args);
  void updateStatistics();
  void guess(const std::vector<std::string>& args);
  void guessRegex(const std::vector<std::string>& args);
  void plan(const std::vector<std::string>& args);
  void memory(const std::vector<std::string> args);

private:
  // our main database
  boost::filesystem::path currentDBPath;
  annis::DBCache dbCache;

  std::shared_ptr<annis::DB> db;
};

}

