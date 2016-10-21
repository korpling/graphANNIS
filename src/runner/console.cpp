#include "console.h"

#include <humblelogging/api.h>
#include <iomanip>

#include <annis/util/helper.h>

HUMBLE_LOGGER(logger, "default");

using namespace annis;

Console::Console()
 : dbCache(1073741824l*8l), dbPtr(dbCache.get(currentDBPath.string(), true))
{
  currentDBPath = boost::filesystem::unique_path(
          boost::filesystem::temp_directory_path().string() + "/annis-temporary-workspace-%%%%-%%%%-%%%%-%%%%");
  HL_INFO(logger, "Using " + currentDBPath.string() + " as temporary path");
}

bool Console::execute(const std::string &cmd, const std::vector<std::string> &args)
{
  try
  {
    if (cmd == "import")
    {
      import(args);
    }
    else if(cmd == "save")
    {
      save(args);
    }
    else if(cmd == "load")
    {
      load(args);
    }
    else if(cmd == "info")
    {
      info();
    }
    else if(cmd == "optimize")
    {
      optimize();
    }
    else if(cmd == "count")
    {
      count(args);
    }
    else if(cmd == "find")
    {
      find(args);
    }
    else if(cmd == "update_statistics")
    {
      updateStatistics();
    }
    else if(cmd == "guess")
    {
      guess(args);
    }
    else if(cmd == "guess_regex")
    {
      guessRegex(args);
    }
    else if(cmd == "plan")
    {
      plan(args);
    }
    else if(cmd == "memory")
    {
      memory(args);
    }
    else if (cmd == "quit" || cmd == "exit")
    {
      return true;
    }
    else
    {
      std::cout << "Unknown command \"" << cmd << "\"" << std::endl;
    }

  }
  catch(std::string ex)
  {
    std::cerr << "Exception: " << ex << std::endl;
  }

  return false;
}

void Console::import(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() > 0)
    {
      std::cout << "Import relANNIS from " << args[0] << std::endl;
      db->loadRelANNIS(args[0]);
      if(args.size() > 1)
      {
        // directly save the imported corpus to directory
        HL_INFO(logger, "saving to " +  args[1]);
        db->save(args[1]);
      }
    }
    else
    {
      std::cout << "You have to give a path as argument" << std::endl;
    }
  }
}

void Console::save(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() > 0)
    {
      std::cout << "Save to " << args[0] << std::endl;
      db->save(args[0]);
    }
    else
    {
      std::cout << "You have to give a path as argument" << std::endl;
    }
  }
}

void Console::load(const std::vector<std::string> &args)
{

  if(args.size() > 0)
  {
    std::cout << "Loading from " << args[0] << std::endl;
    dbPtr = dbCache.get(args[0], args.size() > 1 && args[1] == "preload");
  }
  else
  {
    std::cout << "You have to give a path as argument" << std::endl;
  }

}

void Console::info()
{
  if(auto db = dbPtr.lock())
  {
    std::cout << db->info() << std::endl;
  }
}

void Console::optimize()
{
  if(auto db = dbPtr.lock())
  {
    std::cout << "Optimizing..." << std::endl;
    db->optimizeAll();
    std::cout << "Finished." << std::endl;
  }
}

void Console::count(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() > 0)
    {
      std::string json = boost::join(args, " ");
      std::cout << "Counting..." << std::endl;
      std::stringstream ss;
      ss << json;
      try
      {
        std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
        int counter =0;
        auto startTime = annis::Helper::getSystemTimeInMilliSeconds();
        while(q->next())
        {
          counter++;
        }
        auto endTime = annis::Helper::getSystemTimeInMilliSeconds();
        std::cout << counter << " matches in " << (endTime - startTime) << " ms" << std::endl;
      }
      catch(Json::RuntimeError err)
      {
        std::cout << "JSON error: " << err.what() << std::endl;
      }

    }
    else
    {
      std::cout << "you need to give the query JSON as argument" << std::endl;
    }
  }
}

void Console::find(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() > 0)
    {
      std::string json = boost::join(args, " ");
      std::cout << "Finding..." << std::endl;
      std::stringstream ss;
      ss << json;
      try
      {
        std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
        int counter =0;
        while(q->next())
        {
          std::vector<annis::Match> m = q->getCurrent();
          for(size_t i = 0; i < m.size(); i++)
          {
            const auto& n = m[i];
            std::cout << db->getNodeDebugName(n.node);
            if(n.anno.ns != 0 && n.anno.name != 0)
            {
              std::cout << " " << db->strings.str(n.anno.ns)
                << "::" << db->strings.str(n.anno.name);
            }
            if(i < m.size()-1)
            {
             std::cout << ", ";
            }
          }
          std::cout << std::endl;
          counter++;
        }
        std::cout << counter << " matches" << std::endl;

      }
      catch(Json::RuntimeError err)
      {
        std::cout << "JSON error: " << err.what() << std::endl;
      }

    }
    else
    {
      std::cout << "you need to give the query JSON as argument" << std::endl;
    }
  }
}

void Console::updateStatistics()
{
  if(auto db = dbPtr.lock())
  {
    std::cout << "Updating statistics...";
    db->nodeAnnos.calculateStatistics();
    std::cout << " Done" << std::endl;
  }
}

void Console::guess(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() == 3)
    {
      std::cout << "Guessed maximum count: " << db->nodeAnnos.guessMaxCount(args[0], args[1], args[2]) << std::endl;
    }
    else if(args.size() == 2)
    {
      std::cout << "Guessed maximum count: " << db->nodeAnnos.guessMaxCount(args[0], args[1]) << std::endl;
    }
    else
    {
      std::cout << "Must provide at two (name and value) or three (namespace name value) arguments" << std::endl;
    }
  }
}

void Console::guessRegex(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() == 3)
    {
      std::cout << "Guessed maximum count: " << db->nodeAnnos.guessMaxCountRegex(args[0], args[1], args[2]) << std::endl;
    }
    else if(args.size() == 2)
    {
      std::cout << "Guessed maximum count: " << db->nodeAnnos.guessMaxCountRegex(args[0], args[1]) << std::endl;
    }
    else
    {
      std::cout << "Must provide at two (name and regex) or three (namespace name regex) arguments" << std::endl;
    }
  }
}

void Console::plan(const std::vector<std::string> &args)
{
  if(auto db = dbPtr.lock())
  {
    if(args.size() > 0)
    {
      std::string json = boost::join(args, " ");
      std::cout << "Planning..." << std::endl;
      std::stringstream ss;
      ss << json;
      try
      {
        std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(*db, db->edges, ss);
        std::cout << q->getBestPlan()->debugString() << std::endl;
      }
      catch(Json::RuntimeError err)
      {
        std::cout << "JSON error: " << err.what() << std::endl;
      }

    }
    else
    {
      std::cout << "you need to give the query JSON as argument" << std::endl;
    }
  }
}

void Console::memory(const std::vector<std::string> args)
{
  if(args.empty())
  {

    auto corpusSizes = dbCache.estimateCorpusSizes();
    for(auto it = corpusSizes.begin();
        it != corpusSizes.end(); it++)

    {
      if(!it->first.corpusPath.empty())
      {
        const DBCache::CorpusSize& size = it->second;
        std::cout << it->first.corpusPath << ": " << Helper::inMB(size.estimated) << " MB (estimated) " << Helper::inMB(size.measured) << " MB (measured)" << std::endl;
      }
    }
    DBCache::CorpusSize total = dbCache.calculateTotalSize();
    std::cout << "Used total memory (estimated): "  << Helper::inMB(total.estimated) << " MB" << std::endl;
    std::cout << "Used total memory (measured): "  << Helper::inMB(total.measured) << " MB" << std::endl;

  }
  else if(args[0] == "clear")
  {
    dbCache.releaseAll();
    std::cout << "Cleared cache" << std::endl;
  }
}

