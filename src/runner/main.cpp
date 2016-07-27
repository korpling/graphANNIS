#include <iostream>
#include <string>
#include <cstdint>
#include <memory>

#include "linenoise.h"

#include "console.h"

#include <humblelogging/api.h>


using namespace std;


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
  else if(boost::starts_with(buf, "m"))
  {
    linenoiseAddCompletion(lc, "memory");
  }
}

int main(int argc, char** argv)
{
  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setConfiguration(humble::logging::DefaultConfiguration::createFromString(
    "logger.level(*)=debug\n"
  ));
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::ConsoleAppender());


  annis::Console console;

  if(argc > 1)
  {
    // command line mode
    std::string cmd(argv[1]);
    std::vector<std::string> args;

    for(int i=2; i < argc; i++)
    {
      args.push_back(std::string(argv[i]));
    }
    console.execute(cmd, args);
  }
  else
  {
    // interactive mode

    char* lineBuffer = NULL;

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

      exit = console.execute(cmd, args);

      free(lineBuffer);
    }
    std::cout << "Exiting" << std::endl;

  }


  return 0;
}

