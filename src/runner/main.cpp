#include <iostream>
#include <string>
#include <cstdint>

#include "linenoise.h"
#include <annis/db.h>
#include <annis/util/helper.h>
#include <annis/json/jsonqueryparser.h>

#include <humblelogging/api.h>
#include <boost/algorithm/string.hpp>

using namespace std;

HUMBLE_LOGGER(logger, "default");

void completion(const char *bufRaw, linenoiseCompletions *lc)
{
  std::string buf(bufRaw);
  if(boost::starts_with(buf, "q"))
  {
    linenoiseAddCompletion(lc,"quit");
  }
  else if(boost::starts_with(buf, "e"))
  {
    linenoiseAddCompletion(lc,"exit");
  }
  else if(boost::starts_with(buf, "i"))
  {
    linenoiseAddCompletion(lc,"import");
  }
  else if(boost::starts_with(buf, "s"))
  {
    linenoiseAddCompletion(lc, "save");
  }
  else if(boost::starts_with(buf, "l"))
  {
    linenoiseAddCompletion(lc, "load");
  }
  else if(boost::starts_with(buf, "o"))
  {
    linenoiseAddCompletion(lc, "optimize");
  }
  else if(boost::starts_with(buf, "c"))
  {
    linenoiseAddCompletion(lc, "count");
  }
  else if(boost::starts_with(buf, "f"))
  {
    linenoiseAddCompletion(lc, "find");
  }
  else if(boost::starts_with(buf, "g"))
  {
    linenoiseAddCompletion(lc, "guess");
    linenoiseAddCompletion(lc, "guess_regex");
  }
  else if(boost::starts_with(buf, "p"))
  {
    linenoiseAddCompletion(lc, "plan");
  }
  else if(boost::starts_with(buf, "u"))
  {
    linenoiseAddCompletion(lc, "update_statistics");
  }
}


int main(int argc, char** argv)
{
  char* lineBuffer = NULL;

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
    "logger.level(*)=info\n"
  ));
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::ConsoleAppender());

  linenoiseHistoryLoad("annis4_history.txt");
  linenoiseSetCompletionCallback(completion);

  // our main database
  annis::DB db;


  bool exit = false;
  while(!exit && (lineBuffer = linenoise("annis4> ")) != NULL)
  {
    std::string line(lineBuffer);
    linenoiseHistoryAdd(lineBuffer);
    linenoiseHistorySave("annis4_history.txt");

    // split the line into it's components
    vector<string> args;
    boost::split(args,line, boost::is_any_of(" "));
    std::string cmd = "";
    if(args.size() > 0)
    {
      cmd = args[0];
      args.erase(args.begin());
    }
    try
    {
      if (cmd == "import")
      {
        if(args.size() > 0)
        {
          std::cout << "Import relANNIS from " << args[0] << std::endl;
          db.loadRelANNIS(args[0]);
        }
        else
        {
          std::cout << "You have to give a path as argument" << std::endl;
        }
      }
      else if(cmd == "save")
      {
        if(args.size() > 0)
        {
          std::cout << "Save to " << args[0] << std::endl;
          db.save(args[0]);
        }
        else
        {
          std::cout << "You have to give a path as argument" << std::endl;
        }
      }
      else if(cmd == "load")
      {
        if(args.size() > 0)
        {
          std::cout << "Loading from " << args[0] << std::endl;
          db.load(args[0]);
        }
        else
        {
          std::cout << "You have to give a path as argument" << std::endl;
        }
      }
      else if(cmd == "info")
      {
        std::cout << db.info() << std::endl;
      }
      else if(cmd == "optimize")
      {
        std::cout << "Optimizing..." << std::endl;
        db.optimizeAll();
        std::cout << "Finished." << std::endl;
      }
      else if(cmd == "count")
      {
        if(args.size() > 0)
        {
          std::string json = boost::join(args, " ");
          std::cout << "Counting..." << std::endl;
          std::stringstream ss;
          ss << json;
          std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(db, ss); 
          int counter =0;
          auto startTime = annis::Helper::getSystemTimeInMilliSeconds();
          while(q->next())
          {
            counter++;
          }
          auto endTime = annis::Helper::getSystemTimeInMilliSeconds();
          std::cout << counter << " matches in " << (endTime - startTime) << " ms" << std::endl;
        }
        else
        {
          std::cout << "you need to give the query JSON as argument" << std::endl;
        }
      }
      else if(cmd == "find")
      {
        if(args.size() > 0)
        {
          std::string json = boost::join(args, " ");
          std::cout << "Finding..." << std::endl;
          std::stringstream ss;
          ss << json;
          std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(db, ss); 
          int counter =0;
          while(q->next())
          {
            std::vector<annis::Match> m = q->getCurrent();
            for(auto i = 0; i < m.size(); i++)
            {
              const auto& n = m[i];
              std::cout << db.getNodeDebugName(n.node);
              if(n.anno.ns != 0 && n.anno.name != 0 != 0)
              {
                std::cout << " " << db.strings.str(n.anno.ns) 
                  << "::" << db.strings.str(n.anno.name);
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
        else
        {
          std::cout << "you need to give the query JSON as argument" << std::endl;
        }
      }
      else if(cmd == "update_statistics")
      {
        std::cout << "Updating statistics...";
        db.nodeAnnos.calculateStatistics();
        std::cout << " Done" << std::endl;
      }
      else if(cmd == "guess")
      { 
        if(args.size() == 3)
        {
          std::cout << "Guessed maximum count: " << db.nodeAnnos.guessMaxCount(args[0], args[1], args[2]) << std::endl;
        }
        else if(args.size() == 2)
        {
          std::cout << "Guessed maximum count: " << db.nodeAnnos.guessMaxCount(args[0], args[1]) << std::endl;
        }
        else
        {
          std::cout << "Must provide at two (name and value) or three (namespace name value) arguments" << std::endl;
        }
      }
      else if(cmd == "guess_regex")
      { 
        if(args.size() == 3)
        {
          std::cout << "Guessed maximum count: " << db.nodeAnnos.guessMaxCountRegex(args[0], args[1], args[2]) << std::endl;
        }
        else if(args.size() == 2)
        {
          std::cout << "Guessed maximum count: " << db.nodeAnnos.guessMaxCountRegex(args[0], args[1]) << std::endl;
        }
        else
        {
          std::cout << "Must provide at two (name and regex) or three (namespace name regex) arguments" << std::endl;
        }
      }
      else if(cmd == "plan")
      {
        if(args.size() > 0)
        {
          std::string json = boost::join(args, " ");
          std::cout << "Planning..." << std::endl;
          std::stringstream ss;
          ss << json;
          std::shared_ptr<annis::Query> q = annis::JSONQueryParser::parse(db, ss); 
          std::cout << q->getBestPlan()->debugString() << std::endl;

        }
        else
        {
          std::cout << "you need to give the query JSON as argument" << std::endl;
        }
      }
      else if (cmd == "quit" || cmd == "exit")
      {
        exit = true;
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
    free(lineBuffer);
  }
  std::cout << "Exiting" << std::endl;


  return 0;
}

