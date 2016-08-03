#include <annis/api/admin.h>

#include <annis/db.h>

using namespace annis::api;

Admin::Admin()
{

}

Admin::~Admin()
{

}

void Admin::import(std::string sourceFolder, std::string targetFolder)
{
  DB targetDB;
  targetDB.loadRelANNIS(sourceFolder);
  targetDB.save(targetFolder);
}
