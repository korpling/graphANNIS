#include "dbloader.h"

using namespace annis;

DBLoader::DBLoader(std::string location, std::function<void()> onloadCalback)
  : location(location), dbLoaded(false), onloadCalback(onloadCalback)
{
}
