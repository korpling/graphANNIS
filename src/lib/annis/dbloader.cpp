#include "dbloader.h"

using namespace annis;

DBLoader::DBLoader(std::string location, std::function<void()> onloadCalback)
  : dbLoaded(false), location(location), onloadCalback(onloadCalback)
{
}
