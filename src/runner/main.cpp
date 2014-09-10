#include <iostream>
#include <string>
#include <cstdint>

#include "linenoise.h"
#include <db.h>

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
}


int main(int argc, char** argv)
{
  char* lineBuffer = NULL;

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setDefaultLogLevel(humble::logging::LogLevel::All);
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::ConsoleAppender());

  linenoiseHistoryLoad("annis4_history.txt");
  linenoiseSetCompletionCallback(completion);

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
    if (cmd == "import")
    {
      if(args.size() > 0)
      {
        std::cout << "Import relANNIS from " << args[0] << std::endl;
        annis::DB db;
        db.loadRelANNIS(args[0]);
      }
      else
      {
        std::cout << "You have to give a path as argument" << std::endl;
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
    free(lineBuffer);
  }
  std::cout << "Exiting" << std::endl;


  return 0;
}

