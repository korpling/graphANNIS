#include <iostream>
#include <string>
#include <cstdint>

#include <db.h>

using namespace std;

int main(int argc, char** argv)
{
  if(argc > 1)
  {
    annis::DB db;
    std::string path(argv[1]);
    db.loadNodeStorage(path);
  }
  else
  {
    std::cerr << "You have to specicy an corpus input directory" << std::endl;
  }
  return 0;
}

