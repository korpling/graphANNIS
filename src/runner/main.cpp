#include <iostream>
#include <string>
#include <cstdint>

#include "linenoise.h"
#include <db.h>

#include <humblelogging/api.h>

using namespace std;

HUMBLE_LOGGER(logger, "default");

int main(int argc, char** argv)
{
  char* lineBuffer = NULL;

  humble::logging::Factory &fac = humble::logging::Factory::getInstance();
  fac.setDefaultLogLevel(humble::logging::LogLevel::All);
  fac.setDefaultFormatter(new humble::logging::PatternFormatter("[%date] %m\n"));
  fac.registerAppender(new humble::logging::ConsoleAppender());

  bool exit = false;
  while(!exit && (lineBuffer = linenoise("annis4> ")) != NULL)
  {
    std::string line(lineBuffer);
    // split the line into it's component
    if (line == "import")
    {
      std::cout << "Import relANNIS" << std::endl;
      annis::DB db;
      std::string path(argv[argc-1]);
      db.loadRelANNIS(path);
    }
    else if (line == "quit" || line == "exit")
    {
      exit = true;
    }
    else
    {
      std::cout << "Unknown command" << std::endl;
    }
    free(lineBuffer);
  }
  std::cout << "Exiting" << std::endl;


  return 0;
}

